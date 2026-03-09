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
