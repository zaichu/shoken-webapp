-- stock テーブルのスキーマを修正
-- 1. PK を (code) から (date, code) に変更（日次データ対応）
-- 2. industry_category_33/17 の長さをモデルに合わせて 100 に拡張

-- 既存の PK を削除
ALTER TABLE stock DROP CONSTRAINT IF EXISTS stock_pkey;

-- 新しい複合 PK を追加
ALTER TABLE stock ADD PRIMARY KEY (date, code);

-- カラム長を拡張（VARCHAR は拡張のみ可能、縮小は不可）
ALTER TABLE stock ALTER COLUMN industry_category_33 TYPE VARCHAR(100);
ALTER TABLE stock ALTER COLUMN industry_category_17 TYPE VARCHAR(100);

-- 既存のインデックスを再作成（PK 変更に伴い最適化）
DROP INDEX IF EXISTS idx_stock_code_name;
CREATE INDEX idx_stock_code_name ON stock (code, name);
