-- 現在のスキーマのベースライン。旧 0001〜0017 を順に全適用した結果と同じ schema を作る。
-- 旧17本を適用した DB の pg_dump --schema-only 出力から、所有者・権限・SET・コメント行を
-- 除いて生成している(手書きではない)。再生成手順は backend/migrations/README.md を参照。

CREATE EXTENSION IF NOT EXISTS citext WITH SCHEMA public;

COMMENT ON EXTENSION citext IS 'data type for case-insensitive character strings';

CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;

COMMENT ON EXTENSION pg_trgm IS 'text similarity measurement and index searching based on trigrams';

CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;

COMMENT ON EXTENSION pgcrypto IS 'cryptographic functions';

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

CREATE TABLE public.dividend_per_share_cache (
    security_code character varying(10) NOT NULL,
    dividend_per_share double precision,
    status character varying(20) DEFAULT 'pending'::character varying NOT NULL,
    fetched_at timestamp with time zone,
    provider character varying(50) DEFAULT 'jquants'::character varying NOT NULL,
    error_message text,
    stale_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

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

CREATE TABLE public.market_data_provider_rate_control (
    provider character varying(50) NOT NULL,
    next_available_at timestamp with time zone DEFAULT now() NOT NULL
);

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

CREATE TABLE public.sessions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone DEFAULT (now() + '7 days'::interval) NOT NULL,
    token_hash bytea NOT NULL
);

CREATE TABLE public.stock (
    date date NOT NULL,
    code character varying(10) NOT NULL,
    name public.citext NOT NULL,
    market_category character varying(50) NOT NULL,
    industry_code_33 character varying(10),
    industry_category_33 character varying(100),
    industry_code_17 character varying(10),
    industry_category_17 character varying(100),
    size_code character varying(10),
    size_category character varying(50)
);

CREATE TABLE public.users (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    google_id character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    name character varying(255),
    picture_url text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

ALTER TABLE ONLY public.asset_balances
    ADD CONSTRAINT asset_balances_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.dividend_per_share_cache
    ADD CONSTRAINT dividend_per_share_cache_pkey PRIMARY KEY (security_code);

ALTER TABLE ONLY public.dividends
    ADD CONSTRAINT dividends_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.domestic_stocks
    ADD CONSTRAINT domestic_stocks_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.market_data_provider_rate_control
    ADD CONSTRAINT market_data_provider_rate_control_pkey PRIMARY KEY (provider);

ALTER TABLE ONLY public.mutualfunds
    ADD CONSTRAINT mutualfunds_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.stock
    ADD CONSTRAINT stock_pkey PRIMARY KEY (date, code);

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_google_id_key UNIQUE (google_id);

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);

CREATE INDEX idx_asset_balances_security_code_trgm ON public.asset_balances USING gin (security_code public.gin_trgm_ops);

CREATE INDEX idx_asset_balances_security_name_trgm ON public.asset_balances USING gin (security_name public.gin_trgm_ops);

CREATE UNIQUE INDEX idx_asset_balances_user_security ON public.asset_balances USING btree (user_id, security_code);

CREATE INDEX idx_asset_balances_user_security_code_id ON public.asset_balances USING btree (user_id, security_code, id);

CREATE INDEX idx_asset_balances_user_security_name ON public.asset_balances USING btree (user_id, security_name);

CREATE INDEX idx_dividends_account_trgm ON public.dividends USING gin (account public.gin_trgm_ops);

CREATE INDEX idx_dividends_product_trgm ON public.dividends USING gin (product public.gin_trgm_ops);

CREATE INDEX idx_dividends_security_code_trgm ON public.dividends USING gin (security_code public.gin_trgm_ops);

CREATE INDEX idx_dividends_security_name_trgm ON public.dividends USING gin (security_name public.gin_trgm_ops);

CREATE UNIQUE INDEX idx_dividends_unique ON public.dividends USING btree (user_id, settlement_date, security_code, security_name, shares, dividends_before_tax);

