-- stock テーブル(銘柄マスタ)。现行 schema を1ファイルに畳んだベースライン。

CREATE TABLE public.stock (
    date date NOT NULL,
    code character varying(10) NOT NULL,
    name public.citext NOT NULL,
    market_category character varying(50) NOT NULL,
    industry_category_33 character varying(100),
    industry_category_17 character varying(100),
    size_category character varying(50)
);


ALTER TABLE ONLY public.stock
    ADD CONSTRAINT stock_pkey PRIMARY KEY (code);


CREATE INDEX idx_stock_name_trgm ON public.stock USING gin (name public.gin_trgm_ops);
