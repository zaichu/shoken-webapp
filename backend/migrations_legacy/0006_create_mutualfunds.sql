-- 投資信託テーブルを作成
CREATE TABLE IF NOT EXISTS mutualfunds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    trade_date DATE NOT NULL,
    settlement_date DATE NOT NULL,
    fund_name VARCHAR(300) NOT NULL,
    dividends VARCHAR(100),
    account VARCHAR(100) NOT NULL,
    shares DOUBLE PRECISION NOT NULL,
    exchange_rate DOUBLE PRECISION NOT NULL,
    cancellation_unit_price_yen DOUBLE PRECISION NOT NULL,
    cancellation_amount_yen DOUBLE PRECISION NOT NULL,
    average_acquisition_price_yen DOUBLE PRECISION NOT NULL,
    realized_profit_and_loss DOUBLE PRECISION NOT NULL,
    taxes DOUBLE PRECISION NOT NULL,
    realized_profit_and_loss_after_tax DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ユーザーごとのデータ取得用インデックス
CREATE INDEX IF NOT EXISTS idx_mutualfunds_user_id ON mutualfunds (user_id);

-- 重複判定用ユニーク制約
CREATE UNIQUE INDEX IF NOT EXISTS idx_mutualfunds_unique
    ON mutualfunds (user_id, trade_date, fund_name, shares, cancellation_amount_yen);
