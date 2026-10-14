-- mutualfunds テーブル(投資信託)。旧 0001〜0017 を全適用した結果と同じ schema を作るベースライン。

CREATE TABLE public.mutualfunds (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    trade_date date NOT NULL,
    settlement_date date NOT NULL,
    fund_name character varying(300) NOT NULL,
    dividends character varying(100),
    account character varying(100) NOT NULL,
    shares numeric NOT NULL,
    exchange_rate numeric NOT NULL,
    cancellation_unit_price_yen numeric NOT NULL,
    cancellation_amount_yen numeric NOT NULL,
    average_acquisition_price_yen numeric NOT NULL,
    realized_profit_and_loss numeric NOT NULL,
    taxes numeric NOT NULL,
    realized_profit_and_loss_after_tax numeric NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

ALTER TABLE ONLY public.mutualfunds
    ADD CONSTRAINT mutualfunds_pkey PRIMARY KEY (id);

CREATE INDEX idx_mutualfunds_account_trgm ON public.mutualfunds USING gin (account public.gin_trgm_ops);

CREATE INDEX idx_mutualfunds_dividends_trgm ON public.mutualfunds USING gin (dividends public.gin_trgm_ops);

CREATE INDEX idx_mutualfunds_fund_name_trgm ON public.mutualfunds USING gin (fund_name public.gin_trgm_ops);

CREATE UNIQUE INDEX idx_mutualfunds_unique ON public.mutualfunds USING btree (user_id, trade_date, fund_name, shares, cancellation_amount_yen);

CREATE INDEX idx_mutualfunds_user_account ON public.mutualfunds USING btree (user_id, account);

CREATE INDEX idx_mutualfunds_user_dividends ON public.mutualfunds USING btree (user_id, dividends);

CREATE INDEX idx_mutualfunds_user_fund_name ON public.mutualfunds USING btree (user_id, fund_name);

CREATE INDEX idx_mutualfunds_user_trade_date_id ON public.mutualfunds USING btree (user_id, trade_date DESC, id DESC);

ALTER TABLE ONLY public.mutualfunds
    ADD CONSTRAINT mutualfunds_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
