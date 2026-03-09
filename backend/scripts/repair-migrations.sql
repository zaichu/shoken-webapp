-- 既存 DB の migration 履歴をリセットするスクリプト
-- 対象: migrations/0001〜0017 を適用済みの DB を migrations/0001〜0009 に移行する場合
--
-- 実行方法:
--   psql $DATABASE_URL -f scripts/repair-migrations.sql
--   cargo sqlx migrate run  # 新しいファイルを "applied" としてマーク
--
-- 注意:
--   - 全ての CREATE TABLE / CREATE INDEX に IF NOT EXISTS が付いているため
--     テーブルが存在していても SQL は安全に再実行される
--   - このスクリプトはスキーマには一切変更を加えない

TRUNCATE TABLE _sqlx_migrations;
