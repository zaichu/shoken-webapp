-- 現在値の最終問い合わせ時刻。祝日や取得失敗で price_as_of が進まない銘柄を
-- 毎分取り直し続けないよう、「前回問い合わせから1時間以上経過」の判定に使う。
ALTER TABLE asset_balances ADD COLUMN IF NOT EXISTS price_checked_at TIMESTAMPTZ;
