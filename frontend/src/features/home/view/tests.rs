use super::{
    CHART_COLOR_OTHER, CHART_COLORS, account_chart_index, chart_color, product_chart_index,
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
