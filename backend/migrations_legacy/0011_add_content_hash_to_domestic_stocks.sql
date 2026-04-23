-- 内容ハッシュ + 出現回数によるユニーク制約を追加
-- 同一内容の行を複数保持しつつ、同一CSVの再アップロードによる重複増殖を防ぐ

-- カラム追加（デフォルト値付きで既存行への対応を確保）
ALTER TABLE domestic_stocks ADD COLUMN content_hash TEXT NOT NULL DEFAULT '';
ALTER TABLE domestic_stocks ADD COLUMN occurrence_index INTEGER NOT NULL DEFAULT 1;

-- 既存行のハッシュ値をバックフィル
-- ハッシュ形式は Rust の compute_content_hash と一致させるため
-- PostgreSQL の float8::text と Rust の f64 フォーマットが同一表現を使う前提
UPDATE domestic_stocks
SET content_hash = md5(
    trade_date::text || '|' || settlement_date::text || '|' ||
    security_code || '|' || security_name || '|' || account || '|' ||
    shares::text || '|' || asked_price::text || '|' || proceeds::text || '|' ||
    purchase_price::text || '|' || realized_profit_and_loss::text || '|' ||
    taxes::text || '|' || realized_profit_and_loss_after_tax::text
);

-- 既存行の occurrence_index をバックフィル
-- 同一ユーザー・同一ハッシュ内で created_at 昇順に連番を付与
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

-- ユニーク制約を追加（内容ハッシュ + 出現回数で同一内容・同一件数の重複を排除）
CREATE UNIQUE INDEX idx_domestic_stocks_hash_occurrence
    ON domestic_stocks (user_id, content_hash, occurrence_index);

-- バックフィル完了後はデフォルト値を削除（以降は必ず Rust 側でセット）
ALTER TABLE domestic_stocks ALTER COLUMN content_hash DROP DEFAULT;
ALTER TABLE domestic_stocks ALTER COLUMN occurrence_index DROP DEFAULT;
