CREATE TABLE IF NOT EXISTS users (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    google_id   VARCHAR(255) UNIQUE NOT NULL,
    email       VARCHAR(255) NOT NULL,
    name        VARCHAR(255),
    picture_url TEXT,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_users_google_id ON users (google_id);
CREATE INDEX IF NOT EXISTS idx_users_email ON users (email);
