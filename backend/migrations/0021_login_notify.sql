ALTER TABLE public.users
    ADD COLUMN login_notify_enabled BOOLEAN NOT NULL DEFAULT TRUE;

COMMENT ON COLUMN public.users.login_notify_enabled IS '新しい端末・場所変化のログイン通知を受け取るか';

CREATE TABLE login_notify_tokens (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash BYTEA NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '24 hours',
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_login_notify_tokens_user_created ON login_notify_tokens (user_id, created_at DESC);

COMMENT ON TABLE login_notify_tokens IS 'ログイン通知メール内の全端末ログアウト用使い捨てトークン(平文は保存しない)';
