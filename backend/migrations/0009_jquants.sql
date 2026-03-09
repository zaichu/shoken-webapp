-- jquants_dividend_cache: dividend_per_share は意図的に DOUBLE PRECISION のまま
CREATE TABLE IF NOT EXISTS jquants_dividend_cache (
    security_code      VARCHAR(10)      PRIMARY KEY,
    dividend_per_share DOUBLE PRECISION,
    status             VARCHAR(20)      NOT NULL DEFAULT 'pending',
    fetched_at         TIMESTAMPTZ,
    source             VARCHAR(50)      NOT NULL DEFAULT 'jquants',
    error_message      TEXT,
    stale_at           TIMESTAMPTZ,
    created_at         TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS jquants_rate_control (
    id                INTEGER PRIMARY KEY,
    next_available_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT single_row CHECK (id = 1)
);
INSERT INTO jquants_rate_control (id, next_available_at)
    VALUES (1, NOW())
    ON CONFLICT (id) DO NOTHING;
