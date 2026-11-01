use super::{SummaryBreakdownEntry, daily_rate};
use crate::ui::amount::AmountKind;

#[test]
fn daily_rate_uses_previous_market_as_denominator() {
    let rate = daily_rate(Some(110.0), Some(10.0)).expect("率が出る");
    assert!((rate - 10.0).abs() < 1e-9);
}

#[test]
fn daily_rate_returns_none_without_base() {
    assert_eq!(daily_rate(None, Some(10.0)), None);
    assert_eq!(daily_rate(Some(100.0), None), None);
    assert_eq!(daily_rate(Some(100.0), Some(100.0)), None);
    assert_eq!(daily_rate(Some(90.0), Some(100.0)), None);
}

#[test]
fn daily_rate_keeps_sign_of_daily_change() {
    let rate = daily_rate(Some(90.0), Some(-10.0)).expect("率が出る");
    assert!(rate < 0.0);
}

#[test]
fn breakdown_keeps_item_order() {
    let entries = [
        SummaryBreakdownEntry {
            label: "前日比".to_string(),
            value: "+¥100".to_string(),
            kind: AmountKind::Delta,
            negative: false,
            positive: true,
            rate: "+1.0%".to_string(),
        },
        SummaryBreakdownEntry {
            label: "年間予想配当(税引前)".to_string(),
            value: "¥500".to_string(),
            kind: AmountKind::Plain,
            negative: false,
            positive: false,
            rate: String::new(),
        },
        SummaryBreakdownEntry {
            label: "取得総額".to_string(),
            value: "¥1,000".to_string(),
            kind: AmountKind::Plain,
            negative: false,
            positive: false,
            rate: String::new(),
        },
    ];
    let labels: Vec<&str> = entries.iter().map(|entry| entry.label.as_str()).collect();
    assert_eq!(labels, ["前日比", "年間予想配当(税引前)", "取得総額"]);
}
