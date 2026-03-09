-- baseline migration: 0001〜0017 の最終状態を1ファイルに集約
-- 新規環境（CI / ローカル Docker / fresh install）向け。
-- 既存環境は backend/migrations/ の 0001〜0017 を通じてアップグレードする。

-- ---- 拡張機能 ----------------------------------------
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS citext;

-- ---- stock（0001 + 0008: PK を (date, code) に変更、VARCHAR 長拡張）-------
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

-- ---- users（0002）--------------------------------------
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

-- ---- sessions（0003）-----------------------------------
CREATE TABLE IF NOT EXISTS sessions (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '7 days')
);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions (expires_at);

-- ---- dividends（0004 + 0015: unique に security_name 追加 + 0016: NUMERIC）
CREATE TABLE IF NOT EXISTS dividends (
    id                    UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id               UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    settlement_date       DATE        NOT NULL,
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
-- 重複判定キー（0015 で security_name を追加済み）
CREATE UNIQUE INDEX IF NOT EXISTS idx_dividends_unique
    ON dividends (user_id, settlement_date, security_code, security_name, shares, dividends_before_tax);

-- ---- domestic_stocks（0005 + 0010〜0013 + 0016 + 0017）-----------------
-- 0010: old unique index を削除
-- 0011〜0013: content_hash + occurrence_index による unique に変更
-- 0016: NUMERIC
-- 0017: content_hash を NUMERIC::text ベースで再計算（新規 insert では最初から正しく生成される）
CREATE TABLE IF NOT EXISTS domestic_stocks (
    id                                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                           UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    trade_date                        DATE        NOT NULL,
    settlement_date                   DATE        NOT NULL,
    security_code                     VARCHAR(10) NOT NULL,
    security_name                     VARCHAR(200) NOT NULL,
    account                           VARCHAR(100) NOT NULL,
    shares                            NUMERIC      NOT NULL,
    asked_price                       NUMERIC      NOT NULL,
    proceeds                          NUMERIC      NOT NULL,
    purchase_price                    NUMERIC      NOT NULL,
    realized_profit_and_loss          NUMERIC      NOT NULL,
    taxes                             NUMERIC      NOT NULL,
    realized_profit_and_loss_after_tax NUMERIC     NOT NULL,
    content_hash                      TEXT         NOT NULL,
    occurrence_index                  INTEGER      NOT NULL,
    created_at                        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at                        TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_domestic_stocks_user_id ON domestic_stocks (user_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_domestic_stocks_hash_occurrence
    ON domestic_stocks (user_id, content_hash, occurrence_index);

-- ---- mutualfunds（0006 + 0016: NUMERIC）--------------------------------
CREATE TABLE IF NOT EXISTS mutualfunds (
    id                               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                          UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    trade_date                       DATE         NOT NULL,
    settlement_date                  DATE         NOT NULL,
    fund_name                        VARCHAR(300) NOT NULL,
    dividends                        VARCHAR(100),
    account                          VARCHAR(100) NOT NULL,
    shares                           NUMERIC      NOT NULL,
    exchange_rate                    NUMERIC      NOT NULL,
    cancellation_unit_price_yen      NUMERIC      NOT NULL,
    cancellation_amount_yen          NUMERIC      NOT NULL,
    average_acquisition_price_yen    NUMERIC      NOT NULL,
    realized_profit_and_loss         NUMERIC      NOT NULL,
    taxes                            NUMERIC      NOT NULL,
    realized_profit_and_loss_after_tax NUMERIC    NOT NULL,
    created_at                       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at                       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_mutualfunds_user_id ON mutualfunds (user_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_mutualfunds_unique
    ON mutualfunds (user_id, trade_date, fund_name, shares, cancellation_amount_yen);

-- ---- asset_balances（0007 + 0016: NUMERIC）-----------------------------
CREATE TABLE IF NOT EXISTS asset_balances (
    id                    UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id               UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    security_code         VARCHAR(10)  NOT NULL,
    security_name         VARCHAR(200) NOT NULL,
    shares                NUMERIC      NOT NULL,
    executing_shares      NUMERIC      NOT NULL DEFAULT 0,
    average_purchase_price NUMERIC     NOT NULL,
    total_purchase_amount  NUMERIC     NOT NULL,
    current_price         NUMERIC      NOT NULL DEFAULT 0,
    daily_change          NUMERIC      NOT NULL DEFAULT 0,
    market_value          NUMERIC      NOT NULL DEFAULT 0,
    profit_loss_rate      NUMERIC      NOT NULL DEFAULT 0,
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_asset_balances_user_security ON asset_balances (user_id, security_code);
CREATE INDEX IF NOT EXISTS idx_asset_balances_user_id ON asset_balances (user_id);

-- ---- jquants_dividend_cache（0009 + 0014: stale_at 追加）---------------
-- 注意: dividend_per_share は意図的に DOUBLE PRECISION のまま（0016 の対象外）
CREATE TABLE IF NOT EXISTS jquants_dividend_cache (
    security_code      VARCHAR(10)  PRIMARY KEY,
    dividend_per_share DOUBLE PRECISION,
    status             VARCHAR(20)  NOT NULL DEFAULT 'pending',
    fetched_at         TIMESTAMPTZ,
    source             VARCHAR(50)  NOT NULL DEFAULT 'jquants',
    error_message      TEXT,
    stale_at           TIMESTAMPTZ,
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ---- jquants_rate_control（0009）--------------------------------------
CREATE TABLE IF NOT EXISTS jquants_rate_control (
    id                INTEGER PRIMARY KEY,
    next_available_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT single_row CHECK (id = 1)
);
INSERT INTO jquants_rate_control (id, next_available_at)
    VALUES (1, NOW())
    ON CONFLICT (id) DO NOTHING;
