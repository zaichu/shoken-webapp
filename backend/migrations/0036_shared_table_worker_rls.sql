-- 共有表(利用者の行なし)を app_worker・app_restricted から読めるようにする。
-- 本番(Supabase)では共有表の RLS が有効でポリシーが無く、NOBYPASSRLS の app_worker の
-- SELECT がエラーなしで 0 行になっていた。RLS は外さない(Supabase の anon / authenticated
-- からの読み書きを止めているため)。どの環境でも同じ状態になるよう有効化も明示する。
-- 書き込みは BYPASSRLS の限定関数(0033・0035)だけなので、読み取りのポリシーだけを足す。
SET LOCAL lock_timeout = '5s';

ALTER TABLE public.dividend_per_share_cache ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.market_data_provider_rate_control ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.stock ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.security_prices ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.fund_prices ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.fund_codes ENABLE ROW LEVEL SECURITY;

CREATE POLICY app_worker_shared_read ON public.dividend_per_share_cache
    FOR SELECT TO app_worker, app_restricted USING (true);
CREATE POLICY app_worker_shared_read ON public.market_data_provider_rate_control
    FOR SELECT TO app_worker, app_restricted USING (true);
CREATE POLICY app_worker_shared_read ON public.stock
    FOR SELECT TO app_worker, app_restricted USING (true);
CREATE POLICY app_worker_shared_read ON public.security_prices
    FOR SELECT TO app_worker, app_restricted USING (true);
CREATE POLICY app_worker_shared_read ON public.fund_prices
    FOR SELECT TO app_worker, app_restricted USING (true);
CREATE POLICY app_worker_shared_read ON public.fund_codes
    FOR SELECT TO app_worker, app_restricted USING (true);
