-- 取引: dividends・domestic_stocks・mutualfunds(CSV 取込の明細)と csv_import_history。
-- ポリシーの式は 0101 の利用者の表と同じ。

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
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT dividends_pkey PRIMARY KEY (id),
    CONSTRAINT dividends_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE
);

CREATE INDEX idx_dividends_security_name_trgm ON public.dividends USING gin (security_name public.gin_trgm_ops);
CREATE UNIQUE INDEX idx_dividends_unique ON public.dividends USING btree (user_id, settlement_date, security_code, security_name, shares, dividends_before_tax, product, account);
CREATE INDEX idx_dividends_user_account ON public.dividends USING btree (user_id, account);
CREATE INDEX idx_dividends_user_product ON public.dividends USING btree (user_id, product);
CREATE INDEX idx_dividends_user_security_code ON public.dividends USING btree (user_id, security_code);
CREATE INDEX idx_dividends_user_security_name ON public.dividends USING btree (user_id, security_name);
CREATE INDEX idx_dividends_user_settlement_date_id ON public.dividends USING btree (user_id, settlement_date DESC, id DESC);

COMMENT ON TABLE public.dividends IS '配当・分配金の受取明細(CSV 取込)';

ALTER TABLE public.dividends ENABLE ROW LEVEL SECURITY;
CREATE POLICY dividends_isolation ON public.dividends
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));
COMMENT ON POLICY dividends_isolation ON public.dividends
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';


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
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT domestic_stocks_pkey PRIMARY KEY (id),
    CONSTRAINT domestic_stocks_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX idx_domestic_stocks_hash_occurrence ON public.domestic_stocks USING btree (user_id, content_hash, occurrence_index);
CREATE INDEX idx_domestic_stocks_security_name_trgm ON public.domestic_stocks USING gin (security_name public.gin_trgm_ops);
CREATE INDEX idx_domestic_stocks_user_account ON public.domestic_stocks USING btree (user_id, account);
CREATE INDEX idx_domestic_stocks_user_security_code ON public.domestic_stocks USING btree (user_id, security_code);
CREATE INDEX idx_domestic_stocks_user_security_name ON public.domestic_stocks USING btree (user_id, security_name);
CREATE INDEX idx_domestic_stocks_user_trade_date_id ON public.domestic_stocks USING btree (user_id, trade_date DESC, id DESC);

COMMENT ON TABLE public.domestic_stocks IS '国内株式の売買明細(CSV 取込)';

ALTER TABLE public.domestic_stocks ENABLE ROW LEVEL SECURITY;
CREATE POLICY domestic_stocks_isolation ON public.domestic_stocks
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));
COMMENT ON POLICY domestic_stocks_isolation ON public.domestic_stocks
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';


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
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT mutualfunds_pkey PRIMARY KEY (id),
    CONSTRAINT mutualfunds_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE
);

CREATE INDEX idx_mutualfunds_fund_name_trgm ON public.mutualfunds USING gin (fund_name public.gin_trgm_ops);
CREATE UNIQUE INDEX idx_mutualfunds_unique ON public.mutualfunds USING btree (user_id, trade_date, fund_name, shares, cancellation_amount_yen, account);
CREATE INDEX idx_mutualfunds_user_account ON public.mutualfunds USING btree (user_id, account);
CREATE INDEX idx_mutualfunds_user_dividends ON public.mutualfunds USING btree (user_id, dividends);
CREATE INDEX idx_mutualfunds_user_fund_name ON public.mutualfunds USING btree (user_id, fund_name);
CREATE INDEX idx_mutualfunds_user_trade_date_id ON public.mutualfunds USING btree (user_id, trade_date DESC, id DESC);

COMMENT ON TABLE public.mutualfunds IS '投資信託の取引明細(CSV 取込)';

ALTER TABLE public.mutualfunds ENABLE ROW LEVEL SECURITY;
CREATE POLICY mutualfunds_isolation ON public.mutualfunds
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));
COMMENT ON POLICY mutualfunds_isolation ON public.mutualfunds
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';


CREATE TABLE public.csv_import_history (
    user_id uuid NOT NULL,
    kind text NOT NULL,
    last_imported_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT csv_import_history_pkey PRIMARY KEY (user_id, kind),
    CONSTRAINT csv_import_history_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE,
    CONSTRAINT csv_import_history_kind_check CHECK (kind IN ('assetbalance', 'fundbalance', 'dividend', 'domesticstock', 'mutualfund'))
);

COMMENT ON TABLE public.csv_import_history IS 'ユーザー・CSV種類ごとの最終取り込み成功時刻';

ALTER TABLE public.csv_import_history ENABLE ROW LEVEL SECURITY;
CREATE POLICY csv_import_history_isolation ON public.csv_import_history
    USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))
    WITH CHECK (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid));
COMMENT ON POLICY csv_import_history_isolation ON public.csv_import_history
    IS '接続ごとの app.current_user_id と user_id が一致する行だけを許可する。FORCE なしのため所有者ロールは迂回し、現行の挙動は変わらない';
