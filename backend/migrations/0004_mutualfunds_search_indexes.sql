-- 投資信託検索（account / fund_name / dividends での絞り込み・facets集計）向けの複合インデックス
CREATE INDEX IF NOT EXISTS idx_mutualfunds_user_account ON mutualfunds (user_id, account);
CREATE INDEX IF NOT EXISTS idx_mutualfunds_user_fund_name ON mutualfunds (user_id, fund_name);
CREATE INDEX IF NOT EXISTS idx_mutualfunds_user_dividends ON mutualfunds (user_id, dividends);

-- 一覧のデフォルトソート（trade_date DESC, id DESC）向けの複合インデックス
CREATE INDEX IF NOT EXISTS idx_mutualfunds_user_trade_date_id
    ON mutualfunds (user_id, trade_date DESC, id DESC);
