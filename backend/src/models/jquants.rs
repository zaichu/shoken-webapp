use serde::{Deserialize, Serialize};

/// 決算サマリー取得パラメータ（J-Quants API V2）
#[derive(Debug, Deserialize)]
pub struct FinSummaryQuery {
    pub code: String,
    pub from: Option<String>,
    pub to: Option<String>,
}

/// 決算サマリーレスポンス（J-Quants API V2）
/// V2では fins/summary を使用し、ルートフィールドは "data"
#[derive(Debug, Serialize, Deserialize)]
pub struct FinSummaryResponse {
    /// 決算サマリーデータの配列（V2では "data" フィールド）
    pub data: Vec<FinSummaryData>,
    /// ページネーションキー（データが大量の場合に設定される）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination_key: Option<String>,
}

/// 決算サマリーデータ（J-Quants API V2）
/// V2では省略形フィールド名を使用
#[derive(Debug, Serialize, Deserialize)]
pub struct FinSummaryData {
    /// 開示日（DisclosedDate → DiscDate）
    #[serde(rename = "DiscDate")]
    pub disclosed_date: String,

    /// 開示時刻
    #[serde(rename = "DiscTime", default)]
    pub disclosed_time: Option<String>,

    /// 銘柄コード（LocalCode → Code）
    #[serde(rename = "Code")]
    pub local_code: String,

    /// 開示番号
    #[serde(rename = "DiscNum", default)]
    pub disclosure_number: Option<String>,

    /// 書類種別
    #[serde(rename = "DocType")]
    pub type_of_document: String,

    /// 当期種別
    #[serde(rename = "CurPeriod")]
    pub type_of_current_period: String,

    /// 当期開始日
    #[serde(rename = "CurStartDate")]
    pub current_period_start_date: String,

    /// 当期終了日
    #[serde(rename = "CurEndDate")]
    pub current_period_end_date: String,

    /// 当会計年度開始日
    #[serde(rename = "CurFYStartDate")]
    pub current_fiscal_year_start_date: String,

    /// 当会計年度終了日
    #[serde(rename = "CurFYEndDate")]
    pub current_fiscal_year_end_date: String,

    /// 翌会計年度開始日
    #[serde(rename = "NxFYStartDate", default)]
    pub next_fiscal_year_start_date: Option<String>,

    /// 翌会計年度終了日
    #[serde(rename = "NxFYEndDate", default)]
    pub next_fiscal_year_end_date: Option<String>,

    /// 売上高
    #[serde(rename = "Sales", default)]
    pub net_sales: Option<String>,

    /// 営業利益
    #[serde(rename = "OpProfit", default)]
    pub operating_profit: Option<String>,

    /// 経常利益
    #[serde(rename = "OrdProfit", default)]
    pub ordinary_profit: Option<String>,

    /// 当期純利益
    #[serde(rename = "Profit", default)]
    pub profit: Option<String>,

    /// 1株当たり当期純利益
    #[serde(rename = "EPS", default)]
    pub earnings_per_share: Option<String>,

    /// 希薄化後1株当たり当期純利益
    #[serde(rename = "DilutedEPS", default)]
    pub diluted_earnings_per_share: Option<String>,

    /// 総資産
    #[serde(rename = "TotalAssets", default)]
    pub total_assets: Option<String>,

    /// 純資産
    #[serde(rename = "Equity", default)]
    pub equity: Option<String>,

    /// 自己資本比率
    #[serde(rename = "EquityRatio", default)]
    pub equity_to_asset_ratio: Option<String>,

    /// 1株当たり純資産
    #[serde(rename = "BPS", default)]
    pub book_value_per_share: Option<String>,

    /// 営業活動によるキャッシュフロー
    #[serde(rename = "CashFlowOp", default)]
    pub cash_flows_from_operating_activities: Option<String>,

    /// 投資活動によるキャッシュフロー
    #[serde(rename = "CashFlowInv", default)]
    pub cash_flows_from_investing_activities: Option<String>,

    /// 財務活動によるキャッシュフロー
    #[serde(rename = "CashFlowFin", default)]
    pub cash_flows_from_financing_activities: Option<String>,

    /// 現金及び現金同等物の期末残高
    #[serde(rename = "CashEq", default)]
    pub cash_and_equivalents: Option<String>,

    /// 1株当たり配当金（第1四半期末）実績
    #[serde(rename = "Div1Q", default)]
    pub result_dividend_per_share_1st_quarter: Option<String>,

