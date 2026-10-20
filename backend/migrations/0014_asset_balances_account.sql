-- asset_balances に口座区分を追加する。国内株式の保有 CSV は口座ごとの
-- セクションに分かれており、「○○口座合計」の区切り行から口座名を取って保持する。
-- 既存行は口座不明のまま(null)残す。
ALTER TABLE ONLY public.asset_balances
    ADD COLUMN account character varying(100);

-- 同じ銘柄を口座別に持てるよう、一意キーを口座込みに変える。
-- 口座なし(旧データ)同士の重複は NULLS NOT DISTINCT で従来どおり防ぐ
DROP INDEX public.idx_asset_balances_user_security;
CREATE UNIQUE INDEX idx_asset_balances_user_security
    ON public.asset_balances USING btree (user_id, security_code, account) NULLS NOT DISTINCT;

-- ビュー列の追加は末尾のみ許容されるため、account は prev_close の後ろに足す
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
    sp.prev_close,
    ab.account
   FROM (public.asset_balances ab
     LEFT JOIN public.security_prices sp ON (((sp.security_code)::text = (ab.security_code)::text)));
