-- provider 非依存の table 名へ rename（J-Quants 固有名の解消）
ALTER TABLE jquants_dividend_cache RENAME TO dividend_per_share_cache;
ALTER TABLE jquants_rate_control RENAME TO market_data_provider_rate_control;
