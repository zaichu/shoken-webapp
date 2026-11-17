use super::composition_card::{account_chart_index, product_chart_index};
use super::recent_dividends::{
    home_dividend_dialog_cells, home_dividend_row_texts, signed_gross_text, signed_tax_text,
};
use crate::api::dto::Dividend;
use crate::ui::amount::AmountKind;
use crate::ui::labels;
use leptos::prelude::Set;

#[test]
fn composition_amounts_follow_the_selected_unit_for_total_and_rows() {
    use crate::features::asset_balance;
    use crate::settings;
    use crate::settings::AmountUnit;

    let unit = settings::amount_unit_signal();
    for (selected, expected_total, expected_row) in [
        (AmountUnit::Yen, "¥660,000", "¥360,000"),
        (AmountUnit::Sen, "660千円", "360千円"),
        (AmountUnit::Man, "66万円", "36万円"),
    ] {
        unit.set(selected);
        assert_eq!(
            asset_balance::format_valuation_amount(Some(660_000.0)),
            expected_total
        );
        assert_eq!(
            asset_balance::format_valuation_amount(Some(360_000.0)),
            expected_row
        );
    }
    unit.set(AmountUnit::Yen);
}

fn test_dividend(code: &str, net: i64, before: i64, tax: i64) -> Dividend {
    serde_json::from_value(serde_json::json!({
        "id": format!("div-{code}-{net}"),
        "settlement_date": "2024-03-15",
        "product": "国内株式",
        "account": "特定",
        "security_code": code,
        "security_name": "架空工業",
        "unit_price": 30,
        "shares": 100,
        "dividends_before_tax": before,
        "taxes": tax,
        "net_amount_received": net,
        "created_at": "",
        "updated_at": "2026-10-18T00:00:00Z"
    }))
    .unwrap()
}

#[test]
fn product_chart_index_is_fixed_per_label() {
    assert_eq!(product_chart_index("国内株式"), Some(0));
    assert_eq!(product_chart_index("投資信託"), Some(1));
    assert_eq!(product_chart_index("外国株式"), None);
}

#[test]
fn account_chart_index_normalizes_csv_names() {
    // CSV 由来の長い名称はバッジの表示名と同じ系列に乗る
    for (input, expected) in [
        ("特定", 0),
        ("特定口座", 0),
        ("NISA成長", 1),
        ("NISA成長投資枠", 1),
        ("旧NISA", 2),
        (labels::ACCOUNT_SPECIFIC_GENERAL, 3),
        ("NISAつみたて", 4),
        ("NISAつみたて投資枠", 4),
        ("一般", 5),
        ("一般口座", 5),
    ] {
        assert_eq!(account_chart_index(input), Some(expected), "{input}");
    }
    assert_eq!(account_chart_index("NISA"), None);
}

#[test]
fn dividend_dialog_shows_signed_trio() {
    let (left, right) = home_dividend_dialog_cells(
        "2024-03-15",
        "100".to_string(),
        "¥30".to_string(),
        ("+¥2,000".to_string(), false, true),
        ("-¥406".to_string(), true, false),
        ("+¥1,594".to_string(), false, true),
    );
    assert_eq!(left.len(), 3);
    let before_tax = right
        .iter()
        .find(|cell| cell.label == "税引前")
        .expect("ダイアログに税引前がある");
    assert_eq!(before_tax.value, "+¥2,000");
    assert_eq!(before_tax.kind, AmountKind::Delta);
    assert!(!before_tax.negative);
    assert!(before_tax.positive);
    let tax = right
        .iter()
        .find(|cell| cell.label == "税額")
        .expect("ダイアログに税額がある");
    assert_eq!(tax.value, "-¥406");
    assert_eq!(tax.kind, AmountKind::Delta);
    assert!(tax.negative);
    assert!(!tax.positive);
    let net = right
        .iter()
        .find(|cell| cell.label == "税引後")
        .expect("ダイアログに税引後がある");
    assert_eq!(net.value, "+¥1,594");
    assert_eq!(net.kind, AmountKind::Delta);
}

#[test]
fn dividend_dialog_tax_zero_uses_plain() {
    let (_, right) = home_dividend_dialog_cells(
        "2024-03-15",
        "100".to_string(),
        "¥30".to_string(),
        ("+¥2,000".to_string(), false, true),
        ("¥0".to_string(), false, false),
        ("+¥2,000".to_string(), false, true),
    );
    let tax = right
        .iter()
        .find(|cell| cell.label == "税額")
        .expect("ダイアログに税額がある");
    assert_eq!(tax.value, "¥0");
    assert_eq!(tax.kind, AmountKind::Plain);
    assert!(!tax.negative);
    assert!(!tax.positive);
}

#[test]
fn dividend_dialog_tax_refund_uses_plus() {
    let (_, right) = home_dividend_dialog_cells(
        "2024-03-15",
        "100".to_string(),
        "¥30".to_string(),
        ("+¥2,000".to_string(), false, true),
        ("+¥100".to_string(), false, true),
        ("+¥2,100".to_string(), false, true),
    );
    let tax = right
        .iter()
        .find(|cell| cell.label == "税額")
        .expect("ダイアログに税額がある");
    assert_eq!(tax.value, "+¥100");
    assert_eq!(tax.kind, AmountKind::Delta);
    assert!(!tax.negative);
    assert!(tax.positive);
}

#[test]
fn dividend_dialog_signed_helpers_cover_edges() {
    assert_eq!(signed_gross_text(2000.0, "¥2,000"), "+¥2,000");
    assert_eq!(signed_gross_text(-1000.0, "-¥1,000"), "-¥1,000");
    assert_eq!(signed_gross_text(0.0, "¥0"), "¥0");
    assert_eq!(signed_tax_text(406.0, "¥406"), "-¥406");
    assert_eq!(signed_tax_text(-100.0, "-¥100"), "+¥100");
    assert_eq!(signed_tax_text(0.0, "¥0"), "¥0");
}

#[test]
fn dividend_row_texts_sign_positive_net() {
    let texts = home_dividend_row_texts(&test_dividend("9904", 1594, 2000, 406));
    assert_eq!(texts.code, "9904");
    assert_eq!(texts.net_signed, "+¥1,594");
    assert!(!texts.net_negative);
    assert!(texts.net_positive);
    assert_eq!(texts.gross_signed, "+¥2,000");
    assert_eq!(texts.tax_signed, "-¥406");
    assert!(texts.tax_negative);
    assert!(!texts.tax_positive);
}

#[test]
fn dividend_row_texts_blank_code_uses_missing() {
    let texts = home_dividend_row_texts(&test_dividend("", 0, 0, 0));
    assert_eq!(texts.code, "—");
    assert_eq!(texts.net_signed, "¥0");
    assert!(!texts.net_negative);
    assert!(!texts.net_positive);
}
