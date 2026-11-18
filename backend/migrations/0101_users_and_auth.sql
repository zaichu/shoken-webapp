-- 利用者と認証: users・sessions・login_events・admin_login_views・login_notify_tokens。
-- 利用者の表は app.current_user_id と一致する行だけを許すポリシーを持つ。
-- 列側を変換しない式にして user_id 先頭の索引を使えるようにし、NULLIF で接続に残った空文字を 0 件として扱う。

CREATE TABLE public.users (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    google_id character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    name character varying(255),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    login_notify_enabled boolean DEFAULT true NOT NULL,
    picture_url character varying(2048),
    CONSTRAINT users_pkey PRIMARY KEY (id),
    CONSTRAINT users_google_id_key UNIQUE (google_id)
);

COMMENT ON TABLE public.users IS 'Google 認証のユーザー';
COMMENT ON COLUMN public.users.login_notify_enabled IS '新しい端末・場所変化のログイン通知を受け取るか';
COMMENT ON COLUMN public.users.picture_url IS 'Google プロフィール画像の URL(googleusercontent の https のみ。未設定・他ホストは NULL)';

ALTER TABLE public.users ENABLE ROW LEVEL SECURITY;
CREATE POLICY users_isolation ON public.users
    USING (id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));
COMMENT ON POLICY users_isolation ON public.users
    IS '接続ごとの app.current_user_id と id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';


CREATE TABLE public.sessions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone DEFAULT (now() + '7 days'::interval) NOT NULL,
    token_hash bytea NOT NULL,
    device_hash bytea,
    CONSTRAINT sessions_pkey PRIMARY KEY (id),
    CONSTRAINT sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE
);

CREATE INDEX idx_sessions_expires_at ON public.sessions USING btree (expires_at);
CREATE UNIQUE INDEX idx_sessions_token_hash ON public.sessions USING btree (token_hash);
CREATE INDEX idx_sessions_user_id ON public.sessions USING btree (user_id);
CREATE INDEX idx_sessions_user_device ON public.sessions USING btree (user_id, device_hash);

COMMENT ON TABLE public.sessions IS 'ログインセッション';
COMMENT ON COLUMN public.sessions.device_hash IS 'ログイン時の端末IDのSHA-256(平文は保存しない)';

ALTER TABLE public.sessions ENABLE ROW LEVEL SECURITY;
CREATE POLICY sessions_isolation ON public.sessions
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));
COMMENT ON POLICY sessions_isolation ON public.sessions
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';


CREATE TABLE public.login_events (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    occurred_at timestamp with time zone DEFAULT now() NOT NULL,
    result text NOT NULL,
    device_hash bytea NOT NULL,
    country text,
    region text,
    browser text,
    os text,
    CONSTRAINT login_events_pkey PRIMARY KEY (id),
    CONSTRAINT login_events_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE,
    CONSTRAINT login_events_result_check CHECK (result IN ('success', 'new_device', 'location_changed', 'failure'))
);

CREATE INDEX idx_login_events_user_occurred ON public.login_events USING btree (user_id, occurred_at DESC);
CREATE INDEX idx_login_events_occurred ON public.login_events USING btree (occurred_at);

COMMENT ON TABLE public.login_events IS 'ログインの記録(端末IDはハッシュのみ。IPは保存しない)';
COMMENT ON COLUMN public.login_events.result IS '成功・新しい端末・場所の変化・失敗';
COMMENT ON COLUMN public.login_events.device_hash IS '端末IDのSHA-256(平文は保存しない)';
COMMENT ON COLUMN public.login_events.country IS 'Cloudflare由来の国';
COMMENT ON COLUMN public.login_events.region IS 'Cloudflare由来の都道府県相当';

ALTER TABLE public.login_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY login_events_isolation ON public.login_events
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));
COMMENT ON POLICY login_events_isolation ON public.login_events
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';


CREATE TABLE public.admin_login_views (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    occurred_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT admin_login_views_pkey PRIMARY KEY (id),
    CONSTRAINT admin_login_views_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE
);

CREATE INDEX idx_admin_login_views_user_occurred ON public.admin_login_views USING btree (user_id, occurred_at DESC);

COMMENT ON TABLE public.admin_login_views IS '管理者によるログイン一覧の閲覧監査（閲覧者と日時のみ）';

ALTER TABLE public.admin_login_views ENABLE ROW LEVEL SECURITY;
CREATE POLICY admin_login_views_isolation ON public.admin_login_views
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));
COMMENT ON POLICY admin_login_views_isolation ON public.admin_login_views
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';


CREATE TABLE public.login_notify_tokens (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    token_hash bytea NOT NULL,
    expires_at timestamp with time zone DEFAULT (now() + '24:00:00'::interval) NOT NULL,
    used_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT login_notify_tokens_pkey PRIMARY KEY (id),
    CONSTRAINT login_notify_tokens_token_hash_key UNIQUE (token_hash),
    CONSTRAINT login_notify_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE
);

CREATE INDEX idx_login_notify_tokens_user_created ON public.login_notify_tokens USING btree (user_id, created_at DESC);

COMMENT ON TABLE public.login_notify_tokens IS 'ログイン通知メール内の全端末ログアウト用使い捨てトークン(平文は保存しない)';

ALTER TABLE public.login_notify_tokens ENABLE ROW LEVEL SECURITY;
CREATE POLICY login_notify_tokens_isolation ON public.login_notify_tokens
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));
COMMENT ON POLICY login_notify_tokens_isolation ON public.login_notify_tokens
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';
