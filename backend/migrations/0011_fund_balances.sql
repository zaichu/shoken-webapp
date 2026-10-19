-- fund_balances テーブル(投資信託の保有残高)。資産残高 CSV の投資信託形式を
-- 全件置換で取り込む。基準価額は外部価格を引かず CSV 値をそのまま保持する。

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
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE ONLY public.fund_balances
    ADD CONSTRAINT fund_balances_pkey PRIMARY KEY (id);


CREATE INDEX idx_fund_balances_fund_name_trgm ON public.fund_balances USING gin (fund_name public.gin_trgm_ops);


CREATE INDEX idx_fund_balances_user_fund_name ON public.fund_balances USING btree (user_id, fund_name);


CREATE INDEX idx_fund_balances_user_account ON public.fund_balances USING btree (user_id, account);


ALTER TABLE ONLY public.fund_balances
    ADD CONSTRAINT fund_balances_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
