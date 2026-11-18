-- 限定関数: 利用者 ID が決まる前の認証と、利用者をまたぐ定期処理・共有の表の書き込みを
-- SECURITY DEFINER の関数に閉じ込め、アプリのロールには EXECUTE だけを与える。
-- 認証の関数(auth_*・login_notify_*_by_token)は移行ロールが所有し app_restricted から呼ぶ。
-- *_v2 は BYPASSRLS の所有ロール(0100)が持ち、触る列だけの権限(0105)で動く。
-- 所有者を移すには移す先に schema の CREATE が要るため、関数を作る間だけ与えて最後に外す。

CREATE FUNCTION public.auth_lookup_user_id_by_session(p_token_hash bytea) RETURNS uuid
    LANGUAGE sql STABLE SECURITY DEFINER SET search_path = public, pg_temp
AS $$ SELECT user_id FROM public.sessions WHERE token_hash = p_token_hash AND expires_at > NOW() $$;

REVOKE ALL ON FUNCTION public.auth_lookup_user_id_by_session(bytea) FROM PUBLIC;

CREATE FUNCTION public.auth_delete_session_by_token(p_token_hash bytea) RETURNS void
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER SET search_path = public, pg_temp
AS $$
BEGIN
    DELETE FROM public.sessions WHERE token_hash = p_token_hash;
END;
$$;

REVOKE ALL ON FUNCTION public.auth_delete_session_by_token(bytea) FROM PUBLIC;

CREATE FUNCTION public.login_notify_peek_by_token(p_token_hash bytea)
    RETURNS TABLE(user_id uuid, expires_at timestamptz)
    LANGUAGE sql STABLE SECURITY DEFINER SET search_path = public, pg_temp
AS $$ SELECT user_id, expires_at FROM public.login_notify_tokens WHERE token_hash = p_token_hash AND used_at IS NULL AND expires_at > NOW() $$;

REVOKE ALL ON FUNCTION public.login_notify_peek_by_token(bytea) FROM PUBLIC;

CREATE FUNCTION public.login_notify_consume_by_token(p_token_hash bytea) RETURNS uuid
    LANGUAGE sql VOLATILE SECURITY DEFINER SET search_path = public, pg_temp
AS $$ UPDATE public.login_notify_tokens SET used_at = NOW() WHERE token_hash = p_token_hash AND used_at IS NULL AND expires_at > NOW() RETURNING user_id $$;

REVOKE ALL ON FUNCTION public.login_notify_consume_by_token(bytea) FROM PUBLIC;

CREATE FUNCTION public.auth_rotate_session(p_user_id uuid, p_token_hash bytea, p_device_hash bytea) RETURNS void
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER SET search_path = public, pg_temp
AS $$
BEGIN
    IF p_user_id::text IS DISTINCT FROM current_setting('app.current_user_id', true) THEN
        RAISE EXCEPTION '利用者の不一致';
    END IF;
    WITH expired AS (
        SELECT id FROM public.sessions WHERE expires_at <= NOW() FOR UPDATE SKIP LOCKED
    )
    DELETE FROM public.sessions WHERE id IN (SELECT id FROM expired);
    DELETE FROM public.sessions
    WHERE user_id = p_user_id
      AND (p_device_hash IS NULL OR device_hash = p_device_hash OR device_hash IS NULL);
    INSERT INTO public.sessions (user_id, token_hash, device_hash)
    VALUES (p_user_id, p_token_hash, p_device_hash);
END;
$$;

REVOKE ALL ON FUNCTION public.auth_rotate_session(uuid, bytea, bytea) FROM PUBLIC;

CREATE FUNCTION public.auth_upsert_user(p_google_id text, p_email text, p_name text, p_picture_url text) RETURNS uuid
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER SET search_path = public, pg_temp
AS $$
DECLARE
    v_id uuid;
BEGIN
    INSERT INTO public.users (google_id, email, name, picture_url)
    VALUES (p_google_id, p_email, p_name, p_picture_url)
    ON CONFLICT (google_id) DO UPDATE SET
        email = EXCLUDED.email,
        name = EXCLUDED.name,
        picture_url = EXCLUDED.picture_url,
        updated_at = NOW()
    RETURNING id INTO v_id;
    RETURN v_id;
END;
$$;

REVOKE ALL ON FUNCTION public.auth_upsert_user(text, text, text, text) FROM PUBLIC;

