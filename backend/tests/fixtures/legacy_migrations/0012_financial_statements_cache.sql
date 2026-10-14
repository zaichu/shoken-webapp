-- 決算サマリー（J-Quants fins/summary）取得結果のDBバックTTLキャッシュ。
-- キー: code + from_param + to_param の組み合わせで完全一致（from/to は NULL 許容）。
-- NULL 値を含むキーでも UPSERT できるよう、アプリ側で組み立てたサロゲートキーで一意化する。
-- レスポンスJSON全体を JSONB で格納し、fetched_at が TTL（24時間）以内なら外部APIを呼ばない。
CREATE TABLE IF NOT EXISTS financial_statements_cache (
    cache_key   TEXT        PRIMARY KEY,
    code        TEXT        NOT NULL,
    from_param  TEXT,
    to_param    TEXT,
    response    JSONB       NOT NULL,
    fetched_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_financial_statements_cache_code
    ON financial_statements_cache (code);
