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
    services::{auth, dividend_cache, jquants::JQuantsClient, stock_price},
    state::{AppState, Secrets},
};

// ドメインハンドラーの wasm 移植が済むまでは、チェックイン済みの契約ファイルをそのまま返す
const OPENAPI_JSON: &str = include_str!("../../docs/openapi.json");

/// リクエストボディの上限サイズ（CSV インポートのペイロードを見込んで 10MB）
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

    // 配当キャッシュのエンキューも data_routes 内の dividend-per-share-estimates が担う。
    // stale/pending 銘柄の消化は scheduled イベントが担当
    let data_routes = with_rate_limit(
        handlers::v1::data_routes(),
        rate_limiter(env, "RATE_LIMIT_DATA"),
    );

    let csv_routes = with_rate_limit(
        handlers::v1::csv_upload_routes(),
        rate_limiter(env, "RATE_LIMIT_CSV"),
    );

    let stock_search_routes = with_rate_limit(
        handlers::v1::stock_search_routes(),
        rate_limiter(env, "RATE_LIMIT_STOCK_SEARCH"),
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
        .merge(data_routes)
        .merge(csv_routes)
        .merge(stock_search_routes)
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
        (Some(client_id), Some(client_secret)) => {
            let mut client =
                auth::create_oauth_client(client_id, client_secret, &config.backend_url)
                    .map_err(|e| Error::RustError(e.to_string()))?;
            // dev/検証でモックへ差し替えるための vars。未設定なら Google 本番のまま
            client.override_endpoints(
                env.var("GOOGLE_TOKEN_URL").ok().map(|v| v.to_string()),
                env.var("GOOGLE_TOKENINFO_URL").ok().map(|v| v.to_string()),
            );
            Some(client)
        }
        _ => None,
    };
    // dev/検証でモックへ差し替えるための vars。未設定なら J-Quants 本番のまま
    let jquants_client =
        secrets
            .jquants_api_key
            .clone()
            .map(|key| match env.var("JQUANTS_BASE_URL") {
                Ok(url) => JQuantsClient::with_base_url(key, url.to_string()),
                Err(_) => JQuantsClient::new(key),
            });
    Ok(AppState {
        pool: crate::db::Db::from_hyperdrive(&hyperdrive),
        secrets,
        config,
        google_oauth,
        jquants_client,
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

/// 現在値を更新する日次 cron（UTC 07:00 = JST 16:00、東証の引け後）。wrangler.toml と対応させる
const STOCK_PRICE_CRON: &str = "0 7 * * *";

#[event(scheduled)]
async fn scheduled(event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    let result = match event.cron().as_str() {
        STOCK_PRICE_CRON => refresh_stock_prices(&env).await,
        // 毎分 cron: 配当キャッシュの stale/pending 銘柄を消化する
        _ => drain_dividend_cache(&env).await,
    };
    if let Err(e) = result {
        console_error!("scheduled イベントエラー (cron={}): {e}", event.cron());
    }
}

/// 保有銘柄の現在値を Yahoo chart API から取得して更新する。
/// dev/検証用に vars の YAHOO_CHART_BASE_URL でモックへ差し替えられる
async fn refresh_stock_prices(env: &Env) -> Result<()> {
    let state = build_state(env)?;
    let base_url = env.var("YAHOO_CHART_BASE_URL").ok().map(|v| v.to_string());
    let updated = stock_price::refresh_stock_prices(&state.pool, base_url)
        .await
        .map_err(|e| Error::RustError(e.to_string()))?;
    console_log!("現在値更新: {} 銘柄を取得", updated);
    Ok(())
}

async fn drain_dividend_cache(env: &Env) -> Result<()> {
    let state = build_state(env)?;
    let Some(client) = state.jquants_client.as_ref() else {
        console_log!("JQUANTS_API_KEY 未設定のため配当キャッシュ消化をスキップ");
        return Ok(());
    };
    let processed = dividend_cache::drain_refresh_queue(&state.pool, client)
        .await
        .map_err(|e| Error::RustError(e.to_string()))?;
    console_log!("配当キャッシュ消化: {} 件処理", processed);
    Ok(())
}
