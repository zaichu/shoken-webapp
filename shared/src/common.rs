use serde::{Deserialize, Deserializer, Serialize};
use std::{fmt::Display, str::FromStr};

/// URL query は数値や boolean も文字列として届くため、対象型へ明示的に parse する。
fn deserialize_optional_from_string<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: Display,
{
    let value = Option::<String>::deserialize(deserializer)?;
    value
        .filter(|v| !v.is_empty())
        .map(|v| v.parse::<T>().map_err(serde::de::Error::custom))
        .transpose()
}

/// ページネーションクエリパラメータ（全ドメイン共通）
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PaginationParams {
    #[serde(default, deserialize_with = "deserialize_optional_from_string")]
    pub page: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_from_string")]
    pub per_page: Option<i64>,
}

impl PaginationParams {
    #[must_use]
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }
    #[must_use]
    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(200).clamp(1, 1000)
    }
    #[must_use]
    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.per_page()
    }
}

/// 検索・集計付き一覧の共通クエリパラメータ。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SearchQueryParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    /// フリーワード検索。各ドメインが定義する検索対象カラムへの token AND 条件としてSQLへ変換する。
    pub q: Option<String>,
    /// 開始日（YYYY-MM-DD）
    pub date_from: Option<String>,
    /// 終了日（YYYY-MM-DD）
    pub date_to: Option<String>,
    /// 年（YYYY）
    #[serde(default, deserialize_with = "deserialize_optional_from_string")]
    pub year: Option<i32>,
    /// 年月（YYYY-MM）
    pub year_month: Option<String>,
    /// 単日（YYYY-MM-DD）
    pub date: Option<String>,
    /// summary を返すかどうか
    #[serde(default, deserialize_with = "deserialize_optional_from_string")]
    pub include_summary: Option<bool>,
    /// facets を返すかどうか
    #[serde(default, deserialize_with = "deserialize_optional_from_string")]
    pub include_facets: Option<bool>,
}

impl SearchQueryParams {
    #[must_use]
    pub fn page(&self) -> i64 {
        self.pagination.page()
    }

    #[must_use]
    pub fn per_page(&self) -> i64 {
        self.pagination.per_page()
    }

    #[must_use]
    pub fn offset(&self) -> i64 {
        self.pagination.offset()
    }

    #[must_use]
    pub fn should_include_summary(&self) -> bool {
        self.include_summary.unwrap_or(false)
    }

    #[must_use]
    pub fn should_include_facets(&self) -> bool {
        self.include_facets.unwrap_or(false)
    }
}

/// `#[serde(flatten)] search: SearchQueryParams` を持つドメイン別検索パラメータが
/// page/per_page/offset/should_include_summary/should_include_facets を委譲実装する重複を解消する共通トレイト。
/// 実装側は `search_params()` のみを定義すればよい。
pub trait SearchParamsAccessor {
    fn search_params(&self) -> &SearchQueryParams;

    fn page(&self) -> i64 {
        self.search_params().page()
    }

    fn per_page(&self) -> i64 {
        self.search_params().per_page()
    }

    fn offset(&self) -> i64 {
        self.search_params().offset()
    }

    fn should_include_summary(&self) -> bool {
        self.search_params().should_include_summary()
    }

    fn should_include_facets(&self) -> bool {
        self.search_params().should_include_facets()
    }
}

/// 検索候補の共通表現。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct FacetOption {
    pub value: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
}

/// 検索候補レスポンスの共通枠。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SearchFacets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub products: Option<Vec<FacetOption>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<FacetOption>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub securities: Option<Vec<FacetOption>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funds: Option<Vec<FacetOption>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub years: Option<Vec<FacetOption>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year_months: Option<Vec<FacetOption>>,
}

/// 検索・集計付きページネーションレスポンス。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PaginatedSearchResponse<T, Summary, Facets> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<Summary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Facets>,
}

/// 一括作成レスポンス（全ドメイン共通）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct BulkCreateResponse {
    pub inserted: usize,
    pub skipped: usize,
}

