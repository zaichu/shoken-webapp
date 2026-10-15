-- 配当・投資信託の重複判定キーに口座(配当は商品も)を含める。旧キーの列をすべて含む
-- ため旧キーで一意だった行は必ず通り、違反行があれば CREATE UNIQUE INDEX が失敗して
-- マイグレーション全体がロールバックされる。新しい index を先に作ってから旧 index を消す。

CREATE UNIQUE INDEX idx_dividends_unique_new
    ON public.dividends USING btree
    (user_id, settlement_date, security_code, security_name, shares, dividends_before_tax, product, account);

DROP INDEX public.idx_dividends_unique;

ALTER INDEX public.idx_dividends_unique_new RENAME TO idx_dividends_unique;

CREATE UNIQUE INDEX idx_mutualfunds_unique_new
    ON public.mutualfunds USING btree
    (user_id, trade_date, fund_name, shares, cancellation_amount_yen, account);

DROP INDEX public.idx_mutualfunds_unique;

ALTER INDEX public.idx_mutualfunds_unique_new RENAME TO idx_mutualfunds_unique;
