-- 国内株式取引テーブルを作成
CREATE TABLE IF NOT EXISTS domestic_stocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    trade_date DATE NOT NULL,
    settlement_date DATE NOT NULL,
    security_code VARCHAR(10) NOT NULL,
    security_name VARCHAR(200) NOT NULL,
    account VARCHAR(100) NOT NULL,
    shares DOUBLE PRECISION NOT NULL,
    asked_price DOUBLE PRECISION NOT NULL,
    proceeds DOUBLE PRECISION NOT NULL,
    purchase_price DOUBLE PRECISION NOT NULL,
    realized_profit_and_loss DOUBLE PRECISION NOT NULL,
    taxes DOUBLE PRECISION NOT NULL,
    realized_profit_and_loss_after_tax DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ユーザーごとのデータ取得用インデックス
CREATE INDEX IF NOT EXISTS idx_domestic_stocks_user_id ON domestic_stocks (user_id);

-- 重複判定用ユニーク制約
CREATE UNIQUE INDEX IF NOT EXISTS idx_domestic_stocks_unique
    ON domestic_stocks (user_id, trade_date, security_code, shares, proceeds);
