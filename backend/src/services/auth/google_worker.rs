//! Workers 側の Google OAuth 実装。
//! HTTP は `worker::Fetch`、ID トークン検証は tokeninfo エンドポイントに委譲する
//! （JWKS 署名検証と同じ iss/aud/exp/nonce 検査は validate_tokeninfo_claims で行う）。

use worker::{Fetch, Headers, Method, Request, RequestInit, wasm_bindgen::JsValue};

use crate::errors::{ApiError, ConfigError, UpstreamError};
use crate::models::user::GoogleUserInfo;

use super::GoogleAuthFlow;
use super::google_common::{
    GOOGLE_TOKEN_URL, GOOGLE_TOKENINFO_URL, build_authorize_url, new_oauth_random, new_pkce_pair,
    url_encode, validate_tokeninfo_claims,
};

/// Workers 側の OAuth クライアント設定（認可 URL・トークン交換に必要な値だけを持つ）
#[derive(Clone, Debug)]
pub struct GoogleOAuthClient {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    token_url: String,
    tokeninfo_url: String,
}

impl GoogleOAuthClient {
    /// dev/検証用に Google エンドポイントを差し替える。
    /// vars でのみ注入する想定で、未設定時は本番エンドポイントのままにする
    pub fn override_endpoints(&mut self, token_url: Option<String>, tokeninfo_url: Option<String>) {
        if let Some(url) = token_url {
            self.token_url = url;
        }
        if let Some(url) = tokeninfo_url {
            self.tokeninfo_url = url;
        }
    }
}

pub fn create_oauth_client(
    client_id: &str,
    client_secret: &str,
    backend_url: &str,
) -> Result<GoogleOAuthClient, ApiError> {
    let redirect_uri = format!("{backend_url}/api/v1/oauth/google/callback");
    // backend_url が URL として壊れていると認可 URL も壊れるため構築時に検証する
    url::Url::parse(&redirect_uri).map_err(ConfigError::UrlParse)?;
    Ok(GoogleOAuthClient {
        client_id: client_id.to_string(),
        client_secret: client_secret.to_string(),
        redirect_uri,
        token_url: GOOGLE_TOKEN_URL.to_string(),
        tokeninfo_url: GOOGLE_TOKENINFO_URL.to_string(),
    })
}

pub fn begin_auth(client: &GoogleOAuthClient) -> GoogleAuthFlow {
    let (pkce_verifier, pkce_challenge) = new_pkce_pair();
    let state = new_oauth_random();
    let nonce = new_oauth_random();
    GoogleAuthFlow {
        authorize_url: build_authorize_url(
            &client.client_id,
            &client.redirect_uri,
            &state,
            &nonce,
            &pkce_challenge,
        ),
        state,
        nonce,
        pkce_verifier,
    }
}

/// 認可コードを ID トークンに交換する（POST https://oauth2.googleapis.com/token）。
/// レスポンスの生値（access_token 等）をログ・エラーに載せないよう、
/// 失敗は一律 `UpstreamError::OAuth` に丸める
async fn exchange_code_for_id_token(
    client: &GoogleOAuthClient,
    code: &str,
    pkce_verifier: &str,
) -> Result<String, ApiError> {
    let body = [
        ("code", code),
        ("client_id", &client.client_id),
        ("client_secret", &client.client_secret),
        ("redirect_uri", &client.redirect_uri),
        ("grant_type", "authorization_code"),
        ("code_verifier", pkce_verifier),
    ]
    .iter()
    .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
    .collect::<Vec<_>>()
    .join("&");

    let headers = Headers::new();
    headers
        .set("Content-Type", "application/x-www-form-urlencoded")
        .map_err(|_| UpstreamError::OAuth("Googleトークン交換エラー"))?;
    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_headers(headers)
        .with_body(Some(JsValue::from_str(&body)));

    let request = Request::new_with_init(&client.token_url, &init)
        .map_err(|_| UpstreamError::OAuth("Googleトークン交換エラー"))?;
    // axum ハンドラは Future: Send が必要なため、JsFuture 系の待ち合わせは SendFuture で包む
    let fetch = Fetch::Request(request);
    let mut response = worker::send::SendFuture::new(fetch.send())
        .await
        .map_err(UpstreamError::Transport)?;
    if !(200..300).contains(&response.status_code()) {
        // エラー応答の本文にはトークン等が含まれ得るため読まない
        return Err(UpstreamError::OAuth("Googleトークン交換エラー").into());
    }
    let json: serde_json::Value = worker::send::SendFuture::new(response.json())
        .await
        .map_err(|_| UpstreamError::OAuth("Googleトークン交換エラー"))?;
    json.get("id_token")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| ApiError::OAuth("Google IDトークンがありません".into()))
}

/// tokeninfo エンドポイントで ID トークンを検証しクレームを返す
async fn fetch_tokeninfo_claims(
    client: &GoogleOAuthClient,
    id_token: &str,
) -> Result<serde_json::Value, ApiError> {
    let url = format!("{}?id_token={}", client.tokeninfo_url, url_encode(id_token));
    let parsed_url = url::Url::parse(&url).map_err(ConfigError::UrlParse)?;
    let fetch = Fetch::Url(parsed_url);
    let mut response = worker::send::SendFuture::new(fetch.send())
        .await
        .map_err(UpstreamError::Transport)?;
    if !(200..300).contains(&response.status_code()) {
        return Err(ApiError::OAuth("Google IDトークン検証エラー".into()));
    }
    worker::send::SendFuture::new(response.json())
        .await
        .map_err(|_| ApiError::OAuth("Google IDトークン検証エラー".into()))
}

/// Google OAuth コードを検証し、ユーザー情報を返す（DB への反映は呼び出し側が行う）
pub async fn verify_code_with_google(
    oauth_client: &GoogleOAuthClient,
    code: String,
    pkce_verifier: String,
    nonce: &str,
) -> Result<GoogleUserInfo, ApiError> {
    let id_token = exchange_code_for_id_token(oauth_client, &code, &pkce_verifier).await?;
    let claims = fetch_tokeninfo_claims(oauth_client, &id_token).await?;
    let now_unix = (worker::js_sys::Date::now() / 1000.0) as i64;
    validate_tokeninfo_claims(&claims, &oauth_client.client_id, nonce, now_unix)
}
