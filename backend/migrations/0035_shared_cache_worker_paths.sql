-- 5f: app_worker 接続での共有表の読み取りと、配当キャッシュ・レート制御・
-- 銘柄マスタの限定書き込み関数。個人表には触らない。
-- 読み取りは共有表(利用者の行なし)に SELECT だけを与え、書き込みは
-- SECURITY DEFINER の限定関数に閉じ込める(0033 と同じ定型)。
-- 旧 postgres 接続でも新 Worker でも動くよう、EXECUTE は app_worker と
-- migration 実行ロール(CURRENT_USER)の両方に付ける。app_restricted には付けない。

DO $$
BEGIN
    CREATE ROLE app_shared_write_executor NOLOGIN BYPASSRLS;
EXCEPTION WHEN duplicate_object THEN
    ALTER ROLE app_shared_write_executor NOLOGIN BYPASSRLS;
END
$$;

GRANT app_shared_write_executor TO CURRENT_USER WITH SET TRUE, INHERIT FALSE;
GRANT USAGE ON SCHEMA public TO app_shared_write_executor;
GRANT CREATE ON SCHEMA public TO app_shared_write_executor;
GRANT SELECT, INSERT, UPDATE ON public.dividend_per_share_cache TO app_shared_write_executor;
GRANT SELECT, INSERT, UPDATE ON public.market_data_provider_rate_control TO app_shared_write_executor;
GRANT SELECT, INSERT, DELETE ON public.stock TO app_shared_write_executor;

-- 共有表の読み取り(利用者の行なし)。個人表の権限は付けない。
-- 残高ビュー(asset_balances_view / fund_balances_view)は所有者権限で動くため
-- 利用者 ID なしの app_worker 直からは読めないようにし、利用者 Tx 内の
-- app_restricted 経由(WHERE user_id 付き)の既存経路だけを使う。よって GRANT しない。
GRANT SELECT ON public.stock, public.security_prices,
    public.dividend_per_share_cache, public.market_data_provider_rate_control,
    public.fund_codes, public.fund_prices TO app_worker;
GRANT EXECUTE ON FUNCTION public.fund_name_normalize(text),
    public.fund_expected_nav_date(timestamptz) TO app_worker;

-- 未取得銘柄を pending 行として積む。既存行は触らない。
CREATE FUNCTION public.dividend_cache_enqueue_v2(p_codes varchar[])
RETURNS void
LANGUAGE sql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    INSERT INTO public.dividend_per_share_cache (security_code, status, provider, updated_at)
    SELECT c.code, 'pending', 'jquants', pg_catalog.now()
    FROM pg_catalog.unnest(p_codes) AS c(code)
    ON CONFLICT (security_code) DO NOTHING
$$;
REVOKE ALL ON FUNCTION public.dividend_cache_enqueue_v2(varchar[])
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.dividend_cache_enqueue_v2(varchar[])
    OWNER TO app_shared_write_executor;

-- 取得した配当をキャッシュへ反映する。p_status は 'ok' などの状態文字。
CREATE FUNCTION public.dividend_cache_apply_success_v2(
    p_code text, p_dividend_per_share numeric, p_status varchar
)
RETURNS void
LANGUAGE sql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    INSERT INTO public.dividend_per_share_cache
        (security_code, dividend_per_share, status, fetched_at, stale_at, provider, updated_at)
    VALUES (p_code, p_dividend_per_share, p_status, pg_catalog.now(), pg_catalog.now() + INTERVAL '7 days', 'jquants', pg_catalog.now())
    ON CONFLICT (security_code) DO UPDATE
        SET dividend_per_share = EXCLUDED.dividend_per_share,
            status             = EXCLUDED.status,
            fetched_at         = EXCLUDED.fetched_at,
            stale_at           = pg_catalog.now() + INTERVAL '7 days',
            provider           = EXCLUDED.provider,
            error_message      = NULL,
            updated_at         = pg_catalog.now()
$$;
REVOKE ALL ON FUNCTION public.dividend_cache_apply_success_v2(text, numeric, varchar)
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.dividend_cache_apply_success_v2(text, numeric, varchar)
    OWNER TO app_shared_write_executor;

-- 取得失敗をキャッシュに記録する。p_cooldown_secs 秒だけ再取得を止める。
CREATE FUNCTION public.dividend_cache_apply_error_v2(
    p_code text, p_error_message text, p_cooldown_secs bigint
)
RETURNS void
LANGUAGE sql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    INSERT INTO public.dividend_per_share_cache
        (security_code, dividend_per_share, status, error_message, stale_at, provider, updated_at)
    VALUES (p_code, NULL, 'error', p_error_message, pg_catalog.now() + p_cooldown_secs * INTERVAL '1 second', 'jquants', pg_catalog.now())
    ON CONFLICT (security_code) DO UPDATE
        SET status        = 'error',
            error_message = EXCLUDED.error_message,
            stale_at      = pg_catalog.now() + p_cooldown_secs * INTERVAL '1 second',
            updated_at    = pg_catalog.now()
$$;
REVOKE ALL ON FUNCTION public.dividend_cache_apply_error_v2(text, text, bigint)
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.dividend_cache_apply_error_v2(text, text, bigint)
    OWNER TO app_shared_write_executor;

