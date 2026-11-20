-- no-transaction
-- contract: 使われていない trigram GIN 索引を外す。ILIKE 検索は複数列の OR 条件でこの索引を使わず、facet 集計は GROUP BY のため不要。idx_stock_name_trgm は銘柄検索で使うため残す。
DROP INDEX CONCURRENTLY IF EXISTS public.idx_dividends_security_name_trgm;
