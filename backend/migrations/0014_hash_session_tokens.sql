-- 既存行の id は Cookie の値そのものなので、ハッシュを取ったうえで id を振り直す
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS token_hash BYTEA;

UPDATE sessions
SET token_hash = sha256(convert_to(id::text, 'UTF8')),
    id = gen_random_uuid()
WHERE token_hash IS NULL;

ALTER TABLE sessions ALTER COLUMN token_hash SET NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_token_hash ON sessions (token_hash);

DELETE FROM sessions WHERE expires_at <= NOW();
