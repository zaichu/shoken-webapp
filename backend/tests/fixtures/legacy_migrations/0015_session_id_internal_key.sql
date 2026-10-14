UPDATE sessions
SET token_hash = sha256(convert_to(id::text, 'UTF8'))
WHERE token_hash IS NULL;

ALTER TABLE sessions ALTER COLUMN token_hash SET NOT NULL;

-- id は Cookie の値と同じ平文なので振り直す。列はローリングデプロイ中の旧版が参照するため残す
UPDATE sessions
SET id = gen_random_uuid()
WHERE token_hash = sha256(convert_to(id::text, 'UTF8'));
