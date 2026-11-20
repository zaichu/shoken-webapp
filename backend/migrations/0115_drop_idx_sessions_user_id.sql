-- no-transaction
-- contract: idx_sessions_user_device(user_id, device_hash) の先頭列で同じ検索ができるため重複する idx_sessions_user_id を外す。user_id 単独の等価検索は複合索引の先頭列で満たせる。
DROP INDEX CONCURRENTLY IF EXISTS public.idx_sessions_user_id;