    /// 1株当たり配当金（第2四半期末）実績
    #[serde(rename = "Div2Q", default)]
    pub result_dividend_per_share_2nd_quarter: Option<String>,

    /// 1株当たり配当金（第3四半期末）実績
    #[serde(rename = "Div3Q", default)]
    pub result_dividend_per_share_3rd_quarter: Option<String>,

    /// 1株当たり配当金（期末）実績
    #[serde(rename = "DivYrEnd", default)]
    pub result_dividend_per_share_fiscal_year_end: Option<String>,

    /// 1株当たり年間配当金実績
    #[serde(rename = "DivAnn", default)]
    pub result_dividend_per_share_annual: Option<String>,

    /// 1口当たり分配金（REIT）
    #[serde(rename = "DistREIT", default)]
    pub distributions_per_unit_reit: Option<String>,

    /// 年間配当支払総額実績
    #[serde(rename = "TotalDivPaidAnn", default)]
    pub result_total_dividend_paid_annual: Option<String>,

    /// 配当性向実績
    #[serde(rename = "PayoutRatioAnn", default)]
    pub result_payout_ratio_annual: Option<String>,

    /// 1株当たり配当金（第1四半期末）予想
    #[serde(rename = "FDiv1Q", default)]
    pub forecast_dividend_per_share_1st_quarter: Option<String>,

    /// 1株当たり配当金（第2四半期末）予想
    #[serde(rename = "FDiv2Q", default)]
    pub forecast_dividend_per_share_2nd_quarter: Option<String>,

    /// 1株当たり配当金（第3四半期末）予想
    #[serde(rename = "FDiv3Q", default)]
    pub forecast_dividend_per_share_3rd_quarter: Option<String>,

    /// 1株当たり配当金（期末）予想
    #[serde(rename = "FDivYrEnd", default)]
    pub forecast_dividend_per_share_fiscal_year_end: Option<String>,

    /// 1株当たり年間配当金予想（今期）
    #[serde(rename = "FDivAnn", default)]
    pub forecast_dividend_per_share_annual: Option<String>,

    /// 1口当たり分配金（REIT）予想
    #[serde(rename = "FDistREIT", default)]
    pub forecast_distributions_per_unit_reit: Option<String>,

    /// 年間配当支払総額予想
    #[serde(rename = "FTotalDivPaidAnn", default)]
    pub forecast_total_dividend_paid_annual: Option<String>,

    /// 配当性向予想
    #[serde(rename = "FPayoutRatioAnn", default)]
    pub forecast_payout_ratio_annual: Option<String>,

    /// 1株当たり配当金（第1四半期末）来期予想
    #[serde(rename = "NxFDiv1Q", default)]
    pub next_year_forecast_dividend_per_share_1st_quarter: Option<String>,

    /// 1株当たり配当金（第2四半期末）来期予想
    #[serde(rename = "NxFDiv2Q", default)]
    pub next_year_forecast_dividend_per_share_2nd_quarter: Option<String>,

    /// 1株当たり配当金（第3四半期末）来期予想
    #[serde(rename = "NxFDiv3Q", default)]
    pub next_year_forecast_dividend_per_share_3rd_quarter: Option<String>,

    /// 1株当たり配当金（期末）来期予想
    #[serde(rename = "NxFDivYrEnd", default)]
    pub next_year_forecast_dividend_per_share_fiscal_year_end: Option<String>,

    /// 1株当たり年間配当金来期予想
    #[serde(rename = "NxFDivAnn", default)]
    pub next_year_forecast_dividend_per_share_annual: Option<String>,

    /// 1口当たり分配金（REIT）来期予想
    #[serde(rename = "NxFDistREIT", default)]
    pub next_year_forecast_distributions_per_unit_reit: Option<String>,

    /// 配当性向来期予想
    #[serde(rename = "NxFPayoutRatioAnn", default)]
    pub next_year_forecast_payout_ratio_annual: Option<String>,

    /// 売上高予想（第2四半期）
    #[serde(rename = "FSales2Q", default)]
    pub forecast_net_sales_2nd_quarter: Option<String>,

    /// 営業利益予想（第2四半期）
    #[serde(rename = "FOpProfit2Q", default)]
    pub forecast_operating_profit_2nd_quarter: Option<String>,

