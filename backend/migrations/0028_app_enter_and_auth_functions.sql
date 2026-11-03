-- 5b:app_enter(uuid)(INVOKER)と認証前の照合用 SECURITY DEFINER 関数群。アプリはまだ呼ばない。
CREATE FUNCTION public.app_enter(uid uuid) RETURNS void
    LANGUAGE plpgsql SECURITY INVOKER SET search_path = public, pg_temp
AS $$
BEGIN
    PERFORM set_config('role', 'app_restricted', true);
    PERFORM set_config('app.current_user_id', uid::text, true);
END;
$$;

REVOKE ALL ON FUNCTION public.app_enter(uuid) FROM PUBLIC;

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
    DELETE FROM public.sessions WHERE expires_at <= NOW();
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

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_restricted') THEN
        EXECUTE 'GRANT EXECUTE ON FUNCTION public.auth_lookup_user_id_by_session(bytea) TO app_restricted';
        EXECUTE 'GRANT EXECUTE ON FUNCTION public.auth_delete_session_by_token(bytea) TO app_restricted';
        EXECUTE 'GRANT EXECUTE ON FUNCTION public.login_notify_peek_by_token(bytea) TO app_restricted';
        EXECUTE 'GRANT EXECUTE ON FUNCTION public.login_notify_consume_by_token(bytea) TO app_restricted';
        EXECUTE 'GRANT EXECUTE ON FUNCTION public.auth_rotate_session(uuid, bytea, bytea) TO app_restricted';
        EXECUTE 'GRANT EXECUTE ON FUNCTION public.auth_upsert_user(text, text, text, text) TO app_restricted';
    ELSE
        RAISE NOTICE 'app_restricted がないため照会関数への EXECUTE 付与を省略';
    END IF;
END
$$;
