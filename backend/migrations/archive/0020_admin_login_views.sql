CREATE TABLE admin_login_views (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_admin_login_views_user_occurred ON admin_login_views (user_id, occurred_at DESC);

COMMENT ON TABLE admin_login_views IS '管理者によるログイン一覧の閲覧監査（閲覧者と日時のみ）';
