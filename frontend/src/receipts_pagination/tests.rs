use super::*;

fn rows(range: std::ops::Range<usize>) -> Vec<usize> {
    range.collect()
}

#[test]
fn first_short_page_is_the_last_page() {
    let mut pages = PageCollector::new(1000, 100);
    assert_eq!(pages.next_page(), 1);
    assert!(!pages.push(rows(0..3), 3));
    assert_eq!(pages.into_rows(), rows(0..3));
}

#[test]
fn full_page_then_empty_page_stops_even_if_total_is_larger() {
    // API の total が実データより多い場合でも、空ページで必ず終了する
    let mut pages = PageCollector::new(1000, 100);
    assert!(pages.push(rows(0..1000), 2500));
    assert_eq!(pages.next_page(), 2);
    assert!(!pages.push(Vec::new(), 2500));
    assert_eq!(pages.into_rows().len(), 1000);
}

#[test]
fn exact_per_page_rows_matching_total_stops_without_extra_request() {
    // ちょうど per_page 件なら total 到達で終了し、空ページの取得を省く
    let mut pages = PageCollector::new(1000, 100);
    assert!(!pages.push(rows(0..1000), 1000));
    assert_eq!(pages.into_rows().len(), 1000);
}

#[test]
fn joins_more_than_two_pages_in_order() {
    let mut pages = PageCollector::new(1000, 100);
    assert!(pages.push(rows(0..1000), 2300));
    assert_eq!(pages.next_page(), 2);
    assert!(pages.push(rows(1000..2000), 2300));
    assert!(!pages.push(rows(2000..2300), 2300));
    assert_eq!(pages.into_rows(), rows(0..2300));
}

#[test]
fn empty_first_page_is_not_truncated() {
    let mut pages = PageCollector::<usize>::new(1000, 100);
    assert!(!pages.push(Vec::new(), 0));
    assert!(!pages.truncated());
    assert!(pages.into_rows().is_empty());
}

#[test]
fn natural_end_before_max_pages_is_not_truncated() {
    let mut pages = PageCollector::new(1000, 100);
    assert!(!pages.push(rows(0..3), 3));
    assert!(!pages.truncated());
}

#[test]
fn truncation_depends_on_last_page_fullness_and_total() {
    for (name, last_page, total, want_truncated, want_rows) in [
        ("cap_reached_with_full_pages", rows(3..6), 100, true, 6),
        ("one_row_over_the_cap", rows(3..6), 7, true, 6),
        ("empty_last_page_at_cap", Vec::new(), 100, false, 3),
        ("reaching_total_exactly_at_cap", rows(3..6), 6, false, 6),
        ("short_last_page_at_cap", rows(3..5), 100, false, 5),
    ] {
        let mut pages = PageCollector::new(3, 2);
        assert!(pages.push(rows(0..3), total), "{name}");
        assert!(!pages.push(last_page, total), "{name}");
        assert_eq!(pages.truncated(), want_truncated, "{name}");
        assert_eq!(pages.into_rows().len(), want_rows, "{name}");
    }
}

#[test]
fn total_comes_from_pushed_pages() {
    let mut pages = PageCollector::<usize>::new(3, 2);
    assert_eq!(pages.total(), None);
    pages.push(rows(0..3), 7);
    assert_eq!(pages.total(), Some(7));
    pages.push(rows(3..5), 5);
    assert_eq!(pages.total(), Some(5));
    pages.push(rows(5..5), -1);
    assert_eq!(pages.total(), Some(5));
}
