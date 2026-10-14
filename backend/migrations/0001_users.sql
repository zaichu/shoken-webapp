-- users テーブル(Google 認証ユーザー)。旧 0001〜0017 を全適用した結果と同じ schema を作るベースライン。

CREATE TABLE public.users (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    google_id character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    name character varying(255),
    picture_url text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_google_id_key UNIQUE (google_id);

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);

CREATE INDEX idx_users_email ON public.users USING btree (email);

CREATE INDEX idx_users_google_id ON public.users USING btree (google_id);
