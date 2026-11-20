-- 取得状態と数量の範囲を CHECK で固定する。NOT VALID で追加して同ファイルで VALIDATE するため既存行の書き換えは無い。
-- 既存行の確認: 新規 DB は空、テストの投入値は shares>0・status 4値のみ。本番の status は限定関数が 4値だけを書く。
SET LOCAL lock_timeout = '5s';

ALTER TABLE public.dividend_per_share_cache
    ADD CONSTRAINT dividend_per_share_cache_status_check CHECK (status IN ('ok', 'zero', 'error', 'pending')) NOT VALID;
ALTER TABLE public.dividend_per_share_cache
    ADD CONSTRAINT dividend_per_share_cache_dividend_check CHECK (dividend_per_share IS NULL OR dividend_per_share >= 0) NOT VALID;
ALTER TABLE public.dividends
    ADD CONSTRAINT dividends_shares_check CHECK (shares > 0) NOT VALID;
ALTER TABLE public.domestic_stocks
    ADD CONSTRAINT domestic_stocks_shares_check CHECK (shares > 0) NOT VALID;
ALTER TABLE public.mutualfunds
    ADD CONSTRAINT mutualfunds_shares_check CHECK (shares > 0) NOT VALID;

ALTER TABLE public.dividend_per_share_cache VALIDATE CONSTRAINT dividend_per_share_cache_status_check;
ALTER TABLE public.dividend_per_share_cache VALIDATE CONSTRAINT dividend_per_share_cache_dividend_check;
ALTER TABLE public.dividends VALIDATE CONSTRAINT dividends_shares_check;
ALTER TABLE public.domestic_stocks VALIDATE CONSTRAINT domestic_stocks_shares_check;
ALTER TABLE public.mutualfunds VALIDATE CONSTRAINT mutualfunds_shares_check;
