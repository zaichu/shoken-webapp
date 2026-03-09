-- 既存 DB の migration 履歴をリセットするスクリプト
-- 対象: migrations/0001〜0017 を適用済みの DB を migrations/0001〜0009 に移行する場合
--
-- 実行方法:
--   psql $DATABASE_URL -f backend/scripts/repair-migrations.sql
--   cd backend && cargo sqlx migrate run  # 新しいファイルを "applied" としてマーク
--
-- 注意:
--   - 全ての CREATE TABLE / CREATE INDEX に IF NOT EXISTS が付いているため
--     テーブルが存在していても SQL は安全に再実行される
--   - このスクリプトはスキーマには一切変更を加えない
--   - 事前条件: version=17 (0017_recalculate_domestic_stocks_content_hash) まで適用済みであること

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM _sqlx_migrations WHERE version = 17
  ) THEN
    RAISE EXCEPTION
      'repair を実行するには version=17 まで全て適用済みである必要があります。'
      '現在未達のため中断します。先に cargo sqlx migrate run で 0017 まで適用してください。';
  END IF;
END $$;

TRUNCATE TABLE _sqlx_migrations;