/// メッセージレスポンス（削除・ログアウト等）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct MessageResponse {
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::{
        FacetOption, PaginatedSearchResponse, PaginationParams, SearchFacets, SearchParamsAccessor,
        SearchQueryParams,
    };
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Row {
        id: i32,
    }

    #[derive(Debug, Default)]
    struct StubParams {
        search: SearchQueryParams,
    }

    impl SearchParamsAccessor for StubParams {
        fn search_params(&self) -> &SearchQueryParams {
            &self.search
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Summary {
        amount: i64,
    }

    #[test]
    fn pagination_params_defaults_and_limits() {
        let params = PaginationParams {
            page: Some(0),
            per_page: Some(5000),
        };

        assert_eq!(params.page(), 1);
        assert_eq!(params.per_page(), 1000);
        assert_eq!(params.offset(), 0);
    }

    #[test]
    fn pagination_params_parse_from_string_query() {
        // serde_urlencoded 相当（値がすべて文字列として届くケース）
        let params: PaginationParams =
            serde_json::from_str(r#"{"page": "3", "per_page": "25"}"#).expect("deserialize");
        assert_eq!(params.page(), 3);
        assert_eq!(params.per_page(), 25);

        let empty: PaginationParams =
            serde_json::from_str(r#"{"page": "", "per_page": null}"#).expect("deserialize");
        assert_eq!(empty.page, None);
        assert_eq!(empty.per_page, None);
    }

    #[test]
    fn search_query_params_delegate_pagination_and_flags() {
        let params = SearchQueryParams {
            pagination: PaginationParams {
                page: Some(3),
                per_page: Some(50),
            },
            q: Some("NTT".to_string()),
            date_from: Some("2026-01-01".to_string()),
            date_to: Some("2026-12-31".to_string()),
            year: Some(2026),
            year_month: Some("2026-06".to_string()),
            date: None,
            include_summary: Some(true),
            include_facets: None,
        };

        assert_eq!(params.page(), 3);
        assert_eq!(params.per_page(), 50);
        assert_eq!(params.offset(), 100);
        assert!(params.should_include_summary());
        assert!(!params.should_include_facets());
    }

    #[test]
    fn search_params_accessor_delegates_to_search_query_params() {
        for (include_summary, include_facets) in [
            (Some(true), Some(true)),
            (None, None),
            (Some(false), Some(false)),
        ] {
            let stub = StubParams {
                search: SearchQueryParams {
                    pagination: PaginationParams {
                        page: Some(4),
                        per_page: Some(25),
                    },
                    include_summary,
                    include_facets,
                    ..SearchQueryParams::default()
                },
            };

            assert_eq!(
                (
                    stub.page(),
                    stub.per_page(),
                    stub.offset(),
                    stub.should_include_summary(),
                    stub.should_include_facets(),
                ),
                (
                    stub.search.page(),
                    stub.search.per_page(),
                    stub.search.offset(),
                    stub.search.should_include_summary(),
                    stub.search.should_include_facets(),
                )
            );
        }
    }

    #[test]
    fn paginated_search_response_omits_optional_sections_when_absent() {
        let response: PaginatedSearchResponse<Row, Summary, SearchFacets> =
            PaginatedSearchResponse {
                data: vec![Row { id: 1 }],
                total: 1,
                page: 1,
                per_page: 200,
                summary: None,
                facets: None,
            };

        let json =
            serde_json::to_value(response).expect("PaginatedSearchResponse should serialize");

        assert_eq!(
            json,
            serde_json::json!({
                "data": [{"id": 1}],
                "total": 1,
                "page": 1,
                "per_page": 200
            })
        );
    }

    #[test]
    fn search_facets_support_common_option_groups() {
        let facets = SearchFacets {
            products: Some(vec![FacetOption {
                value: "domestic_stock".to_string(),
                label: "国内株式".to_string(),
                count: Some(2),
            }]),
            ..SearchFacets::default()
        };

        let json = serde_json::to_value(facets).expect("SearchFacets should serialize");

        assert_eq!(
            json,
            serde_json::json!({
                "products": [{
                    "value": "domestic_stock",
                    "label": "国内株式",
                    "count": 2
                }]
            })
        );
    }
}
