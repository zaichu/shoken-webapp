UPDATE public.dividend_per_share_cache
SET stale_at = updated_at
WHERE status = 'error' AND stale_at IS NULL;
