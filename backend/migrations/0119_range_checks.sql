-- 取得状態と配当の範囲を CHECK で固定する。NOT VALID で追加するだけにし、既存行の書き換えも全表スキャンも無い。
-- VALIDATE は別 migration で行う(#2112)。本番の既存行に違反が1件でもあるとデプロイが止まるため、
-- 先に cron の db access ログ(range_check_violations)の件数が 0 であることを確かめてから当てる。
-- 既存行の確認: 新規 DB は空、テストの投入値は status 4値・dividend_per_share >= 0 のみ。本番の status と配当は限定関数だけが書く。
-- dividends・domestic_stocks・mutualfunds の shares > 0 はこの PR には入れない。NOT VALID でも新規 INSERT には効き、
-- 本番の楽天 CSV に数量 0 の行があると取込全体が CHECK 違反で失敗するため、#2112 で件数 0 を確かめてから追加する。
SET LOCAL lock_timeout = '5s';

ALTER TABLE public.dividend_per_share_cache
    ADD CONSTRAINT dividend_per_share_cache_status_check CHECK (status IN ('ok', 'zero', 'error', 'pending')) NOT VALID;
ALTER TABLE public.dividend_per_share_cache
    ADD CONSTRAINT dividend_per_share_cache_dividend_check CHECK (dividend_per_share IS NULL OR dividend_per_share >= 0) NOT VALID;