    /// 経常利益予想（第2四半期）
    #[serde(rename = "FOrdProfit2Q", default)]
    pub forecast_ordinary_profit_2nd_quarter: Option<String>,

    /// 当期純利益予想（第2四半期）
    #[serde(rename = "FProfit2Q", default)]
    pub forecast_profit_2nd_quarter: Option<String>,

    /// 1株当たり当期純利益予想（第2四半期）
    #[serde(rename = "FEPS2Q", default)]
    pub forecast_earnings_per_share_2nd_quarter: Option<String>,

    /// 売上高来期予想（第2四半期）
    #[serde(rename = "NxFSales2Q", default)]
    pub next_year_forecast_net_sales_2nd_quarter: Option<String>,

    /// 営業利益来期予想（第2四半期）
    #[serde(rename = "NxFOpProfit2Q", default)]
    pub next_year_forecast_operating_profit_2nd_quarter: Option<String>,

    /// 経常利益来期予想（第2四半期）
    #[serde(rename = "NxFOrdProfit2Q", default)]
    pub next_year_forecast_ordinary_profit_2nd_quarter: Option<String>,

    /// 当期純利益来期予想（第2四半期）
    #[serde(rename = "NxFProfit2Q", default)]
    pub next_year_forecast_profit_2nd_quarter: Option<String>,

    /// 1株当たり当期純利益来期予想（第2四半期）
    #[serde(rename = "NxFEPS2Q", default)]
    pub next_year_forecast_earnings_per_share_2nd_quarter: Option<String>,

    /// 売上高予想（通期）
    #[serde(rename = "FSales", default)]
    pub forecast_net_sales: Option<String>,

    /// 営業利益予想（通期）
    #[serde(rename = "FOpProfit", default)]
    pub forecast_operating_profit: Option<String>,

    /// 経常利益予想（通期）
    #[serde(rename = "FOrdProfit", default)]
    pub forecast_ordinary_profit: Option<String>,

    /// 当期純利益予想（通期）
    #[serde(rename = "FProfit", default)]
    pub forecast_profit: Option<String>,

    /// 1株当たり当期純利益予想（通期）
    #[serde(rename = "FEPS", default)]
    pub forecast_earnings_per_share: Option<String>,

    /// 売上高来期予想（通期）
    #[serde(rename = "NxFSales", default)]
    pub next_year_forecast_net_sales: Option<String>,

    /// 営業利益来期予想（通期）
    #[serde(rename = "NxFOpProfit", default)]
    pub next_year_forecast_operating_profit: Option<String>,

    /// 経常利益来期予想（通期）
    #[serde(rename = "NxFOrdProfit", default)]
    pub next_year_forecast_ordinary_profit: Option<String>,

    /// 当期純利益来期予想（通期）
    #[serde(rename = "NxFProfit", default)]
    pub next_year_forecast_profit: Option<String>,

    /// 1株当たり当期純利益来期予想（通期）
    #[serde(rename = "NxFEPS", default)]
    pub next_year_forecast_earnings_per_share: Option<String>,

    /// 重要な子会社の異動
    #[serde(rename = "MatChgSub", default)]
    pub material_changes_in_subsidiaries: Option<String>,

    /// 連結範囲の重要な変更
    #[serde(rename = "SigChgScope", default)]
    pub significant_changes_in_the_scope_of_consolidation: Option<String>,

    /// 会計基準の改正に伴う変更
    #[serde(rename = "ChgAccStd", default)]
    pub changes_based_on_revisions_of_accounting_standard: Option<String>,

    /// 会計基準の改正以外の変更
    #[serde(rename = "ChgOther", default)]
    pub changes_other_than_ones_based_on_revisions_of_accounting_standard: Option<String>,

    /// 会計上の見積りの変更
    #[serde(rename = "ChgEstimate", default)]
    pub changes_in_accounting_estimates: Option<String>,

    /// 修正再表示
    #[serde(rename = "Restatement", default)]
    pub retrospective_restatement: Option<String>,

    /// 発行済株式数（期末・自己株式を含む）
    #[serde(rename = "SharesOutstanding", default)]
    pub number_of_issued_and_outstanding_shares_at_the_end_of_fiscal_year_including_treasury_stock:
        Option<String>,

    /// 自己株式数（期末）
    #[serde(rename = "TreasuryStock", default)]
    pub number_of_treasury_stock_at_the_end_of_fiscal_year: Option<String>,

