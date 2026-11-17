DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'app_worker') THEN
        CREATE ROLE app_worker LOGIN NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE NOINHERIT;
    ELSE
        ALTER ROLE app_worker LOGIN NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE NOINHERIT;
    END IF;
END
$$;

GRANT USAGE ON SCHEMA public TO app_worker;
GRANT app_restricted TO app_worker WITH SET TRUE, INHERIT FALSE;
GRANT EXECUTE ON FUNCTION public.app_enter(uuid) TO app_worker;

SET ROLE app_admin_login_executor;
GRANT EXECUTE ON FUNCTION public.admin_login_search_data_v2(text[], text[], text[], text, text, bigint, bigint),
    public.admin_login_search_count_v2(text[], text[], text[])
    TO app_admin_login_executor, app_worker;
RESET ROLE;

SET ROLE app_price_target_executor;
GRANT EXECUTE ON FUNCTION public.scheduled_stock_price_targets_v2(date, bigint),
    public.scheduled_fund_price_targets_v2(date, bigint),
    public.scheduled_stock_price_mark_attempted_v2(text[]),
    public.scheduled_stock_price_apply_v2(text, numeric, numeric, date),
    public.scheduled_fund_price_mark_attempted_v2(text[]),
    public.scheduled_fund_price_apply_v2(text, numeric, numeric, date),
    public.scheduled_fund_price_record_failure_v2(text, text),
    public.scheduled_fund_price_clear_failure_v2(text)
    TO app_price_target_executor, app_worker;
RESET ROLE;

SET ROLE app_login_event_prune_executor;
GRANT EXECUTE ON FUNCTION public.prune_login_events_older_than_90_days_v2()
    TO app_login_event_prune_executor, app_worker;
RESET ROLE;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_catalog.pg_roles
        WHERE rolname = CURRENT_USER AND rolsuper
    ) THEN
        REVOKE app_notify_token_v2_executor, app_admin_login_executor,
            app_price_target_executor, app_login_event_prune_executor
            FROM CURRENT_USER;
    ELSE
        GRANT app_notify_token_v2_executor, app_admin_login_executor,
            app_price_target_executor, app_login_event_prune_executor
            TO CURRENT_USER WITH ADMIN FALSE, SET FALSE, INHERIT FALSE;
    END IF;
END
$$;
