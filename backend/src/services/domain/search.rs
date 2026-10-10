use super::Domain;
use super::facets::{self, FacetKind, FacetSlot, FacetSpec};
use super::search_filters::{
    DateAxisFilter, FilterField, SearchFilter, fetch_if_included, push_search_filters,
    tokens_from_query,
};
use crate::db::{Db, FromRow, QueryBuilder};
use crate::errors::ApiError;
use crate::models::common::{PaginatedSearchResponse, SearchFacets, SearchParamsAccessor};
use serde::Serialize;
use shared::value::UserId;
use std::fmt::Display;
use std::future::Future;
use tracing::info;

/// 検索・集計付き一覧を持つドメイン。
/// ドメインごとに持つのは SELECT 列・ORDER BY・フィルタ/facets の指定・summary の SQL と、
/// params 上のドメイン固有フィールドの取り出しだけ
pub trait Search: Domain {
    /// 一覧に返す行の型
    type Data: FromRow + Serialize + Send + 'static;
    /// ハンドラーが受け取るクエリパラメータの型
    type Params: SearchParamsAccessor;
    /// 検索条件全体の集計の型
    type Summary: FromRow + Serialize + Send + 'static;

    /// SELECT 句に並べる列一覧（固定文字列）
    const COLUMNS: &'static str;
    /// ORDER BY 句（先頭スペース込みの固定文字列）
    const ORDER_BY: &'static str;
    /// date 系パラメータを適用する日付カラム。
    /// 日付軸を持たないドメインは None（date 系パラメータは解釈・検証ともにしない）
    const DATE_COLUMN: Option<&'static str>;
    /// 完全一致・フリーワード（ILIKE OR）共通の対象フィールド（SQL への出力順）
    const FILTER_FIELDS: &'static [FilterField];
    /// 返す facets（SearchFacets のスロットと取得方法の対応表）
    const FACETS: &'static [FacetSpec];

    /// params 上のドメイン固有フィールドを取り出す。対象外フィールドは NO_FIELD を返す
    fn filter_value(params: &Self::Params, field: FilterField) -> &Option<String>;

    /// Params を検証して共通の検索条件へ変換する
    fn make_filter(params: &Self::Params) -> Result<SearchFilter, ApiError> {
        let search = params.search_params();
        let date_axis = match Self::DATE_COLUMN {
            Some(_) => DateAxisFilter::from_search_params(search)?,
            None => DateAxisFilter::default(),
        };
        let mut values: [Option<String>; FilterField::COUNT] = std::array::from_fn(|_| None);
        for &field in Self::FILTER_FIELDS {
            values[field as usize] = Self::filter_value(params, field).clone();
        }
        Ok(SearchFilter {
            date_axis,
            tokens: tokens_from_query(search.q.as_deref()),
            values,
        })
    }

    /// count/data/summary/facets 共通の WHERE 句を積む
    fn push_filters(qb: &mut QueryBuilder, user_id: UserId, filter: &SearchFilter) {
        let exact_match_fields: Vec<_> = Self::FILTER_FIELDS
            .iter()
            .map(|&field| (field.column(), filter.value(field)))
            .collect();
        let token_columns: Vec<_> = Self::FILTER_FIELDS
            .iter()
            .map(|field| field.column())
            .collect();
        push_search_filters(
            qb,
            user_id,
            Self::DATE_COLUMN.map(|column| (column, &filter.date_axis)),
            &exact_match_fields,
            &filter.tokens,
            &token_columns,
        );
    }

    /// 固定の SELECT 文にこのドメインの検索条件を積んだ QueryBuilder を返す。
    /// summary 系など WHERE 句をクエリ途中へ差し込む起点に使う
    fn filtered_query(
        select_sql: impl Display,
        user_id: UserId,
        filter: &SearchFilter,
    ) -> QueryBuilder {
        let mut qb = QueryBuilder::new(select_sql);
        Self::push_filters(&mut qb, user_id, filter);
        qb
    }

    fn fetch_summary(
        pool: &Db,
        user_id: UserId,
        filter: &SearchFilter,
    ) -> impl Future<Output = Result<Self::Summary, ApiError>> + Send;

    /// FACETS の指定どおりに各 facet を並行取得して SearchFacets を組み立てる
    fn fetch_facets(
        pool: &Db,
        user_id: UserId,
        filter: &SearchFilter,
    ) -> impl Future<Output = Result<SearchFacets, ApiError>> + Send {
        collect_facets::<Self>(pool, user_id, filter)
    }
}

/// FACETS 指定どおりに各 facet を並行取得して SearchFacets を組み立てる
async fn collect_facets<D: Search + ?Sized>(
    pool: &Db,
    user_id: UserId,
    filter: &SearchFilter,
) -> Result<SearchFacets, ApiError> {
    let fetches = D::FACETS.iter().map(|spec| async move {
        let options = match spec.kind {
            FacetKind::Group { field, order } => {
                facets::fetch_group_facets(pool, D::TABLE, field.as_sql_expr(), order, |qb| {
                    D::push_filters(qb, user_id, filter)
                })
                .await?
            }
            FacetKind::Security { label_order } => {
                facets::fetch_security_facets(pool, D::TABLE, label_order, |qb| {
                    D::push_filters(qb, user_id, filter)
                })
                .await?
            }
        };
        Ok::<_, ApiError>((spec.slot, options))
    });

    let mut result = SearchFacets::default();
    for (slot, options) in futures_util::future::try_join_all(fetches).await? {
        match slot {
            FacetSlot::Products => result.products = Some(options),
            FacetSlot::Accounts => result.accounts = Some(options),
            FacetSlot::Securities => result.securities = Some(options),
            FacetSlot::Funds => result.funds = Some(options),
            FacetSlot::Years => result.years = Some(options),
            FacetSlot::YearMonths => result.year_months = Some(options),
        }
    }
    Ok(result)
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
    let filter = D::make_filter(&params)?;

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
        futures_util::future::try_join4(count_fut, data_fut, summary_fut, facets_fut).await?;

    Ok(PaginatedSearchResponse {
        data,
        total,
        page,
        per_page,
        summary,
        facets,
    })
}
