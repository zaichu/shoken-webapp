# バックエンドを Fly.io から Cloudflare Workers へ移す設計メモ

対象 Issue: #1210。実装は含まない。結論と根拠だけを書く。

## 1. 今の構成

- Rust/Axum 0.8.9 の単一バイナリ (`backend/`)。`tokio::main` + `TcpListener` + `axum::serve` で起動
  (`backend/src/main.rs`)。Fly.io `shoken-backend` で nrt リージョンに shared-cpu-1x / 256MB を
  常時 1 台実行 (`min_machines_running = 1`。cold start proxy error 回避のため、`fly.toml:22`)。
- DB は Neon PostgreSQL。`sqlx` 0.9 + `PgPool` (lazy connect + 起動時 retry + 起動時 migration。
  `backend/src/db.rs`)。クエリは `sqlx::query!` マクロでコンパイル時検証し、`.sqlx` オフライン
  キャッシュを CI の Docker ビルドに使っている。
- 認証は Google OAuth2 (`oauth2` + `openidconnect` クレート)。authorize で PKCE / state / nonce を
  短命 Cookie に入れ、callback でトークン交換 + `id_token` 検証 + セッション発行
  (`backend/src/handlers/v1/auth/oauth.rs`)。セッションは DB の `sessions` テーブル +
  `session_token` Cookie (HttpOnly, Max-Age=7日, 本番 `SameSite=None; Secure`)。
- レート制限は `governor` のインメモリ keyed limiter (auth / csv / stock_search / data の 4 系)。
  クライアント IP は `fly-client-ip` ヘッダーのみ信頼 (`backend/src/middleware/rate_limit.rs`)。
- J-Quants は `reqwest` で `api.jquants.com/v2/fins/summary` を `x-api-key` 付きで呼ぶ。
  結果は `dividend_per_share_cache` にキャッシュし、未取得/TTL切れ銘柄は `tokio::spawn` の
  バックグラウンド更新で埋める。外部 API の呼び出し間隔は `market_data_provider_rate_control`
  テーブルの UPSERT で全インスタンス共有の 12 秒間隔を保証する
  (`backend/src/services/dividend_cache*.rs`)。
- CSV インポートは 2 段階 (validate → import)。`RequestBodyLimit` 10MB、
  Shift_JIS フォールバック (`encoding_rs`)、バルク INSERT は `UNNEST` 系。
- ミドルウェアは tower-http 中心 (CORS / Origin検証 / RequestBodyLimit / request-id /
  CompressionLayer / security headers / `startup_ready` 503 ゲート)。`utoipa` で
  `/api-docs/openapi.json` を生成し、`docs/openapi.json` が API 契約の正本。
- エンドポイント数は 25 (openapi.json の operationId 数)。観測は `tracing` + sentry。
- デプロイは `main` push → `deploy-backend.yml` → Docker build → `flyctl deploy`
  (`FLY_API_TOKEN` を GitHub secrets に保持)。シークレットは Fly.io Secrets。

## 2. 移す理由

### 費用

- Fly.io の always-on shared-cpu-1x/256MB は最安リージョンで $2.19/月。本アプリは
  `primary_region = 'nrt'` で、リージョン係数 1.3077 を掛けると **約 $2.86/月**。
  加えて APAC の egress は $0.04/GB。
  出典: https://fly.io/docs/about/pricing (確認日 2026-10-08。
  Machines 表「shared-cpu-1x 256MB $2.19/month」・リージョン係数 nrt=1.307692308・
  Outbound data「$0.04/GB in Asia Pacific」)。
- Workers Free ならこの常時起動費用が 0 になる (3節)。Neon は現行どおり Free のまま据え置く
  (https://neon.com/pricing 確認日 2026-10-08: Free は 1GB/プロジェクト・100 CU-hours・
  5 分で scale-to-zero)。
- 差額は小さい (約 $3/月 + egress) が、常時起動を維持する理由自体が cold start 回避であり、
  Workers に移すとその構造的な課金理由が消える。

### 速さ

- Fly の machine 起動は秒単位で、`auto_stop_machines='stop'` と併用すると初回リクエストで
  proxy error が出るため `min_machines_running = 1` を選んでいる (`fly.toml:22` のコメント)。
  #1114 の計測でも本番 `/api/v1/session` は 401 経路で 37-45ms、Neon 起床中はさらに遅い。
