-- 旧 login_notify_*_by_token はコードが *_v2 だけを呼ぶため app_restricted の EXECUTE を外す。
-- auth_upsert_user は現行コードが利用者 Tx 内で呼ぶため残す。関数は消さない(権限の縮小のみ)。
SET LOCAL lock_timeout = '5s';

REVOKE EXECUTE ON FUNCTION public.login_notify_peek_by_token(bytea) FROM app_restricted;
REVOKE EXECUTE ON FUNCTION public.login_notify_consume_by_token(bytea) FROM app_restricted;
