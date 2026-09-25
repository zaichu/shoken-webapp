use crate::{
    errors::{ApiError, ErrorResponse},
    models::{common::validate_length_field, stock::Stock},
    services::stock as stock_service,
    state::AppState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use utoipa::ToSchema;
use validator::{Validate, ValidationErrors};

/// 銘柄検索クエリパラメータ（v1）
#[derive(Debug, Deserialize, ToSchema)]
pub struct StockSearchQuery {
    /// 銘柄コードまたは銘柄名（1〜100文字）
    pub query: String,
}

impl Validate for StockSearchQuery {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        // 上限100は `Stock.name` の上限に合わせる（それより長い文字列は一致し得ない）
        validate_length_field(&mut errors, "query", &self.query, Some(1), Some(100));
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 銘柄情報を検索（v1: クエリパラメータ）
#[utoipa::path(
    get,
    path = "/api/v1/stocks",
    operation_id = "v1_stock_search",
    params(
        ("query" = String, Query, description = "銘柄コードまたは銘柄名")
    ),
    responses(
        (status = 200, body = Stock),
        (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 429, body = ErrorResponse),
    )
)]
pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<StockSearchQuery>,
) -> Result<impl IntoResponse, ApiError> {
    params.validate().map_err(|e| {
        let msg = format!("{e}").replace('\n', ", ");
        ApiError::ValidationError(msg)
    })?;
    let stock = stock_service::search(&state.pool, &params.query).await?;
    Ok((StatusCode::OK, Json(stock)))
}

#[cfg(test)]
mod tests;
