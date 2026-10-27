use super::*;
use crate::api::dto::{AssetBalance, Dividend, PaginatedSearchResponse};
use crate::support::pagination::{LIST_MAX_PAGES, LIST_PER_PAGE, ListEndpoint, collect_list_pages};
use crate::testing::block_on;
use std::future::ready;

#[test]
fn summary_endpoints_share_the_list_paths() {
    assert_eq!(
        <AssetBalance as ListEndpoint>::PATH,
        "/api/v1/asset-balances"
    );
    assert_eq!(<Dividend as ListEndpoint>::PATH, "/api/v1/dividends");
    assert_eq!(<AssetBalance as ListEndpoint>::MAX_PAGES, LIST_MAX_PAGES);
    assert_eq!(<AssetBalance as ListEndpoint>::PER_PAGE, LIST_PER_PAGE);
}

#[test]
fn has_current_summary_only_matches_same_generation() {
    let generation = Generation::new(1);
    let other = Generation::new(2);
    let slot: Option<(Generation, Option<u32>)> = Some((generation, Some(7)));
    assert!(has_current_summary(&slot, generation));
    assert!(!has_current_summary(&slot, other));
    assert!(!has_current_summary::<u32>(
        &Some((generation, None)),
        other
    ));
    assert!(!has_current_summary::<u32>(&None, generation));
}

#[test]
fn summary_fetch_blocks_second_request_while_inflight() {
    let generation = Generation::new(1);
    let mut map = HashMap::new();
    assert_eq!(
        begin_summary_fetch(&mut map, generation, SummaryKind::Assets),
        Some(1)
    );
    assert_eq!(
        begin_summary_fetch(&mut map, generation, SummaryKind::Assets),
        None,
        "取得中の同じ集計を重ねて要求しない"
    );
    assert_eq!(
        begin_summary_fetch(&mut map, generation, SummaryKind::DividendsCurrent),
        Some(1),
        "別の集計は取得中でも取れる"
    );
    assert!(finish_summary_fetch(
        &mut map,
        generation,
        SummaryKind::Assets,
        1
    ));
    assert_eq!(
        begin_summary_fetch(&mut map, generation, SummaryKind::Assets),
        Some(2),
        "応答後は再取得できる"
    );
}

#[test]
fn summary_fetch_drops_stale_response() {
    let generation = Generation::new(1);
    let mut map = HashMap::new();
    let rev = begin_summary_fetch(&mut map, generation, SummaryKind::Assets).unwrap();
    assert!(finish_summary_fetch(
        &mut map,
        generation,
        SummaryKind::Assets,
        rev
    ));
    let next = begin_summary_fetch(&mut map, generation, SummaryKind::Assets).unwrap();
    assert!(
        !finish_summary_fetch(&mut map, generation, SummaryKind::Assets, rev),
        "古い応答は inflight を下ろさない"
    );
    assert!(
        map.get(&(generation, SummaryKind::Assets))
            .is_some_and(|entry| entry.inflight),
        "新しい取得はまだ取得中のまま"
    );
    assert!(finish_summary_fetch(
        &mut map,
        generation,
        SummaryKind::Assets,
        next
    ));
    // 別世代のキーは同じ rev でも混ざらない
    let other = Generation::new(2);
    assert_eq!(
        begin_summary_fetch(&mut map, other, SummaryKind::Assets),
        Some(1)
    );
}

#[test]
fn settle_summary_keeps_displayed_value_on_refresh_error() {
    let generation = Generation::new(1);
    let error = || -> Result<Option<u32>, ApiError> { Err(ApiError::http(500)) };
    // 裏再取得の失敗では表示済みの値を消さない(書き戻さない)
    let current = Some((generation, Some(260_000u32)));
    assert_eq!(settle_summary(&current, generation, error()), None);
    // 初回の失敗は従来どおり失敗として書く(「一部の集計を取得できませんでした」が出る)
    assert_eq!(
        settle_summary::<u32>(&None, generation, error()),
        Some((generation, None))
    );
    assert_eq!(
        settle_summary(&Some((generation, None::<u32>)), generation, error()),
        Some((generation, None))
    );
    // 別世代の成功値は今の世代の失敗を隠さない
    assert_eq!(
        settle_summary(&Some((Generation::new(0), Some(1u32))), generation, error()),
        Some((generation, None))
    );
    // 成功は常に書く(Ok(None) は「集計なし」の確定)
    assert_eq!(
        settle_summary(&current, generation, Ok(Some(1))),
        Some((generation, Some(1)))
    );
    assert_eq!(
        settle_summary(&current, generation, Ok(None)),
        Some((generation, None))
    );
}

#[test]
fn refresh_failed_ignores_other_generations() {
    let generation = Generation::new(1);
    let other = Generation::new(2);
    let mut map = HashMap::new();
    let failed = || SummaryFetch {
        failed: true,
        ..SummaryFetch::default()
    };
    map.insert((other, SummaryKind::Assets), failed());
    assert!(!refresh_failed(&map, generation));
    map.insert((generation, SummaryKind::Recent), failed());
    assert!(refresh_failed(&map, generation));
}

// 年次配当の集計が依存する全ページ取得の挙動。`fetch_dividend_year` は
// 取得結果を `Some(rows)` に包むだけなので、ここでは行の結合だけを固定する
fn year_rows(pages: &[(Vec<usize>, i64)]) -> Result<Option<Vec<usize>>, ApiError> {
    block_on(collect_list_pages(
        LIST_PER_PAGE,
        LIST_MAX_PAGES,
        |page_no| {
            let (rows, total) = &pages[page_no - 1];
            ready(Ok(PaginatedSearchResponse {
                data: rows.clone(),
                total: *total,
                page: page_no as i64,
                per_page: LIST_PER_PAGE as i64,
                summary: None::<()>,
                facets: None,
            }))
        },
    ))
    .map(|page| Some(page.rows))
}

#[test]
fn dividend_year_joins_multiple_pages_in_order() {
    let full: Vec<usize> = (0..LIST_PER_PAGE).collect();
    let rest: Vec<usize> = (LIST_PER_PAGE..LIST_PER_PAGE + 3).collect();
    let total = (LIST_PER_PAGE + 3) as i64;
    let rows = year_rows(&[(full, total), (rest, total)]).expect("fetch");
    assert_eq!(rows, Some((0..LIST_PER_PAGE + 3).collect()));
}

#[test]
fn dividend_year_empty_returns_no_rows() {
    let rows = year_rows(&[(Vec::new(), 0)]).expect("fetch");
    assert_eq!(rows, Some(Vec::new()));
}

#[test]
fn dividend_year_mid_page_error_fails_whole_fetch() {
    let calls = std::cell::Cell::new(0);
    let result: Result<Option<Vec<usize>>, ApiError> = block_on(async {
        collect_list_pages(LIST_PER_PAGE, LIST_MAX_PAGES, |page_no| {
            calls.set(calls.get() + 1);
            if page_no == 1 {
                ready(Ok(PaginatedSearchResponse {
                    data: (0..LIST_PER_PAGE).collect(),
                    total: LIST_PER_PAGE as i64 + 1,
                    page: 1,
                    per_page: LIST_PER_PAGE as i64,
                    summary: None::<()>,
                    facets: None,
                }))
            } else {
                ready(Err(ApiError::http(500)))
            }
        })
        .await
        .map(|page| Some(page.rows))
    });
    assert!(matches!(result, Err(ApiError::Http { .. })));
    assert_eq!(calls.get(), 2, "失敗したページ以降は要求しない");
}
