# セキュリティポリシー

## 脆弱性の報告

セキュリティに関わる問題を発見した場合は、**公開 Issue ではなく**、以下の手順で報告してください。

### 報告先

GitHub の [Security Advisories](https://github.com/zaichu/shoken-webapp/security/advisories/new)（非公開の脆弱性報告）からプライベートレポートを作成してください。

報告内容には以下を含めてください：

- 問題の概要と影響範囲
- 再現手順（できるだけ具体的に）
- 想定される攻撃シナリオ
- 発見バージョン・環境情報

### 対応方針と SLA

| ステップ | 目安 |
|---|---|
| 受領確認 | 3 営業日以内 |
| 初期評価・重大度判定 | 7 日以内 |
| 修正完了 | 重大度 Critical/High: 30 日以内、Medium 以下: 90 日以内 |
| 公開 | 修正リリース後に GitHub Advisory を公開 |

### 対象範囲

- バックエンド API（Rust / Axum、Fly.io で運用）
- フロントエンド（Leptos / Rust・WebAssembly、Vercel で配信）
- 認証・セッション管理（Google OAuth 2.0、Cookie セッション）
- CSV インポート処理

### 対象外

- 開発環境限定の設定ミス
- 第三者サービス（Fly.io・Vercel・Neon・J-Quants API）側の脆弱性

## サポートバージョン

現在は `main` ブランチの最新リリースのみサポートします。

## 現在の構成とセキュリティ方針

### 構成

- フロントエンド: Leptos（CSR）/ WebAssembly。Vercel で配信
- バックエンド: Rust / Axum。Fly.io で運用。DB は PostgreSQL（Neon）
- 認証: Google OAuth 2.0 + セッション Cookie
- API 契約の正本は `docs/openapi.json`

### 認証・セッション

- OAuth の `state` は HttpOnly Cookie（10 分）に保存し、コールバックで照合する
- Google の ID トークンは JWKS による署名検証（RS256）と iss・aud・exp の検証を行う
- ログイン後のリダイレクト先は設定値 `FRONTEND_URL` に固定し、オープンリダイレクトを防ぐ
- セッション Cookie は `HttpOnly`・`Path=/`・`Max-Age=7日` で、DB 側の `expires_at` でも期限を管理する
- 本番では `Secure` + `SameSite=None`（Vercel と Fly.io のクロスサイト構成で必要）。ローカル開発では `SameSite=Lax`
- 本番判定は fail-safe: `RUST_ENV` / `APP_ENV` に設定された値が**すべて**開発用の値（`local` / `dev` / `development` / `test`）のときだけ非本番とする。未設定・不明値・本番値との混在はすべて本番扱い。本番では `SECURE_COOKIE` の値に関わらず Cookie は `Secure`
- 状態を変える GET リクエストは置かない

### CSRF / CORS

- POST / PUT / DELETE は `Origin` ヘッダーを許可一覧と完全一致で照合する。`Origin` が無い場合は `Referer` のオリジン部分を完全一致で照合する
- 本番相当の環境では `Origin` と `Referer` の両方が無い変更系リクエストを拒否する
- CORS は許可一覧との完全一致でのみ credentials を許可し、本番では localhost オリジンを除外する
- デフォルトの許可一覧は `https://shoken-webapp.vercel.app` とローカル開発用のみ

### レート制限・リソース上限

- クライアント IP は `fly-client-ip` のみを信用し、`X-Forwarded-For` は使わない
- IP 単位のレート制限（既定値・環境変数で変更可）:
  - 認証系ルート: 10 req/s（`AUTH_RATE_LIMIT_RPS`）
  - 銘柄検索: 10 req/s（`STOCK_SEARCH_RATE_LIMIT_RPS`）
  - CSV 取り込み: 2 req/s（`CSV_RATE_LIMIT_RPS`）
  - 認証済みデータ系ルート: 10 req/s（`DATA_RATE_LIMIT_RPS`）
- リクエストボディ上限は 10MB（multipart アップロードは axum 既定の 2MB）
- ユーザーごとの保存行数に上限（既定 100,000 行/ドメイン、`USER_ROW_LIMIT`）
- ページネーションの `per_page` は 1〜1000 に制限

### セキュリティヘッダー

- バックエンドの全レスポンスに `X-Content-Type-Options: nosniff`、`X-Frame-Options: DENY`、`Referrer-Policy: strict-origin-when-cross-origin`、`Content-Security-Policy: default-src 'none'` を付与し、本番では HSTS を追加する
- フロントエンド（Vercel）の CSP: `default-src 'self'`、`script-src 'self' 'wasm-unsafe-eval'`（`unsafe-inline` / `unsafe-eval` なし）、`style-src 'self'`、`connect-src 'self' <backend オリジン>`、`object-src 'none'`、`base-uri 'self'`、`form-action 'self'`、`frame-ancestors 'none'`
- フロントエンドは localStorage に認証情報を保存しない

### ログ・エラー応答

- リクエストログはパスのみ記録し、クエリパラメータ（OAuth の `code` / `state` 等）は残さない
- 本番のエラー応答には DB の詳細を含めない
- Sentry は `send_default_pii = false` のまま運用する

### 機密情報・CI

- 秘密情報はローカルの `.env`（git 管理外）と Fly.io Secrets / GitHub Secrets で管理し、リポジトリにコミットしない
- GitHub Actions の `uses:` は 40 桁のコミット SHA で固定する
- `pull_request_target` は PR gate と Dependabot 自動マージの 2 ワークフローに限定し、どちらも PR のコードを checkout しない

## 受け入れたリスク

### エージェントの権限（Issue #1011 M3、判断日: 2026-09-26）

開発に使う AI エージェントの権限を強く絞っていない。範囲:

- Devin の実行環境の許可が広い
- Codex を `danger-full-access` で実行している
- 統合側の `gh` は管理者権限を持ち、必須チェックを通さないマージ（`gh pr merge --admin`）が可能

公開リポジトリでは誰でも Issue・PR にコメントできるため、エージェントが読む文章を介したプロンプトインジェクションでこれらの権限が悪用され得る。このリスクは認識した上で、運用効率を優先してユーザーが受け入れた（2026-09-26）。

## 謝辞

報告していただいた方には、修正公開後に Advisory の謝辞に名前を掲載します（ご希望の場合）。
