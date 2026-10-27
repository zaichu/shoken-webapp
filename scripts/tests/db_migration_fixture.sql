TRUNCATE fund_codes, market_data_provider_rate_control;
INSERT INTO users (id, google_id, email, name) VALUES
('00000000-0000-0000-0000-000000000001', 'fictional-user', 'fictional@example.invalid', '架空の移行確認ユーザー');
INSERT INTO sessions (user_id, expires_at, token_hash) VALUES
('00000000-0000-0000-0000-000000000001', now() + interval '7 days', decode('01', 'hex')),
('00000000-0000-0000-0000-000000000001', now() - interval '1 day', decode('02', 'hex'));
INSERT INTO dividends (user_id, settlement_date, product, account, security_code, security_name,
unit_price, shares, dividends_before_tax, taxes, net_amount_received) VALUES
('00000000-0000-0000-0000-000000000001', '2026-10-01', '株式', '特定', '9999', '架空株式会社', 10.5, 200, 2100, 420, 1680);
INSERT INTO domestic_stocks (user_id, trade_date, settlement_date, security_code, security_name,
account, shares, asked_price, proceeds, purchase_price, realized_profit_and_loss, taxes,
realized_profit_and_loss_after_tax, content_hash, occurrence_index) VALUES
('00000000-0000-0000-0000-000000000001', '2026-10-01', '2026-10-03', '9999', '架空株式会社',
'特定', 200, 1234.56, 246912, 200000, 46912, 9000, 37912, 'fictional-hash', 0);
INSERT INTO mutualfunds (user_id, trade_date, settlement_date, fund_name, account, shares,
exchange_rate, cancellation_unit_price_yen, cancellation_amount_yen, average_acquisition_price_yen,
realized_profit_and_loss, taxes, realized_profit_and_loss_after_tax) VALUES
('00000000-0000-0000-0000-000000000001', '2026-10-01', '2026-10-03', '架空ファンド', 'NISA',
10000, 1, 20000, 20000, 15000, 5000, 0, 5000);
INSERT INTO asset_balances (user_id, security_code, security_name, shares, average_purchase_price,
total_purchase_amount, csv_current_price, csv_daily_change, account) VALUES
('00000000-0000-0000-0000-000000000001', '9999', '架空株式会社', 200, 1000, 200000, 1234.56, -12.34, '特定');
INSERT INTO stock (date, code, name, market_category) VALUES ('2026-10-01', '9999', '架空株式会社', '架空市場');
INSERT INTO dividend_per_share_cache (security_code, dividend_per_share, status) VALUES ('9999', 10.5, 'success');
INSERT INTO market_data_provider_rate_control (provider, next_available_at) VALUES ('fictional-provider', '2026-11-02 00:00:00+00');
INSERT INTO security_prices (security_code, current_price, price_as_of, prev_close) VALUES ('9999', 1234.56, '2026-10-01', 1246.90);
INSERT INTO fund_codes VALUES (public.fund_name_normalize('架空ファンド'), 'TEST0001', '架空ファンド');
INSERT INTO fund_prices (association_fund_cd, nav, prev_day_change, price_as_of, fetch_error, fetch_failed_at)
VALUES ('TEST0001', 20000, -123, '2026-10-01', '架空の取得エラー', '2026-10-02 00:00:00+00');
INSERT INTO fund_balances (user_id, fund_name, account, shares, average_purchase_price,
total_purchase_amount, base_price, market_value, profit_loss, price_as_of) VALUES
('00000000-0000-0000-0000-000000000001', '架空ファンド', 'NISA', 10000, 15000, 15000, 20000, 20000, 5000, '2026-10-01');
INSERT INTO csv_import_history (user_id, kind) VALUES
('00000000-0000-0000-0000-000000000001', 'dividend'),
('00000000-0000-0000-0000-000000000001', 'fundbalance');
SELECT setval('migration_test_seq', 321, true);
