-- dividends テーブル(配当・分配金の受取明細)。現行 schema を1ファイルに畳んだベースライン。

CREATE TABLE public.dividends (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    settlement_date date NOT NULL,
    product character varying(100) NOT NULL,
    account character varying(100) NOT NULL,
    security_code character varying(10) NOT NULL,
    security_name character varying(200) NOT NULL,
    unit_price numeric NOT NULL,
    shares numeric NOT NULL,
    dividends_before_tax numeric NOT NULL,
    taxes numeric NOT NULL,
    net_amount_received numeric NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE ONLY public.dividends
    ADD CONSTRAINT dividends_pkey PRIMARY KEY (id);


CREATE INDEX idx_dividends_security_name_trgm ON public.dividends USING gin (security_name public.gin_trgm_ops);


CREATE UNIQUE INDEX idx_dividends_unique ON public.dividends USING btree (user_id, settlement_date, security_code, security_name, shares, dividends_before_tax, product, account);


CREATE INDEX idx_dividends_user_account ON public.dividends USING btree (user_id, account);


CREATE INDEX idx_dividends_user_product ON public.dividends USING btree (user_id, product);


CREATE INDEX idx_dividends_user_security_code ON public.dividends USING btree (user_id, security_code);


CREATE INDEX idx_dividends_user_security_name ON public.dividends USING btree (user_id, security_name);


CREATE INDEX idx_dividends_user_settlement_date_id ON public.dividends USING btree (user_id, settlement_date DESC, id DESC);


ALTER TABLE ONLY public.dividends
    ADD CONSTRAINT dividends_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
