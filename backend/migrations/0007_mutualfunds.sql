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
