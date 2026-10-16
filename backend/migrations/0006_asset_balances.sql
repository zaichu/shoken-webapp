-- asset_balances テーブル(保有銘柄残高)。现行 schema を1ファイルに畳んだベースライン。

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
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE ONLY public.asset_balances
    ADD CONSTRAINT asset_balances_pkey PRIMARY KEY (id);


CREATE INDEX idx_asset_balances_security_name_trgm ON public.asset_balances USING gin (security_name public.gin_trgm_ops);


CREATE UNIQUE INDEX idx_asset_balances_user_security ON public.asset_balances USING btree (user_id, security_code);


CREATE INDEX idx_asset_balances_user_security_name ON public.asset_balances USING btree (user_id, security_name);


ALTER TABLE ONLY public.asset_balances
    ADD CONSTRAINT asset_balances_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
