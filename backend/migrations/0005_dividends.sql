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
