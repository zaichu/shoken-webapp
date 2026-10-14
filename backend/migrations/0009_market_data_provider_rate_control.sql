-- market_data_provider_rate_control テーブル(外部APIレート制御)と初期行。旧 0001〜0017 を全適用した結果と同じ schema を作るベースライン。

CREATE TABLE public.market_data_provider_rate_control (
    provider character varying(50) NOT NULL,
    next_available_at timestamp with time zone DEFAULT now() NOT NULL
);

ALTER TABLE ONLY public.market_data_provider_rate_control
    ADD CONSTRAINT market_data_provider_rate_control_pkey PRIMARY KEY (provider);

INSERT INTO public.market_data_provider_rate_control (provider, next_available_at)
    VALUES ('jquants', NOW())
    ON CONFLICT (provider) DO NOTHING;
