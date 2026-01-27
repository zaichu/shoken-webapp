use crate::errors::ApiError;
use crate::models::jquants::{StatementsQuery, StatementsResponse};
use crate::services::jquants::JQuantsService;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::Json,
};

/// 財務諸表を取得（J-Quants API V2）
pub async fn get_statements(
    State(state): State<AppState>,
    Query(params): Query<StatementsQuery>,
) -> Result<Json<StatementsResponse>, ApiError> {
    tracing::info!("財務諸表取得パラメータ: {:?}", params);

    let api_key =
        state.secrets.jquants_api_key.as_ref().ok_or_else(|| {
            ApiError::ApiError("JQUANTS_API_KEY が設定されていません".to_string())
        })?;

    let response = JQuantsService::get_statements(&state.client, params, api_key).await?;
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compilation() {
        // モジュールが正常にコンパイルされることを確認
        assert!(true);
    }
}
