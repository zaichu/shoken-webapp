use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::OpenApi;

use crate::{
    errors::{ErrorDetails, ErrorResponse},
    handlers,
    models::{
        asset_balance::{AssetBalance, BulkCreateAssetBalanceRequest, CreateAssetBalanceRequest},
        common::{BulkCreateResponse, MessageResponse},
        csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadForm, CsvUploadResponse},
        dividend::Dividend,
        dividend_cache::{
            DividendPerShareBatchRequest, DividendPerShareBatchResponse, DividendPerShareItem,
        },
        domestic_stock::DomesticStock,
        jquants::{FinSummaryData, FinSummaryResponse},
        mutualfund::Mutualfund,
        stock::Stock,
        user::UserResponse,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::dividend::list,
        handlers::dividend::preview_csv,
        handlers::dividend::upload_csv,
        handlers::dividend::delete_all,
        handlers::domestic_stock::list,
        handlers::domestic_stock::preview_csv,
        handlers::domestic_stock::upload_csv,
        handlers::domestic_stock::delete_all,
        handlers::mutualfund::list,
        handlers::mutualfund::preview_csv,
        handlers::mutualfund::upload_csv,
        handlers::mutualfund::delete_all,
        handlers::asset_balance::list,
        handlers::asset_balance::bulk_create,
        handlers::asset_balance::preview_csv,
        handlers::asset_balance::upload_csv,
        handlers::asset_balance::delete_all,
        handlers::stock::search_stock,
        handlers::stock::create_stock,
        handlers::auth::get_current_user,
        handlers::auth::logout,
        handlers::auth::delete_account,
        handlers::jquants::get_fin_summary,
        handlers::dividend_per_share::batch,
    ),
    components(
        schemas(
            Dividend,
            DomesticStock,
            Mutualfund,
            AssetBalance,
            CreateAssetBalanceRequest,
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
            FinSummaryResponse,
            FinSummaryData,
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
