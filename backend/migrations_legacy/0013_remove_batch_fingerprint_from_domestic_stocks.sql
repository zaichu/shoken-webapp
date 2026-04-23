-- import_batch_fingerprint カラムを廃止し、occurrence_index をグローバル連番に統一する
-- これにより再アップロード防止が移行前データを含む全データで機能する
--
-- 設計:
--   occurrence_index = (user_id, content_hash) 単位での通し番号（created_at, id 順）
--   同一内容の行が N 件 DB にある場合、N+1 件目以降のみ新規挿入される
--   → 累積CSV（既存分を含む再アップロード）: スキップ
--   → 増分CSV（新規分を追加したアップロード）: 新規行のみ挿入
--   注意: 純増分CSV（新規分のみ）で既存と同一ハッシュの行は
--         batch_index <= existing_count でスキップされる。
--         この場合は外部キー（取引ID等）による識別が別途必要。

-- 1. バッチ指紋用インデックスを削除
DROP INDEX IF EXISTS idx_domestic_stocks_batch_hash_occurrence;

-- 2. occurrence_index をグローバル連番で再計算
--    （0012 運用中にバッチ跨ぎの重複が発生している可能性があるため再採番）
WITH renumbered AS (
    SELECT id,
        ROW_NUMBER() OVER (
            PARTITION BY user_id, content_hash
            ORDER BY created_at, id
        )::int4 AS new_occurrence_index
    FROM domestic_stocks
)
UPDATE domestic_stocks ds
SET occurrence_index = r.new_occurrence_index
FROM renumbered r
WHERE ds.id = r.id;

-- 3. グローバル occurrence_index のユニーク制約を作成
CREATE UNIQUE INDEX idx_domestic_stocks_hash_occurrence
    ON domestic_stocks (user_id, content_hash, occurrence_index);

-- 4. import_batch_fingerprint カラムを削除
ALTER TABLE domestic_stocks DROP COLUMN import_batch_fingerprint;
