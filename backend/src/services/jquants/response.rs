use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 配当キャッシュが使用する決算サマリーレスポンス（J-Quants API V2）
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FinSummaryResponse {
    pub data: Vec<FinSummaryData>,
    /// ページネーションキーは従来どおり保持する（追加取得は行わない）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination_key: Option<String>,
}

/// 配当抽出に必要な項目のみ保持し、その他の上流フィールドは無視する。
/// Code と DocType は従来の必須項目の検証を維持するため残す。
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FinSummaryData {
    /// 開示日
    #[serde(rename = "DiscDate")]
    pub disclosed_date: String,

    /// 銘柄コード
    #[serde(rename = "Code")]
    pub local_code: String,

    /// 書類種別
    #[serde(rename = "DocType")]
    pub type_of_document: String,

    // 空文字・不正文字列のフォールバックは extract_dividend に委ねる。
    /// 年間配当実績
    #[serde(rename = "DivAnn", default)]
    pub result_dividend_per_share_annual: Option<String>,

    /// 年間配当予想
    #[serde(rename = "FDivAnn", default)]
    pub forecast_dividend_per_share_annual: Option<String>,

    /// 年間配当来期予想
    #[serde(rename = "NxFDivAnn", default)]
    pub next_year_forecast_dividend_per_share_annual: Option<String>,
}

#[cfg(test)]
mod tests {
    use {super::*, serde_json::json};
    #[test]
    fn test_fin_summary_required_fields_and_dividend_types() {
        let row = json!({"DiscDate": "2024-05-10", "Code": "7203", "DocType": "FY"});
        for key in ["DiscDate", "Code", "DocType"] {
            let mut missing = row.clone();
            missing.as_object_mut().expect("オブジェクト").remove(key);
            assert!(serde_json::from_value::<FinSummaryData>(missing).is_err());
        }
        for key in ["NxFDivAnn", "FDivAnn", "DivAnn"] {
            for invalid in [json!(12.5), json!(true), json!([]), json!({})] {
                let mut invalid_row = row.clone();
                invalid_row[key] = invalid;
                assert!(serde_json::from_value::<FinSummaryData>(invalid_row).is_err());
            }
        }
        for key in [None, Some("next-page")] {
            let response: FinSummaryResponse =
                serde_json::from_value(json!({"data": [row.clone()], "pagination_key": key}))
                    .expect("ページネーション");
            assert_eq!(response.pagination_key.as_deref(), key);
        }
    }

    #[test]
    fn test_fin_summary_data_deserialize_v2_format() {
        let fin_summary_data: FinSummaryData = serde_json::from_value(json!({"DiscDate":"2023-11-14","DiscTime":"15:00:00","Code":"72030","DiscNo":"20231114502171","DocType":"決算短信","CurPerType":"2Q","CurPerSt":"2023-04-01","CurPerEn":"2023-09-30","CurFYSt":"2023-04-01","CurFYEn":"2024-03-31","NxtFYSt":"2024-04-01","NxtFYEn":"2025-03-31","Sales":"18733067000000","OP":"1686297000000","NxFDivAnn":"50.00","FDivAnn":"45.00","DivAnn":"40.00"})).expect("有効なテスト用 JSON");
        assert_eq!(
            (
                fin_summary_data.disclosed_date.as_str(),
                fin_summary_data.local_code.as_str(),
                fin_summary_data
                    .next_year_forecast_dividend_per_share_annual
                    .as_deref(),
                fin_summary_data
                    .forecast_dividend_per_share_annual
                    .as_deref(),
                fin_summary_data.result_dividend_per_share_annual.as_deref()
            ),
            (
                "2023-11-14",
                "72030",
                Some("50.00"),
                Some("45.00"),
                Some("40.00")
            )
        );
        let response: FinSummaryResponse = serde_json::from_value(json!({"data":[{"DiscDate":"2023-11-14","Code":"72030","DocType":"決算短信","CurPerType":"2Q","CurPerSt":"2023-04-01","CurPerEn":"2023-09-30","CurFYSt":"2023-04-01","CurFYEn":"2024-03-31"}]})).expect("有効なテスト用 JSON");
        assert_eq!(
            (
                response.data.len(),
                response.data[0].disclosed_date.as_str(),
                response.data[0].local_code.as_str()
            ),
            (1, "2023-11-14", "72030")
        );
        assert!(
            serde_json::from_value::<FinSummaryResponse>(json!({"data":[]}))
                .expect("有効なテスト用 JSON")
                .data
                .is_empty()
        );
    }
}
