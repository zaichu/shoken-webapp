ALTER TABLE sessions ADD COLUMN device_hash BYTEA;
CREATE INDEX idx_sessions_user_device ON sessions (user_id, device_hash);
COMMENT ON COLUMN sessions.device_hash IS 'ログイン時の端末IDのSHA-256(平文は保存しない)';
