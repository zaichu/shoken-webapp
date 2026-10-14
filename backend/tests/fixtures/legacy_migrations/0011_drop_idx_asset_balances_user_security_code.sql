-- no-transaction
-- idx_asset_balances_user_security_code (0005, asset_balances(user_id, security_code)) は
-- 同ファイルの idx_asset_balances_user_security_code_id (user_id, security_code, id) に
-- 最左2列で包含されるため冗長。DROP INDEX CONCURRENTLY で削除する。
-- なお UNIQUE 制約の idx_asset_balances_user_security (user_id, security_code) は制約のため残す。
-- 注意: DROP INDEX CONCURRENTLY はトランザクション内で実行できない。
-- sqlx は migration ファイル全体を1回の execute として送り、複文は暗黙トランザクションに
-- なるため、このファイルには文を1つだけ置くこと (文を追加しないこと)。
DROP INDEX CONCURRENTLY IF EXISTS idx_asset_balances_user_security_code;
