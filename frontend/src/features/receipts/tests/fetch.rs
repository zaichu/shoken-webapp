use crate::api::ApiError;
use crate::api::dto::{PaginatedSearchResponse, SearchFacets};
use crate::features::receipts::truncated_list_warning;
use crate::support::pagination::{LIST_MAX_PAGES, LIST_PER_PAGE, collect_list_pages};
use crate::testing::block_on;
use std::cell::Cell;
use std::future::{Ready, ready};

type Page = PaginatedSearchResponse<usize, (), SearchFacets>;

fn page(data: Vec<usize>, total: i64, summary: Option<()>) -> Page {
    PaginatedSearchResponse {
        data,
        total,
        page: 0,
        per_page: LIST_PER_PAGE as i64,
        summary,
        facets: None,
    }
}

fn full_page(page_no: usize, total: i64) -> Page {
    let start = (page_no - 1) * LIST_PER_PAGE;
    page((start..start + LIST_PER_PAGE).collect(), total, None)
}

#[test]
fn mid_page_failure_fails_the_whole_fetch_because_partial_rows_would_silently_corrupt_the_list() {
    let calls = Cell::new(0);
    let fetch = |page_no: usize| -> Ready<Result<Page, ApiError>> {
        calls.set(calls.get() + 1);
        ready(match page_no {
            1 => Ok(full_page(1, 3000)),
            _ => Err(ApiError::http(500)),
        })
    };

    let result = block_on(collect_list_pages(LIST_PER_PAGE, LIST_MAX_PAGES, fetch));

    assert!(matches!(result, Err(ApiError::Http { status: 500, .. })));
    assert_eq!(calls.get(), 2, "失敗したページ以降は要求しない");
}

#[test]
fn reaching_the_page_cap_returns_rows_marked_truncated() {
    let result = block_on(collect_list_pages(
        LIST_PER_PAGE,
        LIST_MAX_PAGES,
        |page_no| ready(Ok::<Page, ApiError>(full_page(page_no, i64::MAX))),
    ));

    let page = result.expect("上限到達は失敗ではない");
    assert!(page.truncated);
    assert_eq!(page.rows.len(), LIST_PER_PAGE * LIST_MAX_PAGES);
}

#[test]
fn short_last_page_is_not_truncated_and_keeps_first_page_summary() {
    let result = block_on(collect_list_pages(
        LIST_PER_PAGE,
        LIST_MAX_PAGES,
        |page_no| -> Ready<Result<Page, ApiError>> {
            ready(Ok(match page_no {
                1 => page((0..1000).collect(), 2300, Some(())),
                _ => page((1000..1300).collect(), 2300, None),
            }))
        },
    ));

    let page = result.expect("fetch");
    assert!(!page.truncated);
    assert_eq!(page.rows.len(), 1300);
    assert_eq!(page.summary, Some(()));
}

#[test]
fn truncated_warning_uses_fixed_message() {
    assert_eq!(
        truncated_list_warning(),
        "一覧は最大100,000件まで表示しています。検索条件を絞り込んでください。"
    );
}
