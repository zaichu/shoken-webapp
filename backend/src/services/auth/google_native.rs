use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenUrl,
};
use openidconnect::{
    IssuerUrl, JsonWebKeySetUrl, Nonce, TokenResponse,
    core::{
        CoreIdToken, CoreIdTokenVerifier, CoreJsonWebKeySet, CoreJwsSigningAlgorithm,
        CoreTokenResponse,
    },
};
use std::{
    sync::OnceLock,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

use crate::errors::{ApiError, ConfigError, UpstreamError};
use crate::models::user::GoogleUserInfo;

use super::GoogleAuthFlow;
use super::google_common::{GOOGLE_AUTH_URL, GOOGLE_ISSUER, GOOGLE_TOKEN_URL};

const GOOGLE_JWKS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";

pub type GoogleOAuthClient = oauth2::Client<
    oauth2::basic::BasicErrorResponse,
    CoreTokenResponse,
    oauth2::basic::BasicTokenIntrospectionResponse,
    oauth2::StandardRevocableToken,
    oauth2::basic::BasicRevocationErrorResponse,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

pub fn create_oauth_client(
    client_id: &str,
    client_secret: &str,
    backend_url: &str,
) -> Result<GoogleOAuthClient, ApiError> {
    let redirect_url = format!("{backend_url}/api/v1/oauth/google/callback");

    let client = oauth2::Client::new(ClientId::new(client_id.to_string()))
        .set_client_secret(ClientSecret::new(client_secret.to_string()))
        .set_auth_uri(AuthUrl::new(GOOGLE_AUTH_URL.to_string()).map_err(ConfigError::UrlParse)?)
        .set_token_uri(TokenUrl::new(GOOGLE_TOKEN_URL.to_string()).map_err(ConfigError::UrlParse)?)
        .set_redirect_uri(RedirectUrl::new(redirect_url).map_err(ConfigError::UrlParse)?);

    Ok(client)
}

/// 認可リダイレクト一式を生成する。state/nonce/PKCE は oauth2 クレートの生成器を使う
pub fn begin_auth(client: &GoogleOAuthClient) -> GoogleAuthFlow {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let nonce = Nonce::new_random();
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .add_extra_param("nonce", nonce.secret())
        .url();
    GoogleAuthFlow {
        authorize_url: auth_url.to_string(),
        state: csrf_token.secret().to_string(),
        nonce: nonce.secret().to_string(),
        pkce_verifier: pkce_verifier.secret().to_string(),
    }
}

/// トークン交換用の共有HTTPクライアント（起動時に1つ構築し、TCP/TLSコネクションを使い回す）。
///
/// `oauth2::reqwest` は oauth2 クレートが依存する reqwest 0.12 の再エクスポートであり、
/// アプリ側の `reqwest` 0.13（`AppState.client`）とは型が異なるため、専用の static で保持する。
static OAUTH_HTTP_CLIENT: OnceLock<oauth2::reqwest::Client> = OnceLock::new();

/// OAuth用HTTPクライアントを構築する（10秒タイムアウト・リダイレクト3回制限）。
pub fn build_oauth_http_client() -> Result<oauth2::reqwest::Client, ApiError> {
    oauth2::reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(oauth2::reqwest::redirect::Policy::limited(3))
        .build()
        .map_err(|e| ApiError::OAuth(format!("OAuth HTTPクライアント構築エラー: {e}")))
}

/// 起動時に共有OAuthクライアントを初期化する（`main.rs` の起動処理から呼ぶ）。
pub fn init_oauth_http_client() -> Result<(), ApiError> {
    if OAUTH_HTTP_CLIENT.get().is_none() {
        let client = build_oauth_http_client()?;
        let startup_client = client.clone();
        // 並行して初期化が進んでいた場合は先勝ちした側を共有する。
        if OAUTH_HTTP_CLIENT.set(client).is_err() {
            tracing::debug!("OAuth HTTPクライアントは既に初期化されています");
        } else {
            if let Ok(runtime) = tokio::runtime::Handle::try_current() {
                // 起動をブロックせず先読みする。失敗時は次のログイン時に再取得する。
                runtime.spawn(async move {
                    if shared_google_jwks(&startup_client).await.is_err() {
                        tracing::warn!("Google JWKSの先読みに失敗しました");
                    }
                });
            }
        }
    }
    Ok(())
}

/// 共有OAuthクライアントを取得する。起動時初期化漏れ時はその場で構築する。
fn shared_oauth_http_client() -> Result<&'static oauth2::reqwest::Client, ApiError> {
    if let Some(client) = OAUTH_HTTP_CLIENT.get() {
        return Ok(client);
    }
    init_oauth_http_client()?;
    OAUTH_HTTP_CLIENT
        .get()
        .ok_or_else(|| ApiError::OAuth("OAuth HTTPクライアントが初期化されていません".to_string()))
}

