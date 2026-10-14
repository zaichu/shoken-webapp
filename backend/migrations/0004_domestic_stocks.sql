-- domestic_stocks テーブル(国内株式取引)。旧 0001〜0017 を全適用した結果と同じ schema を作るベースライン。

CREATE TABLE public.domestic_stocks (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    trade_date date NOT NULL,
    settlement_date date NOT NULL,
    security_code character varying(10) NOT NULL,
    security_name character varying(200) NOT NULL,
    account character varying(100) NOT NULL,
    shares numeric NOT NULL,
    asked_price numeric NOT NULL,
    proceeds numeric NOT NULL,
    purchase_price numeric NOT NULL,
    realized_profit_and_loss numeric NOT NULL,
    taxes numeric NOT NULL,
    realized_profit_and_loss_after_tax numeric NOT NULL,
    content_hash text NOT NULL,
    occurrence_index integer NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

ALTER TABLE ONLY public.domestic_stocks
    ADD CONSTRAINT domestic_stocks_pkey PRIMARY KEY (id);

CREATE INDEX idx_domestic_stocks_account_trgm ON public.domestic_stocks USING gin (account public.gin_trgm_ops);

CREATE UNIQUE INDEX idx_domestic_stocks_hash_occurrence ON public.domestic_stocks USING btree (user_id, content_hash, occurrence_index);

CREATE INDEX idx_domestic_stocks_security_code_trgm ON public.domestic_stocks USING gin (security_code public.gin_trgm_ops);

CREATE INDEX idx_domestic_stocks_security_name_trgm ON public.domestic_stocks USING gin (security_name public.gin_trgm_ops);

CREATE INDEX idx_domestic_stocks_user_account ON public.domestic_stocks USING btree (user_id, account);

CREATE INDEX idx_domestic_stocks_user_security_code ON public.domestic_stocks USING btree (user_id, security_code);

CREATE INDEX idx_domestic_stocks_user_security_name ON public.domestic_stocks USING btree (user_id, security_name);

CREATE INDEX idx_domestic_stocks_user_trade_date_id ON public.domestic_stocks USING btree (user_id, trade_date DESC, id DESC);

ALTER TABLE ONLY public.domestic_stocks
    ADD CONSTRAINT domestic_stocks_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
