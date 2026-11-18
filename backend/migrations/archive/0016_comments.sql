-- テーブルとカラムの説明。docs/db/ のテーブル一覧と ER 図に出る。
-- 適用済みの migration は書き換えないので、説明の追記は新しいファイルで行う。

COMMENT ON TABLE public.users IS 'Google 認証のユーザー';
COMMENT ON TABLE public.sessions IS 'ログインセッション';
COMMENT ON TABLE public.dividends IS '配当・分配金の受取明細(CSV 取込)';
COMMENT ON TABLE public.domestic_stocks IS '国内株式の売買明細(CSV 取込)';
COMMENT ON TABLE public.mutualfunds IS '投資信託の取引明細(CSV 取込)';
COMMENT ON TABLE public.asset_balances IS '国内株式の保有残高(現在値)';
COMMENT ON VIEW public.asset_balances_view IS '保有残高と共有株価の合成(読み取りビュー)。current_price / daily_change は共有の確定値を優先';
COMMENT ON TABLE public.stock IS '上場銘柄のマスタ(共有)';
COMMENT ON TABLE public.dividend_per_share_cache IS '1株配当のキャッシュ(共有)';
COMMENT ON TABLE public.market_data_provider_rate_control IS '外部APIの呼び出し間隔(全インスタンス共有)';
COMMENT ON TABLE public.security_prices IS '銘柄ごとの共有株価';
COMMENT ON TABLE public.fund_balances IS '投資信託の保有残高(現在値)';
COMMENT ON TABLE public.fund_codes IS 'ファンド名と投信協会コードの対応表(共有)';
COMMENT ON TABLE public.fund_prices IS 'ファンドごとの共有基準価額';
COMMENT ON VIEW public.fund_balances_view IS '投信の保有残高と共有基準価額の合成(読み取りビュー)。price_source / nav_is_latest は出どころの判定';

COMMENT ON COLUMN public.asset_balances.csv_current_price IS 'CSV 由来の現在値(共有の確定値があるときはそちらを使う)';
COMMENT ON COLUMN public.asset_balances.csv_daily_change IS 'CSV 由来の前日比';
COMMENT ON COLUMN public.security_prices.price_as_of IS '値として確定した基準日';
COMMENT ON COLUMN public.security_prices.prev_close IS '前日終値';
COMMENT ON COLUMN public.dividend_per_share_cache.status IS '取得状態(ok / zero / error / pending)';
COMMENT ON COLUMN public.dividend_per_share_cache.stale_at IS 'TTL 期限。この時刻を過ぎると再取得対象';
COMMENT ON COLUMN public.market_data_provider_rate_control.next_available_at IS '次の呼び出し可能時刻';
COMMENT ON COLUMN public.fund_codes.normalized_name IS '照合用に正規化したファンド名';
COMMENT ON COLUMN public.fund_prices.nav IS '共有の基準価額';
COMMENT ON COLUMN public.fund_prices.prev_day_change IS '共有の基準価額の前日比';
COMMENT ON COLUMN public.fund_prices.price_as_of IS '基準価額の基準日';
COMMENT ON COLUMN public.fund_prices.fetch_error IS '直近の基準価額取得の失敗理由';
COMMENT ON COLUMN public.fund_prices.fetch_failed_at IS '直近の基準価額取得の失敗時刻';
COMMENT ON COLUMN public.fund_balances.base_price IS 'CSV 由来の基準価額';
COMMENT ON COLUMN public.fund_balances.price_as_of IS 'CSV の基準価額の基準日';
COMMENT ON COLUMN public.fund_balances.normalized_fund_name IS '照合用に正規化したファンド名';
