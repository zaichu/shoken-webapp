use super::Domain;
use super::search_filters::fetch_if_included;
use crate::db::{Db, FromRow, QueryBuilder};
use crate::errors::ApiError;
use crate::models::common::{PaginatedSearchResponse, SearchFacets, SearchParamsAccessor};
use serde::Serialize;
use shared::value::UserId;
use std::future::Future;
use tracing::info;

/// 検索・集計付き一覧を持つドメイン。
/// ドメインごとに持つのは SELECT 列・ORDER BY・フィルタ条件・summary/facets の取得だけ
pub trait Search: Domain {
    /// 一覧に返す行の型
    type Data: FromRow + Serialize + Send + 'static;
    /// ハンドラーが受け取るクエリパラメータの型
    type Params: SearchParamsAccessor;
    /// Params を所有したまま検証・変換した検索条件
    type Filter: TryFrom<Self::Params, Error = ApiError>;
    /// 検索条件全体の集計の型
    type Summary: FromRow + Serialize + Send + 'static;

    /// SELECT 句に並べる列一覧（固定文字列）
    const COLUMNS: &'static str;
    /// ORDER BY 句（先頭スペース込みの固定文字列）
    const ORDER_BY: &'static str;

    /// count/data/summary/facets 共通の WHERE 句を積む
    fn push_filters(qb: &mut QueryBuilder, user_id: UserId, filter: &Self::Filter);

    fn fetch_summary(
        pool: &Db,
        user_id: UserId,
        filter: &Self::Filter,
    ) -> impl Future<Output = Result<Self::Summary, ApiError>> + Send;

    fn fetch_facets(
        pool: &Db,
        user_id: UserId,
        filter: &Self::Filter,
    ) -> impl Future<Output = Result<SearchFacets, ApiError>> + Send;
}

/// 4ドメイン共通の検索制御フロー。
/// count/data/summary/facets は相互に依存しないため並行実行する
pub async fn search<D: Search>(
    pool: &Db,
    user_id: UserId,
    params: D::Params,
) -> Result<PaginatedSearchResponse<D::Data, D::Summary, SearchFacets>, ApiError> {
    info!("[{}.search] リクエスト受信", D::NAME);

    let page = params.page();
    let per_page = params.per_page();
    let offset = params.offset();
    let include_summary = params.should_include_summary();
    let include_facets = params.should_include_facets();
    let filter = D::Filter::try_from(params)?;

    // summary/facets は include_* が true の場合だけ実クエリを発行する
    let summary_fut = fetch_if_included(include_summary, D::fetch_summary(pool, user_id, &filter));
    let facets_fut = fetch_if_included(include_facets, D::fetch_facets(pool, user_id, &filter));

    let mut count_qb = QueryBuilder::new(format!("SELECT COUNT(*) FROM {}", D::TABLE));
    D::push_filters(&mut count_qb, user_id, &filter);
    let count_fut = async move {
        let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await?;
        Ok::<_, ApiError>(total)
    };

    let mut data_qb = QueryBuilder::new(format!("SELECT {} FROM {}", D::COLUMNS, D::TABLE));
    D::push_filters(&mut data_qb, user_id, &filter);
    data_qb.push(D::ORDER_BY);
    data_qb.push(" LIMIT ").push_bind(per_page);
    data_qb.push(" OFFSET ").push_bind(offset);
    let data_fut = async move {
        let data = data_qb.build_query_as::<D::Data>().fetch_all(pool).await?;
        Ok::<_, ApiError>(data)
    };

    let (total, data, summary, facets) =
        tokio::try_join!(count_fut, data_fut, summary_fut, facets_fut)?;

    Ok(PaginatedSearchResponse {
        data,
        total,
        page,
        per_page,
        summary,
        facets,
    })
}
