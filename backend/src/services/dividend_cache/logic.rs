use crate::models::jquants::FinSummaryData;
use chrono::{DateTime, Utc};

/// キャッシュエントリの is_stale を判定する
///
/// pending は取得中のため stale_at が NULL でも is_stale = false とする。
/// それ以外は stale_at が NULL または過去なら is_stale = true。
pub fn compute_is_stale(status: &str, stale_at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    status != "pending" && stale_at.map(|t| t < now).unwrap_or(true)
}

/// 決算サマリーから1株配当を抽出する
/// 優先順位: 来期予想(NxFDivAnn) > 今期予想(FDivAnn) > 実績(DivAnn)
pub fn extract_dividend(data: &[FinSummaryData]) -> (Option<f64>, String) {
    // 開示日で降順ソート（最新データを優先）
    let mut sorted: Vec<&FinSummaryData> = data.iter().collect();
    sorted.sort_by(|a, b| b.disclosed_date.cmp(&a.disclosed_date));

    for summary in &sorted {
        for raw in [
            summary
                .next_year_forecast_dividend_per_share_annual
                .as_deref(),
            summary.forecast_dividend_per_share_annual.as_deref(),
            summary.result_dividend_per_share_annual.as_deref(),
        ] {
            let Some(val_str) = raw.filter(|s| !s.is_empty()) else {
                continue;
            };

            if let Ok(val) = val_str.parse::<f64>() {
                let status = if val > 0.0 { "ok" } else { "zero" };
                return (Some(val), status.to_string());
            }
        }
    }

    // 配当情報が見つからない → ゼロ配当として扱う
    (Some(0.0), "zero".to_string())
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use super::*;
    use crate::models::jquants::FinSummaryData;

    fn make_summary(disc_date: &str, nx_div: Option<&str>, f_div: Option<&str>, div: Option<&str>) -> FinSummaryData { serde_json::from_value(serde_json::json!({ "DiscDate": disc_date, "Code": "1234", "DocType": "test", "NxFDivAnn": nx_div, "FDivAnn": f_div, "DivAnn": div, })).expect("FinSummaryData のパースに失敗") }

    #[test]
    fn test_extract_dividend() {
        let dividend_cases = [
            (vec![make_summary("2024-01-01", Some("100.0"), None, None)], (Some(100.0), "ok")),
            (vec![make_summary("2024-01-01", Some("0.0"), None, None)], (Some(0.0), "zero")),
            (vec![make_summary("2024-01-01", Some("200.0"), Some("100.0"), Some("50.0"))], (Some(200.0), "ok")),
            (vec![make_summary("2024-01-01", None, None, Some("75.0"))], (Some(75.0), "ok")),
            (vec![make_summary("2024-01-01", Some(""), Some("100.0"), None)], (Some(100.0), "ok")),
            (vec![make_summary("2024-01-01", Some(""), Some(""), Some(""))], (Some(0.0), "zero")),
            (vec![make_summary("2024-01-01", Some("N/A"), Some("100.0"), None)], (Some(100.0), "ok")),
            (vec![], (Some(0.0), "zero")),
        ];
        for (data, expected) in dividend_cases {
            let (value, status) = extract_dividend(&data);
            assert_eq!((value, status.as_str()), expected);
        }

        assert_eq!(extract_dividend(&[make_summary("2023-01-01", None, None, Some("30.0")), make_summary("2024-01-01", None, None, Some("60.0"))]).0, Some(60.0));

        let now = Utc::now();
        let past = Some(now - chrono::Duration::hours(1));
        let future = Some(now + chrono::Duration::days(7));
        for (status, stale_at, expected) in [
            ("pending", None, false),
            ("error", None, true),
            ("ok", past, true),
            ("ok", future, false),
            ("ok", None, true),
        ] {
            assert_eq!(compute_is_stale(status, stale_at, now), expected);
        }
    }
}
