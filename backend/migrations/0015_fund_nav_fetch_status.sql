-- 直近の基準価額取得の結果(失敗の理由と時刻)を共有値ごとに残す。
-- 画面から取得の成否と失敗の理由を読めるようにする。
-- 既存行は NULL のまま(旧 Worker と共存できる。列の追加のみで既存データを変えない)

ALTER TABLE public.fund_prices
    ADD COLUMN fetch_error text;

ALTER TABLE public.fund_prices
    ADD COLUMN fetch_failed_at timestamp with time zone;
