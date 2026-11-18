CREATE TABLE login_events (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    result TEXT NOT NULL CHECK (result IN ('success', 'new_device', 'location_changed', 'failure')),
    device_hash BYTEA NOT NULL,
    country TEXT,
    region TEXT,
    browser TEXT,
    os TEXT
);

CREATE INDEX idx_login_events_user_occurred ON login_events (user_id, occurred_at DESC);

CREATE INDEX idx_login_events_occurred ON login_events (occurred_at);

COMMENT ON TABLE login_events IS 'ログインの記録(端末IDはハッシュのみ。IPは保存しない)';
COMMENT ON COLUMN login_events.result IS '成功・新しい端末・場所の変化・失敗';
COMMENT ON COLUMN login_events.device_hash IS '端末IDのSHA-256(平文は保存しない)';
COMMENT ON COLUMN login_events.country IS 'Cloudflare由来の国';
COMMENT ON COLUMN login_events.region IS 'Cloudflare由来の都道府県相当';