// 起動時に先読みし、取得成功後5分間は共有する。期限切れ後の最初の要求で更新する。
// 未知の鍵や署名エラーによる強制再取得は行わず、外部リクエストの増幅を避ける。
const JWKS_CACHE_TTL: Duration = Duration::from_secs(300);
static GOOGLE_JWKS_CACHE: GoogleJwksCache = GoogleJwksCache::new();

struct GoogleJwksCache {
    entry: Mutex<Option<(CoreJsonWebKeySet, Instant)>>,
}

impl GoogleJwksCache {
    const fn new() -> Self {
        Self {
            entry: Mutex::const_new(None),
        }
    }

    async fn get(
        &self,
        client: &oauth2::reqwest::Client,
        url: &JsonWebKeySetUrl,
    ) -> Result<CoreJsonWebKeySet, ApiError> {
        // 取得中も排他し、同時ログインによる重複取得を防ぐ。
        let mut entry = self.entry.lock().await;
        if let Some((keys, fetched_at)) = entry.as_ref()
            && fetched_at.elapsed() < JWKS_CACHE_TTL
        {
            return Ok(keys.clone());
        }
        let keys = CoreJsonWebKeySet::fetch_async(url, client)
            .await
            .map_err(|_| UpstreamError::OAuth("Google JWKS取得エラー"))?;
        // 更新失敗時は期限切れの鍵を使用しない。
        *entry = Some((keys.clone(), Instant::now()));
        Ok(keys)
    }
}

async fn shared_google_jwks(
    client: &oauth2::reqwest::Client,
) -> Result<CoreJsonWebKeySet, ApiError> {
    let url = JsonWebKeySetUrl::new(GOOGLE_JWKS_URL.to_string()).map_err(ConfigError::UrlParse)?;
    GOOGLE_JWKS_CACHE.get(client, &url).await
}

fn verify_google_id_token(
    token: &CoreIdToken,
    client_id: &ClientId,
    keys: CoreJsonWebKeySet,
    nonce: &Nonce,
) -> Result<GoogleUserInfo, ApiError> {
    // Googleの非対称署名のみ許可。クライアントシークレットを署名鍵として扱わない。
    let verifier = CoreIdTokenVerifier::new_public_client(
        client_id.clone(),
        IssuerUrl::new(GOOGLE_ISSUER.to_string()).map_err(ConfigError::UrlParse)?,
        keys,
    )
    .set_allowed_algs(vec![CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256]);
    let claims = token
        .claims(&verifier, nonce)
        .map_err(|_| ApiError::OAuth("Google IDトークン検証エラー".into()))?;

    // 署名・iss・aud・expの検証を通ったclaimだけをDB更新に使用する。
    Ok(GoogleUserInfo {
        sub: claims.subject().as_str().to_string(),
        email: claims
            .email()
            .ok_or_else(|| ApiError::OAuth("Google IDトークンにemailがありません".into()))?
            .as_str()
            .to_string(),
        name: claims
            .name()
            .and_then(|name| name.get(None))
            .map(|name| name.as_str().to_string()),
        picture: claims
            .picture()
            .and_then(|picture| picture.get(None))
            .map(|picture| picture.as_str().to_string()),
    })
}

async fn exchange_google_code(
    oauth_client: &GoogleOAuthClient,
    code: String,
    pkce_verifier: String,
) -> Result<CoreTokenResponse, ApiError> {
    oauth_client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier))
        .request_async(shared_oauth_http_client()?)
        .await
        // パースエラーにはレスポンス本文が含まれ得るため、詳細をログや応答に出さない。
        .map_err(|_| UpstreamError::OAuth("Googleトークン交換エラー").into())
}

