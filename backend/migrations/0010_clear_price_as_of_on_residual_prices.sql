-- Yahoo 由来で保存する価格列(current_price・前日終値との差分である daily_change)に、
-- 丸め導入前の浮動小数点の残差(小数第2位で割り切れない値)が残る行は、次の cron 取得で
-- 丸め済みの終値に取り直されるよう基準日を未確定(NULL)に戻す。価格の値自体は更新しない
UPDATE asset_balances
SET price_as_of = NULL
WHERE price_as_of IS NOT NULL
    AND (
        current_price <> ROUND(current_price, 2)
        OR daily_change <> ROUND(daily_change, 2)
    );
