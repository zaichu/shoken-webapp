-- security_prices テーブル(銘柄ごとの共有株価)。
-- asset_balances の価格列は残したままにして、削除は別 migration で行う。

CREATE TABLE public.security_prices (
    security_code character varying(10) NOT NULL,
    current_price numeric DEFAULT 0 NOT NULL,
    daily_change numeric DEFAULT 0 NOT NULL,
    price_as_of date,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

ALTER TABLE ONLY public.security_prices
    ADD CONSTRAINT security_prices_pkey PRIMARY KEY (security_code);

-- 既存の保有行から銘柄ごとの最新値を移す。確定値(price_as_of あり)がある銘柄は
-- 基準日の新しい行を、全行が未確定の銘柄は最後に更新された行を採用する
INSERT INTO public.security_prices (security_code, current_price, daily_change, price_as_of, updated_at)
SELECT DISTINCT ON (security_code)
    security_code, current_price, daily_change, price_as_of, updated_at
FROM public.asset_balances
ORDER BY security_code, price_as_of DESC NULLS LAST, updated_at DESC;

-- 一覧・集計の読み取りはこのビュー経由にする(従来の asset_balances 行と同じ列名を返す)
CREATE VIEW public.asset_balances_view AS
SELECT
    ab.id,
    ab.user_id,
    ab.security_code,
    ab.security_name,
    ab.shares,
    ab.executing_shares,
    ab.average_purchase_price,
    ab.total_purchase_amount,
    COALESCE(sp.current_price, 0) AS current_price,
    COALESCE(sp.daily_change, 0) AS daily_change,
    sp.price_as_of,
    ab.created_at,
    ab.updated_at
FROM public.asset_balances ab
LEFT JOIN public.security_prices sp ON sp.security_code = ab.security_code;