- Workers は V8 isolate モデルで「This model eliminates the cold starts of the virtual
  machine model」(起動がほぼ一瞬) と公式が明記している。
  出典: https://developers.cloudflare.com/workers/reference/how-workers-works/
  (確認日 2026-10-08)。
- #1114 で課題だった起動時のセッション確認も、isolate 起動コストがほぼ無いぶん速くなる見込み。
  速度の優劣は未実測なので断言せず、実装 PR で計測値を記録する。

## 3. Cloudflare 無料枠・制限 (公式確認、確認日 2026-10-08)

出典は断りがない限り https://developers.cloudflare.com/workers/platform/pricing/ と
https://developers.cloudflare.com/workers/platform/limits/ (ともに確認日 2026-10-08)。

| 項目 | 無料枠 | 備考・出典 |
|---|---|---|
| Workers リクエスト | 100,000/日 | 超過時は Error 1027。UTC 0 時リセット |
| CPU 時間 | 10ms/呼び出し | 超過すると Error 1102。公式は「認証や大きなペイロードの parse は 10-20ms 使い得る」と明記 |
| メモリ | 128MB/isolate | wasm 確保を含む |
| Duration (wall clock) | HTTP リクエストは無制限 | `waitUntil()` はレスポンス後 +30 秒まで延長 |
| サブリクエスト | 50/呼び出し | fetch・KV・R2・D1 等への呼び出しを数える |
| 同時アウトバウンド接続 | 6/呼び出し | 応答ヘッダー待ちの接続を数える |
| Worker サイズ | 64 MiB (非圧縮) | 圧縮サイズの制限なし。`wrangler deploy --dry-run` で計測可 |
| Worker 起動時間 | 1 秒 | グローバルスコープの実行時間 |
| 環境変数・シークレット | 64 個/Worker・各 5KB | |
| Cron Triggers | 5 個/account | 実行あたり wall 15 分・CPU 10ms (free) |
| Workers Logs | 200,000 events/日・3日保持 | |
| Hyperdrive クエリ | 100,000/日 | https://developers.cloudflare.com/workers/platform/pricing/ (Hyperdrive 節)。接続 ~20/設定・クエリ 60 秒上限は https://developers.cloudflare.com/hyperdrive/platform/limits/ |
| Rate Limiting バインディング | `simple` (limit, period=10 or 60 秒) | https://developers.cloudflare.com/workers/runtime-apis/bindings/rate-limit/。ロケーション単位・eventually consistent |
| D1 (不採用・参考) | read 5M 行/日・write 100K 行/日・合計 5GB | DB あたり 500MB・DB 数 10・クエリ 50/呼び出し: https://developers.cloudflare.com/d1/platform/limits/ |
| KV (未使用) | read 100K/日・write 1K/日・1GB | 上記 pricing |
| リクエストボディ | 100MB (Cloudflare Free ゾーン) | 現行の RequestBodyLimit 10MB は内側 |

## 4. 移し先の構成

`workers-rs` (`worker` 0.8.7) で Rust を wasm32 にビルドし、Axum ルーターと
tokio-postgres + Hyperdrive で Neon を使い続ける構成:

```
Browser → frontend (Pages) → https://<worker>.workers.dev
                                      ├── #[event(fetch)] → axum Router (現行 routes を流用)
                                      │      ├── tower 系ミドルウェア (CORS/Origin/body limit/security headers)
                                      │      ├── ratelimits バインディング ×4 (cf-connecting-ip)
                                      │      └── services → tokio_postgres → worker::Socket → Hyperdrive → Neon
                                      └── #[event(scheduled)] (毎分) → dividend cache の消化
```

ビルドは `worker-build`、デプロイは `wrangler deploy`、設定は `wrangler.toml`。

### 4.1 Issue の「設計で決めること」への結論

- **Rust/Axum を workers-rs で動かす形**: 実現する。`worker` クレートに `axum` feature
  (dep: axum ^0.8。現行 0.8.9 と同一系列) があり、公式 examples/axum では
  `#[event(fetch)]` が `HttpRequest` を受けて `router(env).call(req)` で
  `axum::http::Response<Body>` を返す形が示されている。Router・middleware・extractor・
  handler はほぼそのまま移植できる。
  出典: https://docs.rs/crate/worker/latest/features ・
  https://github.com/cloudflare/workers-rs/tree/main/examples/axum (確認日 2026-10-08)。
  変わるのはエントリポイント (`tokio::main` + TcpListener → `crate-type = "cdylib"` +
  `#[event(fetch)]`) と起動時処理 (isolate には「起動」が無く、`startup_ready` ゲートは不要化。
  `/health`・`/ready` は契約維持で常時 200 を返すだけにする)。
