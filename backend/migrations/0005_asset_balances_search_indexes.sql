-- 保有銘柄検索（security_code / security_name での絞り込み・facets集計）向けの複合インデックス
-- 注意: 既存データがある本番DBでは CREATE INDEX 実行中にテーブルへの書き込みがブロックされうるため、
-- 本番適用はメンテナンスウィンドウ内で実施すること
CREATE INDEX IF NOT EXISTS idx_asset_balances_user_security_code ON asset_balances (user_id, security_code);
CREATE INDEX IF NOT EXISTS idx_asset_balances_user_security_name ON asset_balances (user_id, security_name);

-- 一覧のデフォルトソート（security_code ASC, id ASC）向けの複合インデックス
CREATE INDEX IF NOT EXISTS idx_asset_balances_user_security_code_id
    ON asset_balances (user_id, security_code ASC, id ASC);
