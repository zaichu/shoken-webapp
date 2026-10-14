-- asset_balances テーブル(保有銘柄残高)。旧 0001〜0017 を全適用した結果と同じ schema を作るベースライン。

CREATE TABLE public.asset_balances (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    security_code character varying(10) NOT NULL,
    security_name character varying(200) NOT NULL,
    shares numeric NOT NULL,
    executing_shares numeric DEFAULT 0 NOT NULL,
    average_purchase_price numeric NOT NULL,
    total_purchase_amount numeric NOT NULL,
    current_price numeric DEFAULT 0 NOT NULL,
    daily_change numeric DEFAULT 0 NOT NULL,
    market_value numeric DEFAULT 0 NOT NULL,
    profit_loss_rate numeric DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    price_as_of date
);

ALTER TABLE ONLY public.asset_balances
    ADD CONSTRAINT asset_balances_pkey PRIMARY KEY (id);

CREATE INDEX idx_asset_balances_security_code_trgm ON public.asset_balances USING gin (security_code public.gin_trgm_ops);

CREATE INDEX idx_asset_balances_security_name_trgm ON public.asset_balances USING gin (security_name public.gin_trgm_ops);

CREATE UNIQUE INDEX idx_asset_balances_user_security ON public.asset_balances USING btree (user_id, security_code);

CREATE INDEX idx_asset_balances_user_security_code_id ON public.asset_balances USING btree (user_id, security_code, id);

CREATE INDEX idx_asset_balances_user_security_name ON public.asset_balances USING btree (user_id, security_name);

ALTER TABLE ONLY public.asset_balances
    ADD CONSTRAINT asset_balances_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
