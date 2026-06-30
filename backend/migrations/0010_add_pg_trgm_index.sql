-- pg_trgm拡張を有効化して name の部分一致検索（ILIKE '%...%'）をGINインデックスで高速化
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX IF NOT EXISTS idx_stock_name_trgm ON stock USING gin (name gin_trgm_ops);