    /// 期中平均株式数
    #[serde(rename = "AvgShares", default)]
    pub average_number_of_shares: Option<String>,

    /// 売上高（個別）
    #[serde(rename = "NonConSales", default)]
    pub non_consolidated_net_sales: Option<String>,

    /// 営業利益（個別）
    #[serde(rename = "NonConOpProfit", default)]
    pub non_consolidated_operating_profit: Option<String>,

    /// 経常利益（個別）
    #[serde(rename = "NonConOrdProfit", default)]
    pub non_consolidated_ordinary_profit: Option<String>,

    /// 当期純利益（個別）
    #[serde(rename = "NonConProfit", default)]
    pub non_consolidated_profit: Option<String>,

    /// 1株当たり当期純利益（個別）
    #[serde(rename = "NonConEPS", default)]
    pub non_consolidated_earnings_per_share: Option<String>,

    /// 総資産（個別）
    #[serde(rename = "NonConTotalAssets", default)]
    pub non_consolidated_total_assets: Option<String>,

    /// 純資産（個別）
    #[serde(rename = "NonConEquity", default)]
    pub non_consolidated_equity: Option<String>,

    /// 自己資本比率（個別）
    #[serde(rename = "NonConEquityRatio", default)]
    pub non_consolidated_equity_to_asset_ratio: Option<String>,

    /// 1株当たり純資産（個別）
    #[serde(rename = "NonConBPS", default)]
    pub non_consolidated_book_value_per_share: Option<String>,

    /// 売上高予想（第2四半期・個別）
    #[serde(rename = "FNonConSales2Q", default)]
    pub forecast_non_consolidated_net_sales_2nd_quarter: Option<String>,

    /// 営業利益予想（第2四半期・個別）
    #[serde(rename = "FNonConOpProfit2Q", default)]
    pub forecast_non_consolidated_operating_profit_2nd_quarter: Option<String>,

    /// 経常利益予想（第2四半期・個別）
    #[serde(rename = "FNonConOrdProfit2Q", default)]
    pub forecast_non_consolidated_ordinary_profit_2nd_quarter: Option<String>,

    /// 当期純利益予想（第2四半期・個別）
    #[serde(rename = "FNonConProfit2Q", default)]
    pub forecast_non_consolidated_profit_2nd_quarter: Option<String>,

    /// 1株当たり当期純利益予想（第2四半期・個別）
    #[serde(rename = "FNonConEPS2Q", default)]
    pub forecast_non_consolidated_earnings_per_share_2nd_quarter: Option<String>,

    /// 売上高来期予想（第2四半期・個別）
    #[serde(rename = "NxFNonConSales2Q", default)]
    pub next_year_forecast_non_consolidated_net_sales_2nd_quarter: Option<String>,

    /// 営業利益来期予想（第2四半期・個別）
    #[serde(rename = "NxFNonConOpProfit2Q", default)]
    pub next_year_forecast_non_consolidated_operating_profit_2nd_quarter: Option<String>,

    /// 経常利益来期予想（第2四半期・個別）
    #[serde(rename = "NxFNonConOrdProfit2Q", default)]
    pub next_year_forecast_non_consolidated_ordinary_profit_2nd_quarter: Option<String>,

    /// 当期純利益来期予想（第2四半期・個別）
    #[serde(rename = "NxFNonConProfit2Q", default)]
    pub next_year_forecast_non_consolidated_profit_2nd_quarter: Option<String>,

    /// 1株当たり当期純利益来期予想（第2四半期・個別）
    #[serde(rename = "NxFNonConEPS2Q", default)]
    pub next_year_forecast_non_consolidated_earnings_per_share_2nd_quarter: Option<String>,

    /// 売上高予想（通期・個別）
    #[serde(rename = "FNonConSales", default)]
    pub forecast_non_consolidated_net_sales: Option<String>,

    /// 営業利益予想（通期・個別）
    #[serde(rename = "FNonConOpProfit", default)]
    pub forecast_non_consolidated_operating_profit: Option<String>,

    /// 経常利益予想（通期・個別）
    #[serde(rename = "FNonConOrdProfit", default)]
    pub forecast_non_consolidated_ordinary_profit: Option<String>,

    /// 当期純利益予想（通期・個別）
    #[serde(rename = "FNonConProfit", default)]
    pub forecast_non_consolidated_profit: Option<String>,

