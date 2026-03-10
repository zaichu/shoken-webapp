-- import_batch_fingerprint カラムを追加
-- バッチ単位での再アップロード防止と増分取込の両立を実現する
-- 同一バッチ（同一CSV）の再アップロード → ON CONFLICT でスキップ
-- 異なるバッチ（増分CSV）のアップロード → 新規挿入を許可

ALTER TABLE domestic_stocks ADD COLUMN import_batch_fingerprint TEXT NOT NULL DEFAULT '';

-- 既存行のバックフィル: 各行を個別バッチとして扱う（id をフィンガープリントとして使用）
-- 注意: マイグレーション前のデータはバッチ情報が不明なため、再アップロード防止は機能しない。
--       既存データを正しく扱うには全削除 → 再取込を推奨する。
UPDATE domestic_stocks SET import_batch_fingerprint = id::text;

-- 旧ユニーク制約（content_hash + occurrence_index）を削除
DROP INDEX IF EXISTS idx_domestic_stocks_hash_occurrence;

-- バッチ対応の新ユニーク制約を作成
-- (user_id, import_batch_fingerprint, content_hash, occurrence_index) の組み合わせで一意
-- 同一バッチ内での重複を防ぎつつ、異なるバッチ間は許容する
CREATE UNIQUE INDEX idx_domestic_stocks_batch_hash_occurrence
    ON domestic_stocks (user_id, import_batch_fingerprint, content_hash, occurrence_index);

-- バックフィル完了後はデフォルト値を削除（以降は必ず Rust 側でセット）
ALTER TABLE domestic_stocks ALTER COLUMN import_batch_fingerprint DROP DEFAULT;
