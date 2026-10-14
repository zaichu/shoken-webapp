-- 配当金検索（product / account / security_code / security_name での絞り込み・facets集計）向けの複合インデックス
CREATE INDEX IF NOT EXISTS idx_dividends_user_product ON dividends (user_id, product);
CREATE INDEX IF NOT EXISTS idx_dividends_user_account ON dividends (user_id, account);
CREATE INDEX IF NOT EXISTS idx_dividends_user_security_code ON dividends (user_id, security_code);
CREATE INDEX IF NOT EXISTS idx_dividends_user_security_name ON dividends (user_id, security_name);

-- 一覧のデフォルトソート（settlement_date DESC, id DESC）向けの複合インデックス
CREATE INDEX IF NOT EXISTS idx_dividends_user_settlement_date_id
    ON dividends (user_id, settlement_date DESC, id DESC);
