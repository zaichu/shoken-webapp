-- 利用者 ID 確定前の認証入口のため token_hash 照合と利用者行取得を所有者権限の関数に閉じ込める。
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
