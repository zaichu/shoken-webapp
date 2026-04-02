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
mod tests {
    use super::*;
    use crate::models::jquants::FinSummaryData;

    fn make_summary(
        disc_date: &str,
        nx_div: Option<&str>,
        f_div: Option<&str>,
        div: Option<&str>,
    ) -> FinSummaryData {
        // FinSummaryData のデフォルト値を生成するためにデシリアライズを利用
        let json = serde_json::json!({
            "DiscDate": disc_date,
            "Code": "1234",
            "DocType": "test",
            "NxFDivAnn": nx_div,
            "FDivAnn": f_div,
            "DivAnn": div,
        });
        serde_json::from_value(json).expect("FinSummaryData のパースに失敗")
    }

    #[test]
    fn test_extract_dividend_ok() {
        let data = vec![make_summary("2024-01-01", Some("100.0"), None, None)];
        let (val, status) = extract_dividend(&data);
        assert_eq!(val, Some(100.0));
        assert_eq!(status, "ok");
    }

    #[test]
    fn test_extract_dividend_zero() {
        let data = vec![make_summary("2024-01-01", Some("0.0"), None, None)];
        let (val, status) = extract_dividend(&data);
        assert_eq!(val, Some(0.0));
        assert_eq!(status, "zero");
    }

    #[test]
    fn test_extract_dividend_priority_nx_over_f() {
        // NxFDivAnn が優先
        let data = vec![make_summary(
            "2024-01-01",
            Some("200.0"),
            Some("100.0"),
            Some("50.0"),
        )];
        let (val, status) = extract_dividend(&data);
        assert_eq!(val, Some(200.0));
        assert_eq!(status, "ok");
    }

    #[test]
    fn test_extract_dividend_falls_back_to_result() {
        // NxFDivAnn, FDivAnn が None → DivAnn を使用
        let data = vec![make_summary("2024-01-01", None, None, Some("75.0"))];
        let (val, status) = extract_dividend(&data);
        assert_eq!(val, Some(75.0));
        assert_eq!(status, "ok");
    }

    #[test]
    fn test_extract_dividend_empty_nx_falls_back_to_f() {
        let data = vec![make_summary("2024-01-01", Some(""), Some("100.0"), None)];
        let (val, status) = extract_dividend(&data);
        assert_eq!(val, Some(100.0));
        assert_eq!(status, "ok");
    }

    #[test]
    fn test_extract_dividend_all_empty_strings_is_zero() {
        let data = vec![make_summary("2024-01-01", Some(""), Some(""), Some(""))];
        let (val, status) = extract_dividend(&data);
        assert_eq!(val, Some(0.0));
        assert_eq!(status, "zero");
    }

    #[test]
    fn test_extract_dividend_invalid_nx_falls_back_to_f() {
        let data = vec![make_summary("2024-01-01", Some("N/A"), Some("100.0"), None)];
        let (val, status) = extract_dividend(&data);
        assert_eq!(val, Some(100.0));
        assert_eq!(status, "ok");
    }

    #[test]
    fn test_extract_dividend_empty_data() {
        // データなし → ゼロ配当
        let (val, status) = extract_dividend(&[]);
        assert_eq!(val, Some(0.0));
        assert_eq!(status, "zero");
    }

    #[test]
    fn test_compute_is_stale_pending_null_is_false() {
        // pending は stale_at=NULL でも is_stale=false（取得中のため）
        let now = Utc::now();
        assert!(!compute_is_stale("pending", None, now));
    }

    #[test]
    fn test_compute_is_stale_error_null_is_true() {
        // error は stale_at=NULL → 即再取得対象
        let now = Utc::now();
        assert!(compute_is_stale("error", None, now));
    }

    #[test]
    fn test_compute_is_stale_ok_past_is_true() {
        // ok で stale_at が過去 → stale
        let now = Utc::now();
        let past = now - chrono::Duration::hours(1);
        assert!(compute_is_stale("ok", Some(past), now));
    }

    #[test]
    fn test_compute_is_stale_ok_future_is_false() {
        // ok で stale_at が未来 → 有効
        let now = Utc::now();
        let future = now + chrono::Duration::days(7);
        assert!(!compute_is_stale("ok", Some(future), now));
    }

    #[test]
    fn test_compute_is_stale_ok_null_is_true() {
        // ok で stale_at=NULL → stale
        let now = Utc::now();
        assert!(compute_is_stale("ok", None, now));
    }

    #[test]
    fn test_extract_dividend_newest_first() {
        // 開示日が新しいほうを優先
        let data = vec![
            make_summary("2023-01-01", None, None, Some("30.0")),
            make_summary("2024-01-01", None, None, Some("60.0")),
        ];
        let (val, _) = extract_dividend(&data);
        assert_eq!(val, Some(60.0));
    }
}
