-- 前日比(派生値)をやめ、前日終値(市場の事実)を保存する
ALTER TABLE public.security_prices ADD COLUMN prev_close numeric;
-- 既存の確定行は current_price - daily_change で前日終値を復元する(値はこの分しか戻せない)
UPDATE public.security_prices
SET prev_close = current_price - daily_change
WHERE price_as_of IS NOT NULL;

-- ビューの前日比を導出に変える。ビューが daily_change を参照しているため
-- 列の DROP より先に差し替える
CREATE OR REPLACE VIEW public.asset_balances_view AS
SELECT
    ab.id, ab.user_id, ab.security_code, ab.security_name, ab.shares,
    ab.average_purchase_price, ab.total_purchase_amount,
    CASE WHEN sp.price_as_of IS NOT NULL THEN sp.current_price ELSE ab.csv_current_price END AS current_price,
    CASE WHEN sp.price_as_of IS NOT NULL THEN COALESCE(sp.current_price - sp.prev_close, 0) ELSE ab.csv_daily_change END AS daily_change,
    sp.price_as_of, ab.created_at, ab.updated_at
FROM public.asset_balances ab
LEFT JOIN public.security_prices sp ON sp.security_code = ab.security_code;

ALTER TABLE public.security_prices DROP COLUMN daily_change;
