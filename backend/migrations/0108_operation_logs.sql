SET LOCAL lock_timeout = '5s';

CREATE TABLE public.operation_logs (
    id bigserial PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    operation text NOT NULL CHECK (operation IN ('csv_delete', 'logout_all', 'device_revoke')),
    target text NOT NULL CHECK (target IN ('dividends', 'domestic_stocks', 'mutualfunds', 'asset_balances', 'fund_balances', 'sessions', 'device')),
    count bigint NOT NULL CHECK (count >= 0),
    occurred_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT operation_logs_operation_target CHECK (
        (operation = 'csv_delete' AND target IN ('dividends', 'domestic_stocks', 'mutualfunds', 'asset_balances', 'fund_balances')) OR
        (operation = 'logout_all' AND target = 'sessions') OR
        (operation = 'device_revoke' AND target = 'device')
    )
);

CREATE INDEX operation_logs_user_occurred_idx ON public.operation_logs (user_id, occurred_at DESC, id DESC);
CREATE INDEX operation_logs_occurred_idx ON public.operation_logs (occurred_at);

ALTER TABLE public.operation_logs ENABLE ROW LEVEL SECURITY;
CREATE POLICY operation_logs_user_policy ON public.operation_logs
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));
COMMENT ON TABLE public.operation_logs IS '重要な操作の記録。金額・銘柄・IP の生の値は保存しない';
COMMENT ON COLUMN public.operation_logs.operation IS '操作の種類';
COMMENT ON COLUMN public.operation_logs.target IS '対象のデータ種類';
COMMENT ON COLUMN public.operation_logs.count IS '操作で影響した件数';
COMMENT ON COLUMN public.operation_logs.occurred_at IS '操作日時';

GRANT SELECT, INSERT ON public.operation_logs TO app_restricted;
GRANT USAGE ON SEQUENCE public.operation_logs_id_seq TO app_restricted;
GRANT SELECT, INSERT ON public.operation_logs TO app_worker;
GRANT USAGE ON SEQUENCE public.operation_logs_id_seq TO app_worker;

GRANT SELECT (occurred_at), DELETE ON public.operation_logs TO app_login_event_prune_executor;

CREATE FUNCTION public.prune_operation_logs_older_than_one_year_v1()
RETURNS bigint
LANGUAGE sql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    WITH deleted AS (
        DELETE FROM public.operation_logs
        WHERE occurred_at < pg_catalog.now() - INTERVAL '1 year'
        RETURNING 1
    )
    SELECT pg_catalog.count(*) FROM deleted
$$;
REVOKE ALL ON FUNCTION public.prune_operation_logs_older_than_one_year_v1()
    FROM PUBLIC, app_restricted;
GRANT CREATE ON SCHEMA public TO app_login_event_prune_executor;
ALTER FUNCTION public.prune_operation_logs_older_than_one_year_v1()
    OWNER TO app_login_event_prune_executor;
REVOKE CREATE ON SCHEMA public FROM app_login_event_prune_executor;

SET ROLE app_login_event_prune_executor;
GRANT EXECUTE ON FUNCTION public.prune_operation_logs_older_than_one_year_v1()
    TO app_login_event_prune_executor, app_worker;
DO $$
BEGIN
    EXECUTE format(
        'GRANT EXECUTE ON FUNCTION public.prune_operation_logs_older_than_one_year_v1() TO %I',
        SESSION_USER
    );
END
$$;
RESET ROLE;
