-- 1株配当キャッシュの値を numeric にし、UNIQUE 制約や参照元の無いインデックス、
-- 値の種類が少ない列(security_code・account・product・dividends)の trgm インデックスを消す。
-- security_name・fund_name・stock.name の trgm はフリーワード検索で使うため残す。

ALTER TABLE public.dividend_per_share_cache
    ALTER COLUMN dividend_per_share TYPE numeric;

DROP INDEX public.idx_users_google_id;
DROP INDEX public.idx_users_email;
DROP INDEX public.idx_asset_balances_user_security_code_id;

DROP INDEX public.idx_dividends_account_trgm;
DROP INDEX public.idx_dividends_product_trgm;
DROP INDEX public.idx_dividends_security_code_trgm;
DROP INDEX public.idx_domestic_stocks_account_trgm;
DROP INDEX public.idx_domestic_stocks_security_code_trgm;
DROP INDEX public.idx_mutualfunds_account_trgm;
DROP INDEX public.idx_mutualfunds_dividends_trgm;
DROP INDEX public.idx_asset_balances_security_code_trgm;
