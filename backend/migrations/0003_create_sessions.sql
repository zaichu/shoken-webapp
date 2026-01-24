-- セッションテーブルを作成
-- セキュリティ向上のため、ユーザーIDを直接使用せずランダムなセッショントークンを使用
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '7 days')
);

-- ユーザーIDでのルックアップを高速化するインデックス
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions (user_id);

-- 期限切れセッションのクリーンアップ用インデックス
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions (expires_at);
