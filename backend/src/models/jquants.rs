use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct StatementsQuery {
    pub code: String,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatementsResponse {
    pub statements: Vec<StatementsData>,
    /// ページネーションキー（データが大量の場合に設定される）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatementsData {
    #[serde(rename = "DisclosedDate")]
    pub disclosed_date: String,

    #[serde(rename = "DisclosedTime")]
    pub disclosed_time: Option<String>,

    #[serde(rename = "LocalCode")]
    pub local_code: String,

    #[serde(rename = "DisclosureNumber")]
    pub disclosure_number: Option<String>,

    #[serde(rename = "TypeOfDocument")]
    pub type_of_document: String,

    #[serde(rename = "TypeOfCurrentPeriod")]
    pub type_of_current_period: String,

    #[serde(rename = "CurrentPeriodStartDate")]
    pub current_period_start_date: String,

    #[serde(rename = "CurrentPeriodEndDate")]
    pub current_period_end_date: String,

    #[serde(rename = "CurrentFiscalYearStartDate")]
    pub current_fiscal_year_start_date: String,

    #[serde(rename = "CurrentFiscalYearEndDate")]
    pub current_fiscal_year_end_date: String,

    #[serde(rename = "NextFiscalYearStartDate")]
    pub next_fiscal_year_start_date: Option<String>,

    #[serde(rename = "NextFiscalYearEndDate")]
    pub next_fiscal_year_end_date: Option<String>,

    #[serde(rename = "NetSales")]
    pub net_sales: Option<String>,

    #[serde(rename = "OperatingProfit")]
    pub operating_profit: Option<String>,

    #[serde(rename = "OrdinaryProfit")]
    pub ordinary_profit: Option<String>,

    #[serde(rename = "Profit")]
    pub profit: Option<String>,

    #[serde(rename = "EarningsPerShare")]
    pub earnings_per_share: Option<String>,

    #[serde(rename = "DilutedEarningsPerShare")]
    pub diluted_earnings_per_share: Option<String>,

    #[serde(rename = "TotalAssets")]
    pub total_assets: Option<String>,

    #[serde(rename = "Equity")]
    pub equity: Option<String>,

    #[serde(rename = "EquityToAssetRatio")]
    pub equity_to_asset_ratio: Option<String>,

    #[serde(rename = "BookValuePerShare")]
    pub book_value_per_share: Option<String>,

    #[serde(rename = "CashFlowsFromOperatingActivities")]
    pub cash_flows_from_operating_activities: Option<String>,

    #[serde(rename = "CashFlowsFromInvestingActivities")]
    pub cash_flows_from_investing_activities: Option<String>,

    #[serde(rename = "CashFlowsFromFinancingActivities")]
    pub cash_flows_from_financing_activities: Option<String>,

    #[serde(rename = "CashAndEquivalents")]
    pub cash_and_equivalents: Option<String>,

    #[serde(rename = "ResultDividendPerShare1stQuarter")]
    pub result_dividend_per_share_1st_quarter: Option<String>,

    #[serde(rename = "ResultDividendPerShare2ndQuarter")]
    pub result_dividend_per_share_2nd_quarter: Option<String>,

    #[serde(rename = "ResultDividendPerShare3rdQuarter")]
    pub result_dividend_per_share_3rd_quarter: Option<String>,

    #[serde(rename = "ResultDividendPerShareFiscalYearEnd")]
    pub result_dividend_per_share_fiscal_year_end: Option<String>,

    #[serde(rename = "ResultDividendPerShareAnnual")]
    pub result_dividend_per_share_annual: Option<String>,

    #[serde(rename = "DistributionsPerUnit(REIT)")]
    pub distributions_per_unit_reit: Option<String>,

    #[serde(rename = "ResultTotalDividendPaidAnnual")]
    pub result_total_dividend_paid_annual: Option<String>,

    #[serde(rename = "ResultPayoutRatioAnnual")]
    pub result_payout_ratio_annual: Option<String>,

    #[serde(rename = "ForecastDividendPerShare1stQuarter")]
    pub forecast_dividend_per_share_1st_quarter: Option<String>,

    #[serde(rename = "ForecastDividendPerShare2ndQuarter")]
    pub forecast_dividend_per_share_2nd_quarter: Option<String>,

    #[serde(rename = "ForecastDividendPerShare3rdQuarter")]
    pub forecast_dividend_per_share_3rd_quarter: Option<String>,

    #[serde(rename = "ForecastDividendPerShareFiscalYearEnd")]
    pub forecast_dividend_per_share_fiscal_year_end: Option<String>,

    #[serde(rename = "ForecastDividendPerShareAnnual")]
    pub forecast_dividend_per_share_annual: Option<String>,

    #[serde(rename = "ForecastDistributionsPerUnit(REIT)")]
    pub forecast_distributions_per_unit_reit: Option<String>,

    #[serde(rename = "ForecastTotalDividendPaidAnnual")]
    pub forecast_total_dividend_paid_annual: Option<String>,

    #[serde(rename = "ForecastPayoutRatioAnnual")]
    pub forecast_payout_ratio_annual: Option<String>,

    #[serde(rename = "NextYearForecastDividendPerShare1stQuarter")]
    pub next_year_forecast_dividend_per_share_1st_quarter: Option<String>,

    #[serde(rename = "NextYearForecastDividendPerShare2ndQuarter")]
    pub next_year_forecast_dividend_per_share_2nd_quarter: Option<String>,

    #[serde(rename = "NextYearForecastDividendPerShare3rdQuarter")]
    pub next_year_forecast_dividend_per_share_3rd_quarter: Option<String>,

    #[serde(rename = "NextYearForecastDividendPerShareFiscalYearEnd")]
    pub next_year_forecast_dividend_per_share_fiscal_year_end: Option<String>,

    #[serde(rename = "NextYearForecastDividendPerShareAnnual")]
    pub next_year_forecast_dividend_per_share_annual: Option<String>,

    #[serde(rename = "NextYearForecastDistributionsPerUnit(REIT)")]
    pub next_year_forecast_distributions_per_unit_reit: Option<String>,

    #[serde(rename = "NextYearForecastPayoutRatioAnnual")]
    pub next_year_forecast_payout_ratio_annual: Option<String>,

    #[serde(rename = "ForecastNetSales2ndQuarter")]
    pub forecast_net_sales_2nd_quarter: Option<String>,

    #[serde(rename = "ForecastOperatingProfit2ndQuarter")]
    pub forecast_operating_profit_2nd_quarter: Option<String>,

    #[serde(rename = "ForecastOrdinaryProfit2ndQuarter")]
    pub forecast_ordinary_profit_2nd_quarter: Option<String>,

    #[serde(rename = "ForecastProfit2ndQuarter")]
    pub forecast_profit_2nd_quarter: Option<String>,

    #[serde(rename = "ForecastEarningsPerShare2ndQuarter")]
    pub forecast_earnings_per_share_2nd_quarter: Option<String>,

    #[serde(rename = "NextYearForecastNetSales2ndQuarter")]
    pub next_year_forecast_net_sales_2nd_quarter: Option<String>,

    #[serde(rename = "NextYearForecastOperatingProfit2ndQuarter")]
    pub next_year_forecast_operating_profit_2nd_quarter: Option<String>,

    #[serde(rename = "NextYearForecastOrdinaryProfit2ndQuarter")]
    pub next_year_forecast_ordinary_profit_2nd_quarter: Option<String>,

    #[serde(rename = "NextYearForecastProfit2ndQuarter")]
    pub next_year_forecast_profit_2nd_quarter: Option<String>,

    #[serde(rename = "NextYearForecastEarningsPerShare2ndQuarter")]
    pub next_year_forecast_earnings_per_share_2nd_quarter: Option<String>,

    #[serde(rename = "ForecastNetSales")]
    pub forecast_net_sales: Option<String>,

    #[serde(rename = "ForecastOperatingProfit")]
    pub forecast_operating_profit: Option<String>,

    #[serde(rename = "ForecastOrdinaryProfit")]
    pub forecast_ordinary_profit: Option<String>,

    #[serde(rename = "ForecastProfit")]
    pub forecast_profit: Option<String>,

    #[serde(rename = "ForecastEarningsPerShare")]
    pub forecast_earnings_per_share: Option<String>,

    #[serde(rename = "NextYearForecastNetSales")]
    pub next_year_forecast_net_sales: Option<String>,

    #[serde(rename = "NextYearForecastOperatingProfit")]
    pub next_year_forecast_operating_profit: Option<String>,

    #[serde(rename = "NextYearForecastOrdinaryProfit")]
    pub next_year_forecast_ordinary_profit: Option<String>,

    #[serde(rename = "NextYearForecastProfit")]
    pub next_year_forecast_profit: Option<String>,

    #[serde(rename = "NextYearForecastEarningsPerShare")]
    pub next_year_forecast_earnings_per_share: Option<String>,

    #[serde(rename = "MaterialChangesInSubsidiaries")]
    pub material_changes_in_subsidiaries: Option<String>,

    #[serde(rename = "SignificantChangesInTheScopeOfConsolidation")]
    pub significant_changes_in_the_scope_of_consolidation: Option<String>,

    #[serde(rename = "ChangesBasedOnRevisionsOfAccountingStandard")]
    pub changes_based_on_revisions_of_accounting_standard: Option<String>,

    #[serde(rename = "ChangesOtherThanOnesBasedOnRevisionsOfAccountingStandard")]
    pub changes_other_than_ones_based_on_revisions_of_accounting_standard: Option<String>,

    #[serde(rename = "ChangesInAccountingEstimates")]
    pub changes_in_accounting_estimates: Option<String>,

    #[serde(rename = "RetrospectiveRestatement")]
    pub retrospective_restatement: Option<String>,

    #[serde(
        rename = "NumberOfIssuedAndOutstandingSharesAtTheEndOfFiscalYearIncludingTreasuryStock"
    )]
    pub number_of_issued_and_outstanding_shares_at_the_end_of_fiscal_year_including_treasury_stock:
        Option<String>,

    #[serde(rename = "NumberOfTreasuryStockAtTheEndOfFiscalYear")]
    pub number_of_treasury_stock_at_the_end_of_fiscal_year: Option<String>,

    #[serde(rename = "AverageNumberOfShares")]
    pub average_number_of_shares: Option<String>,

    #[serde(rename = "NonConsolidatedNetSales")]
    pub non_consolidated_net_sales: Option<String>,

    #[serde(rename = "NonConsolidatedOperatingProfit")]
    pub non_consolidated_operating_profit: Option<String>,

    #[serde(rename = "NonConsolidatedOrdinaryProfit")]
    pub non_consolidated_ordinary_profit: Option<String>,

    #[serde(rename = "NonConsolidatedProfit")]
    pub non_consolidated_profit: Option<String>,

    #[serde(rename = "NonConsolidatedEarningsPerShare")]
    pub non_consolidated_earnings_per_share: Option<String>,

    #[serde(rename = "NonConsolidatedTotalAssets")]
    pub non_consolidated_total_assets: Option<String>,

    #[serde(rename = "NonConsolidatedEquity")]
    pub non_consolidated_equity: Option<String>,

    #[serde(rename = "NonConsolidatedEquityToAssetRatio")]
    pub non_consolidated_equity_to_asset_ratio: Option<String>,

    #[serde(rename = "NonConsolidatedBookValuePerShare")]
    pub non_consolidated_book_value_per_share: Option<String>,

    #[serde(rename = "ForecastNonConsolidatedNetSales2ndQuarter")]
    pub forecast_non_consolidated_net_sales_2nd_quarter: Option<String>,

    #[serde(rename = "ForecastNonConsolidatedOperatingProfit2ndQuarter")]
    pub forecast_non_consolidated_operating_profit_2nd_quarter: Option<String>,

    #[serde(rename = "ForecastNonConsolidatedOrdinaryProfit2ndQuarter")]
    pub forecast_non_consolidated_ordinary_profit_2nd_quarter: Option<String>,

    #[serde(rename = "ForecastNonConsolidatedProfit2ndQuarter")]
    pub forecast_non_consolidated_profit_2nd_quarter: Option<String>,

    #[serde(rename = "ForecastNonConsolidatedEarningsPerShare2ndQuarter")]
    pub forecast_non_consolidated_earnings_per_share_2nd_quarter: Option<String>,

    #[serde(rename = "NextYearForecastNonConsolidatedNetSales2ndQuarter")]
    pub next_year_forecast_non_consolidated_net_sales_2nd_quarter: Option<String>,

    #[serde(rename = "NextYearForecastNonConsolidatedOperatingProfit2ndQuarter")]
    pub next_year_forecast_non_consolidated_operating_profit_2nd_quarter: Option<String>,

    #[serde(rename = "NextYearForecastNonConsolidatedOrdinaryProfit2ndQuarter")]
    pub next_year_forecast_non_consolidated_ordinary_profit_2nd_quarter: Option<String>,

    #[serde(rename = "NextYearForecastNonConsolidatedProfit2ndQuarter")]
    pub next_year_forecast_non_consolidated_profit_2nd_quarter: Option<String>,

    #[serde(rename = "NextYearForecastNonConsolidatedEarningsPerShare2ndQuarter")]
    pub next_year_forecast_non_consolidated_earnings_per_share_2nd_quarter: Option<String>,

    #[serde(rename = "ForecastNonConsolidatedNetSales")]
    pub forecast_non_consolidated_net_sales: Option<String>,

    #[serde(rename = "ForecastNonConsolidatedOperatingProfit")]
    pub forecast_non_consolidated_operating_profit: Option<String>,

    #[serde(rename = "ForecastNonConsolidatedOrdinaryProfit")]
    pub forecast_non_consolidated_ordinary_profit: Option<String>,

    #[serde(rename = "ForecastNonConsolidatedProfit")]
    pub forecast_non_consolidated_profit: Option<String>,

    #[serde(rename = "ForecastNonConsolidatedEarningsPerShare")]
    pub forecast_non_consolidated_earnings_per_share: Option<String>,

    #[serde(rename = "NextYearForecastNonConsolidatedNetSales")]
    pub next_year_forecast_non_consolidated_net_sales: Option<String>,

    #[serde(rename = "NextYearForecastNonConsolidatedOperatingProfit")]
    pub next_year_forecast_non_consolidated_operating_profit: Option<String>,

    #[serde(rename = "NextYearForecastNonConsolidatedOrdinaryProfit")]
    pub next_year_forecast_non_consolidated_ordinary_profit: Option<String>,

    #[serde(rename = "NextYearForecastNonConsolidatedProfit")]
    pub next_year_forecast_non_consolidated_profit: Option<String>,

    #[serde(rename = "NextYearForecastNonConsolidatedEarningsPerShare")]
    pub next_year_forecast_non_consolidated_earnings_per_share: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_statements_query_deserialize() {
        let query = StatementsQuery {
            code: "7203".to_string(),
            from: Some("2023-01-01".to_string()),
            to: Some("2023-12-31".to_string()),
        };

        assert_eq!(query.code, "7203");
        assert_eq!(query.from.unwrap(), "2023-01-01");
        assert_eq!(query.to.unwrap(), "2023-12-31");
    }

    #[test]
    fn test_statements_query_optional_params() {
        let query = StatementsQuery {
            code: "7203".to_string(),
            from: None,
            to: None,
        };

        assert_eq!(query.code, "7203");
        assert!(query.from.is_none());
        assert!(query.to.is_none());
    }

    #[test]
    fn test_statements_data_deserialize() {
        let json_data = json!({
            "DisclosedDate": "2023-11-14",
            "DisclosedTime": "15:00:00",
            "LocalCode": "72030",
            "DisclosureNumber": "20231114502171",
            "TypeOfDocument": "決算短信",
            "TypeOfCurrentPeriod": "2Q",
            "CurrentPeriodStartDate": "2023-04-01",
            "CurrentPeriodEndDate": "2023-09-30",
            "CurrentFiscalYearStartDate": "2023-04-01",
            "CurrentFiscalYearEndDate": "2024-03-31",
            "NextFiscalYearStartDate": "2024-04-01",
            "NextFiscalYearEndDate": "2025-03-31",
            "NetSales": "18733067000000",
            "OperatingProfit": "1686297000000",
            "NextYearForecastDividendPerShareAnnual": "50.00"
        });

        let statements_data: StatementsData = serde_json::from_value(json_data).unwrap();

        assert_eq!(statements_data.disclosed_date, "2023-11-14");
        assert_eq!(statements_data.local_code, "72030");
        assert_eq!(
            statements_data.net_sales,
            Some("18733067000000".to_string())
        );
        assert_eq!(
            statements_data.next_year_forecast_dividend_per_share_annual,
            Some("50.00".to_string())
        );
    }

    #[test]
    fn test_statements_response_deserialize() {
        let json_data = json!({
            "statements": [
                {
                    "DisclosedDate": "2023-11-14",
                    "LocalCode": "72030",
                    "TypeOfDocument": "決算短信",
                    "TypeOfCurrentPeriod": "2Q",
                    "CurrentPeriodStartDate": "2023-04-01",
                    "CurrentPeriodEndDate": "2023-09-30",
                    "CurrentFiscalYearStartDate": "2023-04-01",
                    "CurrentFiscalYearEndDate": "2024-03-31"
                }
            ]
        });

        let response: StatementsResponse = serde_json::from_value(json_data).unwrap();

        assert_eq!(response.statements.len(), 1);
        assert_eq!(response.statements[0].disclosed_date, "2023-11-14");
        assert_eq!(response.statements[0].local_code, "72030");
    }

    #[test]
    fn test_statements_response_empty() {
        let json_data = json!({ "statements": [] });
        let response: StatementsResponse = serde_json::from_value(json_data).unwrap();

        assert_eq!(response.statements.len(), 0);
    }
}
