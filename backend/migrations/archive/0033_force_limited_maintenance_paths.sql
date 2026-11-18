DO $$
BEGIN
    CREATE ROLE app_notify_token_v2_executor NOLOGIN BYPASSRLS;
EXCEPTION WHEN duplicate_object THEN
    ALTER ROLE app_notify_token_v2_executor NOLOGIN BYPASSRLS;
END
$$;

DO $$
BEGIN
    CREATE ROLE app_admin_login_executor NOLOGIN BYPASSRLS;
EXCEPTION WHEN duplicate_object THEN
    ALTER ROLE app_admin_login_executor NOLOGIN BYPASSRLS;
END
$$;

DO $$
BEGIN
    CREATE ROLE app_price_target_executor NOLOGIN BYPASSRLS;
EXCEPTION WHEN duplicate_object THEN
    ALTER ROLE app_price_target_executor NOLOGIN BYPASSRLS;
END
$$;

DO $$
BEGIN
    CREATE ROLE app_login_event_prune_executor NOLOGIN BYPASSRLS;
EXCEPTION WHEN duplicate_object THEN
    ALTER ROLE app_login_event_prune_executor NOLOGIN BYPASSRLS;
END
$$;

GRANT app_notify_token_v2_executor, app_admin_login_executor,
    app_price_target_executor, app_login_event_prune_executor
    TO CURRENT_USER WITH SET TRUE, INHERIT FALSE;
GRANT CREATE ON SCHEMA public TO app_notify_token_v2_executor,
    app_admin_login_executor, app_price_target_executor, app_login_event_prune_executor;

GRANT USAGE ON SCHEMA public TO app_notify_token_v2_executor;
GRANT SELECT (token_hash, user_id, expires_at, used_at), UPDATE (used_at)
    ON public.login_notify_tokens TO app_notify_token_v2_executor;
GRANT USAGE ON SCHEMA public TO app_admin_login_executor;
GRANT SELECT (id, occurred_at, user_id, result, country, region, browser, os)
    ON public.login_events TO app_admin_login_executor;
GRANT SELECT (id, email) ON public.users TO app_admin_login_executor;
GRANT USAGE ON SCHEMA public TO app_price_target_executor;
GRANT SELECT (security_code) ON public.asset_balances TO app_price_target_executor;
GRANT SELECT (security_code, current_price, prev_close, price_as_of, updated_at)
    ON public.security_prices TO app_price_target_executor;
GRANT INSERT, UPDATE ON public.security_prices TO app_price_target_executor;
GRANT SELECT (normalized_fund_name) ON public.fund_balances TO app_price_target_executor;
GRANT SELECT (normalized_name, association_fund_cd)
    ON public.fund_codes TO app_price_target_executor;
GRANT SELECT (association_fund_cd, nav, prev_day_change, price_as_of, updated_at, fetch_error, fetch_failed_at)
    ON public.fund_prices TO app_price_target_executor;
GRANT INSERT, UPDATE ON public.fund_prices TO app_price_target_executor;
GRANT USAGE ON SCHEMA public TO app_login_event_prune_executor;
GRANT SELECT (occurred_at), DELETE ON public.login_events TO app_login_event_prune_executor;

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

REVOKE CREATE ON SCHEMA public FROM app_notify_token_v2_executor,
    app_admin_login_executor, app_price_target_executor, app_login_event_prune_executor;
