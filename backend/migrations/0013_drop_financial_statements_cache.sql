-- financial-statements エンドポイント廃止に伴い、DBバックTTLキャッシュ用テーブルを削除する。
-- 0012 で作成したテーブルは適用済み環境があるため、新規 migration で打ち消す。
DROP TABLE IF EXISTS financial_statements_cache;