-- 配当取得の実行スロットを取る。取れた回だけ取得するため並走しても 12 秒間隔になる。
-- 枠が埋まっている回は行を返さない。
CREATE FUNCTION public.rate_control_acquire_jquants_v2()
RETURNS timestamptz
LANGUAGE sql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    INSERT INTO public.market_data_provider_rate_control (provider, next_available_at)
        VALUES ('jquants', pg_catalog.now() + INTERVAL '12 seconds')
    ON CONFLICT (provider) DO UPDATE
        SET next_available_at =
            GREATEST(public.market_data_provider_rate_control.next_available_at, pg_catalog.now()) + INTERVAL '12 seconds'
    RETURNING
        GREATEST(next_available_at - INTERVAL '12 seconds', pg_catalog.now() - INTERVAL '1 second')
$$;
REVOKE ALL ON FUNCTION public.rate_control_acquire_jquants_v2()
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.rate_control_acquire_jquants_v2()
    OWNER TO app_shared_write_executor;

-- レート制御の次回時刻を少なくとも p_cooldown_secs 秒先へ延ばす。
-- 銘柄マスタの失敗時の再試行延期(3600 秒)にも使う。
CREATE FUNCTION public.rate_control_cooldown_v2(p_provider text, p_cooldown_secs bigint)
RETURNS void
LANGUAGE sql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    INSERT INTO public.market_data_provider_rate_control (provider, next_available_at)
        VALUES (p_provider, pg_catalog.now() + p_cooldown_secs * INTERVAL '1 second')
    ON CONFLICT (provider) DO UPDATE
        SET next_available_at =
            GREATEST(public.market_data_provider_rate_control.next_available_at, pg_catalog.now() + p_cooldown_secs * INTERVAL '1 second')
$$;
REVOKE ALL ON FUNCTION public.rate_control_cooldown_v2(text, bigint)
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.rate_control_cooldown_v2(text, bigint)
    OWNER TO app_shared_write_executor;

-- 銘柄マスタの更新枠を取る。取れた(true 相当の行を返す)回だけ全置換する。
-- 枠が埋まっている回は行を返さない。
CREATE FUNCTION public.stock_master_claim_v2()
RETURNS TABLE (claimed integer)
LANGUAGE sql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    INSERT INTO public.market_data_provider_rate_control (provider, next_available_at)
        VALUES ('jquants-equities-master', pg_catalog.now() + INTERVAL '7 days')
    ON CONFLICT (provider) DO UPDATE
        SET next_available_at = pg_catalog.now() + INTERVAL '7 days'
        WHERE public.market_data_provider_rate_control.next_available_at <= pg_catalog.now()
    RETURNING 1
$$;
REVOKE ALL ON FUNCTION public.stock_master_claim_v2()
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.stock_master_claim_v2()
    OWNER TO app_shared_write_executor;

-- stock を全置換する。戻り値は投入件数。
CREATE FUNCTION public.stock_master_replace_v2(
    p_dates date[], p_codes text[], p_names text[], p_markets text[],
    p_industry33 text[], p_industry17 text[], p_sizes text[]
)
RETURNS bigint
LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
DECLARE
    inserted bigint;
BEGIN
    DELETE FROM public.stock;
    INSERT INTO public.stock
        (date, code, name, market_category,
         industry_category_33, industry_category_17, size_category)
    SELECT * FROM unnest(
        p_dates, p_codes, p_names, p_markets, p_industry33, p_industry17, p_sizes
    );
    GET DIAGNOSTICS inserted = ROW_COUNT;
    RETURN inserted;
END
$$;
REVOKE ALL ON FUNCTION public.stock_master_replace_v2(date[], text[], text[], text[], text[], text[], text[])
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.stock_master_replace_v2(date[], text[], text[], text[], text[], text[], text[])
    OWNER TO app_shared_write_executor;

-- 銘柄マスタ更新の失敗時は再試行を1時間後に遅らせる(毎分のやり直しを止める)。
-- 直前の枠取り(7日後)を上書きする。
CREATE FUNCTION public.stock_master_postpone_v2()
RETURNS void
LANGUAGE sql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    UPDATE public.market_data_provider_rate_control
       SET next_available_at = pg_catalog.now() + INTERVAL '1 hour'
     WHERE provider = 'jquants-equities-master'
$$;
REVOKE ALL ON FUNCTION public.stock_master_postpone_v2()
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.stock_master_postpone_v2()
    OWNER TO app_shared_write_executor;

REVOKE CREATE ON SCHEMA public FROM app_shared_write_executor;

SET ROLE app_shared_write_executor;
GRANT EXECUTE ON FUNCTION public.dividend_cache_enqueue_v2(varchar[]),
    public.dividend_cache_apply_success_v2(text, numeric, varchar),
    public.dividend_cache_apply_error_v2(text, text, bigint),
    public.rate_control_acquire_jquants_v2(),
    public.rate_control_cooldown_v2(text, bigint),
    public.stock_master_claim_v2(),
    public.stock_master_postpone_v2(),
    public.stock_master_replace_v2(date[], text[], text[], text[], text[], text[], text[])
    TO app_shared_write_executor, app_worker;
DO $$
BEGIN
    EXECUTE format(
        'GRANT EXECUTE ON FUNCTION public.dividend_cache_enqueue_v2(varchar[]), public.dividend_cache_apply_success_v2(text, numeric, varchar), public.dividend_cache_apply_error_v2(text, text, bigint), public.rate_control_acquire_jquants_v2(), public.rate_control_cooldown_v2(text, bigint), public.stock_master_claim_v2(), public.stock_master_postpone_v2(), public.stock_master_replace_v2(date[], text[], text[], text[], text[], text[], text[]) TO %I',
        SESSION_USER
    );
END
$$;
RESET ROLE;
