-- 保有: asset_balances・fund_balances(CSV 取込の現在値)と、共有の価格を合成する読み取りビュー。
-- ビューは security_invoker で呼び出し側の権限で動かし、参照先の表の RLS を迂回させない。
-- ポリシーの式は 0101 の利用者の表と同じ。

CREATE TABLE public.asset_balances (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    security_code character varying(10) NOT NULL,
    security_name character varying(200) NOT NULL,
    shares numeric NOT NULL,
    average_purchase_price numeric NOT NULL,
    total_purchase_amount numeric NOT NULL,
    csv_current_price numeric DEFAULT 0 NOT NULL,
    csv_daily_change numeric DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    account character varying(100),
    CONSTRAINT asset_balances_pkey PRIMARY KEY (id),
    CONSTRAINT asset_balances_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE
);

CREATE INDEX idx_asset_balances_security_name_trgm ON public.asset_balances USING gin (security_name public.gin_trgm_ops);
-- 口座なし(旧データ)同士の重複も防ぐため NULLS NOT DISTINCT
CREATE UNIQUE INDEX idx_asset_balances_user_security ON public.asset_balances USING btree (user_id, security_code, account) NULLS NOT DISTINCT;
CREATE INDEX idx_asset_balances_user_security_name ON public.asset_balances USING btree (user_id, security_name);

COMMENT ON TABLE public.asset_balances IS '国内株式の保有残高(現在値)';
COMMENT ON COLUMN public.asset_balances.csv_current_price IS 'CSV 由来の現在値(共有の確定値があるときはそちらを使う)';
COMMENT ON COLUMN public.asset_balances.csv_daily_change IS 'CSV 由来の前日比';

ALTER TABLE public.asset_balances ENABLE ROW LEVEL SECURITY;
CREATE POLICY asset_balances_isolation ON public.asset_balances
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));
COMMENT ON POLICY asset_balances_isolation ON public.asset_balances
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';

-- 列の追加は末尾のみ許されるため、prev_close と account は後から足した順に並ぶ
CREATE VIEW public.asset_balances_view WITH (security_invoker = true) AS
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
    ab.updated_at,
    sp.prev_close,
    ab.account
   FROM (public.asset_balances ab
     LEFT JOIN public.security_prices sp ON (((sp.security_code)::text = (ab.security_code)::text)));

COMMENT ON VIEW public.asset_balances_view IS '保有残高と共有株価の合成(読み取りビュー)。current_price / daily_change は共有の確定値を優先';


-- 資産残高 CSV の投資信託形式を全件置換で取り込む
CREATE TABLE public.fund_balances (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    fund_name character varying(200) NOT NULL,
    account character varying(100) NOT NULL,
    dividends_course character varying(50),
    shares numeric NOT NULL,
    average_purchase_price numeric NOT NULL,
    total_purchase_amount numeric NOT NULL,
    base_price numeric NOT NULL,
    base_price_day_change numeric,
    base_price_month_change numeric,
    market_value numeric NOT NULL,
    profit_loss numeric NOT NULL,
    profit_loss_rate numeric,
    total_return numeric,
    price_as_of date NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    normalized_fund_name text GENERATED ALWAYS AS (public.fund_name_normalize(fund_name)) STORED,
    CONSTRAINT fund_balances_pkey PRIMARY KEY (id),
    CONSTRAINT fund_balances_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE
);

CREATE INDEX idx_fund_balances_fund_name_trgm ON public.fund_balances USING gin (fund_name public.gin_trgm_ops);
CREATE INDEX idx_fund_balances_user_fund_name ON public.fund_balances USING btree (user_id, fund_name);
CREATE INDEX idx_fund_balances_user_account ON public.fund_balances USING btree (user_id, account);

COMMENT ON TABLE public.fund_balances IS '投資信託の保有残高(現在値)';
COMMENT ON COLUMN public.fund_balances.base_price IS 'CSV 由来の基準価額';
COMMENT ON COLUMN public.fund_balances.price_as_of IS 'CSV の基準価額の基準日';
COMMENT ON COLUMN public.fund_balances.normalized_fund_name IS '照合用に正規化したファンド名';

