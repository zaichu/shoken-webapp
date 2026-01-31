use crate::errors::ApiError;
use crate::models::jquants::{FinSummaryQuery, FinSummaryResponse};
use crate::services::jquants::JQuantsService;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::Json,
};

/// 決算サマリーを取得（J-Quants API V2）
/// V2では fins/statements → fins/summary に変更
pub async fn get_fin_summary(
    State(state): State<AppState>,
    Query(params): Query<FinSummaryQuery>,
) -> Result<Json<FinSummaryResponse>, ApiError> {
    tracing::info!("決算サマリー取得パラメータ: {:?}", params);

    let api_key =
        state.secrets.jquants_api_key.as_ref().ok_or_else(|| {
            ApiError::ApiError("JQUANTS_API_KEY が設定されていません".to_string())
        })?;

    let response = JQuantsService::get_fin_summary(&state.client, params, api_key).await?;
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
