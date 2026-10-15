# backend/migrations

migration ファイルの置き場所と運用メモ。
ファイル形式と `_sqlx_migrations` テーブルは sqlx 互換だが、実行は自前ランナー
（`backend/src/bin/migrate.rs` = `backend/src/db/migrate.rs` の `run_migrations`）で行う。
新規ファイルを `MIGRATION_FILES`（`backend/src/db/migrate.rs`）に `include_str!` で
登録しないと適用されない。

## 現行方針

現在の schema はテーブルごと1ファイルの baseline 群(`0000〜0009`)で表す。
旧 `0001〜0017` を順に全適用した結果と同じ schema を作る baseline で、
旧17本を適用した DB の `pg_dump --schema-only` 出力から所有者・権限・SET・
コメント行を除き、テーブル単位に分割して生成した(手書きではない)。
`pg_dump --schema-only` に含まれない初期行(`market_data_provider_rate_control` の
INSERT)は対応するテーブルのファイル末尾に追記してある。

- `0000_extensions.sql`: 共通拡張(citext / pg_trgm / pgcrypto)
- `0001_users.sql` 〜 `0006_asset_balances.sql`: 外部キーで参照される側
  (`users`)から番号順。各テーブルの index・constraint は同じファイル内
- `0007_stock.sql` 〜 `0009_market_data_provider_rate_control.sql`: 外部キーを
  持たない独立テーブル

## _sqlx_migrations 履歴の扱い

旧バージョン(1〜17)が適用済みの DB で migrate を実行すると、`_sqlx_migrations`
の内容をベースライン 0〜9 の記録に置き換える(一度だけ。テーブル・データには
触れない)。旧履歴かどうかは version=1 の checksum が新 `0001_users.sql` と
異なるかで判定する。旧履歴が 1〜17 の途中までしか無い DB は schema が現行と
異なる可能性があるためエラーで停止する。

## デプロイ時（Cloudflare Workers）の動作：自動適用

デプロイ時の migration 適用は自動。手動実行は不要。

- `deploy-cloudflare-worker.yml` の deploy ジョブが `cargo run --bin migrate`（`backend/src/bin/migrate.rs`）を Neon へ直接実行してから `wrangler deploy` する
- Worker のリクエストパスに migration は置けない（起動という概念がないため）
- 手動 `cargo run --bin migrate` が必要なのは新規 DB・ローカル DB のみ

## 新規 DB

```bash
cd backend
cargo run --bin migrate
```

## 新しい schema 変更を入れる場合

baseline(0000〜0009)は適用済み環境があるため編集しない。
`0010_<description>.sql` のように次番号の新規 migration を追加し、
`backend/src/db/migrate.rs` の `MIGRATION_FILES` に `include_str!` エントリを足す。

## baseline の再生成(baseline ファイル群を張り直す場合)

新規 DB に全マイグレーションを適用した状態に対して `pg_dump --schema-only` を取り、
SET・`SELECT pg_catalog.set_config`・コメント・`OWNER TO`・権限行を除いて整形し、
テーブル単位に分割する。`--schema-only` にはデータが含まれないため、必要な
初期行 INSERT は別途対象テーブルのファイル末尾に追記する。

```bash
cd backend && make db-up   # ローカル PostgreSQL (docker-compose)
cargo run --bin migrate
pg_dump --schema-only "$DATABASE_URL"
```

## 検証

```bash
cd backend && cargo fmt --check
cd backend && cargo clippy --all-targets -- -D warnings
cd backend && cargo test
cd backend && cargo test --test db_integration -- --ignored --nocapture
```
