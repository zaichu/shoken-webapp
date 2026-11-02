ALTER TABLE dividends ENABLE ROW LEVEL SECURITY;

CREATE POLICY dividends_isolation ON dividends
    USING (user_id::text = current_setting('app.current_user_id', true))
    WITH CHECK (user_id::text = current_setting('app.current_user_id', true));

COMMENT ON POLICY dividends_isolation ON dividends
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';

ALTER TABLE domestic_stocks ENABLE ROW LEVEL SECURITY;

CREATE POLICY domestic_stocks_isolation ON domestic_stocks
    USING (user_id::text = current_setting('app.current_user_id', true))
    WITH CHECK (user_id::text = current_setting('app.current_user_id', true));

COMMENT ON POLICY domestic_stocks_isolation ON domestic_stocks
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';

ALTER TABLE mutualfunds ENABLE ROW LEVEL SECURITY;

CREATE POLICY mutualfunds_isolation ON mutualfunds
    USING (user_id::text = current_setting('app.current_user_id', true))
    WITH CHECK (user_id::text = current_setting('app.current_user_id', true));

COMMENT ON POLICY mutualfunds_isolation ON mutualfunds
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';

ALTER TABLE asset_balances ENABLE ROW LEVEL SECURITY;

CREATE POLICY asset_balances_isolation ON asset_balances
    USING (user_id::text = current_setting('app.current_user_id', true))
    WITH CHECK (user_id::text = current_setting('app.current_user_id', true));

COMMENT ON POLICY asset_balances_isolation ON asset_balances
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';

ALTER TABLE fund_balances ENABLE ROW LEVEL SECURITY;

CREATE POLICY fund_balances_isolation ON fund_balances
    USING (user_id::text = current_setting('app.current_user_id', true))
    WITH CHECK (user_id::text = current_setting('app.current_user_id', true));

COMMENT ON POLICY fund_balances_isolation ON fund_balances
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';
