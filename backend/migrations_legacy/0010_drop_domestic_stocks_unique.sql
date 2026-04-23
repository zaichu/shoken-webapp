-- 同一日・同銘柄・同数量・同売却額の取引が複数回発生しうるため、ユニーク制約を削除
DROP INDEX IF EXISTS idx_domestic_stocks_unique;
