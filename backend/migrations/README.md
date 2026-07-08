# backend/migrations

SQLx migration の baseline 管理。

## 現行方針

`backend/migrations` は、新規 DB を現在の最終 schema にするための baseline だけを置く。
過去の段階的な migration は保持しない。

| No. | ファイル | 内容 |
|-----|---------|------|
| 0001 | `0001_initial_schema.sql` | 拡張、全テーブル、全 index、初期 seed を作成 |
| 0002 | `0002_dividends_search_indexes.sql` | 配当金検索（product/account/security_code/security_name の絞り込み、settlement_date/id 順の一覧取得）向け index を追加 |
| 0003 | `0003_domestic_stocks_search_indexes.sql` | 国内株式検索（account/security_code/security_name の絞り込み、trade_date/id 順の一覧取得）向け index を追加 |

## 重要な注意

SQLx は `_sqlx_migrations` に migration の version と checksum を保存する。
そのため、既存 DB に古い `0001〜0011` を適用済みの状態で、この baseline をそのまま `migrate run` すると checksum 不一致で失敗する。

既存 DB をこの baseline に切り替える場合は、先に `backend/scripts/repair-migrations.sql` を実行して migration 履歴だけを空にする。
この repair script は schema/data を変更しない。`CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF NOT EXISTS` / `ON CONFLICT DO NOTHING` により、次の `migrate run` で baseline を安全に applied として記録する。

本番 DB では、必ずバックアップ取得後に手動で実行する。アプリ起動時に勝手に repair はしない。

## 新規 DB

```bash
cd backend
cargo sqlx migrate run
```

## 既存 DB の baseline 切り替え

前提:
- 現行 `0001〜0011` を全て適用済み
- schema はアプリが期待する最終状態になっている
- 事前に DB backup を取得済み

手順:

```bash
psql "$DATABASE_URL" -f backend/scripts/repair-migrations.sql
cd backend
cargo sqlx migrate run
```

ローカル Docker DB では以下でもよい。

```bash
cd backend
make repair-and-migrate-local
```

## 新しい schema 変更を入れる場合

baseline ファイルは、既存環境がある限り安易に編集しない。
通常は `0002_<description>.sql` のように新規 migration を追加する。
次に再ベースライン化する時だけ、baseline を作り直し、repair script と README を同じ PR で更新する。

## 検証

```bash
cd backend && cargo fmt --check
cd backend && cargo clippy --all-targets -- -D warnings
cd backend && cargo test
cd backend && cargo test db_integration_with_docker_and_migrations -- --ignored --nocapture
```
