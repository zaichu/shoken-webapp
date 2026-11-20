-- no-transaction
-- contract: 使われていない trigram GIN 索引を外す。残高の銘柄名検索は ILIKE OR 条件でこの索引を使わず、facet 集計は GROUP BY のため不要。
DROP INDEX CONCURRENTLY IF EXISTS public.idx_asset_balances_security_name_trgm;
