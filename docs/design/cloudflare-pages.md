# フロントを Vercel から Cloudflare Pages へ移す設計メモ

対象 Issue: #1209。実装は含まない。結論と根拠だけを書く。

> 移行は完了済み（#1231 で Vercel 経路を撤去。切り戻し経路は存在しない）。
> 本文は移行当時の検討記録で、Vercel・Fly.io など旧構成の固有名詞が残る。
> 現行構成・運用は `docs/architecture.md` と `docs/runbook.md` を参照。

## 1. 今の構成

- フロントは Leptos CSR を Trunk でビルドした静的ファイル一式。サーバ側の処理は無い
  (API は Fly.io 上の Rust/Axum バックエンド `https://shoken-backend.fly.dev` が担う)。
- 配信は Vercel。本番 URL は `https://shoken-webapp.vercel.app`
  (CORS 許可 origin の既定値は `backend/src/config.rs:41`、本番値は Fly secrets の `FRONTEND_URL`。`docs/runbook.md:80` 参照)。
- `frontend/vercel.json` が配信の振る舞いを決めている:
  - rewrite `/(.*)` → `/index.html` (SPA フォールバック)。
  - 全体に CSP・`Cache-Control: public, max-age=0, must-revalidate`・`nosniff`・`DENY`・
    `Referrer-Policy`・`Permissions-Policy` を付与。
  - `/:name.wasm` に `Content-Type: application/wasm` と immutable 1 年キャッシュ、
    `/:name.js`・`/:name.css` に immutable 1 年キャッシュ。
- 本番 backend の URL は `vercel.json` の CSP `connect-src` が正本で、
  `frontend/scripts/backend-origin.cjs` 経由でのみ読み出す決まり。
  `prepare-vercel-dist.mjs` がビルド成果物に API origin の meta 埋め・init script 外部化・
  session-probe ハッシュ化を行う。
- CI は #1184 で一本化済み (`frontend.yml` → `deploy-frontend.yml` を `use_prebuilt_dist: true` で呼ぶ)。
  検証済みの `vercel-dist` artifact をそのまま配信し、二重ビルドしない。
  PR プレビューは `deploy-frontend.yml` の `pull_request` トリガーで独立ビルドし、
  投稿者が OWNER/MEMBER/COLLABORATOR の場合のみ、URL を PR コメントに投稿する。

## 2. 移す理由

### 費用

- Vercel Hobby は転送量 100 GB/月・CDN リクエスト 100 万回/月が上限で、超過分の購入はできない
  (Hobby は上限キャップ、追加購入不可)。
  出典: https://vercel.com/pricing (確認日 2026-10-08。
  Fast Data Transfer「Hobby: 100 GB / month included」、
  CDN Requests「Hobby: 1M / month included」)。
- Vercel Hobby は個人の非商用が対象
  (同ページ FAQ: "Our Hobby plan is for personal, non-commercial use")。
  公開サービスとしての利用と相性が悪い。
- Cloudflare Pages は静的アセットへのリクエストが無料・無制限で、転送量 (egress/帯域) の課金が無い。
  出典: https://developers.cloudflare.com/pages/functions/pricing/ (確認日 2026-10-08。
  "On both free and paid plans, requests to static assets are free and unlimited")、
  https://developers.cloudflare.com/workers/platform/pricing/ (確認日 2026-10-08。
  "There are no additional charges for data transfer (egress) or throughput (bandwidth).
  Requests to static assets are free and unlimited")、
  https://www.cloudflare.com/products/pages (確認日 2026-10-08。
  "All plans come with unlimited sites, seats, requests, and bandwidth")。
- 結論: 静的しか配信しない本フロントは、帯域上限による停止リスクが消える分だけ Cloudflare が安い。
  速度の優劣は実測していないので主張しない (どちらも世界分散 CDN の静的配信で、体感差は誤差の範囲と見込む)。

## 3. Cloudflare 無料枠・制限 (公式確認、確認日 2026-10-08)

