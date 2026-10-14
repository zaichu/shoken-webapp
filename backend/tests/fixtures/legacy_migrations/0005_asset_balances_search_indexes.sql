-- 保有銘柄検索（security_code / security_name での絞り込み・facets集計）向けの複合インデックス
-- 注意: 既存データがある本番DBでは CREATE INDEX 実行中にテーブルへの書き込みがブロックされうるため、
-- 本番適用はメンテナンスウィンドウ内で実施すること
CREATE INDEX IF NOT EXISTS idx_asset_balances_user_security_code ON asset_balances (user_id, security_code);
CREATE INDEX IF NOT EXISTS idx_asset_balances_user_security_name ON asset_balances (user_id, security_name);

-- q の部分一致検索（ILIKE '%token%'）向け。pg_trgm は 0001_initial_schema.sql で有効化済み。
CREATE INDEX IF NOT EXISTS idx_asset_balances_security_code_trgm
    ON asset_balances USING gin (security_code gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_asset_balances_security_name_trgm
    ON asset_balances USING gin (security_name gin_trgm_ops);

-- 一覧のデフォルトソート（security_code ASC, id ASC）向けの複合インデックス
CREATE INDEX IF NOT EXISTS idx_asset_balances_user_security_code_id
    ON asset_balances (user_id, security_code ASC, id ASC);
