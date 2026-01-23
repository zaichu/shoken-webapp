use crate::errors::ApiError;
use crate::models::jquants::{
    AuthResponse, IdTokenResponse, RefreshTokenRequest, StatementsQuery, StatementsResponse,
};
use crate::state::Secrets;
use reqwest::Client;

pub struct JQuantsService;

impl JQuantsService {
    pub async fn authenticate(client: &Client, secrets: &Secrets) -> Result<AuthResponse, ApiError> {
        let url = "https://api.jquants.com/v1/token/auth_user";

        let mailaddress = secrets
            .jquants_email
            .clone()
            .ok_or_else(|| ApiError::ApiError("JQUANTS_EMAIL がシークレットに見つかりません".to_string()))?;
        let password = secrets
            .jquants_password
            .clone()
            .ok_or_else(|| ApiError::ApiError("JQUANTS_PASSWORD がシークレットに見つかりません".to_string()))?;

        let payload = serde_json::json!({
            "mailaddress": mailaddress,
            "password": password
        });

        let response = client
            .post(url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(format!("認証リクエストエラー: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApiError::ApiError(format!(
                "JQuants認証エラー: {}",
                error_text
            )));
        }

        let json_result = response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| ApiError::NetworkError(format!("レスポンス解析エラー: {}", e)))?;

        let refresh_token = json_result
            .get("refreshToken")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ApiError::ApiError("レスポンスにrefreshTokenが見つかりません".to_string()))?
            .to_string();

        Ok(AuthResponse { refresh_token })
    }

    pub async fn refresh_token(
        client: &Client,
        payload: RefreshTokenRequest,
    ) -> Result<IdTokenResponse, ApiError> {
        let url = format!(
            "https://api.jquants.com/v1/token/auth_refresh?refreshtoken={}",
            payload.refresh_token
        );

        let response = client
            .post(url)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(format!("トークンリフレッシュエラー: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApiError::ApiError(format!(
                "JQuantsトークンリフレッシュエラー: {}",
                error_text
            )));
        }

        let json_result = response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| ApiError::NetworkError(format!("レスポンス解析エラー: {}", e)))?;

        let id_token = json_result
            .get("idToken")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ApiError::ApiError("レスポンスにidTokenが見つかりません".to_string()))?
            .to_string();

        Ok(IdTokenResponse { id_token })
    }

    pub async fn get_statements(
        client: &Client,
        params: StatementsQuery,
        token: &str,
    ) -> Result<StatementsResponse, ApiError> {
        let mut url = format!(
            "https://api.jquants.com/v1/fins/statements?code={}",
            params.code
        );

        if let Some(from) = params.from {
            url.push_str(&format!("&from={}", from));
        }

        if let Some(to) = params.to {
            url.push_str(&format!("&to={}", to));
        }

        tracing::info!("JQuants API URL: {}", url);

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(format!("財務諸表取得エラー: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApiError::ApiError(format!(
                "JQuants財務諸表取得エラー: {}",
                error_text
            )));
        }

        let statements_response = response
            .json::<StatementsResponse>()
            .await
            .map_err(|e| ApiError::NetworkError(format!("財務諸表レスポンス解析エラー: {}", e)))?;

        Ok(statements_response)
    }

    pub fn extract_bearer_token(auth_header: &str) -> Result<&str, ApiError> {
        auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| ApiError::ApiError("無効なAuthorizationヘッダー形式です".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_token_valid() {
        let auth_header = "Bearer abc123token";
        let result = JQuantsService::extract_bearer_token(auth_header);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "abc123token");
    }

    #[test]
    fn test_extract_bearer_token_invalid() {
        let auth_header = "Basic abc123";
        let result = JQuantsService::extract_bearer_token(auth_header);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_bearer_token_empty() {
        let auth_header = "Bearer ";
        let result = JQuantsService::extract_bearer_token(auth_header);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_extract_bearer_token_no_space() {
        let auth_header = "Bearertoken";
        let result = JQuantsService::extract_bearer_token(auth_header);
        assert!(result.is_err());
    }
}
