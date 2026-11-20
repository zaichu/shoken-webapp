-- 取得状態と数量の範囲を CHECK で固定する。NOT VALID で追加するだけにし、既存行の書き換えも全表スキャンも無い。
-- VALIDATE は別 migration で行う(#2112)。本番の既存行に違反が1件でもあるとデプロイが止まるため、
-- 先に cron の db access ログ(range_check_violations)の件数が 0 であることを確かめてから当てる。
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
