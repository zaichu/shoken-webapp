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
fn stops_at_max_pages_when_pages_never_run_short() {
    let mut pages = PageCollector::new(3, 2);
    assert!(pages.push(rows(0..3), 100));
    assert!(!pages.push(rows(3..6), 100));
    assert_eq!(pages.into_rows(), rows(0..6));
}
