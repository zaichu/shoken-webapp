ALTER TABLE sessions ENABLE ROW LEVEL SECURITY;

CREATE POLICY sessions_isolation ON sessions
    USING (user_id::text = current_setting('app.current_user_id', true))
    WITH CHECK (user_id::text = current_setting('app.current_user_id', true));

COMMENT ON POLICY sessions_isolation ON sessions
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';

ALTER TABLE users ENABLE ROW LEVEL SECURITY;

CREATE POLICY users_isolation ON users
    USING (id::text = current_setting('app.current_user_id', true))
    WITH CHECK (id::text = current_setting('app.current_user_id', true));

COMMENT ON POLICY users_isolation ON users
    IS '接続ごとの app.current_user_id と id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';
