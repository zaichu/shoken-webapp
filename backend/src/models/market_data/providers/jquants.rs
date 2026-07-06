#[cfg(test)]
mod query;
mod response;

pub use response::{FinSummaryData, FinSummaryResponse};

use crate::models::market_data::financial_statement::{
    FinancialStatementData, FinancialStatementsResponse,
};

impl From<FinSummaryResponse> for FinancialStatementsResponse {
    fn from(value: FinSummaryResponse) -> Self {
        Self {
            data: value.data.into_iter().map(Into::into).collect(),
            pagination_key: value.pagination_key,
        }
    }
}

impl From<FinSummaryData> for FinancialStatementData {
    fn from(value: FinSummaryData) -> Self {
        Self {
            disclosed_date: value.disclosed_date,
            disclosed_time: value.disclosed_time,
            local_code: value.local_code,
            disclosure_number: value.disclosure_number,
            type_of_document: value.type_of_document,
            type_of_current_period: value.type_of_current_period,
            current_period_start_date: value.current_period_start_date,
            current_period_end_date: value.current_period_end_date,
            current_fiscal_year_start_date: value.current_fiscal_year_start_date,
            current_fiscal_year_end_date: value.current_fiscal_year_end_date,
            next_fiscal_year_start_date: value.next_fiscal_year_start_date,
            next_fiscal_year_end_date: value.next_fiscal_year_end_date,
            net_sales: value.net_sales,
            operating_profit: value.operating_profit,
            ordinary_profit: value.ordinary_profit,
            profit: value.profit,
            earnings_per_share: value.earnings_per_share,
            diluted_earnings_per_share: value.diluted_earnings_per_share,
            total_assets: value.total_assets,
            equity: value.equity,
            equity_to_asset_ratio: value.equity_to_asset_ratio,
            book_value_per_share: value.book_value_per_share,
            cash_flows_from_operating_activities: value.cash_flows_from_operating_activities,
            cash_flows_from_investing_activities: value.cash_flows_from_investing_activities,
            cash_flows_from_financing_activities: value.cash_flows_from_financing_activities,
            cash_and_equivalents: value.cash_and_equivalents,
            result_dividend_per_share_1st_quarter: value.result_dividend_per_share_1st_quarter,
            result_dividend_per_share_2nd_quarter: value.result_dividend_per_share_2nd_quarter,
            result_dividend_per_share_3rd_quarter: value.result_dividend_per_share_3rd_quarter,
            result_dividend_per_share_fiscal_year_end: value
                .result_dividend_per_share_fiscal_year_end,
            result_dividend_per_share_annual: value.result_dividend_per_share_annual,
            distributions_per_unit_reit: value.distributions_per_unit_reit,
            result_total_dividend_paid_annual: value.result_total_dividend_paid_annual,
            result_payout_ratio_annual: value.result_payout_ratio_annual,
            forecast_dividend_per_share_1st_quarter: value
                .forecast_dividend_per_share_1st_quarter,
            forecast_dividend_per_share_2nd_quarter: value
                .forecast_dividend_per_share_2nd_quarter,
            forecast_dividend_per_share_3rd_quarter: value
                .forecast_dividend_per_share_3rd_quarter,
            forecast_dividend_per_share_fiscal_year_end: value
                .forecast_dividend_per_share_fiscal_year_end,
            forecast_dividend_per_share_annual: value.forecast_dividend_per_share_annual,
            forecast_distributions_per_unit_reit: value.forecast_distributions_per_unit_reit,
            forecast_total_dividend_paid_annual: value.forecast_total_dividend_paid_annual,
            forecast_payout_ratio_annual: value.forecast_payout_ratio_annual,
            next_year_forecast_dividend_per_share_1st_quarter: value
                .next_year_forecast_dividend_per_share_1st_quarter,
            next_year_forecast_dividend_per_share_2nd_quarter: value
                .next_year_forecast_dividend_per_share_2nd_quarter,
            next_year_forecast_dividend_per_share_3rd_quarter: value
                .next_year_forecast_dividend_per_share_3rd_quarter,
            next_year_forecast_dividend_per_share_fiscal_year_end: value
                .next_year_forecast_dividend_per_share_fiscal_year_end,
            next_year_forecast_dividend_per_share_annual: value
                .next_year_forecast_dividend_per_share_annual,
            next_year_forecast_distributions_per_unit_reit: value
                .next_year_forecast_distributions_per_unit_reit,
            next_year_forecast_payout_ratio_annual: value.next_year_forecast_payout_ratio_annual,
            forecast_net_sales_2nd_quarter: value.forecast_net_sales_2nd_quarter,
            forecast_operating_profit_2nd_quarter: value.forecast_operating_profit_2nd_quarter,
            forecast_ordinary_profit_2nd_quarter: value.forecast_ordinary_profit_2nd_quarter,
            forecast_profit_2nd_quarter: value.forecast_profit_2nd_quarter,
            forecast_earnings_per_share_2nd_quarter: value
                .forecast_earnings_per_share_2nd_quarter,
            next_year_forecast_net_sales_2nd_quarter: value
                .next_year_forecast_net_sales_2nd_quarter,
            next_year_forecast_operating_profit_2nd_quarter: value
                .next_year_forecast_operating_profit_2nd_quarter,
            next_year_forecast_ordinary_profit_2nd_quarter: value
                .next_year_forecast_ordinary_profit_2nd_quarter,
            next_year_forecast_profit_2nd_quarter: value.next_year_forecast_profit_2nd_quarter,
            next_year_forecast_earnings_per_share_2nd_quarter: value
                .next_year_forecast_earnings_per_share_2nd_quarter,
            forecast_net_sales: value.forecast_net_sales,
            forecast_operating_profit: value.forecast_operating_profit,
            forecast_ordinary_profit: value.forecast_ordinary_profit,
            forecast_profit: value.forecast_profit,
            forecast_earnings_per_share: value.forecast_earnings_per_share,
            next_year_forecast_net_sales: value.next_year_forecast_net_sales,
            next_year_forecast_operating_profit: value.next_year_forecast_operating_profit,
            next_year_forecast_ordinary_profit: value.next_year_forecast_ordinary_profit,
            next_year_forecast_profit: value.next_year_forecast_profit,
            next_year_forecast_earnings_per_share: value.next_year_forecast_earnings_per_share,
            material_changes_in_subsidiaries: value.material_changes_in_subsidiaries,
            significant_changes_in_the_scope_of_consolidation: value
                .significant_changes_in_the_scope_of_consolidation,
            changes_based_on_revisions_of_accounting_standard: value
                .changes_based_on_revisions_of_accounting_standard,
            changes_other_than_ones_based_on_revisions_of_accounting_standard: value
                .changes_other_than_ones_based_on_revisions_of_accounting_standard,
            changes_in_accounting_estimates: value.changes_in_accounting_estimates,
            retrospective_restatement: value.retrospective_restatement,
            number_of_issued_and_outstanding_shares_at_the_end_of_fiscal_year_including_treasury_stock: value
                .number_of_issued_and_outstanding_shares_at_the_end_of_fiscal_year_including_treasury_stock,
            number_of_treasury_stock_at_the_end_of_fiscal_year: value
                .number_of_treasury_stock_at_the_end_of_fiscal_year,
            average_number_of_shares: value.average_number_of_shares,
            non_consolidated_net_sales: value.non_consolidated_net_sales,
            non_consolidated_operating_profit: value.non_consolidated_operating_profit,
            non_consolidated_ordinary_profit: value.non_consolidated_ordinary_profit,
            non_consolidated_profit: value.non_consolidated_profit,
            non_consolidated_earnings_per_share: value.non_consolidated_earnings_per_share,
            non_consolidated_total_assets: value.non_consolidated_total_assets,
            non_consolidated_equity: value.non_consolidated_equity,
            non_consolidated_equity_to_asset_ratio: value.non_consolidated_equity_to_asset_ratio,
            non_consolidated_book_value_per_share: value.non_consolidated_book_value_per_share,
            forecast_non_consolidated_net_sales_2nd_quarter: value
                .forecast_non_consolidated_net_sales_2nd_quarter,
            forecast_non_consolidated_operating_profit_2nd_quarter: value
                .forecast_non_consolidated_operating_profit_2nd_quarter,
            forecast_non_consolidated_ordinary_profit_2nd_quarter: value
                .forecast_non_consolidated_ordinary_profit_2nd_quarter,
            forecast_non_consolidated_profit_2nd_quarter: value
                .forecast_non_consolidated_profit_2nd_quarter,
            forecast_non_consolidated_earnings_per_share_2nd_quarter: value
                .forecast_non_consolidated_earnings_per_share_2nd_quarter,
            next_year_forecast_non_consolidated_net_sales_2nd_quarter: value
                .next_year_forecast_non_consolidated_net_sales_2nd_quarter,
            next_year_forecast_non_consolidated_operating_profit_2nd_quarter: value
                .next_year_forecast_non_consolidated_operating_profit_2nd_quarter,
            next_year_forecast_non_consolidated_ordinary_profit_2nd_quarter: value
                .next_year_forecast_non_consolidated_ordinary_profit_2nd_quarter,
            next_year_forecast_non_consolidated_profit_2nd_quarter: value
                .next_year_forecast_non_consolidated_profit_2nd_quarter,
            next_year_forecast_non_consolidated_earnings_per_share_2nd_quarter: value
                .next_year_forecast_non_consolidated_earnings_per_share_2nd_quarter,
            forecast_non_consolidated_net_sales: value.forecast_non_consolidated_net_sales,
            forecast_non_consolidated_operating_profit: value
                .forecast_non_consolidated_operating_profit,
            forecast_non_consolidated_ordinary_profit: value
                .forecast_non_consolidated_ordinary_profit,
            forecast_non_consolidated_profit: value.forecast_non_consolidated_profit,
            forecast_non_consolidated_earnings_per_share: value
                .forecast_non_consolidated_earnings_per_share,
            next_year_forecast_non_consolidated_net_sales: value
                .next_year_forecast_non_consolidated_net_sales,
            next_year_forecast_non_consolidated_operating_profit: value
                .next_year_forecast_non_consolidated_operating_profit,
            next_year_forecast_non_consolidated_ordinary_profit: value
                .next_year_forecast_non_consolidated_ordinary_profit,
            next_year_forecast_non_consolidated_profit: value
                .next_year_forecast_non_consolidated_profit,
            next_year_forecast_non_consolidated_earnings_per_share: value
                .next_year_forecast_non_consolidated_earnings_per_share,
        }
    }
}
