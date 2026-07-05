use crate::models::market_data::providers::jquants::FinSummaryData;
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
    // 有効な配当値を持つ summary の中で開示日が最大のものを選ぶ
    // 同じ開示日の場合は入力順先勝ち（元の sort_by + 線形走査と同挙動）
    let best = data
        .iter()
        .filter_map(|summary| {
            [
                summary
                    .next_year_forecast_dividend_per_share_annual
                    .as_deref(),
                summary.forecast_dividend_per_share_annual.as_deref(),
                summary.result_dividend_per_share_annual.as_deref(),
            ]
            .into_iter()
            .flatten()
            .filter(|s| !s.is_empty())
            .find_map(|s| s.parse::<f64>().ok())
            .map(|v| (&summary.disclosed_date, v))
        })
        .fold(None, |acc: Option<(&String, f64)>, (date, val)| match acc {
            None => Some((date, val)),
            Some((best_date, _)) if date > best_date => Some((date, val)),
            Some(prev) => Some(prev),
        });

    if let Some((_, val)) = best {
        let status = if val > 0.0 { "ok" } else { "zero" };
        return (Some(val), status.to_string());
    }

    // 配当情報が見つからない → ゼロ配当として扱う
    (Some(0.0), "zero".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::market_data::providers::jquants::FinSummaryData;

    fn make_summary(
        disc_date: &str,
        nx_div: Option<&str>,
        f_div: Option<&str>,
        div: Option<&str>,
    ) -> FinSummaryData {
        serde_json::from_value(serde_json::json!({
            "DiscDate": disc_date,
            "Code": "1234",
            "DocType": "test",
            "NxFDivAnn": nx_div,
            "FDivAnn": f_div,
            "DivAnn": div,
        }))
        .expect("FinSummaryData のパースに失敗")
    }

    #[test]
    fn test_extract_dividend() {
        for (data, expected_value, expected_status) in [
            (
                vec![make_summary("2024-01-01", Some("100.0"), None, None)],
                Some(100.0),
                "ok",
            ),
            (
                vec![make_summary("2024-01-01", Some("0.0"), None, None)],
                Some(0.0),
                "zero",
            ),
            (
                vec![make_summary(
                    "2024-01-01",
                    Some("200.0"),
                    Some("100.0"),
                    Some("50.0"),
                )],
                Some(200.0),
                "ok",
            ),
            (
                vec![make_summary("2024-01-01", None, None, Some("75.0"))],
                Some(75.0),
                "ok",
            ),
            (
                vec![make_summary("2024-01-01", Some(""), Some("100.0"), None)],
                Some(100.0),
                "ok",
            ),
            (
                vec![make_summary("2024-01-01", Some(""), Some(""), Some(""))],
                Some(0.0),
                "zero",
            ),
            (
                vec![make_summary("2024-01-01", Some("N/A"), Some("100.0"), None)],
                Some(100.0),
                "ok",
            ),
            (vec![], Some(0.0), "zero"),
        ] {
            let (value, status) = extract_dividend(&data);

            assert_eq!(value, expected_value);
            assert_eq!(status, expected_status);
        }

        let (value, status) = extract_dividend(&[
            make_summary("2023-01-01", None, None, Some("30.0")),
            make_summary("2024-01-01", None, None, Some("60.0")),
        ]);

        assert_eq!(value, Some(60.0));
        assert_eq!(status, "ok");
    }

    #[test]
    fn test_extract_dividend_same_date_uses_first_input_order() {
        // 同じ disclosed_date を持つ複数 summary がある場合、入力順で最初に現れた
        // valid dividend が採用される。この値が persistence.rs によって DB に書き込まれるため、
        // 後続の summary の値に変わらないことを保証する回帰テスト。
        let (value, status) = extract_dividend(&[
            make_summary("2024-01-01", None, None, Some("30.0")),
            make_summary("2024-01-01", None, None, Some("99.0")),
        ]);
        assert_eq!(value, Some(30.0));
        assert_eq!(status, "ok");
    }

    #[test]
    fn test_compute_is_stale() {
        let now = DateTime::parse_from_rfc3339("2024-01-10T00:00:00Z")
            .expect("固定時刻のパースに失敗")
            .with_timezone(&Utc);
        let past = Some(now - chrono::Duration::hours(1));
        let future = Some(now + chrono::Duration::days(7));

        for (status, stale_at, expected) in [
            ("pending", None, false),
            ("error", None, true),
            // 429 cooldown 中は error + future stale_at でも stale 扱いしない
            ("error", future, false),
            ("ok", past, true),
            ("ok", future, false),
            ("ok", None, true),
        ] {
            assert_eq!(compute_is_stale(status, stale_at, now), expected);
        }
    }
}
