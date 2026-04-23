-- JQuants 配当キャッシュテーブル
-- 銘柄コード単位で1株配当をキャッシュし、レート制限を吸収する
CREATE TABLE IF NOT EXISTS jquants_dividend_cache (
    security_code       VARCHAR(10) PRIMARY KEY,
    dividend_per_share  DOUBLE PRECISION,
    -- ok: 有配当、zero: ゼロ配当、error: 取得失敗、pending: 未取得/更新待ち
    status              VARCHAR(20) NOT NULL DEFAULT 'pending',
    fetched_at          TIMESTAMPTZ,
    source              VARCHAR(50) NOT NULL DEFAULT 'jquants',
    error_message       TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- JQuants グローバルレート制御テーブル（単一行、全インスタンス共有）
-- 1分5回 = 12秒間隔を DB トランザクションで原子的に保証する
CREATE TABLE IF NOT EXISTS jquants_rate_control (
    id                INTEGER PRIMARY KEY,
    next_available_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT single_row CHECK (id = 1)
);

INSERT INTO jquants_rate_control (id, next_available_at)
    VALUES (1, NOW())
    ON CONFLICT (id) DO NOTHING;
