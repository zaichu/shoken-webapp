-- dividends テーブル: DOUBLE PRECISION → NUMERIC(18,6)
ALTER TABLE dividends
    ALTER COLUMN unit_price              TYPE NUMERIC(18,6) USING unit_price::NUMERIC,
    ALTER COLUMN shares                  TYPE NUMERIC(18,6) USING shares::NUMERIC,
    ALTER COLUMN dividends_before_tax    TYPE NUMERIC(18,6) USING dividends_before_tax::NUMERIC,
    ALTER COLUMN taxes                   TYPE NUMERIC(18,6) USING taxes::NUMERIC,
    ALTER COLUMN net_amount_received     TYPE NUMERIC(18,6) USING net_amount_received::NUMERIC;

-- domestic_stocks テーブル: DOUBLE PRECISION → NUMERIC(18,6)
ALTER TABLE domestic_stocks
    ALTER COLUMN shares                              TYPE NUMERIC(18,6) USING shares::NUMERIC,
    ALTER COLUMN asked_price                         TYPE NUMERIC(18,6) USING asked_price::NUMERIC,
    ALTER COLUMN proceeds                            TYPE NUMERIC(18,6) USING proceeds::NUMERIC,
    ALTER COLUMN purchase_price                      TYPE NUMERIC(18,6) USING purchase_price::NUMERIC,
    ALTER COLUMN realized_profit_and_loss            TYPE NUMERIC(18,6) USING realized_profit_and_loss::NUMERIC,
    ALTER COLUMN taxes                               TYPE NUMERIC(18,6) USING taxes::NUMERIC,
    ALTER COLUMN realized_profit_and_loss_after_tax  TYPE NUMERIC(18,6) USING realized_profit_and_loss_after_tax::NUMERIC;

-- mutualfunds テーブル: DOUBLE PRECISION → NUMERIC(18,6)
ALTER TABLE mutualfunds
    ALTER COLUMN shares                              TYPE NUMERIC(18,6) USING shares::NUMERIC,
    ALTER COLUMN exchange_rate                       TYPE NUMERIC(18,6) USING exchange_rate::NUMERIC,
    ALTER COLUMN cancellation_unit_price_yen         TYPE NUMERIC(18,6) USING cancellation_unit_price_yen::NUMERIC,
    ALTER COLUMN cancellation_amount_yen             TYPE NUMERIC(18,6) USING cancellation_amount_yen::NUMERIC,
    ALTER COLUMN average_acquisition_price_yen       TYPE NUMERIC(18,6) USING average_acquisition_price_yen::NUMERIC,
    ALTER COLUMN realized_profit_and_loss            TYPE NUMERIC(18,6) USING realized_profit_and_loss::NUMERIC,
    ALTER COLUMN taxes                               TYPE NUMERIC(18,6) USING taxes::NUMERIC,
    ALTER COLUMN realized_profit_and_loss_after_tax  TYPE NUMERIC(18,6) USING realized_profit_and_loss_after_tax::NUMERIC;

-- asset_balances テーブル: DOUBLE PRECISION → NUMERIC(18,6)
ALTER TABLE asset_balances
    ALTER COLUMN shares                  TYPE NUMERIC(18,6) USING shares::NUMERIC,
    ALTER COLUMN executing_shares        TYPE NUMERIC(18,6) USING executing_shares::NUMERIC,
    ALTER COLUMN average_purchase_price  TYPE NUMERIC(18,6) USING average_purchase_price::NUMERIC,
    ALTER COLUMN total_purchase_amount   TYPE NUMERIC(18,6) USING total_purchase_amount::NUMERIC,
    ALTER COLUMN current_price           TYPE NUMERIC(18,6) USING current_price::NUMERIC,
    ALTER COLUMN daily_change            TYPE NUMERIC(18,6) USING daily_change::NUMERIC,
    ALTER COLUMN market_value            TYPE NUMERIC(18,6) USING market_value::NUMERIC,
    ALTER COLUMN profit_loss_rate        TYPE NUMERIC(18,6) USING profit_loss_rate::NUMERIC;