CREATE INDEX idx_dividends_user_account ON public.dividends USING btree (user_id, account);

CREATE INDEX idx_dividends_user_product ON public.dividends USING btree (user_id, product);

CREATE INDEX idx_dividends_user_security_code ON public.dividends USING btree (user_id, security_code);

CREATE INDEX idx_dividends_user_security_name ON public.dividends USING btree (user_id, security_name);

CREATE INDEX idx_dividends_user_settlement_date_id ON public.dividends USING btree (user_id, settlement_date DESC, id DESC);

CREATE INDEX idx_domestic_stocks_account_trgm ON public.domestic_stocks USING gin (account public.gin_trgm_ops);

CREATE UNIQUE INDEX idx_domestic_stocks_hash_occurrence ON public.domestic_stocks USING btree (user_id, content_hash, occurrence_index);

CREATE INDEX idx_domestic_stocks_security_code_trgm ON public.domestic_stocks USING gin (security_code public.gin_trgm_ops);

CREATE INDEX idx_domestic_stocks_security_name_trgm ON public.domestic_stocks USING gin (security_name public.gin_trgm_ops);

CREATE INDEX idx_domestic_stocks_user_account ON public.domestic_stocks USING btree (user_id, account);

CREATE INDEX idx_domestic_stocks_user_security_code ON public.domestic_stocks USING btree (user_id, security_code);

CREATE INDEX idx_domestic_stocks_user_security_name ON public.domestic_stocks USING btree (user_id, security_name);

CREATE INDEX idx_domestic_stocks_user_trade_date_id ON public.domestic_stocks USING btree (user_id, trade_date DESC, id DESC);

CREATE INDEX idx_mutualfunds_account_trgm ON public.mutualfunds USING gin (account public.gin_trgm_ops);

CREATE INDEX idx_mutualfunds_dividends_trgm ON public.mutualfunds USING gin (dividends public.gin_trgm_ops);

CREATE INDEX idx_mutualfunds_fund_name_trgm ON public.mutualfunds USING gin (fund_name public.gin_trgm_ops);

CREATE UNIQUE INDEX idx_mutualfunds_unique ON public.mutualfunds USING btree (user_id, trade_date, fund_name, shares, cancellation_amount_yen);

CREATE INDEX idx_mutualfunds_user_account ON public.mutualfunds USING btree (user_id, account);

CREATE INDEX idx_mutualfunds_user_dividends ON public.mutualfunds USING btree (user_id, dividends);

CREATE INDEX idx_mutualfunds_user_fund_name ON public.mutualfunds USING btree (user_id, fund_name);

CREATE INDEX idx_mutualfunds_user_trade_date_id ON public.mutualfunds USING btree (user_id, trade_date DESC, id DESC);

CREATE INDEX idx_sessions_expires_at ON public.sessions USING btree (expires_at);

CREATE UNIQUE INDEX idx_sessions_token_hash ON public.sessions USING btree (token_hash);

CREATE INDEX idx_sessions_user_id ON public.sessions USING btree (user_id);

CREATE INDEX idx_stock_code_name ON public.stock USING btree (code, name);

CREATE INDEX idx_stock_name_trgm ON public.stock USING gin (name public.gin_trgm_ops);

CREATE INDEX idx_users_email ON public.users USING btree (email);

CREATE INDEX idx_users_google_id ON public.users USING btree (google_id);

ALTER TABLE ONLY public.asset_balances
    ADD CONSTRAINT asset_balances_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.dividends
    ADD CONSTRAINT dividends_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.domestic_stocks
    ADD CONSTRAINT domestic_stocks_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.mutualfunds
    ADD CONSTRAINT mutualfunds_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

-- アプリが参照する初期行(pg_dump --schema-only には含まれないため末尾に置く)
INSERT INTO public.market_data_provider_rate_control (provider, next_available_at)
    VALUES ('jquants', NOW())
    ON CONFLICT (provider) DO NOTHING;
