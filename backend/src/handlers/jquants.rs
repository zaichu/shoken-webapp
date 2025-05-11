use crate::errors::AppError;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    refresh_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RefreshTokenRequest {
    refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct IdTokenResponse {
    id_token: String,
}

#[derive(Debug, Deserialize)]
pub struct StatementsQuery {
    code: String,
    from: Option<String>,
    to: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatementsResponse {
    statements: Vec<StatementsData>,
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

pub async fn authenticate(State(state): State<AppState>) -> Result<Json<AuthResponse>, AppError> {
    let client = &state.client;
    let url = "https://api.jquants.com/v1/token/auth_user";

    // SecretsからJQuantsの認証情報を取得
    let mailaddress = state
        .secrets
        .get("JQUANTS_EMAIL")
        .ok_or_else(|| AppError::ApiError("JQUANTS_EMAIL not found in secrets".to_string()))?;
    let password = state
        .secrets
        .get("JQUANTS_PASSWORD")
        .ok_or_else(|| AppError::ApiError("JQUANTS_PASSWORD not found in secrets".to_string()))?;

    let payload = serde_json::json!({
        "mailaddress": mailaddress,
        "password": password
    });

    let response = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::ApiError(format!(
            "JQuants API error: {}",
            error_text
        )));
    }

    let json_result = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    let refresh_token = json_result
        .get("refreshToken")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ApiError("refreshToken not found in response".to_string()))?
        .to_string();

    Ok(Json(AuthResponse { refresh_token }))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<IdTokenResponse>, AppError> {
    let client = &state.client;
    let url = format!(
        "https://api.jquants.com/v1/token/auth_refresh?refreshtoken={}",
        payload.refresh_token
    );

    let response = client
        .post(url)
        .send()
        .await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::ApiError(format!(
            "JQuants API error: {}",
            error_text
        )));
    }

    let json_result = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    let id_token = json_result
        .get("idToken")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ApiError("idToken not found in response".to_string()))?
        .to_string();

    Ok(Json(IdTokenResponse { id_token }))
}

pub async fn get_statements(
    State(state): State<AppState>,
    Query(params): Query<StatementsQuery>,
    headers: axum::http::HeaderMap,
) -> Result<Json<StatementsResponse>, AppError> {
    let client = &state.client;
    println!("params: {:?}", params);

    // Authorization headerからトークンを取得
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::ApiError("Authorization header not found".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::ApiError("Invalid Authorization header format".to_string()))?;

    let mut url = format!(
        "https://api.jquants.com/v1/fins/statements?code={}",
        params.code
    );

    if let Some(from) = params.from {
        url.push_str(&format!("&from={}", from));
    }

    if let Some(to) = params.to {
        url.push_str(&format!("&to={}", to));
    }

    println!("URL: {}", url);
    // println!("token: {}", token);

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::ApiError(format!(
            "JQuants API error: {}",
            error_text
        )));
    }

    let statements_response = response
        .json::<StatementsResponse>()
        .await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    Ok(Json(statements_response))
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
