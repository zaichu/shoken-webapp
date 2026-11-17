-- 残高ビューを呼び出し側の権限で動かし、利用者表の RLS をビュー経由でも効かせる。
-- 利用者表のポリシーは user_id を text に変換せず比べる形に揃え、user_id 先頭の索引を使えるようにする。
-- NULLIF は、トランザクション後に接続へ残る空文字の設定を NULL(0 件)として扱い、uuid 変換のエラーにしないため。
SET LOCAL lock_timeout = '5s';

ALTER VIEW public.asset_balances_view SET (security_invoker = true);
ALTER VIEW public.fund_balances_view SET (security_invoker = true);

ALTER POLICY users_isolation ON public.users
    USING (id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));

ALTER POLICY sessions_isolation ON public.sessions
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));

ALTER POLICY dividends_isolation ON public.dividends
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));

ALTER POLICY domestic_stocks_isolation ON public.domestic_stocks
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));

ALTER POLICY mutualfunds_isolation ON public.mutualfunds
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));

ALTER POLICY asset_balances_isolation ON public.asset_balances
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));

ALTER POLICY fund_balances_isolation ON public.fund_balances
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));

ALTER POLICY csv_import_history_isolation ON public.csv_import_history
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));

ALTER POLICY login_events_isolation ON public.login_events
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));

ALTER POLICY admin_login_views_isolation ON public.admin_login_views
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));

ALTER POLICY login_notify_tokens_isolation ON public.login_notify_tokens
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));
