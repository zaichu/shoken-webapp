use std::future::Future;

use serde::de::DeserializeOwned;

use crate::api::dto::{PaginatedSearchResponse, SearchFacets};
use crate::api::{ApiClient, ApiError};

/// 一覧エンドポイントの取得上限。API の total が実データより大きい等の
/// 不整合でも必ず終了するためのページ数上限と、1 ページあたりの件数。
pub const LIST_PER_PAGE: usize = 1000;
pub const LIST_MAX_PAGES: usize = 100;

/// 検索対応の一覧 API を型で表す。実体は持たないマーカー型。
/// 一覧取得・ページ結合・切り詰め判定はすべて `fetch_all_pages` に集約する。
pub trait ListEndpoint {
    type Row: DeserializeOwned;
    type Summary: DeserializeOwned;

    const PATH: &'static str;
    const PER_PAGE: usize = LIST_PER_PAGE;
    const MAX_PAGES: usize = LIST_MAX_PAGES;
    const INCLUDE_FACETS: bool = false;
}

/// `fetch_all_pages` の返却。summary は先頭ページのものを採用する。
pub struct ListPage<T, S> {
    pub rows: Vec<T>,
    pub summary: Option<S>,
    pub truncated: bool,
}

/// 指定エンドポイントの一覧を全ページ取得する。
pub async fn fetch_all_pages<E: ListEndpoint>() -> Result<ListPage<E::Row, E::Summary>, ApiError> {
    let client = ApiClient::read_client();
    collect_list_pages(E::PER_PAGE, E::MAX_PAGES, |page_no| {
        let client = client.clone();
        async move {
            let page = page_no.to_string();
            let per_page = E::PER_PAGE.to_string();
            // summary・facets はどのページも同じ全体集計を返すため、1ページ目だけ取る
            let include_aggregates = if page_no == 1 { "true" } else { "false" };
            let mut query = vec![
                ("page", page.as_str()),
                ("per_page", per_page.as_str()),
                ("include_summary", include_aggregates),
            ];
            if E::INCLUDE_FACETS {
                query.push(("include_facets", include_aggregates));
            }
            client
                .get_json::<PaginatedSearchResponse<E::Row, E::Summary, SearchFacets>>(
                    E::PATH,
                    &query,
                )
                .await
        }
    })
    .await
}

/// ページ取得を差し替えられる `fetch_all_pages` の心臓部。
pub(crate) async fn collect_list_pages<T, S, Fut>(
    per_page: usize,
    max_pages: usize,
    fetch_page: impl Fn(usize) -> Fut,
) -> Result<ListPage<T, S>, ApiError>
where
    Fut: Future<Output = Result<PaginatedSearchResponse<T, S, SearchFacets>, ApiError>>,
{
    let mut pages = PageCollector::new(per_page, max_pages);
    let mut summary = None;
    loop {
        let page_no = pages.next_page();
        let page = fetch_page(page_no).await?;
        if page_no == 1 {
            summary = page.summary;
        }
        if !pages.push(page.data, page.total) {
            break;
        }
    }
    Ok(ListPage {
        truncated: pages.truncated(),
        rows: pages.into_rows(),
        summary,
    })
}

/// 一覧 API のページ結合状態。
/// API は 1 ページあたり最大 `per_page` 件しか返さないため、
/// 返却件数が `per_page` 未満のページ(=末尾)まで逐次取得して 1 つの Vec に結合する。
pub struct PageCollector<T> {
    per_page: usize,
    max_pages: usize,
    rows: Vec<T>,
    fetched: usize,
    last_len: usize,
    total: Option<usize>,
}

impl<T> PageCollector<T> {
    pub fn new(per_page: usize, max_pages: usize) -> Self {
        Self {
            per_page,
            max_pages,
            rows: Vec::new(),
            fetched: 0,
            last_len: 0,
            total: None,
        }
    }

    pub fn next_page(&self) -> usize {
        self.fetched + 1
    }

    /// 1 ページ分を結合し、次ページを取得すべきなら true を返す。
    /// `total` が取得済み件数以下なら、ちょうど `per_page` 件で終わる場合の
    /// 空振りリクエストを省いて打ち切る。`total` が実データより大きい等の
    /// API 不整合で終端に達しない場合に備え、`max_pages` を上限とする。
    pub fn push(&mut self, page: Vec<T>, total: i64) -> bool {
        self.fetched += 1;
        self.last_len = page.len();
        self.rows.extend(page);
        if total >= 0 {
            self.total = Some(total as usize);
        }
        self.last_len >= self.per_page
            && self.total.is_none_or(|total| self.rows.len() < total)
            && self.fetched < self.max_pages
    }

    pub fn total(&self) -> Option<usize> {
        self.total
    }

    pub fn truncated(&self) -> bool {
        self.fetched >= self.max_pages
            && self.last_len >= self.per_page
            && self.total.is_none_or(|total| self.rows.len() < total)
    }

    pub fn into_rows(self) -> Vec<T> {
        self.rows
    }
}

#[cfg(test)]
pub(crate) mod tests;
