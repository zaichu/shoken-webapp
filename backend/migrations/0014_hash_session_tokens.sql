-- 段階的移行の第1段: 既存行の id はそのまま残し、token_hash を NULL 可で追加して埋める。
-- ローリングデプロイ中も旧版が id で照合・発行できるよう、平文の列は消さず NOT NULL も付けない。
-- 平文の削除と token_hash の必須化は、全インスタンスが新版になった後の migration で行う。
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS token_hash BYTEA;

UPDATE sessions
SET token_hash = sha256(convert_to(id::text, 'UTF8'))
WHERE token_hash IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_token_hash ON sessions (token_hash);

DELETE FROM sessions WHERE expires_at <= NOW();
