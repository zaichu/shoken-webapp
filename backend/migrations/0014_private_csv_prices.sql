-- 共有価格(security_prices)は確定日付(price_as_of)がある値だけ表示に採用し、
-- 未確定 shared seed は表示に使わない。個人 CSV の価格は asset_balances に残す
CREATE OR REPLACE VIEW public.asset_balances_view AS
SELECT
    ab.id,
    ab.user_id,
    ab.security_code,
    ab.security_name,
    ab.shares,
    ab.executing_shares,
    ab.average_purchase_price,
    ab.total_purchase_amount,
    CASE WHEN sp.price_as_of IS NOT NULL THEN sp.current_price ELSE ab.current_price END AS current_price,
    CASE WHEN sp.price_as_of IS NOT NULL THEN sp.daily_change ELSE ab.daily_change END AS daily_change,
    sp.price_as_of,
    ab.created_at,
    ab.updated_at
FROM public.asset_balances ab
LEFT JOIN public.security_prices sp ON sp.security_code = ab.security_code;
