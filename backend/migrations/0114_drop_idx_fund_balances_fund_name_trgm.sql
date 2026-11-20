-- no-transaction
-- contract: 使われていない trigram GIN 索引を外す。残高のファンド名検索は複数列 OR 条件でこの索引を使わず、facet 集計は GROUP BY のため不要。
DROP INDEX CONCURRENTLY IF EXISTS public.idx_fund_balances_fund_name_trgm;
