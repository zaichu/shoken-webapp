-- 0119 の CHECK 制約(dividend_per_share_cache の status・dividend_per_share。#2112 で VALIDATE 予定)と、
-- #2112 で追加予定の shares > 0 制約に先立つ違反件数だけを返す監視用関数。
-- 個人表の行は件数だけを返し、値・利用者は出さない。
-- SECURITY DEFINER(所有は移行ロール)のため、呼び出し側の権限・RLS に関わらず全行を数えられる。
-- 個人表へ直接触る SQL を cron ログの検査文に書くと、権限のないロールで検査自体が落ちる
-- (プラン全体の権限を事前に検査するため CASE では回避できない)。そのため件数の取得はこの関数に閉じ込める。
-- cron の db access ログ(cron_log::DB_ACCESS_STATE_SQL)から呼び出す。
SET LOCAL lock_timeout = '5s';

CREATE FUNCTION public.range_check_violation_counts_v2()
RETURNS text
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    SELECT format('dividends_shares_nonpositive=%s domestic_stocks_shares_nonpositive=%s mutualfunds_shares_nonpositive=%s dividend_cache status_ok=%s status_zero=%s status_error=%s status_pending=%s status_other=%s dividend_negative=%s',
        (SELECT count(*) FROM public.dividends WHERE shares <= 0),
        (SELECT count(*) FROM public.domestic_stocks WHERE shares <= 0),
        (SELECT count(*) FROM public.mutualfunds WHERE shares <= 0),
        (SELECT count(*) FROM public.dividend_per_share_cache WHERE status = 'ok'),
        (SELECT count(*) FROM public.dividend_per_share_cache WHERE status = 'zero'),
        (SELECT count(*) FROM public.dividend_per_share_cache WHERE status = 'error'),
        (SELECT count(*) FROM public.dividend_per_share_cache WHERE status = 'pending'),
        (SELECT count(*) FROM public.dividend_per_share_cache WHERE status NOT IN ('ok', 'zero', 'error', 'pending')),
        (SELECT count(*) FROM public.dividend_per_share_cache WHERE dividend_per_share < 0))
$$;

REVOKE ALL ON FUNCTION public.range_check_violation_counts_v2() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.range_check_violation_counts_v2() TO app_worker;
