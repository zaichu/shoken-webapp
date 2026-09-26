use std::borrow::Cow;
use validator::{ValidateLength, ValidationError, ValidationErrors};

// PaginationParams は bin クレート（main.rs）から直接参照されないが、公開面として揃える
#[allow(unused_imports)]
pub use shared::common::{
    BulkCreateResponse, FacetOption, MessageResponse, PaginatedSearchResponse, PaginationParams,
    SearchFacets, SearchParamsAccessor, SearchQueryParams,
};

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

#[cfg(test)]
mod tests {
    use super::SearchQueryParams;
    use axum::{extract::Query, http::Uri};

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
}
