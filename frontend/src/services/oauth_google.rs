/*
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenUrl,
};
use thiserror::Error;
use url::Url;

use crate::env;

static OAUTH2_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
static OAUTH2_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

#[derive(Error, Debug)]
pub enum OAuthError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] oauth2::url::ParseError),
    #[error("OAuth client creation failed")]
    _ClientCreationFailed,
}

fn create_oauth_client() -> Result<BasicClient, OAuthError> {
    let client_id = ClientId::new(env::SHOKEN_WEBAPI_GOOGLE_OAUTH_CLIENT_ID.to_string());
    let client_secret_id =
        ClientSecret::new(env::SHOKEN_WEBAPI_GOOGLE_OAUTH_CLIENT_SECRET_ID.to_string());

    let redirect_url = RedirectUrl::new(env::SHOKEN_WEBAPI_REDIRECT_URL.to_string())?;
    let auth_url = AuthUrl::new(OAUTH2_AUTH_URL.to_string())?;
    let token_url = TokenUrl::new(OAUTH2_TOKEN_URL.to_string())?;

    Ok(
        BasicClient::new(client_id, Some(client_secret_id), auth_url, Some(token_url))
            .set_redirect_uri(redirect_url),
    )
}

pub async fn google_oauth() -> Result<(Url, CsrfToken), OAuthError> {
    let client = create_oauth_client()?;
    Ok(client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url())
}
 */
