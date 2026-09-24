use super::*;
use std::cell::Cell;
use std::future::{ready, Ready};
use std::task::{Context, Poll, Waker};

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    let mut context = Context::from_waker(Waker::noop());
    match std::pin::pin!(future).poll(&mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("スタブの Future は即座に完了するはず"),
    }
}

type Page = (Vec<usize>, i64, Option<()>);

fn full_page(page_no: usize, total: i64) -> Page {
    let start = (page_no - 1) * RECEIPT_LIST_PER_PAGE;
    (
        (start..start + RECEIPT_LIST_PER_PAGE).collect(),
        total,
        None,
    )
}

#[test]
fn mid_page_failure_fails_the_whole_fetch_because_partial_rows_would_silently_corrupt_the_list() {
    let calls = Cell::new(0);
    let fetch = |page_no: usize| -> Ready<Result<Page, ApiError>> {
        calls.set(calls.get() + 1);
        ready(match page_no {
            1 => Ok(full_page(1, 3000)),
            _ => Err(ApiError::Http { status: 500 }),
        })
    };

    let result = block_on(fetch_pages(fetch, |page: Page| page));

    assert!(matches!(result, Err(ApiError::Http { status: 500 })));
    assert_eq!(calls.get(), 2, "失敗したページ以降は要求しない");
}

#[test]
fn reaching_the_page_cap_returns_rows_marked_truncated() {
    let result = block_on(fetch_pages(
        |page_no| ready(Ok::<Page, ApiError>(full_page(page_no, i64::MAX))),
        |page: Page| page,
    ));

    let (rows, _summary, truncated) = result.expect("上限到達は失敗ではない");
    assert!(truncated);
    assert_eq!(rows.len(), RECEIPT_LIST_PER_PAGE * RECEIPT_LIST_MAX_PAGES);
}

#[test]
fn short_last_page_is_not_truncated_and_keeps_first_page_summary() {
    let result = block_on(fetch_pages(
        |page_no| -> Ready<Result<Page, ApiError>> {
            ready(Ok(match page_no {
                1 => ((0..1000).collect(), 2300, Some(())),
                _ => ((1000..2300).collect(), 2300, None),
            }))
        },
        |page: Page| page,
    ));

    let (rows, summary, truncated) = result.expect("fetch");
    assert!(!truncated);
    assert_eq!(rows.len(), 2300);
    assert_eq!(summary, Some(()));
}

#[test]
fn truncated_warning_uses_the_same_wording_as_react_db_warning() {
    assert_eq!(
        truncated_list_warning(),
        "一覧は最大100,000件まで表示しています。検索条件を絞り込んでください。"
    );
}
