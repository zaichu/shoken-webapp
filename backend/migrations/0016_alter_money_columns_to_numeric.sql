-- dividends テーブル: DOUBLE PRECISION → NUMERIC（無制限精度）
-- NUMERIC(18,6) ではなく NUMERIC にすることで ::text 変換時に trailing zeros が付かず
-- INSERT 時の content_hash と一致する表現を保証する
ALTER TABLE dividends
    ALTER COLUMN unit_price              TYPE NUMERIC USING unit_price::NUMERIC,
    ALTER COLUMN shares                  TYPE NUMERIC USING shares::NUMERIC,
    ALTER COLUMN dividends_before_tax    TYPE NUMERIC USING dividends_before_tax::NUMERIC,
    ALTER COLUMN taxes                   TYPE NUMERIC USING taxes::NUMERIC,
    ALTER COLUMN net_amount_received     TYPE NUMERIC USING net_amount_received::NUMERIC;

-- domestic_stocks テーブル: DOUBLE PRECISION → NUMERIC
ALTER TABLE domestic_stocks
    ALTER COLUMN shares                              TYPE NUMERIC USING shares::NUMERIC,
    ALTER COLUMN asked_price                         TYPE NUMERIC USING asked_price::NUMERIC,
    ALTER COLUMN proceeds                            TYPE NUMERIC USING proceeds::NUMERIC,
    ALTER COLUMN purchase_price                      TYPE NUMERIC USING purchase_price::NUMERIC,
    ALTER COLUMN realized_profit_and_loss            TYPE NUMERIC USING realized_profit_and_loss::NUMERIC,
    ALTER COLUMN taxes                               TYPE NUMERIC USING taxes::NUMERIC,
    ALTER COLUMN realized_profit_and_loss_after_tax  TYPE NUMERIC USING realized_profit_and_loss_after_tax::NUMERIC;

-- mutualfunds テーブル: DOUBLE PRECISION → NUMERIC
ALTER TABLE mutualfunds
    ALTER COLUMN shares                              TYPE NUMERIC USING shares::NUMERIC,
    ALTER COLUMN exchange_rate                       TYPE NUMERIC USING exchange_rate::NUMERIC,
    ALTER COLUMN cancellation_unit_price_yen         TYPE NUMERIC USING cancellation_unit_price_yen::NUMERIC,
    ALTER COLUMN cancellation_amount_yen             TYPE NUMERIC USING cancellation_amount_yen::NUMERIC,
    ALTER COLUMN average_acquisition_price_yen       TYPE NUMERIC USING average_acquisition_price_yen::NUMERIC,
    ALTER COLUMN realized_profit_and_loss            TYPE NUMERIC USING realized_profit_and_loss::NUMERIC,
    ALTER COLUMN taxes                               TYPE NUMERIC USING taxes::NUMERIC,
    ALTER COLUMN realized_profit_and_loss_after_tax  TYPE NUMERIC USING realized_profit_and_loss_after_tax::NUMERIC;

-- asset_balances テーブル: DOUBLE PRECISION → NUMERIC
ALTER TABLE asset_balances
    ALTER COLUMN shares                  TYPE NUMERIC USING shares::NUMERIC,
    ALTER COLUMN executing_shares        TYPE NUMERIC USING executing_shares::NUMERIC,
    ALTER COLUMN average_purchase_price  TYPE NUMERIC USING average_purchase_price::NUMERIC,
    ALTER COLUMN total_purchase_amount   TYPE NUMERIC USING total_purchase_amount::NUMERIC,
    ALTER COLUMN current_price           TYPE NUMERIC USING current_price::NUMERIC,
    ALTER COLUMN daily_change            TYPE NUMERIC USING daily_change::NUMERIC,
    ALTER COLUMN market_value            TYPE NUMERIC USING market_value::NUMERIC,
    ALTER COLUMN profit_loss_rate        TYPE NUMERIC USING profit_loss_rate::NUMERIC;
