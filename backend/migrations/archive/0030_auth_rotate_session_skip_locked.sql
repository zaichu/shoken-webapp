-- auth_rotate_session の期限切れ掃除が他者のロック待ちで停滞しないよう SKIP LOCKED に戻す。
-- 署名・所有者・権限は変えないため CREATE OR REPLACE で置き換える。
CREATE OR REPLACE FUNCTION public.auth_rotate_session(p_user_id uuid, p_token_hash bytea, p_device_hash bytea) RETURNS void
    LANGUAGE plpgsql VOLATILE SECURITY DEFINER SET search_path = public, pg_temp
AS $$
BEGIN
    IF p_user_id::text IS DISTINCT FROM current_setting('app.current_user_id', true) THEN
        RAISE EXCEPTION '利用者の不一致';
    END IF;
    WITH expired AS (
        SELECT id FROM public.sessions WHERE expires_at <= NOW() FOR UPDATE SKIP LOCKED
    )
    DELETE FROM public.sessions WHERE id IN (SELECT id FROM expired);
    DELETE FROM public.sessions
    WHERE user_id = p_user_id
      AND (p_device_hash IS NULL OR device_hash = p_device_hash OR device_hash IS NULL);
    INSERT INTO public.sessions (user_id, token_hash, device_hash)
    VALUES (p_user_id, p_token_hash, p_device_hash);
END;
$$;