/// Google OAuth コードを検証し、ユーザー情報を返す（DB への反映は呼び出し側が行う）
pub async fn verify_code_with_google(
    oauth_client: &GoogleOAuthClient,
    code: String,
    pkce_verifier: String,
    nonce: &str,
) -> Result<GoogleUserInfo, ApiError> {
    let token_result = exchange_google_code(oauth_client, code, pkce_verifier).await?;

    let id_token = token_result
        .id_token()
        .ok_or_else(|| ApiError::OAuth("Google IDトークンがありません".into()))?;
    let keys = shared_google_jwks(shared_oauth_http_client()?).await?;
    verify_google_id_token(
        id_token,
        oauth_client.client_id(),
        keys,
        &Nonce::new(nonce.to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use openidconnect::{
        JsonWebKeyId, PrivateSigningKey,
        core::{CoreIdTokenClaims, CoreJwsSigningAlgorithm, CoreRsaPrivateSigningKey},
    };

    // このテスト専用に生成した公開済みの鍵。本番の認証には使用しない。
    const TEST_SIGNING_KEY: &str = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEAremwhaI5aHwKo/1GEBrS4qeZx+V1xkXI3X9uU1LcU/rd9Ng0
rybnrTH6VSoCBTbE4eGzH3nQoKYB/mgQwS1vd++zXWja/VuYGFut7hzVOgWBqmGo
F5UL7wcEGl5veYCV9p1lmjQXuBDsX/rZD8ACVG9DMx26YmrVC8U9vmZyQagNG2vm
z5YB3zctZ9hP/0KAy0ixIdcB05u8uErbON/ODwHJp4K4v5wMaD/O5O1KBHrHIpi+
A0F+dj070aWovQ0erUEM8dX9FCAr7/4AzAWpBhPylRVu4HOUridjDW4W5cJlN/Au
GKW8auBqF2wBdngPBCJIr7KwOrHp4VyJGQzjDQIDAQABAoIBAAteAbdNjSo/jh2x
FkoP/Sy3ZVCXHOv8VhuA+SyuqjWknHe+eGMVmRsAiasIKq2jxAA61Y+Yt+kByl7F
XbyV3a1Qy4ziQT4O8is7z57t34bO77siB7yfqPIGQV+Zc8VugPhnluNNxScdl0Jt
J11LxygBrCDEY3eMIaftTBKmQfcsizXgHuiKX5OCn2Hjlx6dZzl9od5oG7fhM/2O
qalx793ItsSfLKp/EftnzdoVIqvX18uLmEBYcrVOGv4eb2sV7IeCMwNll62Tc6rT
Diz/rRoF+p4+AVebzm6sNXMDnJ0mXFWTt7xC6a/oFMDOuted1Mga5CuLigo/5Ha/
cczxM7kCgYEA1UEJ+9OlUFXjrcqgVuRyOWmlEnarHG1mfbEDM5u3Km+kVErYANa2
L+d6+ccFbvHwF6PPqvjjUA2VK6sazoDJm8xmYyovOlsKkLW5Y6LNdvWaLQBSQfil
+x2CPXDeOT19NcjibWnrN7XyRrPo5j241/dob43hZJR9rZ5Ikc81s/8CgYEA0MXj
UTwSob36LvOZ+LvhBaL+CxmXMuBhPUkltVQ5afTEbsGMyK/aJKTm/U7gR9SASNXc
SEfhvWyNXf8De0PbZ01EprhjPHMkZqrp4coWAVb6oUGJCgDPmv6eQUS4DHdBGrxV
/LvEFWEh+RJD/mtQV9nydoVylZOWAzXEsA1M+PMCgYAhaIYC4J5GXp5DjLnfwvwu
CGHm6ZZW5sCmskN5I0znpgPNfMgoIXr7OD1owggU4Gwnl+8hrsoVsXsME0sozL5I
3RWxNVuevcKC9yUq+cdMep+Dq0g3s5d1JqNPss3tk7d45JasY2qJGMTy1J6I62R4
2PaQe16zHhwuRdzCkv6rywKBgHwTL37m6dfQVTC0O+y0lA5KiRrFsbNd4MyQfWWf
0aNkAZ4lT2sx/75JdrJSvz5RT5B58TnP5pwyOG4FkecfM/TX2hYPfYK+l4KgzvEO
rjdLnxZZIX2db8SY0CrQEWXvNfUSuzPBz8449PzW2ywIUS507AF+W9QDa2MrAGL0
9Kr7AoGBAMR/3jv99iniFERcsZjqs50jxQsXOC7o7F2fmtJMYFRKg53/WrDUXz7r
Dyhh5nzzuOz2uZ/ZrViITixw5Pgk5EhS8RF2cTgd4kvR/zHa9j0s2ndhvJfhYGqi
jFdlNnWmQn907d0UZvjZ6tAIt52ONB+xgyv/FkqX/KzCKxPtxnFW
-----END RSA PRIVATE KEY-----
"#;

    fn signed_token(claims: serde_json::Value) -> (CoreIdToken, CoreJsonWebKeySet) {
        let key = CoreRsaPrivateSigningKey::from_pem(
            TEST_SIGNING_KEY,
            Some(JsonWebKeyId::new("test-key".into())),
        )
        .unwrap();
        let claims: CoreIdTokenClaims = serde_json::from_value(claims).unwrap();
        let token = CoreIdToken::new(
            claims,
            &key,
            CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256,
            None,
            None,
        )
        .unwrap();
        (
            token,
            CoreJsonWebKeySet::new(vec![key.as_verification_key()]),
        )
    }

    fn valid_claims() -> serde_json::Value {
        serde_json::json!({
            "iss": "https://accounts.google.com", "aud": "client-id", "sub": "test-subject",
            "exp": chrono::Utc::now().timestamp() + 3600, "iat": chrono::Utc::now().timestamp(),
            "email": "oidc@example.com", "name": "テスト利用者", "nonce": "test-nonce",
            "picture": "https://example.com/avatar.png"
        })
    }

    #[test]
    fn test_oidc_verified_profile() {
        let (token, keys) = signed_token(valid_claims());
        let user = verify_google_id_token(
            &token,
            &ClientId::new("client-id".into()),
            keys,
            &Nonce::new("test-nonce".into()),
        )
        .unwrap();
        assert_eq!(user.sub, "test-subject");
        assert_eq!(user.email, "oidc@example.com");
        assert_eq!(user.name.as_deref(), Some("テスト利用者"));
        assert_eq!(
            user.picture.as_deref(),
            Some("https://example.com/avatar.png")
        );
    }

    #[test]
    fn test_oidc_rejects_invalid_claims_and_signature() {
        for (field, value) in [
            ("iss", serde_json::json!("https://attacker.example.com")),
            ("aud", serde_json::json!("other-client")),
            (
                "exp",
                serde_json::json!(chrono::Utc::now().timestamp() - 60),
            ),
            ("nonce", serde_json::json!("unsolicited-nonce")),
            ("email", serde_json::Value::Null),
        ] {
            let mut claims = valid_claims();
            claims[field] = value;
            let (token, keys) = signed_token(claims);
            assert!(
                verify_google_id_token(
                    &token,
                    &ClientId::new("client-id".into()),
                    keys,
                    &Nonce::new("test-nonce".into())
                )
                .is_err(),
                "{field}"
            );
        }
        let mut claims = valid_claims();
        claims.as_object_mut().unwrap().remove("nonce");
        let (token, keys) = signed_token(claims);
        assert!(
            verify_google_id_token(
                &token,
                &ClientId::new("client-id".into()),
                keys,
                &Nonce::new("test-nonce".into())
            )
            .is_err(),
            "nonce 欠落"
        );
        let (token, keys) = signed_token(valid_claims());
        let serialized = serde_json::to_value(&token).unwrap();
        let mut jwt = serialized.as_str().unwrap().as_bytes().to_vec();
        let start = jwt.iter().rposition(|c| *c == b'.').unwrap() + 1;
        jwt[start] = if jwt[start] == b'A' { b'B' } else { b'A' };
        let tampered: CoreIdToken =
            serde_json::from_value(serde_json::json!(String::from_utf8(jwt).unwrap())).unwrap();
        assert!(
            verify_google_id_token(
                &tampered,
                &ClientId::new("client-id".into()),
                keys,
                &Nonce::new("test-nonce".into())
            )
            .is_err()
        );
        assert!(
            verify_google_id_token(
                &token,
                &ClientId::new("client-id".into()),
                CoreJsonWebKeySet::new(vec![]),
                &Nonce::new("test-nonce".into())
            )
            .is_err()
        );
    }

    #[tokio::test]
    async fn test_jwks_cache_shares_fetch_and_refreshes_after_expiry() {
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{method, path},
        };
        let server = MockServer::start().await;
        let (_, keys) = signed_token(valid_claims());
        Mock::given(method("GET"))
            .and(path("/keys"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&keys))
            .expect(1)
            .mount(&server)
            .await;
        let cache = GoogleJwksCache::new();
        let url = openidconnect::JsonWebKeySetUrl::new(format!("{}/keys", server.uri())).unwrap();
        let client = build_oauth_http_client().unwrap();
        let (first, second) = tokio::join!(cache.get(&client, &url), cache.get(&client, &url));
        assert!(first.is_ok());
        assert!(second.is_ok());
        server.verify().await;
        server.reset().await;
        cache.entry.lock().await.as_mut().unwrap().1 = std::time::Instant::now() - JWKS_CACHE_TTL;
        Mock::given(method("GET"))
            .and(path("/keys"))
            .respond_with(ResponseTemplate::new(503))
            .expect(1)
            .mount(&server)
            .await;
        assert!(
            cache.get(&client, &url).await.is_err(),
            "期限切れの鍵にフォールバックしない"
        );
        server.verify().await;
        server.reset().await;
        Mock::given(method("GET"))
            .and(path("/keys"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&keys))
            .expect(1)
            .mount(&server)
            .await;
        assert!(cache.get(&client, &url).await.is_ok());
        assert!(cache.get(&client, &url).await.is_ok());
    }

    #[tokio::test]
    async fn test_oidc_token_exchange_preserves_id_token() {
        use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};
        let server = MockServer::start().await;
        let (token, keys) = signed_token(valid_claims());
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "test-access-token", "token_type": "Bearer",
                "expires_in": 3600, "id_token": token
            })))
            .expect(1)
            .mount(&server)
            .await;
        let client = create_oauth_client("client-id", "client-secret", "http://localhost:3001")
            .unwrap()
            .set_token_uri(TokenUrl::new(server.uri()).unwrap());
        let response = client
            .exchange_code(AuthorizationCode::new("test-code".into()))
            .request_async(&build_oauth_http_client().unwrap())
            .await
            .unwrap();
        let user = verify_google_id_token(
            response.id_token().unwrap(),
            client.client_id(),
            keys,
            &Nonce::new("test-nonce".into()),
        )
        .unwrap();
        assert_eq!(user.sub, "test-subject");
    }

    #[test]
    fn test_oidc_optional_profile_claims() {
        let mut claims = valid_claims();
        claims.as_object_mut().unwrap().remove("name");
        claims.as_object_mut().unwrap().remove("picture");
        let (token, keys) = signed_token(claims);
        let user = verify_google_id_token(
            &token,
            &ClientId::new("client-id".into()),
            keys,
            &Nonce::new("test-nonce".into()),
        )
        .unwrap();
        assert!(user.name.is_none());
        assert!(user.picture.is_none());
    }

    #[test]
    fn test_oauth_http_client_is_shared() {
        init_oauth_http_client().expect("共有OAuthクライアントの初期化");
        let first = shared_oauth_http_client().expect("共有クライアント取得") as *const _;
        init_oauth_http_client().expect("再初期化は冪等");
        let second = shared_oauth_http_client().expect("共有クライアント再取得") as *const _;
        assert_eq!(
            first, second,
            "OAuth用HTTPクライアントは同一インスタンスを共有すること"
        );
    }

    #[tokio::test]
    async fn test_oauth_redirect_uri_is_v1_path() {
        let client = create_oauth_client(
            "client-id",
            "client-secret",
            "https://shoken-backend.fly.dev",
        )
        .unwrap();
        let (auth_url, _csrf) = client.authorize_url(oauth2::CsrfToken::new_random).url();
        let url_str = auth_url.to_string();
        assert!(
            url_str.contains("redirect_uri=https%3A%2F%2Fshoken-backend.fly.dev%2Fapi%2Fv1%2Foauth%2Fgoogle%2Fcallback"),
            "redirect_uri が /api/v1/oauth/google/callback でない: {url_str}"
        );
    }

    #[tokio::test]
    async fn test_oauth_login_to_token_exchange_round_trip() {
        use axum::{extract::State, response::IntoResponse};
        use axum_extra::extract::CookieJar;
        use oauth2::PkceCodeChallenge;
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{body_string_contains, method},
        };

        let secrets = std::sync::Arc::new(crate::state::Secrets {
            database_url: String::new(),
            jquants_api_key: None,
            google_client_id: Some("client-id".to_string()),
            google_client_secret: Some("client-secret".to_string()),
            frontend_url: "http://localhost:8080".to_string(),
        });
        let app_state = crate::state::AppState {
            pool: crate::db::connect_pool_lazy("postgresql://user:password@localhost/test_db", 1)
                .expect("pool"),
            secrets,
            client: reqwest::Client::new(),
            dividend_cache: crate::state::DividendCacheState::default(),
            config: std::sync::Arc::new(crate::config::Config::default()),
            google_oauth: Some(
                create_oauth_client("client-id", "client-secret", "http://localhost:3001").unwrap(),
            ),
        };
        let (jar, redirect) =
            crate::handlers::v1::auth::oauth::google_auth(State(app_state), CookieJar::new())
                .await
                .expect("認可リダイレクト発行失敗");
        let response = redirect.into_response();
        let location = response
            .headers()
            .get(axum::http::header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .expect("Location ヘッダがありません");
        let url = url::Url::parse(location).expect("認可URLのパース失敗");
        assert_eq!(url.as_str().split('?').next(), Some(GOOGLE_AUTH_URL));
        let params: std::collections::HashMap<String, String> =
            url.query_pairs().into_owned().collect();
        assert_eq!(
            params.get("client_id").map(String::as_str),
            Some("client-id")
        );
        assert_eq!(
            params.get("code_challenge_method").map(String::as_str),
            Some("S256")
        );
        let challenge = params
            .get("code_challenge")
            .expect("認可URLに code_challenge がありません");
        let nonce_param = params
            .get("nonce")
            .expect("認可URLに nonce がありません")
            .clone();
        let state_param = params.get("state").expect("認可URLに state がありません");
        assert_eq!(
            params
                .get("redirect_uri")
                .map(|uri| uri.ends_with("/api/v1/oauth/google/callback")),
            Some(true)
        );

        for name in [
            crate::services::auth::OAUTH_STATE_COOKIE_NAME,
            crate::services::auth::OAUTH_PKCE_VERIFIER_COOKIE_NAME,
            crate::services::auth::OAUTH_NONCE_COOKIE_NAME,
        ] {
            let cookie = jar
                .get(name)
                .unwrap_or_else(|| panic!("Cookie {name} がありません"));
            assert_eq!(cookie.http_only(), Some(true), "{name} は HttpOnly");
        }
        assert_eq!(
            jar.get(crate::services::auth::OAUTH_STATE_COOKIE_NAME)
                .unwrap()
                .value(),
            state_param
        );
        assert_eq!(
            jar.get(crate::services::auth::OAUTH_NONCE_COOKIE_NAME)
                .unwrap()
                .value(),
            nonce_param
        );
        let verifier = jar
            .get(crate::services::auth::OAUTH_PKCE_VERIFIER_COOKIE_NAME)
            .unwrap()
            .value()
            .to_string();
        assert_eq!(
            PkceCodeChallenge::from_code_verifier_sha256(&PkceCodeVerifier::new(verifier.clone()))
                .as_str(),
            challenge,
            "code_challenge は Cookie の verifier の S256 ハッシュであること"
        );

        let server = MockServer::start().await;
        let mut claims = valid_claims();
        claims["nonce"] = serde_json::json!(nonce_param.as_str());
        let (token, keys) = signed_token(claims);
        Mock::given(method("POST"))
            .and(body_string_contains("grant_type=authorization_code"))
            .and(body_string_contains("code=auth-code"))
            .and(body_string_contains(format!("code_verifier={verifier}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "test-access-token", "token_type": "Bearer",
                "expires_in": 3600, "id_token": token
            })))
            .expect(1)
            .mount(&server)
            .await;
        let client = create_oauth_client("client-id", "client-secret", "http://localhost:3001")
            .unwrap()
            .set_token_uri(TokenUrl::new(server.uri()).unwrap());
        let token_result = exchange_google_code(&client, "auth-code".to_string(), verifier)
            .await
            .expect("トークン交換失敗");

        let id_token = token_result.id_token().expect("IDトークンがありません");
        let user = verify_google_id_token(
            id_token,
            client.client_id(),
            keys.clone(),
            &Nonce::new(nonce_param),
        )
        .expect("nonce 一致の ID トークンは受理されること");
        assert_eq!(user.sub, "test-subject");
        assert!(
            verify_google_id_token(
                id_token,
                client.client_id(),
                keys,
                &Nonce::new("attacker-nonce".to_string()),
            )
            .is_err(),
            "nonce 不一致の ID トークンは拒否されること"
        );
    }
}
