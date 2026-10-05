use super::*;
use crate::testing::block_on;
use std::future::ready;
use std::ops::Range;

fn rows(range: Range<usize>) -> Vec<usize> {
    range.collect()
}

fn collect(
    pages: &[(Range<usize>, i64)],
    per_page: usize,
    max_pages: usize,
) -> ListPage<usize, ()> {
    let calls = std::cell::Cell::new(0);
    let result = block_on(collect_list_pages(per_page, max_pages, |page_no| {
        calls.set(calls.get() + 1);
        assert_eq!(page_no, calls.get());
        let (range, total) = &pages[page_no - 1];
        ready(Ok(PaginatedSearchResponse {
            data: rows(range.clone()),
            total: *total,
            page: page_no as i64,
            per_page: per_page as i64,
            summary: None,
            facets: None,
        }))
    }))
    .unwrap();
    assert_eq!(calls.get(), pages.len());
    result
}

#[test]
fn first_short_page_is_the_last_page() {
    let page = collect(&[(0..3, 3)], 1000, 100);
    assert_eq!(page.rows, rows(0..3));
}

#[test]
fn full_page_then_empty_page_stops_even_if_total_is_larger() {
    // API の total が実データより多い場合でも、空ページで必ず終了する
    let page = collect(&[(0..1000, 2500), (0..0, 2500)], 1000, 100);
    assert_eq!(page.rows, rows(0..1000));
}

#[test]
fn exact_per_page_rows_matching_total_stops_without_extra_request() {
    // ちょうど per_page 件なら total 到達で終了し、空ページの取得を省く
    let page = collect(&[(0..1000, 1000)], 1000, 100);
    assert_eq!(page.rows, rows(0..1000));
}

#[test]
fn joins_more_than_two_pages_in_order() {
    let page = collect(
        &[(0..1000, 2300), (1000..2000, 2300), (2000..2300, 2300)],
        1000,
        100,
    );
    assert_eq!(page.rows, rows(0..2300));
}

#[test]
fn empty_first_page_is_not_truncated() {
    let page = collect(&[(0..0, 0)], 1000, 100);
    assert!(!page.truncated);
    assert!(page.rows.is_empty());
}

#[test]
fn natural_end_before_max_pages_is_not_truncated() {
    let page = collect(&[(0..3, 3)], 1000, 100);
    assert!(!page.truncated);
}

#[test]
fn truncation_depends_on_last_page_fullness_and_total() {
    for (name, last_page, total, want_truncated, want_rows) in [
        ("cap_reached_with_full_pages", 3..6, 100, true, rows(0..6)),
        ("one_row_over_the_cap", 3..6, 7, true, rows(0..6)),
        ("empty_last_page_at_cap", 0..0, 100, false, rows(0..3)),
        ("reaching_total_exactly_at_cap", 3..6, 6, false, rows(0..6)),
        ("short_last_page_at_cap", 3..5, 100, false, rows(0..5)),
    ] {
        let page = collect(&[(0..3, total), (last_page, total)], 3, 2);
        assert_eq!(page.truncated, want_truncated, "{name}");
        assert_eq!(page.rows, want_rows, "{name}");
    }
}

#[test]
fn total_comes_from_pushed_pages() {
    assert_eq!(collect(&[(0..3, 7), (3..5, 5)], 3, 2).total, Some(5));
    assert_eq!(collect(&[(0..3, 7), (3..5, -1)], 3, 2).total, Some(7));
    assert_eq!(collect(&[(0..0, -1)], 3, 2).total, None);
}

#[test]
fn unknown_total_stops_at_a_short_page_or_the_cap() {
    let page = collect(&[(0..3, -1), (3..6, -1)], 3, 2);
    assert_eq!(page.rows, rows(0..6));
    assert_eq!(page.total, None);
    assert!(page.truncated);
    assert!(!collect(&[(0..3, -1), (3..4, -1)], 3, 2).truncated);
}

#[test]
fn zero_page_cap_still_fetches_the_first_page() {
    assert!(collect(&[(0..3, 4)], 3, 0).truncated);
    assert!(!collect(&[(0..3, 3)], 3, 0).truncated);
}
