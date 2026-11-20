-- 0119 の CHECK 2件を VALIDATE し、shares > 0 の3件を NOT VALID で追加して VALIDATE する(#2112)。
-- 本番の違反件数が cron の range_check_violations で全て 0 であることを確かめた後に当てる。
-- VALIDATE は書き込みを止めないが全表スキャンする。対象表の行数は少ない(個人表は利用者ごとの明細、キャッシュは銘柄数)。
SET LOCAL lock_timeout = '5s';

ALTER TABLE public.dividends
    ADD CONSTRAINT dividends_shares_check CHECK (shares > 0) NOT VALID;
ALTER TABLE public.domestic_stocks
    ADD CONSTRAINT domestic_stocks_shares_check CHECK (shares > 0) NOT VALID;
ALTER TABLE public.mutualfunds
    ADD CONSTRAINT mutualfunds_shares_check CHECK (shares > 0) NOT VALID;

ALTER TABLE public.dividend_per_share_cache
    VALIDATE CONSTRAINT dividend_per_share_cache_status_check;
ALTER TABLE public.dividend_per_share_cache
    VALIDATE CONSTRAINT dividend_per_share_cache_dividend_check;
ALTER TABLE public.dividends
    VALIDATE CONSTRAINT dividends_shares_check;
ALTER TABLE public.domestic_stocks
    VALIDATE CONSTRAINT domestic_stocks_shares_check;
ALTER TABLE public.mutualfunds
    VALIDATE CONSTRAINT mutualfunds_shares_check;