    /// 1株当たり当期純利益予想（通期・個別）
    #[serde(rename = "FNonConEPS", default)]
    pub forecast_non_consolidated_earnings_per_share: Option<String>,

    /// 売上高来期予想（通期・個別）
    #[serde(rename = "NxFNonConSales", default)]
    pub next_year_forecast_non_consolidated_net_sales: Option<String>,

    /// 営業利益来期予想（通期・個別）
    #[serde(rename = "NxFNonConOpProfit", default)]
    pub next_year_forecast_non_consolidated_operating_profit: Option<String>,

    /// 経常利益来期予想（通期・個別）
    #[serde(rename = "NxFNonConOrdProfit", default)]
    pub next_year_forecast_non_consolidated_ordinary_profit: Option<String>,

    /// 当期純利益来期予想（通期・個別）
    #[serde(rename = "NxFNonConProfit", default)]
    pub next_year_forecast_non_consolidated_profit: Option<String>,

    /// 1株当たり当期純利益来期予想（通期・個別）
    #[serde(rename = "NxFNonConEPS", default)]
    pub next_year_forecast_non_consolidated_earnings_per_share: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_fin_summary_query_deserialize() {
        let query = FinSummaryQuery {
            code: "7203".to_string(),
            from: Some("2023-01-01".to_string()),
            to: Some("2023-12-31".to_string()),
        };

        assert_eq!(query.code, "7203");
        assert_eq!(query.from.unwrap(), "2023-01-01");
        assert_eq!(query.to.unwrap(), "2023-12-31");
    }

    #[test]
    fn test_fin_summary_query_optional_params() {
        let query = FinSummaryQuery {
            code: "7203".to_string(),
            from: None,
            to: None,
        };

        assert_eq!(query.code, "7203");
        assert!(query.from.is_none());
        assert!(query.to.is_none());
    }

    #[test]
    fn test_fin_summary_data_deserialize_v2_format() {
        // V2 API の省略形フィールド名でテスト
        let json_data = json!({
            "DiscDate": "2023-11-14",
            "DiscTime": "15:00:00",
            "Code": "72030",
            "DiscNum": "20231114502171",
            "DocType": "決算短信",
            "CurPeriod": "2Q",
            "CurStartDate": "2023-04-01",
            "CurEndDate": "2023-09-30",
            "CurFYStartDate": "2023-04-01",
            "CurFYEndDate": "2024-03-31",
            "NxFYStartDate": "2024-04-01",
            "NxFYEndDate": "2025-03-31",
            "Sales": "18733067000000",
            "OpProfit": "1686297000000",
            "NxFDivAnn": "50.00",
            "FDivAnn": "45.00",
            "DivAnn": "40.00"
        });

        let fin_summary_data: FinSummaryData = serde_json::from_value(json_data).unwrap();

        assert_eq!(fin_summary_data.disclosed_date, "2023-11-14");
        assert_eq!(fin_summary_data.local_code, "72030");
        assert_eq!(
            fin_summary_data.net_sales,
            Some("18733067000000".to_string())
        );
        assert_eq!(
            fin_summary_data.next_year_forecast_dividend_per_share_annual,
            Some("50.00".to_string())
        );
        assert_eq!(
            fin_summary_data.forecast_dividend_per_share_annual,
            Some("45.00".to_string())
        );
        assert_eq!(
            fin_summary_data.result_dividend_per_share_annual,
            Some("40.00".to_string())
        );
    }

    #[test]
    fn test_fin_summary_response_deserialize_v2_format() {
        // V2 API では "data" フィールド名を使用
        let json_data = json!({
            "data": [
                {
                    "DiscDate": "2023-11-14",
                    "Code": "72030",
                    "DocType": "決算短信",
                    "CurPeriod": "2Q",
                    "CurStartDate": "2023-04-01",
                    "CurEndDate": "2023-09-30",
                    "CurFYStartDate": "2023-04-01",
                    "CurFYEndDate": "2024-03-31"
                }
            ]
        });

        let response: FinSummaryResponse = serde_json::from_value(json_data).unwrap();

        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].disclosed_date, "2023-11-14");
        assert_eq!(response.data[0].local_code, "72030");
    }

    #[test]
    fn test_fin_summary_response_empty() {
        let json_data = json!({ "data": [] });
        let response: FinSummaryResponse = serde_json::from_value(json_data).unwrap();

        assert_eq!(response.data.len(), 0);
    }
}