CREATE FUNCTION public.auth_session_user_by_token(p_token_hash bytea)
RETURNS TABLE(id uuid, google_id text, email text, name text, picture_url text, created_at timestamptz, updated_at timestamptz)
    LANGUAGE sql STABLE SECURITY DEFINER SET search_path = public, pg_temp
AS $$
SELECT u.id, u.google_id, u.email, u.name, u.picture_url, u.created_at, u.updated_at
FROM public.users u
INNER JOIN public.sessions s ON u.id = s.user_id
WHERE s.token_hash = p_token_hash AND s.expires_at > NOW()
$$;

REVOKE ALL ON FUNCTION public.auth_session_user_by_token(bytea) FROM PUBLIC;

GRANT EXECUTE ON FUNCTION public.auth_lookup_user_id_by_session(bytea),
    public.auth_delete_session_by_token(bytea),
    public.login_notify_peek_by_token(bytea),
    public.login_notify_consume_by_token(bytea),
    public.auth_rotate_session(uuid, bytea, bytea),
    public.auth_upsert_user(text, text, text, text),
    public.auth_session_user_by_token(bytea)
    TO app_restricted;

GRANT CREATE ON SCHEMA public TO app_notify_token_v2_executor,
    app_admin_login_executor, app_price_target_executor, app_login_event_prune_executor,
    app_shared_write_executor;

CREATE FUNCTION public.login_notify_peek_v2(p_token_hash bytea)
    RETURNS TABLE(user_id uuid, expires_at timestamptz)
    LANGUAGE sql STABLE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    SET row_security = off
AS $$
    SELECT t.user_id, t.expires_at
    FROM public.login_notify_tokens t
    WHERE t.token_hash = p_token_hash
      AND t.used_at IS NULL
      AND t.expires_at > pg_catalog.now()
$$;
REVOKE ALL ON FUNCTION public.login_notify_peek_v2(bytea) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.login_notify_peek_v2(bytea) TO app_restricted;
ALTER FUNCTION public.login_notify_peek_v2(bytea) OWNER TO app_notify_token_v2_executor;

CREATE FUNCTION public.login_notify_consume_v2(p_token_hash bytea)
    RETURNS uuid
    LANGUAGE sql VOLATILE SECURITY DEFINER
    SET search_path = pg_catalog, public, pg_temp
    SET row_security = off
AS $$
    UPDATE public.login_notify_tokens t
    SET used_at = pg_catalog.now()
    WHERE t.token_hash = p_token_hash
      AND t.used_at IS NULL
      AND t.expires_at > pg_catalog.now()
    RETURNING t.user_id
$$;
REVOKE ALL ON FUNCTION public.login_notify_consume_v2(bytea) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.login_notify_consume_v2(bytea) TO app_restricted;
ALTER FUNCTION public.login_notify_consume_v2(bytea) OWNER TO app_notify_token_v2_executor;

