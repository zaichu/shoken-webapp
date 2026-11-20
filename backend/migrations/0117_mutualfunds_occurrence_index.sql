-- contract: 投信の取り込みで同一内容の行が2行あると2行目が落ちるため、国内株式と同じ content_hash + occurrence_index で区別する。旧一意キー(idx_mutualfunds_unique)は内容列そのもので重複行を許さないため外す。既存行は occurrence_index=0 で埋める(旧キーで重複は無いため新キーは保たれる)。
SET LOCAL lock_timeout = '5s';

ALTER TABLE public.mutualfunds
    ADD COLUMN content_hash text,
    ADD COLUMN occurrence_index integer;

UPDATE public.mutualfunds SET
    content_hash = md5(
        trade_date::text || '|' || settlement_date::text || '|' || fund_name || '|' ||
        COALESCE(dividends, '') || '|' || account || '|' ||
        shares::text || '|' || exchange_rate::text || '|' || cancellation_unit_price_yen::text || '|' ||
        cancellation_amount_yen::text || '|' || average_acquisition_price_yen::text || '|' ||
        realized_profit_and_loss::text || '|' || taxes::text || '|' || realized_profit_and_loss_after_tax::text
    ),
    occurrence_index = 0
WHERE content_hash IS NULL;

ALTER TABLE public.mutualfunds
    ALTER COLUMN content_hash SET NOT NULL,
    ALTER COLUMN occurrence_index SET NOT NULL;

CREATE UNIQUE INDEX idx_mutualfunds_hash_occurrence ON public.mutualfunds USING btree (user_id, content_hash, occurrence_index);

DROP INDEX IF EXISTS public.idx_mutualfunds_unique;
