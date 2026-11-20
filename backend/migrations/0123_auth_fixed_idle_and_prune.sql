-- 無操作期限の迂回防止・認証の1回化・上限・掃除の expand。
-- 古い期限付き認証関数の削除は別 PR の contract で行う(デプロイ中の旧 Worker が使うため)。未使用トークン関数の消去は 0124 で行う。
SET LOCAL lock_timeout = '5s';

CREATE FUNCTION public.auth_session_user_by_token_v2(p_token_hash bytea)
RETURNS TABLE(id uuid, google_id text, email text, name text, picture_url text, created_at timestamptz, updated_at timestamptz)
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER SET search_path = public, pg_temp
AS $$
DECLARE
    v_user_id uuid;
    v_last_used_at timestamptz;
BEGIN
    SELECT s.user_id, s.last_used_at INTO v_user_id, v_last_used_at
    FROM public.sessions s
    WHERE s.token_hash = p_token_hash
      AND s.expires_at > NOW()
      AND s.last_used_at > NOW() - INTERVAL '30 minutes';

    IF v_user_id IS NULL THEN
        RETURN;
    END IF;

    IF v_last_used_at < NOW() - INTERVAL '1 minute' THEN
        UPDATE public.sessions
        SET last_used_at = NOW()
        WHERE token_hash = p_token_hash
          AND last_used_at < NOW() - INTERVAL '1 minute';
    END IF;

    RETURN QUERY
    SELECT u.id, u.google_id::text, u.email::text, u.name::text, u.picture_url::text, u.created_at, u.updated_at
    FROM public.users u
    WHERE u.id = v_user_id;
END;
$$;

REVOKE ALL ON FUNCTION public.auth_session_user_by_token_v2(bytea) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.auth_session_user_by_token_v2(bytea) TO app_restricted;

CREATE FUNCTION public.auth_lookup_user_id_by_session_v2(p_token_hash bytea) RETURNS uuid
    LANGUAGE sql STABLE SECURITY DEFINER SET search_path = public, pg_temp
AS $$
    SELECT user_id FROM public.sessions
    WHERE token_hash = p_token_hash
      AND expires_at > NOW()
      AND last_used_at > NOW() - INTERVAL '30 minutes'
$$;

REVOKE ALL ON FUNCTION public.auth_lookup_user_id_by_session_v2(bytea) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.auth_lookup_user_id_by_session_v2(bytea) TO app_restricted;

GRANT CREATE ON SCHEMA public TO app_admin_login_executor;
SET ROLE app_admin_login_executor;
CREATE OR REPLACE FUNCTION public.admin_login_search_data_v2(
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
    LIMIT LEAST(GREATEST(p_limit, 0), 200) OFFSET p_offset
$$;
RESET ROLE;
REVOKE CREATE ON SCHEMA public FROM app_admin_login_executor;

GRANT SELECT (created_at, expires_at, used_at), DELETE ON public.login_notify_tokens TO app_login_event_prune_executor;
GRANT SELECT (occurred_at), DELETE ON public.admin_login_views TO app_login_event_prune_executor;

CREATE FUNCTION public.prune_login_notify_tokens_and_admin_views_v2()
RETURNS bigint
LANGUAGE sql VOLATILE SECURITY DEFINER
SET search_path = pg_catalog, public, pg_temp
SET row_security = off
AS $$
    WITH deleted_tokens AS (
        DELETE FROM public.login_notify_tokens
        WHERE (used_at IS NOT NULL OR expires_at < pg_catalog.now())
          AND created_at < pg_catalog.now() - INTERVAL '90 days'
        RETURNING 1
    ), deleted_views AS (
        DELETE FROM public.admin_login_views
        WHERE occurred_at < pg_catalog.now() - INTERVAL '1 year'
        RETURNING 1
    )
    SELECT (SELECT pg_catalog.count(*) FROM deleted_tokens)
        + (SELECT pg_catalog.count(*) FROM deleted_views)
$$;
REVOKE ALL ON FUNCTION public.prune_login_notify_tokens_and_admin_views_v2()
    FROM PUBLIC, app_restricted;
GRANT CREATE ON SCHEMA public TO app_login_event_prune_executor;
ALTER FUNCTION public.prune_login_notify_tokens_and_admin_views_v2()
    OWNER TO app_login_event_prune_executor;
REVOKE CREATE ON SCHEMA public FROM app_login_event_prune_executor;

SET ROLE app_login_event_prune_executor;
GRANT EXECUTE ON FUNCTION public.prune_login_notify_tokens_and_admin_views_v2()
    TO app_login_event_prune_executor, app_worker;
DO $$
BEGIN
    EXECUTE format(
        'GRANT EXECUTE ON FUNCTION public.prune_login_notify_tokens_and_admin_views_v2() TO %I',
        SESSION_USER
    );
END
$$;
RESET ROLE;
