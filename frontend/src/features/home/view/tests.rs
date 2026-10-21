use super::{account_chart_index, fallback_chart_index, product_chart_index};

#[test]
fn product_chart_index_is_fixed_per_label() {
    assert_eq!(product_chart_index("国内株式"), 0);
    assert_eq!(product_chart_index("投資信託"), 1);
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
        assert_eq!(account_chart_index(input), expected, "{input}");
    }
}

#[test]
fn account_chart_index_is_stable_for_unknown_names() {
    assert_eq!(account_chart_index("iDeCo"), account_chart_index("iDeCo"));
    assert_eq!(fallback_chart_index("iDeCo"), account_chart_index("iDeCo"));
}
