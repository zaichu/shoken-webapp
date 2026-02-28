ALTER TABLE jquants_dividend_cache
  ADD COLUMN stale_at TIMESTAMPTZ;

-- ok/zero レコードは fetched_at + 7日を設定
UPDATE jquants_dividend_cache
SET stale_at = fetched_at + INTERVAL '7 days'
WHERE status IN ('ok', 'zero') AND fetched_at IS NOT NULL;

-- error/pending は NULL のまま（is_stale = true = 即再取得対象）
