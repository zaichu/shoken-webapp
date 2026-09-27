use crate::errors::ApiError;
use crate::models::common::{MessageResponse, PaginatedSearchResponse, SearchFacets};
use crate::services::domain::search::{search, Search};
use axum::{http::StatusCode, response::Json};
use shared::value::UserId;
use sqlx::PgPool;

pub fn ok_message(message: &str) -> (StatusCode, Json<MessageResponse>) {
    (
        StatusCode::OK,
        Json(MessageResponse {
            message: message.to_string(),
        }),
    )
}

/// 検索の定型処理（汎用 search + 200 OK）
///
/// 各ドメインの一覧ハンドラーから
/// `handle_search::<DividendDomain>(&state.pool, auth_user.id(), params).await` のように呼ぶ。
pub async fn handle_search<D: Search>(
    pool: &PgPool,
    user_id: UserId,
    params: D::Params,
) -> Result<
    (
        StatusCode,
        Json<PaginatedSearchResponse<D::Data, D::Summary, SearchFacets>>,
    ),
    ApiError,
> {
    let result = search::<D>(pool, user_id, params).await?;
    Ok((StatusCode::OK, Json(result)))
}
