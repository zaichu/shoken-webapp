use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenUrl,
};

use crate::{errors::ApiError, AppState};

static OAUTH2_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
static OAUTH2_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

fn oauth_client(state: AppState) -> Result<BasicClient, ApiError> {
    let client_id = ClientId::new(state.secrets.get("GOOGLE_OAUTH_CLIENT_ID").unwrap());
    // let client_secret_id =
    //     ClientSecret::new(state.secrets.get("GOOGLE_OAUTH_CLIENT_SECRET_ID").unwrap());
    let redirect_url = RedirectUrl::new(
        state
            .secrets
            .get("REDIRECT_URL")
            .unwrap_or("http://localhost:8080/shoken-webapp-wasm/".to_string()),
    )?;

    // let auth_url = AuthUrl::new(OAUTH2_AUTH_URL.to_string())?;
    // let token_url = TokenUrl::new(OAUTH2_TOKEN_URL.to_string())?;

    Ok(BasicClient::new(client_id).set_redirect_uri(redirect_url))
}

pub async fn google_oauth(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    // let client = oauth_client(state)?;
    // let (auth_url, _csrf_token) = client
    // .authorize_url(CsrfToken::new_random)
    // .add_scope(Scope::new("email".to_string()))
    // .add_scope(Scope::new("profile".to_string()))
    // .url();

    // Ok((StatusCode::OK, Json(auth_url.to_string())))
    Ok((StatusCode::OK, Json("".to_string())))
}