ALTER TABLE public.fund_balances ENABLE ROW LEVEL SECURITY;
CREATE POLICY fund_balances_isolation ON public.fund_balances
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));
COMMENT ON POLICY fund_balances_isolation ON public.fund_balances
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';

-- 基準価額まわりの値(基準価額・前日比・評価額・評価損益・損益率)は必ず1つの出どころでそろえる。
-- 出どころは use_shared で1度だけ決める:
--   (1) 共有値が最新(基準日 >= E)なら共有値
--   (2) そうでなく CSV が最新(取込時刻が E の23:00以降)なら CSV
--   (3) どちらも古いときだけ「共有値の基準日が CSV 基準日の3日前以降なら共有値、それより古ければ CSV」
--       (API が止まっていても新しい CSV を古い共有値で上書きしない)
-- nav_is_latest は前日比が最新の基準価額の日の変動か。price_as_of は出どころ側の日付に差し替える
CREATE VIEW public.fund_balances_view WITH (security_invoker = true) AS
WITH decided AS (
 SELECT fb.id,
    fb.user_id,
    fb.fund_name,
    fb.account,
    fb.dividends_course,
    fb.shares,
    fb.average_purchase_price,
    fb.total_purchase_amount,
    fb.base_price,
    fb.base_price_day_change,
    fb.base_price_month_change,
    fb.market_value,
    fb.profit_loss,
    fb.profit_loss_rate,
    fb.total_return,
    fb.price_as_of,
    fb.created_at,
    fb.updated_at,
    fp.nav AS shared_nav,
    fp.prev_day_change AS shared_day_change,
    fp.price_as_of AS shared_as_of,
    public.fund_expected_nav_date(now()) AS expected_nav_date,
    CASE WHEN fp.price_as_of >= public.fund_expected_nav_date(now()) THEN true
         WHEN fb.created_at >= (public.fund_expected_nav_date(now()) + time '23:00')
              AT TIME ZONE 'Asia/Tokyo' THEN false
         ELSE fp.price_as_of IS NOT NULL AND fp.price_as_of >= fb.price_as_of - 3
    END AS use_shared
   FROM (public.fund_balances fb
     LEFT JOIN public.fund_codes fc ON fc.normalized_name = fb.normalized_fund_name
     LEFT JOIN public.fund_prices fp
        ON (fp.association_fund_cd)::text = (fc.association_fund_cd)::text)
)
 SELECT d.id,
    d.user_id,
    d.fund_name,
    d.account,
    d.dividends_course,
    d.shares,
    d.average_purchase_price,
    d.total_purchase_amount,
    CASE WHEN d.use_shared THEN d.shared_nav ELSE d.base_price END AS base_price,
    CASE WHEN d.use_shared THEN d.shared_day_change ELSE d.base_price_day_change
         END AS base_price_day_change,
    d.base_price_month_change,
    CASE WHEN d.use_shared THEN floor(d.shares * d.shared_nav / 10000)
         ELSE d.market_value END AS market_value,
    CASE WHEN d.use_shared
         THEN floor(d.shares * d.shared_nav / 10000) - d.total_purchase_amount
         ELSE d.profit_loss END AS profit_loss,
    CASE WHEN d.use_shared
         THEN CASE WHEN d.total_purchase_amount > 0
                   THEN round((floor(d.shares * d.shared_nav / 10000)
                        - d.total_purchase_amount) / d.total_purchase_amount * 100, 4)
                   ELSE NULL::numeric END
         ELSE d.profit_loss_rate END AS profit_loss_rate,
    d.total_return,
    CASE WHEN d.use_shared THEN d.shared_as_of ELSE d.price_as_of END AS price_as_of,
    d.created_at,
    d.updated_at,
    CASE WHEN d.use_shared THEN 'api'::text ELSE 'csv'::text END AS price_source,
    CASE WHEN d.use_shared THEN d.shared_as_of >= d.expected_nav_date
         ELSE d.created_at >= (d.expected_nav_date + time '23:00') AT TIME ZONE 'Asia/Tokyo'
    END AS nav_is_latest,
    d.expected_nav_date
   FROM decided d;

COMMENT ON VIEW public.fund_balances_view IS '投信の保有残高と共有基準価額の合成(読み取りビュー)。price_source / nav_is_latest は出どころの判定';
