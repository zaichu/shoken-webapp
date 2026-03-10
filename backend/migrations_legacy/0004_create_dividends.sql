-- 配当金テーブルを作成
CREATE TABLE IF NOT EXISTS dividends (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    settlement_date DATE NOT NULL,
    product VARCHAR(100) NOT NULL,
    account VARCHAR(100) NOT NULL,
    security_code VARCHAR(10) NOT NULL,
    security_name VARCHAR(200) NOT NULL,
    unit_price DOUBLE PRECISION NOT NULL,
    shares DOUBLE PRECISION NOT NULL,
    dividends_before_tax DOUBLE PRECISION NOT NULL,
    taxes DOUBLE PRECISION NOT NULL,
    net_amount_received DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ユーザーごとのデータ取得用インデックス
CREATE INDEX IF NOT EXISTS idx_dividends_user_id ON dividends (user_id);

-- 重複判定用ユニーク制約
CREATE UNIQUE INDEX IF NOT EXISTS idx_dividends_unique
    ON dividends (user_id, settlement_date, security_code, shares, dividends_before_tax);
