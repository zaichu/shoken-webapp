UPDATE public.dividend_per_share_cache
SET stale_at = updated_at
WHERE status <> 'pending' AND stale_at IS NULL;
