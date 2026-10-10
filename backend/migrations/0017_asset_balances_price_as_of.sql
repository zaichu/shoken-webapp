-- 現在値の基準日。CSV 取込値と cron 自動取得値の「日付の新しいほう」を判定するために使う。
-- 既存行は最終更新時刻の JST 日付を暫定の基準日として backfill する。
ALTER TABLE asset_balances ADD COLUMN IF NOT EXISTS price_as_of DATE;
UPDATE asset_balances
    SET price_as_of = (updated_at AT TIME ZONE 'Asia/Tokyo')::date
    WHERE price_as_of IS NULL;
