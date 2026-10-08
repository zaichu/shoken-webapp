use std::sync::Arc;

use axum::{Router, http::StatusCode, http::header, middleware, routing::get};
use tower_http::{
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
};
use tower_service::Service;
use worker::*;

use crate::{
    config::{self, Config},
    handlers,
    middleware::{add_security_headers, binding_rate_limit, validate_origin},
    services::auth,
    state::{AppState, Secrets},
};

// ドメインハンドラーの wasm 移植が済むまでは、チェックイン済みの契約ファイルをそのまま返す
const OPENAPI_JSON: &str = include_str!("../../docs/openapi.json");

/// リクエストボディの上限サイズ（native の REQUEST_BODY_LIMIT と同値）
const REQUEST_BODY_LIMIT: usize = 10 * 1024 * 1024;

/// `[[ratelimits]]` バインディングを取り出す。未設定（ローカル dev 等）なら None で制限なし
fn rate_limiter(env: &Env, name: &str) -> Option<Arc<RateLimiter>> {
    env.rate_limiter(name).ok().map(Arc::new)
}

/// `Some(limiter)` のときだけバインディングレート制限を付与する
fn with_rate_limit(
    routes: Router<AppState>,
    limiter: Option<Arc<RateLimiter>>,
) -> Router<AppState> {
    if let Some(l) = limiter {
        routes.layer(middleware::from_fn(move |req, next| {
            let l = l.clone();
            async move { binding_rate_limit(l, req, next).await }
        }))
    } else {
        routes
    }
}

fn router(state: AppState, env: &Env) -> Router {
    let config = &state.config;
    let allowed_origins = Arc::new(config.cors_origins.clone());
    let strict_origin_check = config.is_production();
    let secure_cookie = config.secure_cookie;

    let auth_routes = with_rate_limit(
        handlers::v1::auth_routes(),
        rate_limiter(env, "RATE_LIMIT_AUTH"),
    );

    let ready_db = state.pool.clone();
    let probe_routes = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route(
            "/ready",
            get(move || async move {
                match crate::db::query_scalar::<i32>("SELECT 1", vec![])
                    .fetch_one(&ready_db)
                    .await
                {
                    Ok(_) => StatusCode::OK,
                    Err(e) => {
                        console_error!("ready check の DB クエリに失敗: {e}");
                        StatusCode::SERVICE_UNAVAILABLE
                    }
                }
            }),
        )
        .route(
            "/api-docs/openapi.json",
            get(|| async { ([(header::CONTENT_TYPE, "application/json")], OPENAPI_JSON) }),
        );

    auth_routes
        .merge(probe_routes)
        .layer(middleware::from_fn(move |req, next| {
            let origins = allowed_origins.clone();
            async move { validate_origin(origins, strict_origin_check, req, next).await }
        }))
        .layer(RequestBodyLimitLayer::new(REQUEST_BODY_LIMIT))
        .layer(config::build_cors_layer(&config.cors_origins))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        // セキュリティヘッダーは最外層: 403/413 を含む全レスポンスに付与する
        .layer(middleware::from_fn(move |req, next| {
            add_security_headers(secure_cookie, req, next)
        }))
        .with_state(state)
}

fn build_state(env: &Env) -> Result<AppState> {
    let config = Arc::new(Config::from_worker_env(env));
    crate::errors::init_runtime_env(config.runtime_env);
    let secrets = Arc::new(Secrets::from_worker_env(env).map_err(Error::RustError)?);
    let hyperdrive = env.hyperdrive("HYPERDRIVE")?;
    let google_oauth = match (
        secrets.google_client_id.as_deref(),
        secrets.google_client_secret.as_deref(),
    ) {
        (Some(client_id), Some(client_secret)) => Some(
            auth::create_oauth_client(client_id, client_secret, &config.backend_url)
                .map_err(|e| Error::RustError(e.to_string()))?,
        ),
        _ => None,
    };
    Ok(AppState {
        pool: crate::db::Db::from_hyperdrive(&hyperdrive),
        secrets,
        config,
        google_oauth,
    })
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    let state = build_state(&env)?;
    Ok(router(state, &env).call(req).await?)
}