- **DB**: **Neon + Hyperdrive + tokio-postgres を採用し、D1 は採用しない**。
  - `worker` クレートの `tokio-postgres` feature と `Hyperdrive` バインディングで、
    `env.hyperdrive().connect() → worker::Socket` (tokio の AsyncRead/AsyncWrite と
    `TlsStream`/`TlsConnect` を実装済み、`PassthroughTls` で直通) から
    `config.connect_raw(socket, PassthroughTls)` で接続する公式サンプルがある。
    出典: https://docs.rs/worker/latest/worker/struct.Socket.html ・
    https://github.com/cloudflare/workers-rs/blob/main/examples/tokio-postgres/src/lib.rs
    (確認日 2026-10-08)。
  - sqlx → tokio-postgres: `$1..$n` プレースホルダ・`ANY($1)` 配列・`INTERVAL`・
    `RETURNING`・`NUMERIC`/`chrono`/`uuid`/`rust_decimal` 型 (postgres-types の with-* 機能)
    はそのまま使える。**失うものは `sqlx::query!` マクロのコンパイル時クエリ検証だけ**で、
    全クエリを文字列の `client.query/query_one/execute` 系へ機械的に置き換える。
    既存の testcontainers 実 DB 統合テスト (`backend/tests/db_integration.rs`) が実クエリを
    実行するため、同じテストを tokio-postgres 経路で維持してカバーする。tokio-postgres は
    ネイティブでも動くので、移植直後も Fly 側のビルド・テストを壊さず進められる。
  - 接続形態: PgPool は持てないので、リクエストごとに `connect_raw` するか isolate 内
    `thread_local` で `tokio_postgres::Client` を使い回すかを実装で決める。
    Hyperdrive が上流 (Cloudflare 内) で Neon への接続をプールするため、都度接続でも
    handshake コストは小さい。
  - **Hyperdrive のクエリキャッシュは既定 ON (max-age 60 秒) で、書き込みに連動して
    無効化されない** (get-started: "Hyperdrive does not invalidate cached read results
    when your application writes")。セッション・ユーザー CRUD・import 直後の再読は
    read-after-write が必須なので、本アプリでは **`--caching-disabled` で作成する**。
    キャッシュを切ってもエッジ内プーリングの効果 (7 round trips → プール済み接続) は残る。
    出典: https://developers.cloudflare.com/hyperdrive/get-started/ (確認日 2026-10-08)。
    なお公式 tokio-postgres サンプルに「caching 無効時は `query()` の prepared statement が
    使えない」との注記があるため、simple query プロトコル (`simple_query` 系) を使う前提とし、
    実装 PR で挙動を確かめる。
  - マイグレーションとデータ移行: Neon を使い続けるので **データ移行は無い**。起動時
    migration を Worker のリクエストパスに置けないため、デプロイジョブで `sqlx migrate run`
    を実行する (`DATABASE_URL` は GitHub secret から Neon へ直接接続)。
    移行期間は Fly 側と同じ DB を見るため、スキーマ・データ・レート制御テーブルは
    常に共有される。
- **J-Quants 取得・キャッシュ・レート制限・Cookie 認証・Google OAuth の移し方**:
  - J-Quants 呼び出し: `reqwest` → `worker::Fetch`。外部 fetch はサブリクエスト
    (50/呼び出し) を消費するが、1 リクエストあたり数個で収まる。
  - 配当キャッシュ: `dividend_per_share_cache` テーブルと TTL/stale/status のロジックは
    そのまま (同じ Neon を見る)。DB レート制御 `market_data_provider_rate_control` の
    12 秒 UPSERT もそのまま機能し、並走期間は新旧両バックエンドで共有される。
  - バックグラウンド更新: `tokio::spawn` は無く、`ctx.wait_until` は +30 秒で 12 秒間隔の
    連続更新には足りない。→ **Cron Trigger (毎分) で stale/pending 銘柄を消化する
    `#[event(scheduled)]` ハンドラー**にする。12 秒間隔×1 回実行で最大 4 銘柄/分処理でき、
    15 分 wall 制限内。API から見た到達時間は現行とほぼ同じ (取得は I/O 待ち主体で cron の
    CPU 10ms も収まる)。cron は無料枠 5 個のうち 1 個。
  - IP レート制限: governor インメモリ → **`[[ratelimits]]` バインディング ×4**
    (auth / csv / stock_search / data)。キーは `cf-connecting-ip` (Cloudflare が上書きするため
    偽装不可。`fly-client-ip` と同じ役割)。公式は IP キーを推奨していない (共有 IP で他者を
    巻き込み得る) が、DoS 緩和が目的なので per-IP が要件そのもの。差分として、カウンタが
    ロケーション単位・eventually consistent になる点は明記する (現行も 1 マシン限定の
    インメモリで、保護強度は同等と見なす)。
  - Cookie 認証: `SameSite=None; Secure` のクロスサイト運用は変更なし (frontend と backend が
    別オリジンのまま)。`session_token` Cookie 属性と `secure_cookie` の環境判定はそのまま移植。
  - Google OAuth: `oauth2`/`openidconnect` クレートは `reqwest` 前提で wasm では動かないため、
    認可 URL 生成 + PKCE/state/nonce Cookie は純ロジックとして移植し、トークン交換と
    `id_token` 検証は `worker::Fetch` で実装する。`id_token` は Google 公式の tokeninfo
    エンドポイント (`https://oauth2.googleapis.com/tokeninfo?id_token=`) で検証する方式を採る
    — wasm32-unknown-unknown での JWT 検証クレート (jsonwebtoken + ring) の動作が未確認なため。
    トレードオフとしてログイン 1 回あたり +1 外部呼び出しになるが、ログイン頻度は低い。
    nonce・aud・exp は tokeninfo 応答値で自前検証する。
  - `openapi.json`: `utoipa` は純 Rust のため wasm 化できる。`/api-docs/openapi.json`
    エンドポイントと `bin/generate_openapi` (ネイティブ実行) を維持し、`check-openapi.sh`
    を同じ契約検証として残す。**API 契約 (docs/openapi.json) は変えない**。
- **主な依存差分**: `tokio` (full) → worker の wasm 実行環境 (`tokio::time::sleep` →
  `worker::Delay`、`tokio::spawn` → `wasm_bindgen_futures::spawn_local`)、`reqwest` →
  `worker::Fetch`、`tracing` → `console_log`/`worker` のログ (Workers Logs は 200K events/日)、
  sentry → Workers Logs に切り替え (sentry の wasm 対応は未確認のため継続しない)、
  `fly-client-ip` → `cf-connecting-ip`、DB シークレット → Hyperdrive に集約
  (Worker 環境変数に DB 認証情報を置かない)。

### 4.2 選ばなかった案と理由

- **D1 (SQLite)**: (a) 16 本の Postgres マイグレーションと全クエリを方言書き換えする必要がある
  (`pg_trgm` GIN → FTS5/LIKE で検索挙動が変わる、`citext` → `COLLATE NOCASE`、`ANY(配列)` →
  `IN` 展開、`INTERVAL`/`GREATEST`/`NOW()` → SQLite 式、`NUMERIC` → 精度差、UUID/timestamptz
  → TEXT)。(b) Neon→D1 のデータ移行が必要。(c) 無料枠で DB あたり 500MB・クエリ 50/呼び出し
  (bulk INSERT は 1 呼び出し内に収まるか要検証)。(d) 並走期間に 2 つの DB を同期する設計が
  必要になる (Hyperdrive+Neon なら同一 DB なのでこの問題自体が無い)。
  「移行の手間は考えなくてよい」とは別に、意味の変わる書き換えとデータ移行のリスクは残るため
  不採用。
- **Neon へ direct TCP (Hyperdrive なし)**: `worker::Socket` + `start_tls` で技術的には可能だが、
  リクエストごとに TCP+TLS+認証の約 7 round trips を払う。Hyperdrive のエッジ内プールは
  まさにこのコストを消すもので (get-started 記載)、Hyperdrive 無料枠 (10万クエリ/日) も
  十分なため不採用。
- **TypeScript/JavaScript で書き直し**: `node-pg`/`postgres.js` で Postgres も用意に扱えるが、
  Issue の前提である Rust/Axum 維持に反し、既存コードを捨てる理由がない。
- **Workers Paid ($5/月)**: 無料枠で成立する見込み (6節)。残るリスクは CSV の CPU 上限だけで、
  超過が確定した場合だけ検討する。

## 5. 移行の手順 (旧環境を残したまま切り替え、問題なければ止める)

前提: DB (Neon) は新旧で共有するため、データ移行・二重書き込み・同期は一切不要。
セッション Cookie はホスト依存のため、切替時にユーザーは一度再ログインする (許容と判断)。

1. Cloudflare に Hyperdrive 設定を作成 (`wrangler hyperdrive create --caching-disabled`。
   Neon 接続文字列はダッシュボード/`wrangler` 経由で投入し、実値は git に書かない)。
2. Worker 骨格を `*.workers.dev` にデプロイ (health/ready + openapi.json のみ)。
3. backend を Workers へ段階移植 (分割は 8節)。デプロイごとに wrangler で反映。
4. CI deploy ジョブに `sqlx migrate run` を組み込む (`DATABASE_URL` は GitHub secret、
   Neon へ直接接続)。
5. Google Cloud Console の承認済みリダイレクト URI に
   `https://<worker>.workers.dev/api/v1/oauth/google/callback` を追加
   (旧 `fly.dev` URI は残す)。Worker の secrets/vars を設定。
6. Worker URL で OAuth ログイン一巡 + CSV 一巡を検証。
7. frontend の backend-origin (CSP `connect-src` が正本) を `workers.dev` に切り替えて本番
   デプロイ。#1209 (Vercel→Pages) が先に入っていればその origin を更新する。
8. 動作確認 + 1〜2 週間様子見 → `fly scale count 0` で Fly machine を停止 (削除はしない)。
9. 問題なければ Fly アプリ削除、`fly.toml`/`deploy-backend.yml` の Fly 経路、`FLY_API_TOKEN`
   を撤去。

   → 実施済み (#1231)。Fly アプリ `shoken-backend` を削除し、`fly.toml`・
   Fly デプロイ経路・`backend/Dockerfile`・`fly-client-ip` 参照を撤去した
   (backend の CI 部分は `backend.yml` に移管)。`FLY_API_TOKEN` 等の GitHub
   secrets の削除はユーザーが行う。

切り戻し: 手順 8 までなら、frontend の backend-origin を `fly.dev` に戻して再デプロイする
(Fly machine は手順 8 まで生きている)。手順 9 後は Fly アプリ再作成 + `fly deploy` +
secrets 再投入が必要。

  → 手順 9 実施済みのため、Fly.io への切り戻し経路は存在しない。

## 6. 無料枠に収まる根拠

- **リクエスト 10万/日**: 実測値は本環境から取れない (`fly` CLI 未認証、postgres skill 未設定)。
  個人規模の証券管理アプリが常時 1 台の 256MB インスタンスで捌いている負荷なので、
  10万/日 (=約 70/分 持続) を大きく下回る想定。**実装 PR で fly dashboard のリクエスト
  メトリクスを転記して裏付ける** (この環境では確認不可)。
- **Hyperdrive クエリ 10万/日**: 1 リクエストあたりのクエリはセッション検証 1 + ドメイン
  1-3 + キャッシュ系で数本。リクエスト数と同程度以下の枠に収まる。
- **CPU 10ms/呼び出し**: 大半のエンドポイントは DB 往復中心で、I/O 待ちは CPU に数えない
  (limits ページ)。**CSV インポートが最大のリスク**: `docs/performance.md` の実測
  (ネイティブ debug ビルド) で 1000 行デコード 0.44ms・`parse_number` 0.23µs・
  `parse_date` 1.22µs。wasm 化で数倍悪化しても典型的な数百〜千行の証券 CSV なら数 ms
  見込みだが、`USER_ROW_LIMIT` (既定 100K 行) 規模や 10MB 上限では 10ms を超え得る。
  実装 PR で計測し、超過が確定した場合は (a) CPU 削減最適化、(b) Workers Paid $5/月、
  (c) 当該エンドポイントのみ Fly 残留、の順で判断する (この時点では未確定のまま残す)。
- **メモリ 128MB**: CSV 10MB + パース構造で十分内側。
- **Worker サイズ 64MiB・起動 1 秒**: Rust wasm バイナリは依存含め数十 MB 見込み。
  `wrangler deploy --dry-run` で実測して確認する (実装 PR)。
- **サブリクエスト 50/呼び出し**: J-Quants 1 + OAuth 時 token+tokeninfo 2 程度。
- **同時接続 6/呼び出し**: tokio-postgres 1 + fetch 数個で収まる。
- **環境変数 64 個**: secrets/vars 合わせて 10 前後の見込み。
- **Cron 5 個**: 1 個のみ使用。
- **Neon (変更なし)**: Free の 1GB/プロジェクト・100 CU-hours・5 分 scale-to-zero は現行と同一。
- **Logs 200K events/日**: リクエスト数から見て十分。

## 7. 必要な Cloudflare API トークンの権限 (実値は書かない)

- Account / **Workers Scripts: Edit** — `wrangler deploy`、`wrangler secret put`、
  cron/ratelimits/hyperdrive バインディング設定 (スクリプトの一部) 用。
- Account / **Hyperdrive: Edit** — `wrangler hyperdrive create` 用
  (ダッシュボードで 1 回作る運用なら CI トークンには不要)。
- User / **User Details: Read** — wrangler のアカウント確認用 (無いと警告が出る)。
- 変数名は `CLOUDFLARE_API_TOKEN`・`CLOUDFLARE_ACCOUNT_ID` とし、GitHub secrets に保持
  (#1209 と同じ命名)。Zone 系の権限は不要 (`workers.dev` のみ使用、カスタムドメイン無し)。
- 出典: https://developers.cloudflare.com/fundamentals/api/reference/permissions/
  (確認日 2026-10-08)。
- 足りなければ実装 Issue に必要な権限を追記し、ユーザーに発行を依頼する。
  トークンの実値は扱わない。

## 8. 実装の分け方 (PR の分け方)

1. **Worker 骨格**: `worker` クレート導入、`crate-type = "cdylib"` 化、`#[event(fetch)]` で
   axum router を呼ぶ雛形、`wrangler.toml`、health/ready + openapi.json のみ動作、
   CI で `worker-build` ビルド。
   受け入れ: `wrangler dev` で `/health` が応答し、`wrangler deploy --dry-run` で
   バンドルサイズと `startup_time_ms` を記録する。
2. **DB 層**: sqlx → tokio-postgres 全置換 + Hyperdrive 接続ヘルパー + isolate 内
   client 再利用方針。既存 DB 統合テストを tokio-postgres で維持。
   受け入れ: `cargo test` 全緑、`wrangler dev` で実クエリが動く。
3. **認証・ミドルウェア**: OAuth (fetch 化 + tokeninfo 検証)・セッション Cookie・
   CSRF/CORS/security headers・`ratelimits` バインディング化・`cf-connecting-ip`。
   受け入れ: ログイン一巡 + 429/403 の挙動テスト。
4. **J-Quants・配当キャッシュ・cron**: `worker::Fetch` 化、`#[event(scheduled)]` で
   stale 消化、レート制御テーブル共用。
   受け入れ: `wrangler dev` の `/cdn-cgi/local/scheduled` で消化が動く。
5. **CSV 系**: multipart + Shift_JIS + bulk insert の wasm 動作確認と CPU 実測
   (10ms 判定の結論を Issue に記録)。
6. **デプロイ CI + 切替**: deploy ワークフロー、migration ジョブ、frontend origin 切替、
   `runbook.md`/`architecture.md`/deploy skill 更新、切り戻し手順書。
7. **Fly 撤去** (様子見後・別 PR): `fly.toml`/`FLY_API_TOKEN`/deploy 経路削除
   (→ #1231 で実施)。

各 PR は Issue #1210 に `Refs` で紐づけ、本番切替完了をもって Issue を閉じる。

## 未検証事項 (実装で確かめる)

- wasm バンドルサイズと起動時間 1 秒 (`wrangler deploy --dry-run` で実測)
- CSV インポートの CPU 10ms 到達可否
- Hyperdrive caching 無効時の prepared/simple query 挙動
- `tokeninfo` 応答形式と nonce/aud/exp 検証項目
- `CompressionLayer` の要否 (Cloudflare edge が既に圧縮する可能性)
- sentry 代替 (Workers Logs で足りるか)
- `uuid`/`getrandom` の wasm32-unknown-unknown 対応 (js feature)
- `openidconnect` を fetch ベース HTTP クライアントで wasm 化する選択肢 (採用しないが一応)
- Fly の実リクエスト量/DB 実測値 (fly dashboard メトリクス転記)
