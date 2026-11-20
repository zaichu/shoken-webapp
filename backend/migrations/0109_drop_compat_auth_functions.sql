-- contract: #2100 で残した旧 Worker 互換の認証関数を削除する(#2102)。#2100 のデプロイ完了後で、旧関数を呼ぶコードは main にない。
SET LOCAL lock_timeout = '5s';

DROP FUNCTION public.auth_session_user_by_token(bytea);
DROP FUNCTION public.auth_lookup_user_id_by_session(bytea);
