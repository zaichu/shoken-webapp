# 用語集

画面の言葉は `frontend/src/ui/labels.rs`、通信する型は `shared/src/`、保存する列は `backend/migrations/` を根拠にする。
この表はコードを読むための意味を説明し、投資判断や税制の解説は扱わない。

## 画面とデータの言葉

| 用語 | このアプリでの意味 | 根拠 |
| --- | --- | --- |
| 保有銘柄・保有残高 | 現在保有している数量・取得額など。国内株式と投資信託は別の表で保持する | [shared/src/domain.rs:180](../../shared/src/domain.rs#L180)、[同:264](../../shared/src/domain.rs#L264) |
| 取引明細 | CSVから取り込む配当・国内株式取引・投資信託取引の明細。現在残高とは別に保持する | [backend/migrations/0003_dividends.sql:3](../../backend/migrations/0003_dividends.sql#L3)、[0004_domestic_stocks.sql:3](../../backend/migrations/0004_domestic_stocks.sql#L3)、[0005_mutualfunds.sql:3](../../backend/migrations/0005_mutualfunds.sql#L3) |
| 約定日・受渡日 | 取引が成立した日と受渡の日。列はそれぞれ `trade_date`・`settlement_date` | [backend/migrations/0004_domestic_stocks.sql:6](../../backend/migrations/0004_domestic_stocks.sql#L6) |
| 口座区分 | 取引・保有を分ける区分。`account` で保持し、国内株式の保有では未設定も許す | [backend/migrations/0014_asset_balances_account.sql:4](../../backend/migrations/0014_asset_balances_account.sql#L4)、[shared/src/domain.rs:193](../../shared/src/domain.rs#L193) |
| 銘柄コード | 銘柄を識別するコード。保有の任意コードと上場銘柄マスタのコードを同じ集合と決めつけない | [backend/migrations/0006_asset_balances.sql:6](../../backend/migrations/0006_asset_balances.sql#L6)、[docs/adr/0002-separate-stock-master-and-caches.md](../adr/0002-separate-stock-master-and-caches.md) |
| 評価額 | 保有資産の時価での金額。投資信託で共有値を使う場合は数量×基準価額÷1万の切り捨て | [frontend/src/ui/labels.rs:7](../../frontend/src/ui/labels.rs#L7)、[backend/migrations/0013_fund_nav.sql:340](../../backend/migrations/0013_fund_nav.sql#L340) |
| 評価損益 | 評価額から取得総額を引いた値。画面ではこの言葉に統一する | [frontend/src/ui/labels.rs:11](../../frontend/src/ui/labels.rs#L11) |
| 実現損益 | 売却・解約などで確定した取引の損益。`realized_profit_and_loss` と税引後の列を分けて持つ | [backend/migrations/0004_domestic_stocks.sql:15](../../backend/migrations/0004_domestic_stocks.sql#L15)、[0005_mutualfunds.sql:16](../../backend/migrations/0005_mutualfunds.sql#L16) |
| 前日比 | 前の営業日との価格などの差。前日終値が欠ける国内株式は、画面と合計で欠測を考慮する | [frontend/src/ui/labels.rs:13](../../frontend/src/ui/labels.rs#L13)、[shared/src/domain.rs:212](../../shared/src/domain.rs#L212) |
| 基準価額・NAV | 投資信託の1万口あたりの価格。共有表の列名が `nav`、画面向けの型では `base_price` | [shared/src/domain.rs:289](../../shared/src/domain.rs#L289)、[backend/migrations/0013_fund_nav.sql:269](../../backend/migrations/0013_fund_nav.sql#L269) |
| 基準日 | 価格がいつの値かを表す日。列名は `price_as_of`。取得処理の実行時刻とは異なる | [shared/src/domain.rs:210](../../shared/src/domain.rs#L210)、[同:307](../../shared/src/domain.rs#L307) |
| 年間配当（予想） | 会社予想の年間配当。過去に受け取った配当明細とは別の情報 | [frontend/src/ui/labels.rs:19](../../frontend/src/ui/labels.rs#L19)、[shared/src/dividend_per_share.rs](../../shared/src/dividend_per_share.rs) |
| CSV種類 | 国内株式残高・投資信託残高・配当・国内株式取引・投資信託取引の5種類 | [frontend/src/features/csv_sync.rs:15](../../frontend/src/features/csv_sync.rs#L15)、[backend/migrations/0017_csv_import_history.sql:3](../../backend/migrations/0017_csv_import_history.sql#L3) |
| 最終取り込み成功時刻 | ユーザー・CSV種類ごとに保持する最新の成功時刻。CSV取得時刻や全履歴ではない | [backend/migrations/0017_csv_import_history.sql:1](../../backend/migrations/0017_csv_import_history.sql#L1) |

## 認証・保存・通信の言葉

| 用語 | このアプリでの意味 | 根拠 |
| --- | --- | --- |
| API・JSON | APIはデータ操作の窓口。JSONは窓口で受け渡すデータ形式 | [frontend/src/api/client.rs:311](../../frontend/src/api/client.rs#L311)、[同:326](../../frontend/src/api/client.rs#L326) |
| OAuth・callback | Googleの認可画面を経由するログイン。callbackは認可後に戻るバックエンドの入口 | [backend/src/handlers/v1/auth/oauth.rs:99](../../backend/src/handlers/v1/auth/oauth.rs#L99)、[同:140](../../backend/src/handlers/v1/auth/oauth.rs#L140) |
| セッション | ログイン状態の識別。CookieのトークンをハッシュにしてDBと照合し、有効期限も確認する | [backend/src/services/auth.rs:28](../../backend/src/services/auth.rs#L28)、[同:119](../../backend/src/services/auth.rs#L119) |
| Cookie・HttpOnly | ブラウザが保存・送信する値。HttpOnlyは画面のJavaScriptから読み取れない属性。セッションCookieは7日で発行する | [backend/src/handlers/v1/auth/oauth.rs:36](../../backend/src/handlers/v1/auth/oauth.rs#L36)、[同:67](../../backend/src/handlers/v1/auth/oauth.rs#L67) |
| ハッシュ | 元の値から計算する照合用の値。セッションではSHA-256を使い、DBにトークンの生値を保存しない | [backend/src/services/auth.rs:28](../../backend/src/services/auth.rs#L28)、[同:153](../../backend/src/services/auth.rs#L153) |
| PK・FK・ER図 | PKは主キー、FKは他の表を参照する制約。ER図は表と関係を表現する図 | [backend/migrations/0002_sessions.sql:13](../../backend/migrations/0002_sessions.sql#L13)、[同:26](../../backend/migrations/0002_sessions.sql#L26) |
| マスタ・キャッシュ | マスタは参照の基礎となる一覧、キャッシュは外部取得値の再利用のための保存領域 | [backend/migrations/0007_stock.sql:3](../../backend/migrations/0007_stock.sql#L3)、[0008_dividend_per_share_cache.sql:3](../../backend/migrations/0008_dividend_per_share_cache.sql#L3) |
| ビュー・LEFT JOIN | ビューは読み取り時に表を組み合わせる定義。LEFT JOINは対応する共有値がなくても保有行を残す結合 | [backend/migrations/0014_asset_balances_account.sql:16](../../backend/migrations/0014_asset_balances_account.sql#L16)、[0013_fund_nav.sql:324](../../backend/migrations/0013_fund_nav.sql#L324) |
| `price_source`・`nav_is_latest` | 投信価格の採用元と、前日比を最新として表示できるか。共有値採用と最新判定は別に持つ | [shared/src/domain.rs:252](../../shared/src/domain.rs#L252)、[同:309](../../shared/src/domain.rs#L309) |
| マイグレーション | DB構造の変更を順に適用するSQLと実行の仕組み | [backend/src/db/migrate.rs:15](../../backend/src/db/migrate.rs#L15) |

## 実行環境・自動化の言葉

| 用語 | このアプリでの意味 | 根拠 |
| --- | --- | --- |
| Leptos・CSR・Wasm | Rustで画面を書く仕組み・ブラウザ側で画面を作る方式・ブラウザなどで動くコンパイル形式 | [frontend/Cargo.toml:12](../../frontend/Cargo.toml#L12)、[frontend/src/main.rs:13](../../frontend/src/main.rs#L13)、[.github/workflows/frontend.yml:67](../../.github/workflows/frontend.yml#L67) |
| Cloudflare Pages | ビルドした静的な画面の配信先 | [.github/workflows/deploy-cloudflare-pages.yml:190](../../.github/workflows/deploy-cloudflare-pages.yml#L190) |
| Cloudflare Workers・Worker | APIと定期処理の実行環境と、配置するプログラム | [backend/src/worker_entry.rs:167](../../backend/src/worker_entry.rs#L167)、[同:199](../../backend/src/worker_entry.rs#L199) |
| Axum | Worker内でAPIの入口や前後処理を組み立てるRustの仕組み | [backend/src/worker_entry.rs:49](../../backend/src/worker_entry.rs#L49) |
| Hyperdrive | WorkerからPostgreSQLへの接続を仲介する仕組み | [backend/src/worker_entry.rs:127](../../backend/src/worker_entry.rs#L127)、[backend/src/db.rs:454](../../backend/src/db.rs#L454) |
| PostgreSQL・Supabase | PostgreSQLはDB、Supabaseはその提供サービス。本番配置先の記録は構成文書にあり、接続コードは提供会社を固定しない | [backend/src/db.rs:472](../../backend/src/db.rs#L472)、[docs/architecture.md:36](../architecture.md#L36) |
| J-Quants | 決算サマリーと上場銘柄一覧を取得する外部API | [backend/src/services/jquants.rs:13](../../backend/src/services/jquants.rs#L13) |
| Cron Triggers・scheduled | 定期時刻の設定と、Workerに届く定期イベント | [backend/wrangler.toml:44](../../backend/wrangler.toml#L44)、[backend/src/worker_entry.rs:199](../../backend/src/worker_entry.rs#L199) |
| CI・CD・E2E | 自動検証・配信・ブラウザを通じた動作検証 | [.github/workflows/frontend.yml:50](../../.github/workflows/frontend.yml#L50)、[同:150](../../.github/workflows/frontend.yml#L150)、[同:281](../../.github/workflows/frontend.yml#L281) |
| アーティファクト | Actionsで受け渡す成果物。画面ビルドと暗号化バックアップは用途も保存期間も別 | [.github/workflows/frontend.yml:134](../../.github/workflows/frontend.yml#L134)、[.github/workflows/db-backup.yml:55](../../.github/workflows/db-backup.yml#L55) |
| OpenAPI・shared | APIの入出力契約の形式と、両側で共用するRustコードのクレート | [backend/src/openapi.rs](../../backend/src/openapi.rs)、[frontend/src/api/dto.rs](../../frontend/src/api/dto.rs) |
| Dependabot | 依存更新をPRにする自動化。配信物に組み込む機能ではない | [.github/dependabot.yml:1](../../.github/dependabot.yml#L1) |

## 未確認

- 各外部サービスのアカウント・契約プラン・実環境の設定値。ここでの説明はコードを読むための意味に限る。
