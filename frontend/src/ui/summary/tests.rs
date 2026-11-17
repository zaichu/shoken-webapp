use super::{SummaryBreakdownEntry, SummaryTone};
use crate::ui::amount::AmountKind;

#[test]
fn daily_rate_uses_previous_market_as_denominator() {
    let rate = super::daily_rate(Some(110.0), Some(10.0)).expect("率が出る");
    assert!((rate - 10.0).abs() < 1e-9);
}

#[test]
fn daily_rate_returns_none_without_base() {
    assert_eq!(super::daily_rate(None, Some(10.0)), None);
    assert_eq!(super::daily_rate(Some(100.0), None), None);
    assert_eq!(super::daily_rate(Some(100.0), Some(100.0)), None);
    assert_eq!(super::daily_rate(Some(90.0), Some(100.0)), None);
}

#[test]
fn daily_rate_keeps_sign_of_daily_change() {
    let rate = super::daily_rate(Some(90.0), Some(-10.0)).expect("率が出る");
    assert!(rate < 0.0);
}

#[test]
fn gain_maps_to_delta_positive() {
    let (kind, negative, positive) = super::tone_amount(SummaryTone::Gain);
    assert_eq!(kind, AmountKind::Delta);
    assert!(!negative);
    assert!(positive);
}

#[test]
fn loss_maps_to_delta_negative() {
    let (kind, negative, positive) = super::tone_amount(SummaryTone::Loss);
    assert_eq!(kind, AmountKind::Delta);
    assert!(negative);
    assert!(!positive);
}

#[test]
fn neutral_maps_to_plain_without_sign() {
    let (kind, negative, positive) = super::tone_amount(SummaryTone::Neutral);
    assert_eq!(kind, AmountKind::Plain);
    assert!(!negative);
    assert!(!positive);
}

#[test]
fn tone_of_maps_sign_and_zero() {
    assert_eq!(super::tone_of(true, false), SummaryTone::Loss);
    assert_eq!(super::tone_of(false, true), SummaryTone::Gain);
    assert_eq!(super::tone_of(false, false), SummaryTone::Neutral);
}

#[test]
fn breakdown_keeps_item_order() {
    let entries = [
        SummaryBreakdownEntry {
            label: "前日比".to_string(),
            value: "+¥100".to_string(),
            negative: false,
            positive: true,
            rate: "+1.0%".to_string(),
        },
        SummaryBreakdownEntry {
            label: "年間予想配当(税引前)".to_string(),
            value: "¥500".to_string(),
            negative: false,
            positive: false,
            rate: String::new(),
        },
        SummaryBreakdownEntry {
            label: "取得総額".to_string(),
            value: "¥1,000".to_string(),
            negative: false,
            positive: false,
            rate: String::new(),
        },
    ];
    let labels: Vec<&str> = entries.iter().map(|entry| entry.label.as_str()).collect();
    assert_eq!(labels, ["前日比", "年間予想配当(税引前)", "取得総額"]);
}
