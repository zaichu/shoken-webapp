SET LOCAL lock_timeout = '5s';

ALTER TABLE public.sessions
    ADD COLUMN last_used_at timestamp with time zone;

UPDATE public.sessions
SET last_used_at = created_at;

ALTER TABLE public.sessions
    ALTER COLUMN last_used_at SET DEFAULT now(),
    ALTER COLUMN last_used_at SET NOT NULL;

COMMENT ON COLUMN public.sessions.last_used_at IS '最後に利用した時刻。セッション作成時に開始し、API利用時に最大1分ごとに更新';

DROP FUNCTION public.auth_session_user_by_token(bytea);

CREATE FUNCTION public.auth_session_user_by_token(
    p_token_hash bytea,
    p_idle_cutoff timestamptz,
    p_touch_session boolean
)
RETURNS TABLE(id uuid, google_id text, email text, name text, picture_url text, created_at timestamptz, updated_at timestamptz)
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER SET search_path = public, pg_temp
AS $$
DECLARE
    v_user_id uuid;
    v_last_used_at timestamptz;
BEGIN
    SELECT user_id, last_used_at INTO v_user_id, v_last_used_at
    FROM public.sessions
    WHERE token_hash = p_token_hash
      AND expires_at > NOW()
      AND last_used_at > p_idle_cutoff
    FOR UPDATE;

    IF v_user_id IS NULL THEN
        RETURN;
    END IF;

    IF p_touch_session AND v_last_used_at < NOW() - INTERVAL '1 minute' THEN
        UPDATE public.sessions
        SET last_used_at = NOW()
        WHERE token_hash = p_token_hash;
    END IF;

    RETURN QUERY
    SELECT u.id, u.google_id::text, u.email::text, u.name::text, u.picture_url::text, u.created_at, u.updated_at
    FROM public.users u
    WHERE u.id = v_user_id;
END;
$$;

REVOKE ALL ON FUNCTION public.auth_session_user_by_token(bytea, timestamptz, boolean) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.auth_session_user_by_token(bytea, timestamptz, boolean) TO app_restricted;

DROP FUNCTION public.auth_lookup_user_id_by_session(bytea);

CREATE FUNCTION public.auth_lookup_user_id_by_session(
    p_token_hash bytea,
    p_idle_cutoff timestamptz
) RETURNS uuid
    LANGUAGE sql STABLE SECURITY DEFINER SET search_path = public, pg_temp
AS $$
    SELECT user_id FROM public.sessions
    WHERE token_hash = p_token_hash
      AND expires_at > NOW()
      AND last_used_at > p_idle_cutoff
$$;

REVOKE ALL ON FUNCTION public.auth_lookup_user_id_by_session(bytea, timestamptz) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.auth_lookup_user_id_by_session(bytea, timestamptz) TO app_restricted;
