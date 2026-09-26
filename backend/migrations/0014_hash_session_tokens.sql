-- ローリングデプロイ中も旧版が id で照合・発行できるよう、既存行の id は残し、
-- token_hash を NULL 可で追加して埋める。
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS token_hash BYTEA;

UPDATE sessions
SET token_hash = sha256(convert_to(id::text, 'UTF8'))
WHERE token_hash IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_token_hash ON sessions (token_hash);

DELETE FROM sessions WHERE expires_at <= NOW();
