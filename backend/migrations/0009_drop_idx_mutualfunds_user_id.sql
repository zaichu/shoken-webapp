-- no-transaction
-- idx_mutualfunds_user_id (0001, mutualfunds(user_id)) は 0004 の複合索引
-- (user_id, account / fund_name / dividends / trade_date, id) に
-- 最左一致で包含されるため冗長。DROP INDEX CONCURRENTLY で削除する。
-- 注意: DROP INDEX CONCURRENTLY はトランザクション内で実行できない。
-- sqlx は migration ファイル全体を1回の execute として送り、複文は暗黙トランザクションに
-- なるため、このファイルには文を1つだけ置くこと (文を追加しないこと)。
DROP INDEX CONCURRENTLY IF EXISTS idx_mutualfunds_user_id;
