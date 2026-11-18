ALTER TABLE csv_import_history ENABLE ROW LEVEL SECURITY;

CREATE POLICY csv_import_history_isolation ON csv_import_history
    USING (user_id::text = current_setting('app.current_user_id', true))
    WITH CHECK (user_id::text = current_setting('app.current_user_id', true));

COMMENT ON POLICY csv_import_history_isolation ON csv_import_history
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';
