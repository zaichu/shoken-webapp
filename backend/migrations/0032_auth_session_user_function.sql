-- 5e-1:セッション認証の利用者行取得を引数と返却値を限定した照会関数へ移す。
-- token_hash だけを受け取り、期限切れでない本人行の最小列だけを返す。
-- STABLE/SECURITY DEFINER、search_path 固定、PUBLIC からは取り上げる。
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

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_restricted') THEN
        EXECUTE 'GRANT EXECUTE ON FUNCTION public.auth_session_user_by_token(bytea) TO app_restricted';
    ELSE
        RAISE NOTICE 'app_restricted がないため照会関数への EXECUTE 付与を省略';
    END IF;
END
$$;
