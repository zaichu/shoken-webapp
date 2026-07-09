use serde::{Deserialize, Deserializer, Serialize};
use sqlx::FromRow;
use std::{borrow::Cow, fmt::Display, str::FromStr};
use utoipa::ToSchema;
use validator::{ValidateLength, ValidationError, ValidationErrors};

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

/// `validator` derive の length rule 相当を手実装するヘルパー。
/// `String` / `Option<String>` / `Vec<T>` など `ValidateLength` 実装型に共通で使う。
pub(crate) fn validate_length_field<T>(
    errors: &mut ValidationErrors,
    field: &'static str,
    value: T,
    min: Option<u64>,
    max: Option<u64>,
) where
    T: ValidateLength<u64>,
{
    if !value.validate_length(min, max, None) {
        let mut error = ValidationError::new("length");
        if let Some(min) = min {
            error.add_param(Cow::from("min"), &min);
        }
        if let Some(max) = max {
            error.add_param(Cow::from("max"), &max);
        }
        errors.add(field, error);
    }
}

/// ページネーションクエリパラメータ（全ドメイン共通）
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct PaginationParams {
    #[serde(default, deserialize_with = "deserialize_optional_from_string")]
    pub page: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_from_string")]
    pub per_page: Option<i64>,
}

impl PaginationParams {
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }
    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(200).clamp(1, 1000)
    }
    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.per_page()
    }
}

/// 検索・集計付き一覧の共通クエリパラメータ。
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
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
    pub fn page(&self) -> i64 {
        self.pagination.page()
    }

    pub fn per_page(&self) -> i64 {
        self.pagination.per_page()
    }

    pub fn offset(&self) -> i64 {
        self.pagination.offset()
    }

    pub fn should_include_summary(&self) -> bool {
        self.include_summary.unwrap_or(false)
    }

    pub fn should_include_facets(&self) -> bool {
        self.include_facets.unwrap_or(false)
    }
}

/// 検索候補の共通表現。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct FacetOption {
    pub value: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
}

/// 検索候補レスポンスの共通枠。
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedSearchResponse<
    T: ToSchema + 'static,
    Summary: ToSchema + 'static,
    Facets: ToSchema + 'static,
> {
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
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BulkCreateResponse {
    pub inserted: usize,
    pub skipped: usize,
}

/// メッセージレスポンス（削除・ログアウト等）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::{
        BulkCreateResponse, FacetOption, MessageResponse, PaginatedSearchResponse,
        PaginationParams, SearchFacets, SearchQueryParams,
    };
    use axum::{extract::Query, http::Uri};
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
    struct Row {
        id: i32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
    struct Summary {
        amount: i64,
    }

    #[test]
    fn test_serde_round_trips() {
        let r = BulkCreateResponse {
            inserted: 3,
            skipped: 2,
        };
        let json = serde_json::to_string(&r).expect("BulkCreateResponse should serialize");
        let d: BulkCreateResponse =
            serde_json::from_str(&json).expect("BulkCreateResponse should deserialize");
        assert_eq!((d.inserted, d.skipped), (r.inserted, r.skipped));
        let r = MessageResponse {
            message: "ok".to_string(),
        };
        let json = serde_json::to_string(&r).expect("MessageResponse should serialize");
        let d: MessageResponse =
            serde_json::from_str(&json).expect("MessageResponse should deserialize");
        assert_eq!(d.message, r.message);
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
    fn search_query_params_deserialize_from_url_query_strings() {
        let uri: Uri = "/api/v1/dividends?per_page=1000&page=2&year=2026&include_summary=true&include_facets=false"
            .parse()
            .expect("valid URI");

        let Query(params) =
            Query::<SearchQueryParams>::try_from_uri(&uri).expect("query params should parse");

        assert_eq!(params.page(), 2);
        assert_eq!(params.per_page(), 1000);
        assert_eq!(params.year, Some(2026));
        assert!(params.should_include_summary());
        assert!(!params.should_include_facets());
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
