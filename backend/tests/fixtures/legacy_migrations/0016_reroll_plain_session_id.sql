UPDATE sessions
SET id = gen_random_uuid()
WHERE token_hash = sha256(convert_to(id::text, 'UTF8'));
