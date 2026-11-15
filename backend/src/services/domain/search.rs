use super::Domain;
use super::facets::{self, FacetKind, FacetSlot, FacetSpec};
use super::search_filters::{
    DateAxisFilter, FilterField, SearchFilter, push_search_filters, tokens_from_query,
};
use crate::db::{Db, FromRow, QueryBuilder, Tx};
use crate::errors::ApiError;
use crate::models::common::{
    FacetOption, PaginatedSearchResponse, SearchFacets, SearchParamsAccessor,
};
use serde::Serialize;
use shared::value::UserId;
use std::fmt::Display;
use std::future::Future;
use tracing::info;

pub type AggregateResult<S> = (i64, Option<S>, Option<SearchFacets>);

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
    /// 一覧データの読み取り元。JOIN 済みビューなど TABLE と違う名前で読む場合だけ上書きする
    const FROM: &'static str = Self::TABLE;
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
            Some(_) => DateAxisFilter::try_from(search)?,
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

    /// count 用の SELECT COUNT(*) クエリに共通の WHERE 句を積む
    fn count_query(user_id: UserId, filter: &SearchFilter) -> QueryBuilder {
        Self::filtered_query(
            format!("SELECT COUNT(*) FROM {}", Self::TABLE),
            user_id,
            filter,
        )
    }

    /// 一覧 data 用の SELECT クエリに共通の WHERE 句・ORDER BY・LIMIT/OFFSET を積む
    fn data_query(
        user_id: UserId,
        filter: &SearchFilter,
        per_page: i64,
        offset: i64,
    ) -> QueryBuilder {
        let mut qb = Self::filtered_query(
            format!("SELECT {} FROM {}", Self::COLUMNS, Self::FROM),
            user_id,
            filter,
        );
        qb.push(Self::ORDER_BY);
        qb.push(" LIMIT ").push_bind(per_page);
        qb.push(" OFFSET ").push_bind(offset);
        qb
    }

    fn fetch_summary(
        tx: &mut Tx,
        user_id: UserId,
        filter: &SearchFilter,
    ) -> impl Future<Output = Result<Self::Summary, ApiError>> + Send;

    /// FACETS の指定どおりに各 facet を順に取得して SearchFacets を組み立てる
    fn fetch_facets(
        tx: &mut Tx,
        user_id: UserId,
        filter: &SearchFilter,
    ) -> impl Future<Output = Result<SearchFacets, ApiError>> + Send {
        collect_facets::<Self>(tx, user_id, filter)
    }

    fn fetch_aggregates(
        tx: &mut Tx,
        user_id: UserId,
        filter: &SearchFilter,
        include_summary: bool,
        include_facets: bool,
    ) -> impl Future<Output = Result<AggregateResult<Self::Summary>, ApiError>> + Send {
        collect_aggregates::<Self>(tx, user_id, filter, include_summary, include_facets)
    }
}

/// FACETS の1件分を取得する。Group/Security の違いだけをここで吸収する
async fn fetch_facet_option<D: Search + ?Sized>(
    tx: &mut Tx,
    user_id: UserId,
    filter: &SearchFilter,
    spec: &FacetSpec,
) -> Result<(FacetSlot, Vec<FacetOption>), ApiError> {
    let push = |qb: &mut QueryBuilder| D::push_filters(qb, user_id, filter);
    let options = match spec.kind {
        FacetKind::Group { field, order } => {
            facets::fetch_group_facets(
                &mut *tx,
                user_id,
                D::TABLE,
                field.as_sql_expr(),
                order,
                push,
            )
            .await?
        }
        FacetKind::Security { label_order } => {
            facets::fetch_security_facets(&mut *tx, user_id, D::TABLE, label_order, push).await?
        }
    };
    Ok((spec.slot, options))
}
async fn collect_facets<D: Search + ?Sized>(
    tx: &mut Tx,
    user_id: UserId,
    filter: &SearchFilter,
) -> Result<SearchFacets, ApiError> {
    let mut result = SearchFacets::default();
    for spec in D::FACETS {
        let (slot, options) = fetch_facet_option::<D>(&mut *tx, user_id, filter, spec).await?;
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

async fn collect_aggregates<D: Search + ?Sized>(
    tx: &mut Tx,
    user_id: UserId,
    filter: &SearchFilter,
    include_summary: bool,
    include_facets: bool,
) -> Result<AggregateResult<D::Summary>, ApiError> {
    let mut count_qb = D::count_query(user_id, filter);
    let total: i64 = count_qb.build_query_scalar().fetch_one(&mut *tx).await?;
    let summary = if include_summary {
        Some(D::fetch_summary(&mut *tx, user_id, filter).await?)
    } else {
        None
    };
    let facets = if include_facets {
        Some(D::fetch_facets(&mut *tx, user_id, filter).await?)
    } else {
        None
    };
    Ok((total, summary, facets))
}

/// 4ドメイン共通の検索制御フロー。
/// 一覧の読み取りは1つの利用者Txで順に実行する
pub async fn search<D: Search>(
    pool: &Db,
    user_id: UserId,
    params: D::Params,
) -> Result<PaginatedSearchResponse<D::Data, D::Summary>, ApiError> {
    info!("[{}.search] リクエスト受信", D::NAME);

    // 1. 条件作成: page 情報を取り出し、params を検証して共通の検索条件へ変換する
    let page = params.page();
    let per_page = params.per_page();
    let offset = params.offset();
    let include_summary = params.should_include_summary();
    let include_facets = params.should_include_facets();
    let filter = D::make_filter(&params)?;

    let mut data_qb = D::data_query(user_id, &filter, per_page, offset);

    let mut tx = pool.begin_with_user(user_id).await?;
    let data = data_qb
        .build_query_as::<D::Data>()
        .fetch_all(&mut tx)
        .await?;
    let (total, summary, facets) =
        D::fetch_aggregates(&mut tx, user_id, &filter, include_summary, include_facets).await?;
    tx.rollback().await?;

    // 4. 応答組立
    Ok(PaginatedSearchResponse {
        data,
        total,
        page,
        per_page,
        summary,
        facets,
    })
}
