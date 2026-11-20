-- contract: login_notify_peek/consume_by_token は 0118 で EXECUTE を外したあと、
-- コードも DB の ACL も使っていない。所有者だけが持つ関数を削除する。
-- 古い Worker も *_v2 だけを呼ぶため、削除しても旧 Worker の動作は変わらない。
SET LOCAL lock_timeout = '5s';

DROP FUNCTION public.login_notify_peek_by_token(bytea);
DROP FUNCTION public.login_notify_consume_by_token(bytea);
