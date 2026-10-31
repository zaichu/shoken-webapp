ALTER TABLE public.users
    ADD COLUMN picture_url character varying(2048);

COMMENT ON COLUMN public.users.picture_url IS 'Google プロフィール画像の URL(googleusercontent の https のみ。未設定・他ホストは NULL)';
