-- 5a:制限ロール app_restricted を作り、本人表への読み書きと共有表・ビューへの読み取りだけを与える。FORCE なし。
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_restricted') THEN
        BEGIN
            CREATE ROLE app_restricted WITH NOLOGIN;
        EXCEPTION WHEN insufficient_privilege THEN
            RAISE NOTICE 'app_restricted の作成を省略(CREATE ROLE の権限がないため手動作成が必要)';
        END;
    END IF;
END
$$;

DO $$
DECLARE
    owner_name text := CURRENT_USER;
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_restricted') THEN
        RAISE NOTICE 'app_restricted がないため権限付与を省略';
        RETURN;
    END IF;
    EXECUTE 'GRANT USAGE ON SCHEMA public TO app_restricted';
    EXECUTE 'GRANT SELECT, INSERT, UPDATE, DELETE ON public.users, public.sessions, public.dividends, public.domestic_stocks, public.mutualfunds, public.asset_balances, public.fund_balances, public.csv_import_history, public.login_events, public.login_notify_tokens, public.admin_login_views TO app_restricted';
    EXECUTE 'GRANT SELECT ON public.stock, public.security_prices, public.dividend_per_share_cache, public.market_data_provider_rate_control, public.fund_codes, public.fund_prices TO app_restricted';
    EXECUTE 'GRANT SELECT ON public.asset_balances_view, public.fund_balances_view TO app_restricted';
    EXECUTE 'GRANT EXECUTE ON FUNCTION public.fund_name_normalize(text), public.fund_expected_nav_date(timestamptz) TO app_restricted';
    -- 他の管理者が先に作ったロールには ADMIN 権限がなく所属付与だけ失敗しうるため、
    -- その場合もデプロイを止めず手動付与に委ねる(表権限は所有者なら通るため厳密に扱う)
    IF NOT EXISTS (
        SELECT 1 FROM pg_auth_members m
        JOIN pg_roles r ON r.oid = m.roleid
        JOIN pg_roles u ON u.oid = m.member
        WHERE r.rolname = 'app_restricted' AND u.rolname = owner_name
    ) THEN
        BEGIN
            EXECUTE format('GRANT app_restricted TO %I', owner_name);
        EXCEPTION WHEN insufficient_privilege THEN
            RAISE NOTICE 'app_restricted への所属付与を省略(権限がないため手動付与が必要)';
        END;
    END IF;
END
$$;
