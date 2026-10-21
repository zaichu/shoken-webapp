use super::{account_chart_index, assign_chart_indices, product_chart_index};

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
fn assign_chart_indices_avoids_collision_with_fixed_slots() {
    // 「NISA」は対応表にないが、同じ画面に出る「一般」と同じ色にならないようにする
    let indices = assign_chart_indices(&["一般", "NISA"], account_chart_index);
    assert_eq!(indices[0], 4);
    assert_ne!(indices[1], 4);
}

#[test]
fn assign_chart_indices_keeps_fixed_slots_and_fills_free_ones() {
    let indices = assign_chart_indices(&["特定", "iDeCo", "NISA成長"], account_chart_index);
    assert_eq!(indices[0], 0);
    assert_eq!(indices[2], 1);
    assert_ne!(indices[1], 0);
    assert_ne!(indices[1], 1);
}

#[test]
fn assign_chart_indices_is_independent_of_label_order() {
    // 系列は評価額順に並ぶため、並びが変わっても同じ名前には同じ番号が付く必要がある
    let a = assign_chart_indices(&["NISA", "iDeCo"], account_chart_index);
    let b = assign_chart_indices(&["iDeCo", "NISA"], account_chart_index);
    assert_eq!(a[0], b[1]);
    assert_eq!(a[1], b[0]);
}

#[test]
fn assign_chart_indices_keeps_color_when_other_unknowns_appear() {
    // ほかの表外系列が増減しても、同じ名前には同じ番号が付く
    let solo = assign_chart_indices(&["iDeCo"], account_chart_index);
    let with_nisa = assign_chart_indices(&["NISA", "iDeCo"], account_chart_index);
    assert_eq!(solo[0], with_nisa[1]);
}

#[test]
fn assign_chart_indices_falls_back_deterministically_when_full() {
    // 5 枠すべて埋まった後の表外の名前は、名前だけで決まる番号に割る(毎回同じ)
    let labels = [
        "特定",
        "NISA成長",
        "旧NISA",
        "NISAつみたて",
        "一般",
        "iDeCo",
    ];
    assert_eq!(
        assign_chart_indices(&labels, account_chart_index),
        assign_chart_indices(&labels, account_chart_index)
    );
}
