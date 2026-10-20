-- asset_balances_view に前日終値(prev_close)を追加する。
-- 前日比(daily_change)はビューで計算済みだが、前日終値が取れない銘柄は
-- COALESCE で 0 に丸められ「変動なし」と区別できなかった。prev_close を
-- そのまま返すことで、クライアントが「前日終値なし」を判定できるようにする。
-- 列の追加は末尾のみ許容されるため、既存列の順序はそのままにする。
CREATE OR REPLACE VIEW public.asset_balances_view AS
 SELECT ab.id,
    ab.user_id,
    ab.security_code,
    ab.security_name,
    ab.shares,
    ab.average_purchase_price,
    ab.total_purchase_amount,
        CASE
            WHEN (sp.price_as_of IS NOT NULL) THEN sp.current_price
            ELSE ab.csv_current_price
        END AS current_price,
        CASE
            WHEN (sp.price_as_of IS NOT NULL) THEN COALESCE((sp.current_price - sp.prev_close), (0)::numeric)
            ELSE ab.csv_daily_change
        END AS daily_change,
    sp.price_as_of,
    ab.created_at,
    ab.updated_at,
    sp.prev_close
   FROM (public.asset_balances ab
     LEFT JOIN public.security_prices sp ON (((sp.security_code)::text = (ab.security_code)::text)));
