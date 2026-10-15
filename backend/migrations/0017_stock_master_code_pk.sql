-- 読み取りは最新1行のみ。code ごとに最新 date の1行だけ残して1銘柄1行にする
DELETE FROM public.stock s
WHERE s.ctid <> (
    SELECT latest.ctid FROM public.stock latest
    WHERE latest.code = s.code
    ORDER BY latest.date DESC
    LIMIT 1
);

ALTER TABLE public.stock DROP CONSTRAINT stock_pkey;
ALTER TABLE public.stock ADD CONSTRAINT stock_pkey PRIMARY KEY (code);

-- code の主キーがあれば (code, name) の複合 index は不要
DROP INDEX public.idx_stock_code_name;

-- API 返却だけで UI・取り込みで使わない列(カテゴリ名だけ使う)
ALTER TABLE public.stock DROP COLUMN industry_code_33;
ALTER TABLE public.stock DROP COLUMN industry_code_17;
ALTER TABLE public.stock DROP COLUMN size_code;
