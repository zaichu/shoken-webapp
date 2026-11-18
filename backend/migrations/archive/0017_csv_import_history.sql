CREATE TABLE csv_import_history (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('assetbalance', 'fundbalance', 'dividend', 'domesticstock', 'mutualfund')),
    last_imported_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, kind)
);

COMMENT ON TABLE csv_import_history IS 'ユーザー・CSV種類ごとの最終取り込み成功時刻';
