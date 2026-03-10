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
CREATE UNIQUE INDEX IF NOT EXISTS idx_asset_balances_user_security ON asset_balances (user_id, security_code);
CREATE INDEX IF NOT EXISTS idx_asset_balances_user_id ON asset_balances (user_id);
