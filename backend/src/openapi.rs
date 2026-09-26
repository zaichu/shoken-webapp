use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::OpenApi;

use crate::{
    errors::{ErrorDetails, ErrorResponse},
    handlers,
    models::{
        asset_balance::{AssetBalance, BulkCreateAssetBalanceRequest},
        common::{BulkCreateResponse, MessageResponse},
        csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadForm, CsvUploadResponse},
        dividend::Dividend,
        dividend_cache::{
            DividendPerShareBatchRequest, DividendPerShareBatchResponse, DividendPerShareItem,
        },
        domestic_stock::DomesticStock,
        mutualfund::Mutualfund,
        stock::Stock,
        user::UserResponse,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::v1::auth::get_session,
        handlers::v1::auth::delete_session,
        handlers::v1::auth::delete_account,
        handlers::v1::auth::create_account_deletion_confirmation,
        handlers::v1::auth::google_authorize,
        handlers::v1::auth::google_callback,
        handlers::v1::stocks::search,
        handlers::v1::dividends::list,
        handlers::v1::dividends::delete_all,
        handlers::v1::dividends::validate_import,
        handlers::v1::dividends::import,
        handlers::v1::dividends::estimate_per_share,
        handlers::v1::domestic_stocks::list_transactions,
        handlers::v1::domestic_stocks::delete_transactions,
        handlers::v1::domestic_stocks::validate_import,
        handlers::v1::domestic_stocks::import,
        handlers::v1::mutual_funds::list_transactions,
        handlers::v1::mutual_funds::delete_transactions,
        handlers::v1::mutual_funds::validate_import,
        handlers::v1::mutual_funds::import,
        handlers::v1::asset_balances::list,
        handlers::v1::asset_balances::replace,
        handlers::v1::asset_balances::delete_all,
        handlers::v1::asset_balances::validate_import,
        handlers::v1::asset_balances::import,
    ),
    components(
        schemas(
            Dividend,
            DomesticStock,
            Mutualfund,
            AssetBalance,
            BulkCreateAssetBalanceRequest,
            Stock,
            UserResponse,
            BulkCreateResponse,
            MessageResponse,
            CsvUploadForm,
            CsvUploadResponse,
            CsvPreviewResponse,
            CsvRowError,
            DividendPerShareBatchRequest,
            DividendPerShareItem,
            DividendPerShareBatchResponse,
            ErrorResponse,
            ErrorDetails,
        )
    ),
    modifiers(&SecurityAddon),
    info(
        title = "Shoken WebApp API",
        version = "0.1.0",
        description = "証券管理Webアプリ APIドキュメント",
    ),
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "cookieAuth",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("session_token"))),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_doc_registers_cookie_auth_scheme() {
        let doc = serde_json::to_value(ApiDoc::openapi()).expect("OpenAPI がシリアライズできる");
        let scheme = &doc["components"]["securitySchemes"]["cookieAuth"];
        assert_eq!(scheme["type"], "apiKey");
        assert_eq!(scheme["in"], "cookie");
        assert_eq!(scheme["name"], "session_token");
    }

    #[test]
    fn all_responses_have_description() {
        let doc = serde_json::to_value(ApiDoc::openapi()).expect("OpenAPI がシリアライズできる");
        let mut missing = Vec::new();
        for (path, item) in doc["paths"].as_object().expect("paths はオブジェクト") {
            for (method, operation) in item.as_object().expect("path item はオブジェクト") {
                for (status, response) in operation["responses"]
                    .as_object()
                    .expect("responses はオブジェクト")
                {
                    if response["description"].as_str().is_none_or(str::is_empty) {
                        missing.push(format!("{} {} {}", method.to_uppercase(), path, status));
                    }
                }
            }
        }
        assert!(
            missing.is_empty(),
            "description がないレスポンスがあります:\n{}",
            missing.join("\n")
        );
    }
}
