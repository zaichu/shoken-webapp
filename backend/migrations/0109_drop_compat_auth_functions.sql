SET LOCAL lock_timeout = '5s';

DROP FUNCTION public.auth_session_user_by_token(bytea);
DROP FUNCTION public.auth_lookup_user_id_by_session(bytea);
