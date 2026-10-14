-- stock テーブル(銘柄マスタ)。旧 0001〜0017 を全適用した結果と同じ schema を作るベースライン。

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

ALTER TABLE ONLY public.stock
    ADD CONSTRAINT stock_pkey PRIMARY KEY (date, code);

CREATE INDEX idx_stock_code_name ON public.stock USING btree (code, name);

CREATE INDEX idx_stock_name_trgm ON public.stock USING gin (name public.gin_trgm_ops);
