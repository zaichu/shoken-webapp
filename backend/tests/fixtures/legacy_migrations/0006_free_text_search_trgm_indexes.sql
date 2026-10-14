-- dividends / domestic_stocks / mutualfunds のフリーワード検索（q の部分一致検索 ILIKE '%token%'）向け GIN trgm インデックス
-- 注意: 既存データがある本番DBでは CREATE INDEX 実行中にテーブルへの書き込みがブロックされうるため、
-- 本番適用はメンテナンスウィンドウ内で実施すること
-- pg_trgm は 0001_initial_schema.sql で有効化済み。
-- 対象列は全て VARCHAR のため gin_trgm_ops をそのまま適用できる（citext 列は対象に含まれない）。

-- dividends: product, account, security_code, security_name
CREATE INDEX IF NOT EXISTS idx_dividends_product_trgm
    ON dividends USING gin (product gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_dividends_account_trgm
    ON dividends USING gin (account gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_dividends_security_code_trgm
    ON dividends USING gin (security_code gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_dividends_security_name_trgm
    ON dividends USING gin (security_name gin_trgm_ops);

-- domestic_stocks: account, security_code, security_name
CREATE INDEX IF NOT EXISTS idx_domestic_stocks_account_trgm
    ON domestic_stocks USING gin (account gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_domestic_stocks_security_code_trgm
    ON domestic_stocks USING gin (security_code gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_domestic_stocks_security_name_trgm
    ON domestic_stocks USING gin (security_name gin_trgm_ops);

-- mutualfunds: account, fund_name, dividends
CREATE INDEX IF NOT EXISTS idx_mutualfunds_account_trgm
    ON mutualfunds USING gin (account gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_mutualfunds_fund_name_trgm
    ON mutualfunds USING gin (fund_name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_mutualfunds_dividends_trgm
    ON mutualfunds USING gin (dividends gin_trgm_ops);
