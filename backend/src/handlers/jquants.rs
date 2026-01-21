use crate::errors::ApiError;
use crate::models::jquants::{AuthResponse, IdTokenResponse, RefreshTokenRequest, StatementsQuery, StatementsResponse};
use crate::services::jquants::JQuantsService;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::Json,
};

pub async fn authenticate(State(state): State<AppState>) -> Result<Json<AuthResponse>, ApiError> {
    let response = JQuantsService::authenticate(&state.client, &*state.secrets).await?;
    Ok(Json(response))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<IdTokenResponse>, ApiError> {
    let response = JQuantsService::refresh_token(&state.client, payload).await?;
    Ok(Json(response))
}

pub async fn get_statements(
    State(state): State<AppState>,
    Query(params): Query<StatementsQuery>,
    headers: axum::http::HeaderMap,
) -> Result<Json<StatementsResponse>, ApiError> {
    tracing::info!("財務諸表取得パラメータ: {:?}", params);

    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::ApiError("Authorizationヘッダーが見つかりません".to_string()))?;

    let token = JQuantsService::extract_bearer_token(auth_header)?;
    let response = JQuantsService::get_statements(&state.client, params, token).await?;
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ここではハンドラーのユニットテストではなく、
    // サービス層のテストに依存するため、基本的な構造テストのみ実装
    #[test]
    fn test_module_compilation() {
        // モジュールが正常にコンパイルされることを確認
        assert!(true);
    }
}