-- 既存 DB の migration 履歴を現在の baseline に切り替えるスクリプト。
--
-- 対象:
--   - 現行 0001〜0011 を適用済みの DB
--
-- このスクリプトは schema/data を変更しない。
-- `_sqlx_migrations` だけを空にし、次の `cargo sqlx migrate run` で
-- `0001_initial_schema.sql` を applied として記録できる状態にする。
--
-- 実行前に DB backup を取得すること。
-- 実行後:
--   cd backend && cargo sqlx migrate run

\set ON_ERROR_STOP on

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM _sqlx_migrations WHERE version = 11
  ) THEN
    RAISE EXCEPTION
      'repair を実行するには現行 version=11 まで適用済みである必要があります。';
  END IF;
END $$;

TRUNCATE TABLE _sqlx_migrations;
