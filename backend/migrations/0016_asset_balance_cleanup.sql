-- 既存の asset_balances_view が削除対象列(executing_shares)と改名対象列
-- (current_price/daily_change)を参照しているため、先にビューを落とす。
-- 同一トランザクション内で作り直すので、適用中にビューが欠けることはない
DROP VIEW public.asset_balances_view;

-- 本人CSV価格であることを名前で示す(共有 security_prices 側と区別する)
ALTER TABLE public.asset_balances RENAME COLUMN current_price TO csv_current_price;
ALTER TABLE public.asset_balances RENAME COLUMN daily_change  TO csv_daily_change;

-- 保存されても読み書きされない列を落とす
ALTER TABLE public.asset_balances DROP COLUMN market_value;      -- 常に0。応答DTOにも無い(#1335)
ALTER TABLE public.asset_balances DROP COLUMN profit_loss_rate;  -- 常に0。応答DTOにも無い(#1335)
ALTER TABLE public.asset_balances DROP COLUMN price_as_of;       -- 読み書き経路なし(#1335)
ALTER TABLE public.asset_balances DROP COLUMN executing_shares;  -- UI/ロジックで未使用(#1335)

-- ビューは API の列名(current_price/daily_change/price_as_of)を従来どおり出す。
-- 共有の確定値(price_as_of あり)を優先し、なければ本人CSVの値を返す
CREATE VIEW public.asset_balances_view AS
SELECT
    ab.id, ab.user_id, ab.security_code, ab.security_name, ab.shares,
    ab.average_purchase_price, ab.total_purchase_amount,
    CASE WHEN sp.price_as_of IS NOT NULL THEN sp.current_price ELSE ab.csv_current_price END AS current_price,
    CASE WHEN sp.price_as_of IS NOT NULL THEN sp.daily_change  ELSE ab.csv_daily_change  END AS daily_change,
    sp.price_as_of,
    ab.created_at, ab.updated_at
FROM public.asset_balances ab
LEFT JOIN public.security_prices sp ON sp.security_code = ab.security_code;
