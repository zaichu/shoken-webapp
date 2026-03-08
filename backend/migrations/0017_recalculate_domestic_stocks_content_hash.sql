-- 0016 で DOUBLE PRECISION → NUMERIC(18,6) に変換後、
-- content_hash が float8::text 前提で作られた値のままになるため再計算する
-- 同一表現（numeric::text）でハッシュを作り直すことで新規 insert との互換性を回復する

UPDATE domestic_stocks
SET content_hash = md5(
    trade_date::text || '|' || settlement_date::text || '|' ||
    security_code || '|' || security_name || '|' || account || '|' ||
    shares::text || '|' || asked_price::text || '|' || proceeds::text || '|' ||
    purchase_price::text || '|' || realized_profit_and_loss::text || '|' ||
    taxes::text || '|' || realized_profit_and_loss_after_tax::text
);

-- occurrence_index を新しい content_hash に合わせて再計算
WITH ranked AS (
    SELECT id,
           ROW_NUMBER() OVER (
               PARTITION BY user_id, content_hash
               ORDER BY created_at, id
           ) AS rn
    FROM domestic_stocks
)
UPDATE domestic_stocks d
SET occurrence_index = ranked.rn
FROM ranked
WHERE d.id = ranked.id;