CREATE FUNCTION public.admin_login_search_data_v2(
    p_query_patterns text[], p_email_patterns text[], p_place_patterns text[],
    p_sort_by text, p_sort_order text, p_limit bigint, p_offset bigint
)
RETURNS TABLE(occurred_at timestamptz, email text, result text,
    country text, region text, browser text, os text)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    SELECT le.occurred_at, u.email::text, le.result, le.country, le.region, le.browser, le.os
    FROM public.login_events le
    JOIN public.users u ON u.id = le.user_id
    WHERE NOT EXISTS (
        SELECT 1 FROM pg_catalog.unnest(p_query_patterns) q(pattern)
        WHERE NOT (u.email::text ILIKE q.pattern ESCAPE '\'
            OR pg_catalog.concat_ws(' ', le.country, le.region) ILIKE q.pattern ESCAPE '\')
    )
      AND NOT EXISTS (
        SELECT 1 FROM pg_catalog.unnest(p_email_patterns) q(pattern)
        WHERE u.email::text NOT ILIKE q.pattern ESCAPE '\'
    )
      AND NOT EXISTS (
        SELECT 1 FROM pg_catalog.unnest(p_place_patterns) q(pattern)
        WHERE pg_catalog.concat_ws(' ', le.country, le.region) NOT ILIKE q.pattern ESCAPE '\'
    )
    ORDER BY
        CASE WHEN p_sort_by = 'occurred_at' AND p_sort_order = 'asc' THEN le.occurred_at END ASC,
        CASE WHEN p_sort_by = 'occurred_at' AND p_sort_order = 'desc' THEN le.occurred_at END DESC,
        CASE WHEN p_sort_by = 'email' AND p_sort_order = 'asc' THEN u.email END ASC,
        CASE WHEN p_sort_by = 'email' AND p_sort_order = 'desc' THEN u.email END DESC,
        CASE WHEN p_sort_by = 'result' AND p_sort_order = 'asc' THEN le.result END ASC,
        CASE WHEN p_sort_by = 'result' AND p_sort_order = 'desc' THEN le.result END DESC,
        CASE WHEN p_sort_by = 'place' AND p_sort_order = 'asc' THEN pg_catalog.concat_ws(' ', le.country, le.region) END ASC,
        CASE WHEN p_sort_by = 'place' AND p_sort_order = 'desc' THEN pg_catalog.concat_ws(' ', le.country, le.region) END DESC,
        le.occurred_at DESC, le.id DESC
    LIMIT p_limit OFFSET p_offset
$$;
REVOKE ALL ON FUNCTION public.admin_login_search_data_v2(text[], text[], text[], text, text, bigint, bigint)
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.admin_login_search_data_v2(text[], text[], text[], text, text, bigint, bigint)
    OWNER TO app_admin_login_executor;

CREATE FUNCTION public.admin_login_search_count_v2(
    p_query_patterns text[], p_email_patterns text[], p_place_patterns text[]
)
RETURNS bigint
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    SELECT pg_catalog.count(*)
    FROM public.login_events le
    JOIN public.users u ON u.id = le.user_id
    WHERE NOT EXISTS (
        SELECT 1 FROM pg_catalog.unnest(p_query_patterns) q(pattern)
        WHERE NOT (u.email::text ILIKE q.pattern ESCAPE '\'
            OR pg_catalog.concat_ws(' ', le.country, le.region) ILIKE q.pattern ESCAPE '\')
    )
      AND NOT EXISTS (
        SELECT 1 FROM pg_catalog.unnest(p_email_patterns) q(pattern)
        WHERE u.email::text NOT ILIKE q.pattern ESCAPE '\'
    )
      AND NOT EXISTS (
        SELECT 1 FROM pg_catalog.unnest(p_place_patterns) q(pattern)
        WHERE pg_catalog.concat_ws(' ', le.country, le.region) NOT ILIKE q.pattern ESCAPE '\'
    )
$$;
REVOKE ALL ON FUNCTION public.admin_login_search_count_v2(text[], text[], text[])
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.admin_login_search_count_v2(text[], text[], text[])
    OWNER TO app_admin_login_executor;

CREATE FUNCTION public.scheduled_stock_price_targets_v2(p_target_date date, p_limit bigint)
RETURNS TABLE(security_code character varying)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    SELECT ab.security_code
    FROM public.asset_balances ab
    LEFT JOIN public.security_prices sp ON sp.security_code = ab.security_code
    GROUP BY ab.security_code
    HAVING pg_catalog.bool_or(sp.price_as_of IS NULL)
        OR pg_catalog.min(sp.price_as_of) < p_target_date
    ORDER BY pg_catalog.min(sp.updated_at) ASC NULLS FIRST
    LIMIT p_limit
$$;
REVOKE ALL ON FUNCTION public.scheduled_stock_price_targets_v2(date, bigint)
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.scheduled_stock_price_targets_v2(date, bigint) OWNER TO app_price_target_executor;

CREATE FUNCTION public.scheduled_fund_price_targets_v2(p_target_date date, p_limit bigint)
RETURNS TABLE(association_fund_cd character varying)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    SELECT fc.association_fund_cd
    FROM public.fund_balances fb
    JOIN public.fund_codes fc ON fc.normalized_name = fb.normalized_fund_name
    LEFT JOIN public.fund_prices fp ON fp.association_fund_cd = fc.association_fund_cd
    GROUP BY fc.association_fund_cd
    HAVING pg_catalog.bool_or(fp.price_as_of IS NULL)
        OR pg_catalog.min(fp.price_as_of) < p_target_date
    ORDER BY pg_catalog.min(fp.updated_at) ASC NULLS FIRST
    LIMIT p_limit
$$;
REVOKE ALL ON FUNCTION public.scheduled_fund_price_targets_v2(date, bigint)
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.scheduled_fund_price_targets_v2(date, bigint)
    OWNER TO app_price_target_executor;

CREATE FUNCTION public.scheduled_stock_price_mark_attempted_v2(p_codes text[])
RETURNS void
LANGUAGE sql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    INSERT INTO public.security_prices (security_code, updated_at)
    SELECT code, pg_catalog.now() FROM pg_catalog.unnest(p_codes) AS t(code)
    ON CONFLICT (security_code) DO UPDATE SET updated_at = pg_catalog.now()
$$;
REVOKE ALL ON FUNCTION public.scheduled_stock_price_mark_attempted_v2(text[])
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.scheduled_stock_price_mark_attempted_v2(text[])
    OWNER TO app_price_target_executor;

CREATE FUNCTION public.scheduled_stock_price_apply_v2(
    p_code text, p_current_price numeric, p_prev_close numeric, p_price_as_of date
)
RETURNS bigint
LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
DECLARE
    affected bigint;
BEGIN
    INSERT INTO public.security_prices (security_code, current_price, prev_close, price_as_of)
    VALUES (p_code, p_current_price, p_prev_close, p_price_as_of)
    ON CONFLICT (security_code) DO UPDATE SET
        current_price = EXCLUDED.current_price,
        prev_close = CASE
            WHEN security_prices.price_as_of = EXCLUDED.price_as_of
                THEN COALESCE(EXCLUDED.prev_close, security_prices.prev_close)
            ELSE EXCLUDED.prev_close
        END,
        price_as_of = EXCLUDED.price_as_of,
        updated_at = pg_catalog.now()
    WHERE security_prices.price_as_of IS NULL
       OR security_prices.price_as_of <= EXCLUDED.price_as_of;
    GET DIAGNOSTICS affected = ROW_COUNT;
    RETURN affected;
END
$$;
REVOKE ALL ON FUNCTION public.scheduled_stock_price_apply_v2(text, numeric, numeric, date)
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.scheduled_stock_price_apply_v2(text, numeric, numeric, date)
    OWNER TO app_price_target_executor;

CREATE FUNCTION public.scheduled_fund_price_mark_attempted_v2(p_codes text[])
RETURNS void
LANGUAGE sql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    INSERT INTO public.fund_prices (association_fund_cd, updated_at)
    SELECT code, pg_catalog.now() FROM pg_catalog.unnest(p_codes) AS t(code)
    ON CONFLICT (association_fund_cd) DO UPDATE SET updated_at = pg_catalog.now()
$$;
REVOKE ALL ON FUNCTION public.scheduled_fund_price_mark_attempted_v2(text[])
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.scheduled_fund_price_mark_attempted_v2(text[])
    OWNER TO app_price_target_executor;

CREATE FUNCTION public.scheduled_fund_price_apply_v2(
    p_code text, p_nav numeric, p_prev_day_change numeric, p_price_as_of date
)
RETURNS bigint
LANGUAGE plpgsql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
DECLARE
    affected bigint;
BEGIN
    INSERT INTO public.fund_prices (association_fund_cd, nav, prev_day_change, price_as_of)
    VALUES (p_code, p_nav, p_prev_day_change, p_price_as_of)
    ON CONFLICT (association_fund_cd) DO UPDATE SET
        nav = EXCLUDED.nav,
        prev_day_change = EXCLUDED.prev_day_change,
        price_as_of = EXCLUDED.price_as_of,
        updated_at = pg_catalog.now(),
        fetch_error = NULL,
        fetch_failed_at = NULL
    WHERE fund_prices.price_as_of IS NULL
       OR fund_prices.price_as_of <= EXCLUDED.price_as_of;
    GET DIAGNOSTICS affected = ROW_COUNT;
    RETURN affected;
END
$$;
REVOKE ALL ON FUNCTION public.scheduled_fund_price_apply_v2(text, numeric, numeric, date)
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.scheduled_fund_price_apply_v2(text, numeric, numeric, date)
    OWNER TO app_price_target_executor;

CREATE FUNCTION public.scheduled_fund_price_record_failure_v2(p_code text, p_reason text)
RETURNS void
LANGUAGE sql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    INSERT INTO public.fund_prices (association_fund_cd, updated_at, fetch_error, fetch_failed_at)
    VALUES (p_code, pg_catalog.now(), p_reason, pg_catalog.now())
    ON CONFLICT (association_fund_cd) DO UPDATE SET
        fetch_error = EXCLUDED.fetch_error,
        fetch_failed_at = EXCLUDED.fetch_failed_at
$$;
REVOKE ALL ON FUNCTION public.scheduled_fund_price_record_failure_v2(text, text)
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.scheduled_fund_price_record_failure_v2(text, text)
    OWNER TO app_price_target_executor;

CREATE FUNCTION public.scheduled_fund_price_clear_failure_v2(p_code text)
RETURNS void
LANGUAGE sql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    UPDATE public.fund_prices
       SET fetch_error = NULL, fetch_failed_at = NULL
     WHERE association_fund_cd = p_code
$$;
REVOKE ALL ON FUNCTION public.scheduled_fund_price_clear_failure_v2(text)
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.scheduled_fund_price_clear_failure_v2(text)
    OWNER TO app_price_target_executor;

CREATE FUNCTION public.prune_login_events_older_than_90_days_v2()
RETURNS bigint
LANGUAGE sql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    WITH deleted AS (
        DELETE FROM public.login_events
        WHERE occurred_at < pg_catalog.now() - INTERVAL '90 days'
        RETURNING 1
    )
    SELECT pg_catalog.count(*) FROM deleted
$$;
REVOKE ALL ON FUNCTION public.prune_login_events_older_than_90_days_v2()
    FROM PUBLIC, app_restricted;
ALTER FUNCTION public.prune_login_events_older_than_90_days_v2()
    OWNER TO app_login_event_prune_executor;

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

REVOKE CREATE ON SCHEMA public FROM app_notify_token_v2_executor,
    app_admin_login_executor, app_price_target_executor, app_login_event_prune_executor,
    app_shared_write_executor;

-- EXECUTE は所有ロールから app_worker と移行ロール(SESSION_USER)へ付ける。
-- ログイン通知の *_v2 は利用者 Tx 内の app_restricted から呼ぶため app_worker には付けない。
SET ROLE app_notify_token_v2_executor;
GRANT EXECUTE ON FUNCTION public.login_notify_peek_v2(bytea),
    public.login_notify_consume_v2(bytea)
    TO app_notify_token_v2_executor;
DO $$
BEGIN
    EXECUTE format(
        'GRANT EXECUTE ON FUNCTION public.login_notify_peek_v2(bytea), public.login_notify_consume_v2(bytea) TO %I',
        SESSION_USER
    );
END
$$;
RESET ROLE;

SET ROLE app_admin_login_executor;
GRANT EXECUTE ON FUNCTION public.admin_login_search_data_v2(text[], text[], text[], text, text, bigint, bigint),
    public.admin_login_search_count_v2(text[], text[], text[])
    TO app_admin_login_executor, app_worker;
DO $$
BEGIN
    EXECUTE format(
        'GRANT EXECUTE ON FUNCTION public.admin_login_search_data_v2(text[], text[], text[], text, text, bigint, bigint), public.admin_login_search_count_v2(text[], text[], text[]) TO %I',
        SESSION_USER
    );
END
$$;
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
DO $$
BEGIN
    EXECUTE format(
        'GRANT EXECUTE ON FUNCTION public.scheduled_stock_price_targets_v2(date, bigint), public.scheduled_fund_price_targets_v2(date, bigint), public.scheduled_stock_price_mark_attempted_v2(text[]), public.scheduled_stock_price_apply_v2(text, numeric, numeric, date), public.scheduled_fund_price_mark_attempted_v2(text[]), public.scheduled_fund_price_apply_v2(text, numeric, numeric, date), public.scheduled_fund_price_record_failure_v2(text, text), public.scheduled_fund_price_clear_failure_v2(text) TO %I',
        SESSION_USER
    );
END
$$;
RESET ROLE;

SET ROLE app_login_event_prune_executor;
GRANT EXECUTE ON FUNCTION public.prune_login_events_older_than_90_days_v2()
    TO app_login_event_prune_executor, app_worker;
DO $$
BEGIN
    EXECUTE format(
        'GRANT EXECUTE ON FUNCTION public.prune_login_events_older_than_90_days_v2() TO %I',
        SESSION_USER
    );
END
$$;
RESET ROLE;

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
