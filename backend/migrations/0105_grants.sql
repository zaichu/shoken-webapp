-- 権限: ロールごとの表・ビュー・列・関数への GRANT。
-- app_restricted は本人の表への読み書きと共有の表・ビューの読み取り、限定関数の所有ロールは
-- 関数が触る列だけ、app_worker は共有の表の読み取りだけを持つ(個人の表の権限は持たない)。
-- 同じ対象への付与の順は ACL の並びに残るため、旧 migration と同じ順(app_restricted → 所有ロール → app_worker)にしている。

GRANT USAGE ON SCHEMA public TO app_restricted, app_notify_token_v2_executor,
    app_admin_login_executor, app_price_target_executor, app_login_event_prune_executor,
    app_worker, app_shared_write_executor;

-- app_restricted
GRANT SELECT, INSERT, UPDATE, DELETE ON public.users, public.sessions, public.dividends,
    public.domestic_stocks, public.mutualfunds, public.asset_balances, public.fund_balances,
    public.csv_import_history, public.login_events, public.login_notify_tokens,
    public.admin_login_views TO app_restricted;
GRANT SELECT ON public.stock, public.security_prices, public.dividend_per_share_cache,
    public.market_data_provider_rate_control, public.fund_codes, public.fund_prices TO app_restricted;
GRANT SELECT ON public.asset_balances_view, public.fund_balances_view TO app_restricted;
GRANT EXECUTE ON FUNCTION public.fund_name_normalize(text),
    public.fund_expected_nav_date(timestamptz) TO app_restricted;

-- app_notify_token_v2_executor: ログイン通知トークンの照合と使用済み記録
GRANT SELECT (token_hash, user_id, expires_at, used_at), UPDATE (used_at)
    ON public.login_notify_tokens TO app_notify_token_v2_executor;

-- app_admin_login_executor: 管理者のログイン一覧(端末 ID は読まない)
GRANT SELECT (id, occurred_at, user_id, result, country, region, browser, os)
    ON public.login_events TO app_admin_login_executor;
GRANT SELECT (id, email) ON public.users TO app_admin_login_executor;

-- app_price_target_executor: 株価・基準価額の更新対象の列挙と反映
GRANT SELECT (security_code) ON public.asset_balances TO app_price_target_executor;
GRANT SELECT (security_code, current_price, prev_close, price_as_of, updated_at)
    ON public.security_prices TO app_price_target_executor;
GRANT INSERT, UPDATE ON public.security_prices TO app_price_target_executor;
GRANT SELECT (normalized_fund_name) ON public.fund_balances TO app_price_target_executor;
GRANT SELECT (normalized_name, association_fund_cd)
    ON public.fund_codes TO app_price_target_executor;
GRANT SELECT (association_fund_cd, nav, prev_day_change, price_as_of, updated_at, fetch_error, fetch_failed_at)
    ON public.fund_prices TO app_price_target_executor;
GRANT INSERT, UPDATE ON public.fund_prices TO app_price_target_executor;

-- app_login_event_prune_executor: 90 日より古いログイン記録の削除
GRANT SELECT (occurred_at), DELETE ON public.login_events TO app_login_event_prune_executor;

-- app_shared_write_executor: 配当キャッシュ・レート制御・銘柄マスタの書き込み
GRANT SELECT, INSERT, UPDATE ON public.dividend_per_share_cache TO app_shared_write_executor;
GRANT SELECT, INSERT, UPDATE ON public.market_data_provider_rate_control TO app_shared_write_executor;
GRANT SELECT, INSERT, DELETE ON public.stock TO app_shared_write_executor;

-- app_worker: 利用者 ID なしで読むのは共有の表だけ。残高ビューは利用者 Tx 内の app_restricted 経由で読む
GRANT SELECT ON public.stock, public.security_prices,
    public.dividend_per_share_cache, public.market_data_provider_rate_control,
    public.fund_codes, public.fund_prices TO app_worker;
GRANT EXECUTE ON FUNCTION public.fund_name_normalize(text),
    public.fund_expected_nav_date(timestamptz) TO app_worker;
