# backend/migrations

migration ファイルの置き場所と運用メモ。
ファイル形式と `_sqlx_migrations` テーブルは sqlx 互換だが、実行は自前ランナー
（`backend/src/bin/migrate.rs` = `backend/src/db/migrate.rs` の `run_migrations`）で行う。
新規ファイルを `MIGRATION_FILES`（`backend/src/db/migrate.rs`）に `include_str!` で
登録しないと適用されない。

## 現行方針

現在の schema は `0001_schema.sql` 1本で表す。旧 `0001〜0017` を順に全適用した
結果と同じ schema を作る baseline で、旧17本を適用した DB の `pg_dump --schema-only`
出力から所有者・権限・SET・コメント行を除いて生成した(手書きではない)。
`pg_dump --schema-only` に含まれない初期行(`market_data_provider_rate_control`)は
末尾に追記してある。

新旧の schema が一致することは `backend/tests/db_integration.rs` の
`squashed_schema_matches_legacy_migrations` が pg_dump を比較して確認する。
旧17本は比較用に `backend/tests/fixtures/legacy_migrations/` に保持している。

## _sqlx_migrations 履歴の扱い

旧バージョン(1〜17)が適用済みの DB で migrate を実行すると、`_sqlx_migrations`
の内容を version=1 の1件の記録に置き換える(一度だけ。テーブル・データには触れない)。
旧履歴が 1〜17 の途中までしか無い DB は schema が現行と異なる可能性があるため
エラーで停止する。

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

baseline(0001)は適用済み環境があるため編集しない。
`0002_<description>.sql` のように次番号の新規 migration を追加し、
`backend/src/db/migrate.rs` の `MIGRATION_FILES` に `include_str!` エントリを足す。

## baseline の再生成(0001_schema.sql を張り直す場合)

旧系列を順に適用した DB に対して `pg_dump --schema-only` を取り、
SET・`SELECT pg_catalog.set_config`・コメント・`OWNER TO`・権限行を除いて整形する。
`--schema-only` にはデータが含まれないため、必要な初期行 INSERT は別途末尾に追記する。

```bash
cd backend && make db-up   # ローカル PostgreSQL (docker-compose)
psql "$DATABASE_URL" -f tests/fixtures/legacy_migrations/0001_initial_schema.sql  # 〜0017まで順に
pg_dump --schema-only "$DATABASE_URL"
```

## 検証

```bash
cd backend && cargo fmt --check
cd backend && cargo clippy --all-targets -- -D warnings
cd backend && cargo test
cd backend && cargo test --test db_integration -- --ignored --nocapture
```
