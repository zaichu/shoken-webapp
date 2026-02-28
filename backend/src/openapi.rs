use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::OpenApi;

use crate::{
    errors::{ErrorDetails, ErrorResponse},
    handlers,
    models::{
        asset_balance::{AssetBalance, BulkCreateAssetBalanceRequest, CreateAssetBalanceRequest},
        common::{BulkCreateResponse, MessageResponse},
        dividend::{BulkCreateDividendRequest, CreateDividendRequest, Dividend},
        dividend_cache::{
            DividendPerShareBatchRequest, DividendPerShareBatchResponse, DividendPerShareItem,
        },
        domestic_stock::{
            BulkCreateDomesticStockRequest, CreateDomesticStockRequest, DomesticStock,
        },
        jquants::{FinSummaryData, FinSummaryResponse},
        mutualfund::{BulkCreateMutualfundRequest, CreateMutualfundRequest, Mutualfund},
        stock::Stock,
        user::UserResponse,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::dividend::list,
        handlers::dividend::bulk_create,
        handlers::dividend::delete_all,
        handlers::domestic_stock::list,
        handlers::domestic_stock::bulk_create,
        handlers::domestic_stock::delete_all,
        handlers::mutualfund::list,
        handlers::mutualfund::bulk_create,
        handlers::mutualfund::delete_all,
        handlers::asset_balance::list,
        handlers::asset_balance::bulk_create,
        handlers::asset_balance::delete_all,
        handlers::stock::select_stock_info,
        handlers::stock::add_stock_info,
        handlers::auth::get_current_user,
        handlers::auth::logout,
        handlers::auth::delete_account,
        handlers::jquants::get_fin_summary,
        handlers::dividend_per_share::batch,
    ),
    components(
        schemas(
            Dividend,
            CreateDividendRequest,
            BulkCreateDividendRequest,
            DomesticStock,
            CreateDomesticStockRequest,
            BulkCreateDomesticStockRequest,
            Mutualfund,
            CreateMutualfundRequest,
            BulkCreateMutualfundRequest,
            AssetBalance,
            CreateAssetBalanceRequest,
            BulkCreateAssetBalanceRequest,
            Stock,
            UserResponse,
            BulkCreateResponse,
            MessageResponse,
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
