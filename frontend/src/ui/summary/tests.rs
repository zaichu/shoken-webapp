use super::{SummaryStat, SummaryTone, tone_amount, with_pending_note};
use crate::ui::amount::AmountKind;

#[test]
fn gain_maps_to_delta_positive() {
    let (kind, negative, positive) = tone_amount(SummaryTone::Gain);
    assert_eq!(kind, AmountKind::Delta);
    assert!(!negative);
    assert!(positive);
}

#[test]
fn loss_maps_to_delta_negative() {
    let (kind, negative, positive) = tone_amount(SummaryTone::Loss);
    assert_eq!(kind, AmountKind::Delta);
    assert!(negative);
    assert!(!positive);
}

#[test]
fn neutral_maps_to_plain_without_sign() {
    let (kind, negative, positive) = tone_amount(SummaryTone::Neutral);
    assert_eq!(kind, AmountKind::Plain);
    assert!(!negative);
    assert!(!positive);
}

#[test]
fn stats_keep_item_order() {
    let stats = [
        SummaryStat {
            label: "評価額".to_string(),
            value: "¥1,000".to_string(),
            note: String::new(),
            tone: SummaryTone::Neutral,
        },
        SummaryStat {
            label: "評価損益".to_string(),
            value: "+¥100".to_string(),
            note: "+10.0%".to_string(),
            tone: SummaryTone::Gain,
        },
    ];
    let labels: Vec<&str> = stats.iter().map(|stat| stat.label.as_str()).collect();
    assert_eq!(labels, ["評価額", "評価損益"]);
}

#[test]
fn pending_note_appends_only_when_pending() {
    assert_eq!(
        with_pending_note("価格基準日 10/21", false),
        "価格基準日 10/21"
    );
    assert_eq!(
        with_pending_note("価格基準日 10/21", true),
        "価格基準日 10/21・一部更新待ち"
    );
    assert_eq!(with_pending_note("", true), "一部更新待ち");
    assert_eq!(with_pending_note("", false), "");
}