| 項目 | 無料枠 | 出典 |
|---|---|---|
| 静的アセットのリクエスト数・帯域 | 無制限・課金なし | 上記 pricing 2 件と製品ページ |
| ビルド回数 | 月 500 回・同時 1 本。20 分でタイムアウト | https://developers.cloudflare.com/pages/platform/limits/ |
| ファイル数 | 1 サイト 2 万ファイル | 同上 |
| 単体ファイル | 25 MiB | 同上 |
| カスタムドメイン | 1 プロジェクト 100 件 | 同上 |
| `_headers` | ルール 100 件、1 行 2000 文字まで | 同上 + https://developers.cloudflare.com/pages/configuration/headers/ |
| `_redirects` | 静的 2000 + 動的 100 件 | 同上 + https://developers.cloudflare.com/pages/configuration/redirects/ |
| プレビューデプロイ | 同時数に上限なし | 同上 (limits ページ) |
| プロジェクト数 | アカウント 100 件 | 同上 |

注意: Pages ドキュメントのトップは「新規は Workers を使うこと」を推奨している
(https://developers.cloudflare.com/pages/。「Start new projects with Workers」)。
`_headers`・`_redirects` は Workers の静的アセットでもそのまま使える
(https://developers.cloudflare.com/workers/static-assets/migration-guides/migrate-from-pages/)。
将来 Pages が整理されても移行先は確保されている。

## 4. 移し先の構成

方式は Direct Upload (`wrangler pages deploy` で CI 成果物を上げる。Git 連携は使わない)。
手順の正本は https://developers.cloudflare.com/pages/how-to/use-direct-upload-with-continuous-integration/
(確認日 2026-10-08)。

### 4.1 Issue の「確認すること」への結論

- ヘッダー (CSP・キャッシュ・wasm の MIME): `_headers` で再現する。
  現 CSP は 1 行 272 文字で、2000 文字制限に収まる。
  `_headers` の指定は Pages の既定ヘッダーを上書きする
  (headers ページ: "Headers defined in the `_headers` file override what Cloudflare ordinarily sends")。
  wasm の `Content-Type: application/wasm` が拡張子から自動付与されるかは
  ドキュメントで断定できなかったため、実装 PR で `curl -I` と配信 E2E で確認する。
  未検証の懸念: Pages は既定で `Access-Control-Allow-Origin: *` を付ける
  (https://developers.cloudflare.com/pages/configuration/serving-pages/。Vercel は付けていない)。
  現状と同等に保つため `_headers` の Detach 構文 (`! Access-Control-Allow-Origin`) で外す方針とし、
  効くかどうかも実装時に `curl -I` で確認する。
- リライト (SPA フォールバック): `_redirects` に `/* /index.html 200` と明示する。
  200 プロキシは相対 URL のみ対応だが `/index.html` は相対なので適合する
  (redirects ページの Proxying 節)。
  なおトップレベルの `404.html` が無ければ Pages が自動で SPA 扱いする
  (serving-pages ページ) が、自動挙動への依存を避けるため明示する。`404.html` は置かない。
- PR プレビュー: `wrangler pages deploy <dist> --branch=<ブランチ名>` で出す。
  ブランチごとにハッシュ URL とブランチ名エイリアス (`<branch>.<project>.pages.dev`) が作られ、
  プレビューは既定で `X-Robots-Tag: noindex` が付く
  (https://developers.cloudflare.com/pages/configuration/preview-deployments/)。
  Vercel 同様、デプロイ後に PR コメントへ URL を投稿する。
  Vercel の「投稿者を OWNER/MEMBER/COLLABORATOR に絞る」条件はそのまま残す。
- CI からのデプロイ (#1184 の成果物): `vercel-dist` artifact 相当 (名前は変える) を
  `wrangler pages deploy` にそのまま渡す。二重ビルドしない方針は変えない。
  `_headers`・`_redirects` は `prepare-vercel-dist.mjs` 相当の処理で dist 直下に生成する
  (Pages は dist 直下の `_headers`・`_redirects` を読む)。
  現在 `backend-origin.cjs` が `vercel.json` 読み取りを前提にしているため、
  読み取り元を新設定ファイル (名称は実装 PR で決める) に切り替える。
  CSP の `connect-src` が backend origin の正本である決まりは維持する。

  → 実装結果: artifact は `cloudflare-dist`、生成スクリプトは
  `prepare-dist.mjs`、配信設定の正本は `frontend/_headers`・`frontend/_redirects`
  (`backend-origin.cjs` は `_headers` を読む) に決着した。
- 独自ドメイン/URL の変更と backend への影響: 今回カスタムドメインは付けない
  (URL は `*.pages.dev` に変わるだけ。将来付ける場合は CNAME で `pages.dev` に向ける。
  Pages は Cloudflare ゾーン外のドメインにも対応する)。
  backend への影響は次の 3 点で、いずれも frontend ドメイン変更に伴う設定変更であり、
  動作方式の変更は無い:
  1. CORS 許可 origin: `Config::cors_origins` の既定値と Fly secrets の `CORS_ORIGINS` の
     `https://shoken-webapp.vercel.app` を新 URL に置き換える。
     `is_vercel_preview_origin` (`backend/src/config/cors.rs:53`) は Vercel 専用のため、
     Pages プレビュー用 (`<hash>.<project>.pages.dev` とブランチエイリアス) の述語に置き換える。
     `validate_origin` (`backend/src/middleware/security.rs:67`) は同じ述語を使うため連動する。
  2. `FRONTEND_URL` (OAuth コールバック後のリダイレクト先) を新 URL にする。
  3. CSP 自体は不変 (frontend 自身は `'self'`、backend origin は `fly.dev` のまま)。
     Cookie は Domain 属性なし・本番 `SameSite=None; Secure` のクロスサイト前提
     (`backend/src/handlers/v1/auth/oauth.rs:22`) のため、frontend ドメイン変更による動作変化は無い。
     Google の承認済みリダイレクト URI は backend コールバックのため変更不要。

### 4.2 選ばなかった案と理由

- Git 連携 (Pages が GitHub から自動ビルド): #1184 で CI 成果物の使い回しに一本化した経緯と逆行する
  (二重ビルドが復活する)。また Direct Upload プロジェクトは後から Git 連携に切り替えられず
  作り直しになる (direct-upload ページ: "you cannot switch to Git integration later")。
  最初から Direct Upload で作ればこの制約は踏まない。
- Workers の静的アセット (新規推奨の方式): 本フロントは Functions が不要で、
  Pages の方が `_headers`・`_redirects` による `vercel.json` 再現の文書化が直接対応する。
  移行差分が最小の Pages を選び、将来必要になれば移行ガイドに沿って Workers へ移す。
- カスタムドメインの同時導入: Issue の範囲外。URL 変更の影響確認に絞る。

## 5. 移行の手順 (旧環境を残したまま切り替え、問題なければ止める)

1. Cloudflare に Direct Upload プロジェクトを作り、`*.pages.dev` を取得する (ダッシュボード操作)。
2. CI に Pages デプロイワークフローを足す (Vercel 並行)。最初はプレビューのみに出す。
3. backend の `CORS_ORIGINS` に Pages 本番 origin を追加 (旧は残す)、
   Pages プレビューで表示と未ログイン時の API 疎通を確認する。
4. Pages 本番ブランチへデプロイし、本番 URL で表示とログインを確認する。
5. `FRONTEND_URL` を Pages 本番 URL に切り替える (Fly secrets + backend 再デプロイ)。
6. 本番配信を Pages で確認後、Vercel 本番を止める (削除はしない。
   `LEPTOS_PRODUCTION_ENABLED` を落とすかダッシュボードで停止し、切り戻しに備える)。
   `LEPTOS_PRODUCTION_ENABLED` を落とすのは以降のデプロイ先をプレビューに変えるだけで、
   既存の本番デプロイは配信され続ける。旧 URL の利用者は古いビルドを使い続け、
   CORS にも旧 origin が残るため手順 7 まで動き続ける。
7. 1〜2 週間問題がなければ Vercel プロジェクトを削除し、関連 secrets
   (`VERCEL_TOKEN`・`VERCEL_ORG_ID`・`VERCEL_PROJECT_ID`) を整理する。
   backend 既定 origin の `vercel.app` と `is_vercel_preview_origin` を撤去する。

   → 実施済み (#1231)。Vercel プロジェクト・`deploy-frontend.yml`・`vercel.json`・
   関連スクリプト・`is_vercel_preview_origin`・既定 origin の `vercel.app` を撤去した。

切り戻し手順: `FRONTEND_URL` と `CORS_ORIGINS` を旧に戻して backend を再デプロイし、
`LEPTOS_PRODUCTION_ENABLED` を `true` に戻して Vercel に再デプロイする
(Vercel プロジェクトは手順 7 まで残すことが前提)。

  → 手順 7 実施後は Vercel プロジェクトが無いため、この切り戻しは使えない。
  復旧が必要な場合は Vercel プロジェクト再作成からになる。

## 6. 無料枠に収まる根拠

- 帯域・リクエスト数: 無制限のため上限を気にする必要が無い (3 節)。
- ビルド月 500 回: 1 日 16 回ペースが上限。通常の開発ペース (1 日数回の push・PR) では
  2 桁/月に収まる見込み。超過時はデプロイ失敗として検知できる。
- ファイル数 2 万・単体 25 MiB: 本フロントの dist は `index.html`・wasm・JS・CSS・画像など数十ファイル、
  合計数 MB 規模の見込み。wasm 単体も 25 MiB を大きく下回る見込み。
  正確な値は CI の asset size レポート (frontend.yml の Report compressed asset sizes) で出ているため、
  実装 PR で直近の実測値を転記して裏付ける (本メモ執筆時点では手元に dist が無く、断定的な数字は書かない)。

## 7. 必要な Cloudflare API トークンの権限 (実値は書かない)

公式の CI 連携手順 (4 節の出典) のとおり:

- Permissions: Account / Cloudflare Pages / Edit のカスタムトークン。
- あわせて Account ID が必要。
- GitHub 側の secrets 名は `CLOUDFLARE_API_TOKEN`・`CLOUDFLARE_ACCOUNT_ID` とする (公式手順と同名)。
- 足りなければ実装 Issue に必要な権限を追記し、ユーザーに発行を依頼する。トークンの実値は扱わない。

## 8. 実装の分け方 (PR の分け方)

1. `_headers`・`_redirects` 生成 (prepare スクリプトの拡張と設定ファイル切替え) + 配信 E2E の拡充。
   受け入れ: `serve-dist.mjs` 相当のローカル配信でヘッダー・SPA フォールバックを確認。
2. backend の CORS・preview 述語の Pages 対応。Vercel の origin と述語は残す。
   受け入れ: 新旧両 origin で preflight が通ることの単体テスト。
3. CI の Pages デプロイ (プレビュー+本番、PR コメント投稿)。Vercel 経路は残す。
   受け入れ: PR で Pages プレビュー URL が投稿され、表示・ログイン一巡が通る。
4. 本番切替 + Vercel 無効化 + docs 更新 (`runbook.md`・`architecture.md`・deploy skill)。
   切り戻し手順の記録を含む。Vercel 削除は別 PR にし、様子見期間を空ける
   (→ #1231 で撤去済み)。

各 PR は Issue #1209 に `Refs` で紐づけ、本番切替の完了をもって Issue を閉じる。

## 9. 本番切替時の実測 (2026-10-09)

`https://shoken-webapp.pages.dev` に `curl -I` で確認した結果:

- `Access-Control-Allow-Origin: *` は Pages だけでなく Vercel 側のレスポンスにも付いていた
  (4.1 の「Vercel は付けていない」は誤り)。両者で同じ挙動のため Detach は不要と判断した。
- `_headers` では、複数ルールに一致したリクエストは全ルールのヘッダーを継承し、
  同名ヘッダーはカンマ連結される (Vercel の後勝ち上書きとは違う)。`/*` の
  `Cache-Control: max-age=0` とアセットルールの immutable が
  `public, max-age=0, must-revalidate, public, max-age=31536000, immutable` に連結される
  ことを確認した。`! <name>` の Detach は同一ルール内で外してから付け直せる
  (workers-sdk#1979) ので、個別ルールで `! Cache-Control` → `Cache-Control: ... immutable`
  とする形に修正した。
- `/*.js` のような splat パターンは `/` をまたぐため `/snippets/**/*.js` にも immutable が
  効いてしまう (snippets のファイル名は内容ハッシュではない)。`/:name.js`
  プレースホルダ (パス内で単一セグメントのみ一致) にすれば Vercel の `/:name.js` と
  同じ適用範囲になるため、変換側で splat ではなくプレースホルダを使う形に直した。
- wasm の `Content-Type: application/wasm` は `/:name.wasm` ルールどおりに付いた。
- `CORS_ORIGINS` は Fly secrets 未設定で `Config::default()` の既定値
  (`vercel.app`・`pages.dev` 両方) が使われていた。切替時に
  `vercel.app,pages.dev` を明示設定し、`FRONTEND_URL` を `pages.dev` に切り替えた。
