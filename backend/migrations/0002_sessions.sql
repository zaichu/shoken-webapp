-- sessions テーブル(ログインセッション)。现行 schema を1ファイルに畳んだベースライン。

CREATE TABLE public.sessions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone DEFAULT (now() + '7 days'::interval) NOT NULL,
    token_hash bytea NOT NULL
);


ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_pkey PRIMARY KEY (id);


CREATE INDEX idx_sessions_expires_at ON public.sessions USING btree (expires_at);


CREATE UNIQUE INDEX idx_sessions_token_hash ON public.sessions USING btree (token_hash);


CREATE INDEX idx_sessions_user_id ON public.sessions USING btree (user_id);


ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
