-- security_prices テーブル(銘柄ごとの共有株価)。現行 schema を1ファイルに畳んだベースライン。

CREATE TABLE public.security_prices (
    security_code character varying(10) NOT NULL,
    current_price numeric DEFAULT 0 NOT NULL,
    price_as_of date,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    prev_close numeric
);


ALTER TABLE ONLY public.security_prices
    ADD CONSTRAINT security_prices_pkey PRIMARY KEY (security_code);

-- 既存の保有行から銘柄ごとの最新値を移す。確定値(price_as_of あり)がある銘柄は
-- 基準日の新しい行を、全行が未確定の銘柄は最後に更新された行を採用する
-- 0013 当時の asset_balances の列(current_price / daily_change / price_as_of)は
-- 0015 で csv_current_price / csv_daily_change に変わり price_as_of は削除済みなので、
-- 現行の列名に合わせてSELECTする(新規DBでは asset_balances が空なので seed されない)
INSERT INTO public.security_prices (security_code, current_price, updated_at)
SELECT DISTINCT ON (security_code)
    security_code, csv_current_price, updated_at
FROM public.asset_balances
ORDER BY security_code, updated_at DESC;

-- 一覧・集計の読み取りはこのビュー経由にする(従来の asset_balances 行と同じ列名を返す)。
-- asset_balances と security_prices の両方に依存するため、後に作られる
-- security_prices 側のファイルで定義する
CREATE VIEW public.asset_balances_view AS
 SELECT ab.id,
    ab.user_id,
    ab.security_code,
    ab.security_name,
    ab.shares,
    ab.average_purchase_price,
    ab.total_purchase_amount,
        CASE
            WHEN (sp.price_as_of IS NOT NULL) THEN sp.current_price
            ELSE ab.csv_current_price
        END AS current_price,
        CASE
            WHEN (sp.price_as_of IS NOT NULL) THEN COALESCE((sp.current_price - sp.prev_close), (0)::numeric)
            ELSE ab.csv_daily_change
        END AS daily_change,
    sp.price_as_of,
    ab.created_at,
    ab.updated_at
   FROM (public.asset_balances ab
     LEFT JOIN public.security_prices sp ON (((sp.security_code)::text = (ab.security_code)::text)));
