CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS citext;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE IF NOT EXISTS stock (
    date                  DATE          NOT NULL,
    code                  VARCHAR(10)   NOT NULL,
    name                  CITEXT        NOT NULL,
    market_category       VARCHAR(50)   NOT NULL,
    industry_code_33      VARCHAR(10),
    industry_category_33  VARCHAR(100),
    industry_code_17      VARCHAR(10),
    industry_category_17  VARCHAR(100),
    size_code             VARCHAR(10),
    size_category         VARCHAR(50),
    PRIMARY KEY (date, code)
);

CREATE INDEX IF NOT EXISTS idx_stock_code_name ON stock (code, name);
CREATE INDEX IF NOT EXISTS idx_stock_name_trgm ON stock USING gin (name gin_trgm_ops);

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

CREATE TABLE IF NOT EXISTS sessions (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '7 days')
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions (expires_at);

CREATE TABLE IF NOT EXISTS dividends (
    id                    UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id               UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    settlement_date       DATE         NOT NULL,
    product               VARCHAR(100) NOT NULL,
    account               VARCHAR(100) NOT NULL,
    security_code         VARCHAR(10)  NOT NULL,
    security_name         VARCHAR(200) NOT NULL,
    unit_price            NUMERIC      NOT NULL,
    shares                NUMERIC      NOT NULL,
    dividends_before_tax  NUMERIC      NOT NULL,
    taxes                 NUMERIC      NOT NULL,
    net_amount_received   NUMERIC      NOT NULL,
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dividends_user_id ON dividends (user_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_dividends_unique
    ON dividends (user_id, settlement_date, security_code, security_name, shares, dividends_before_tax);

CREATE TABLE IF NOT EXISTS domestic_stocks (
    id                                 UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                            UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    trade_date                         DATE         NOT NULL,
    settlement_date                    DATE         NOT NULL,
    security_code                      VARCHAR(10)  NOT NULL,
    security_name                      VARCHAR(200) NOT NULL,
    account                            VARCHAR(100) NOT NULL,
    shares                             NUMERIC      NOT NULL,
    asked_price                        NUMERIC      NOT NULL,
    proceeds                           NUMERIC      NOT NULL,
    purchase_price                     NUMERIC      NOT NULL,
    realized_profit_and_loss           NUMERIC      NOT NULL,
    taxes                              NUMERIC      NOT NULL,
    realized_profit_and_loss_after_tax NUMERIC      NOT NULL,
    content_hash                       TEXT         NOT NULL,
    occurrence_index                   INTEGER      NOT NULL,
    created_at                         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at                         TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_domestic_stocks_user_id ON domestic_stocks (user_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_domestic_stocks_hash_occurrence
    ON domestic_stocks (user_id, content_hash, occurrence_index);

CREATE TABLE IF NOT EXISTS mutualfunds (
    id                                 UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                            UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    trade_date                         DATE         NOT NULL,
    settlement_date                    DATE         NOT NULL,
    fund_name                          VARCHAR(300) NOT NULL,
    dividends                          VARCHAR(100),
    account                            VARCHAR(100) NOT NULL,
    shares                             NUMERIC      NOT NULL,
    exchange_rate                      NUMERIC      NOT NULL,
    cancellation_unit_price_yen        NUMERIC      NOT NULL,
    cancellation_amount_yen            NUMERIC      NOT NULL,
    average_acquisition_price_yen      NUMERIC      NOT NULL,
    realized_profit_and_loss           NUMERIC      NOT NULL,
    taxes                              NUMERIC      NOT NULL,
    realized_profit_and_loss_after_tax NUMERIC      NOT NULL,
    created_at                         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at                         TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mutualfunds_user_id ON mutualfunds (user_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_mutualfunds_unique
    ON mutualfunds (user_id, trade_date, fund_name, shares, cancellation_amount_yen);

CREATE TABLE IF NOT EXISTS asset_balances (
    id                     UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    security_code          VARCHAR(10)  NOT NULL,
    security_name          VARCHAR(200) NOT NULL,
    shares                 NUMERIC      NOT NULL,
    executing_shares       NUMERIC      NOT NULL DEFAULT 0,
    average_purchase_price NUMERIC      NOT NULL,
    total_purchase_amount  NUMERIC      NOT NULL,
    current_price          NUMERIC      NOT NULL DEFAULT 0,
    daily_change           NUMERIC      NOT NULL DEFAULT 0,
    market_value           NUMERIC      NOT NULL DEFAULT 0,
    profit_loss_rate       NUMERIC      NOT NULL DEFAULT 0,
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_asset_balances_user_security
    ON asset_balances (user_id, security_code);
CREATE INDEX IF NOT EXISTS idx_asset_balances_user_id ON asset_balances (user_id);

CREATE TABLE IF NOT EXISTS dividend_per_share_cache (
    security_code      VARCHAR(10)      PRIMARY KEY,
    dividend_per_share DOUBLE PRECISION,
    status             VARCHAR(20)      NOT NULL DEFAULT 'pending',
    fetched_at         TIMESTAMPTZ,
    provider           VARCHAR(50)      NOT NULL DEFAULT 'jquants',
    error_message      TEXT,
    stale_at           TIMESTAMPTZ,
    created_at         TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS market_data_provider_rate_control (
    id                INTEGER PRIMARY KEY,
    next_available_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT single_row CHECK (id = 1)
);

INSERT INTO market_data_provider_rate_control (id, next_available_at)
    VALUES (1, NOW())
    ON CONFLICT (id) DO NOTHING;
