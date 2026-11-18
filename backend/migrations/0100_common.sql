-- 共通の土台: 拡張、アプリのロールと所属、利用者 Tx の入口関数 app_enter。
-- ロールはクラスタ共通で既にあることがあるため、作成は既存なら飛ばす。

CREATE EXTENSION IF NOT EXISTS citext WITH SCHEMA public;
COMMENT ON EXTENSION citext IS 'data type for case-insensitive character strings';

CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;
COMMENT ON EXTENSION pg_trgm IS 'text similarity measurement and index searching based on trigrams';

CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;
COMMENT ON EXTENSION pgcrypto IS 'cryptographic functions';

-- app_restricted: 利用者 Tx の中だけで使う制限ロール(NOBYPASSRLS)。
-- 移行ロールは SET で切り替えられるが、権限は継承しない。
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'app_restricted') THEN
        CREATE ROLE app_restricted WITH NOLOGIN;
    END IF;
END
$$;

GRANT app_restricted TO CURRENT_USER WITH SET TRUE, INHERIT FALSE;

-- 限定関数の所有者(BYPASSRLS)。関数の中でだけ使い、アプリの接続ロールは所属しない。
DO $$
DECLARE
    executor text;
BEGIN
    FOREACH executor IN ARRAY ARRAY[
        'app_notify_token_v2_executor',
        'app_admin_login_executor',
        'app_price_target_executor',
        'app_login_event_prune_executor',
        'app_shared_write_executor'
    ] LOOP
        BEGIN
            EXECUTE format('CREATE ROLE %I NOLOGIN BYPASSRLS', executor);
        EXCEPTION WHEN duplicate_object THEN
            EXECUTE format('ALTER ROLE %I NOLOGIN BYPASSRLS', executor);
        END;
    END LOOP;
END
$$;

GRANT app_notify_token_v2_executor, app_admin_login_executor,
    app_price_target_executor, app_login_event_prune_executor, app_shared_write_executor
    TO CURRENT_USER WITH SET TRUE, INHERIT FALSE;

-- app_worker: Workers の接続ロール。表の所有者にせず、所有者ロールにも所属させない
-- (RLS が効く根拠。docs/security/db-rls-plan.md §28)。
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'app_worker') THEN
        CREATE ROLE app_worker LOGIN NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE NOINHERIT;
    ELSE
        ALTER ROLE app_worker LOGIN NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE NOINHERIT;
    END IF;
END
$$;

GRANT app_restricted TO app_worker WITH SET TRUE, INHERIT FALSE;

-- 利用者 Tx の準備を1呼び出しにまとめる(ロールの切り替えと利用者 ID の設定)。
CREATE FUNCTION public.app_enter(uid uuid) RETURNS void
    LANGUAGE plpgsql SECURITY INVOKER SET search_path = public, pg_temp
AS $$
BEGIN
    PERFORM set_config('role', 'app_restricted', true);
    PERFORM set_config('app.current_user_id', uid::text, true);
END;
$$;

REVOKE ALL ON FUNCTION public.app_enter(uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.app_enter(uuid) TO app_worker;
