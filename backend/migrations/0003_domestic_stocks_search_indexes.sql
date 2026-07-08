-- 国内株式取引検索（account / security_code / security_name での絞り込み・facets集計）向けの複合インデックス
CREATE INDEX IF NOT EXISTS idx_domestic_stocks_user_account ON domestic_stocks (user_id, account);
CREATE INDEX IF NOT EXISTS idx_domestic_stocks_user_security_code ON domestic_stocks (user_id, security_code);
CREATE INDEX IF NOT EXISTS idx_domestic_stocks_user_security_name ON domestic_stocks (user_id, security_name);

-- 一覧のデフォルトソート（trade_date DESC, id DESC）向けの複合インデックス
CREATE INDEX IF NOT EXISTS idx_domestic_stocks_user_trade_date_id
    ON domestic_stocks (user_id, trade_date DESC, id DESC);
