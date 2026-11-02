use super::{
    CHART_COLOR_OTHER, CHART_COLORS, account_chart_index, chart_color, home_dividend_dialog_cells,
    product_chart_index,
};

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
        ("NISAつみたて", 3),
        ("NISAつみたて投資枠", 3),
        ("一般", 4),
        ("一般口座", 4),
    ] {
        assert_eq!(account_chart_index(input), Some(expected), "{input}");
    }
    assert_eq!(account_chart_index("NISA"), None);
}

#[test]
fn chart_color_uses_table_slot_or_single_other_color() {
    // 対応表の名前は自分の固定色、表外は名前によらず同じ中立色
    assert_eq!(
        chart_color("国内株式", product_chart_index),
        CHART_COLORS[0]
    );
    assert_eq!(chart_color("一般", account_chart_index), CHART_COLORS[4]);
    for label in ["NISA", "iDeCo", "外国株式"] {
        assert_eq!(
            chart_color(label, account_chart_index),
            CHART_COLOR_OTHER,
            "{label}"
        );
    }
}

#[test]
fn dividend_dialog_shows_signed_tax_trio() {
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
    assert!(before_tax.delta);
    assert!(!before_tax.negative);
    assert!(before_tax.positive);
    let tax = right
        .iter()
        .find(|cell| cell.label == "税額")
        .expect("ダイアログに税額がある");
    assert_eq!(tax.value, "-¥406");
    assert!(tax.delta);
    assert!(tax.negative);
    assert!(!tax.positive);
    let net = right
        .iter()
        .find(|cell| cell.label == "税引後")
        .expect("ダイアログに税引後がある");
    assert_eq!(net.value, "+¥1,594");
    assert!(net.delta);
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
    assert!(!tax.delta);
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
    assert!(tax.delta);
    assert!(!tax.negative);
    assert!(tax.positive);
}

#[test]
fn dividend_dialog_signed_helpers_cover_edges() {
    use super::{signed_gross_text, signed_tax_text};
    assert_eq!(signed_gross_text(2000.0, "¥2,000"), "+¥2,000");
    assert_eq!(signed_gross_text(-1000.0, "-¥1,000"), "-¥1,000");
    assert_eq!(signed_gross_text(0.0, "¥0"), "¥0");
    assert_eq!(signed_tax_text(406.0, "¥406"), "-¥406");
    assert_eq!(signed_tax_text(-100.0, "-¥100"), "+¥100");
    assert_eq!(signed_tax_text(0.0, "¥0"), "¥0");
}
