-- 保有銘柄テーブル
CREATE TABLE IF NOT EXISTS asset_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    security_code VARCHAR(10) NOT NULL,
    security_name VARCHAR(200) NOT NULL,
    shares DOUBLE PRECISION NOT NULL,
    executing_shares DOUBLE PRECISION NOT NULL DEFAULT 0,
    average_purchase_price DOUBLE PRECISION NOT NULL,
    total_purchase_amount DOUBLE PRECISION NOT NULL,
    current_price DOUBLE PRECISION NOT NULL DEFAULT 0,
    daily_change DOUBLE PRECISION NOT NULL DEFAULT 0,
    market_value DOUBLE PRECISION NOT NULL DEFAULT 0,
    profit_loss_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ユーザーごとの銘柄コードで一意制約
CREATE UNIQUE INDEX idx_asset_balances_user_security ON asset_balances(user_id, security_code);

-- ユーザーIDのインデックス
CREATE INDEX idx_asset_balances_user_id ON asset_balances(user_id);
