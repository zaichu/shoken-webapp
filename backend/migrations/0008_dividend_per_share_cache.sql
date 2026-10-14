-- dividend_per_share_cache テーブル(1株配当キャッシュ)。旧 0001〜0017 を全適用した結果と同じ schema を作るベースライン。

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

ALTER TABLE ONLY public.dividend_per_share_cache
    ADD CONSTRAINT dividend_per_share_cache_pkey PRIMARY KEY (security_code);
