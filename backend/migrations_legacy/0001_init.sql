-- pgcrypto 拡張を有効化（gen_random_uuid() のため、PostgreSQL 12 以前で必要）
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- CITEXT 拡張を有効化（大文字小文字を区別しない検索用）
CREATE EXTENSION IF NOT EXISTS citext;

CREATE TABLE IF NOT EXISTS stock (
  date DATE NOT NULL,
  code VARCHAR(10) NOT NULL,
  name CITEXT NOT NULL,
  market_category VARCHAR(50) NOT NULL,
  industry_code_33 VARCHAR(10),
  industry_category_33 VARCHAR(50),
  industry_code_17 VARCHAR(10),
  industry_category_17 VARCHAR(50),
  size_code VARCHAR(10),
  size_category VARCHAR(50),
  PRIMARY KEY (code)
);

CREATE INDEX idx_stock_code_name ON stock (code, name);
