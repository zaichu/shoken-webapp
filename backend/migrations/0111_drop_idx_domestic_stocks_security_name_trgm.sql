-- no-transaction
-- contract: 使われていない trigram GIN 索引を外す。国内株式の銘柄名検索は複数列 OR 条件でこの索引を使わず、facet 集計は GROUP BY のため不要。
DROP INDEX CONCURRENTLY IF EXISTS public.idx_domestic_stocks_security_name_trgm;
