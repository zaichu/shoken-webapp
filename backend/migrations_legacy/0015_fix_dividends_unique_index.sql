-- 配当金の重複判定キーに security_name を追加
-- security_code が空（投資信託など）の場合に異なる銘柄名が同一キーと見なされる問題を修正
DROP INDEX IF EXISTS idx_dividends_unique;

CREATE UNIQUE INDEX idx_dividends_unique
    ON dividends (user_id, settlement_date, security_code, security_name, shares, dividends_before_tax);
