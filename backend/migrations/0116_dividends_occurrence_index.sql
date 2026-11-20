-- contract: 配当の取り込みで同一内容の行が2行あると2行目が落ちるため、国内株式と同じ content_hash + occurrence_index で区別する。旧一意キー(idx_dividends_unique)は内容列そのもので重複行を許さないため外す。既存行は occurrence_index=0 で埋める(旧キーで重複は無いため新キーは保たれる)。
SET LOCAL lock_timeout = '5s';

ALTER TABLE public.dividends
    ADD COLUMN content_hash text,
    ADD COLUMN occurrence_index integer;

UPDATE public.dividends SET
    content_hash = md5(
        settlement_date::text || '|' || product || '|' || account || '|' ||
        security_code || '|' || security_name || '|' ||
        unit_price::text || '|' || shares::text || '|' || dividends_before_tax::text || '|' ||
        taxes::text || '|' || net_amount_received::text
    ),
    occurrence_index = 0
WHERE content_hash IS NULL;

ALTER TABLE public.dividends
    ALTER COLUMN content_hash SET NOT NULL,
    ALTER COLUMN occurrence_index SET NOT NULL;

CREATE UNIQUE INDEX idx_dividends_hash_occurrence ON public.dividends USING btree (user_id, content_hash, occurrence_index);

DROP INDEX IF EXISTS public.idx_dividends_unique;
