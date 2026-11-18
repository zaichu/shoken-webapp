---
name: db-migration
description: |
  PostgreSQL マイグレーションの作成と実行。
  Use when: マイグレーション、テーブル作成、スキーマ変更を依頼された時。
---

# データベースマイグレーション

## マイグレーションファイル

### 配置場所
`backend/migrations/` ディレクトリ(役割ごとの土台 `0100〜0106` と、その後の追記。旧版は `archive/`)

### 命名規則
```
NNNN_<説明>.sql
例: 0107_add_dividends_memo.sql
```

新規ファイルは `backend/src/db/migrate.rs` の `MIGRATION_FILES` に `include_str!` で登録する（登録しないと適用されない）。
土台と適用済みのファイルは書き換えない。新しい表の RLS・ポリシー・GRANT の決まりは `backend/migrations/README.md` に従う。

## 新規テーブル作成テンプレート

```sql
-- マイグレーション: テーブル名の説明
CREATE TABLE IF NOT EXISTS table_name (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 他のカラム
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_table_name_user_id ON table_name(user_id);

-- ユニーク制約（重複防止用）
CREATE UNIQUE INDEX IF NOT EXISTS idx_table_name_unique
ON table_name(user_id, some_column);
```

## 型マッピング

| Rust | PostgreSQL |
|------|------------|
| Uuid | UUID |
| String | TEXT / VARCHAR |
| i32 | INTEGER |
| i64 | BIGINT |
| f64 | DOUBLE PRECISION |
| NaiveDate | DATE |
| DateTime<Utc> | TIMESTAMPTZ |
| bool | BOOLEAN |

## マイグレーション実行

本番（Cloudflare Workers）では `deploy-cloudflare-worker.yml` がデプロイ時に `cargo run --bin migrate` で Neon へ直接適用する。ローカルでは：
```bash
cd backend && make migrate-local
```

## 注意事項

- 本番DBに影響する変更は慎重に
- DROP TABLE は避け、必要なら別マイグレーションで
- 外部キー制約で `ON DELETE CASCADE` を適切に使用
