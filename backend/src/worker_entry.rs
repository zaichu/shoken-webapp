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
    cron_log::{CronStep, CronTimer, format_cron_noop, format_cron_step},
    handlers,
    middleware::{add_security_headers, binding_rate_limit, validate_origin},
    services::{
        auth, dividend_cache, fund_price, jquants, jquants::JQuantsClient, stock_master,
        stock_price,
    },
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
        .layer(config::build_cors_layer(
            &config.cors_origins,
            !config.is_production(),
        ))
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
    let jquants_client = secrets.jquants_api_key.clone().map(|key| {
        let url_or = |name: &str, default: &str| {
            env.var(name)
                .map(|v| v.to_string())
                .unwrap_or_else(|_| default.to_string())
        };
        JQuantsClient::with_urls(
            key,
            url_or("JQUANTS_BASE_URL", jquants::FIN_SUMMARY_URL),
            url_or("JQUANTS_MASTER_BASE_URL", jquants::EQUITIES_MASTER_URL),
        )
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

// 何もしなかった回を noop の1行に抑えるため、処理した回だけ開始・終了を出す
fn log_cron_outcome(
    step: CronStep,
    label: &str,
    outcome: Result<Option<usize>, Error>,
    elapsed_ms: u64,
) -> bool {
    match outcome {
        Ok(Some(count)) if count > 0 => {
            console_log!("cron {}: 開始", step.name());
            console_log!("{}", format_cron_step(step, count, elapsed_ms));
            true
        }
        Ok(_) => false,
        Err(e) => {
            console_error!("scheduled イベントエラー: {label}: {e} ({elapsed_ms}ms)");
            true
        }
    }
}

// 同じ分の cron は1回にまとまって届くため振り分けない
#[event(scheduled)]
async fn scheduled(event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    let total = CronTimer::start();
    let state = match build_state(&env) {
        Ok(state) => state,
        Err(e) => {
            console_error!("scheduled イベントエラー: state 構築: {e}");
            return;
        }
    };
    let mut ran = false;
    {
        let timer = CronTimer::start();
        let outcome = drain_dividend_cache(&state).await;
        ran |= log_cron_outcome(
            CronStep::DividendCache,
            "配当キャッシュ消化",
            outcome,
            timer.elapsed_ms(),
        );
    }
    {
        let timer = CronTimer::start();
        let outcome: Result<Option<usize>, Error> =
            if stock_price::should_refresh_at(event.schedule()) {
                refresh_stock_prices(&env, &state).await.map(Some)
            } else {
                Ok(None)
            };
        ran |= log_cron_outcome(
            CronStep::StockPrice,
            "現在値更新",
            outcome,
            timer.elapsed_ms(),
        );
    }
    {
        let timer = CronTimer::start();
        let outcome: Result<Option<usize>, Error> =
            if fund_price::should_fund_refresh_at(event.schedule()) {
                refresh_fund_prices(&env, &state).await.map(Some)
            } else {
                Ok(None)
            };
        ran |= log_cron_outcome(
            CronStep::FundPrice,
            "基準価額更新",
            outcome,
            timer.elapsed_ms(),
        );
    }
    {
        let timer = CronTimer::start();
        let outcome = refresh_stock_master(&state, event.schedule()).await;
        ran |= log_cron_outcome(
            CronStep::StockMaster,
            "銘柄マスタ更新",
            outcome,
            timer.elapsed_ms(),
        );
    }
    if !ran {
        console_log!("{}", format_cron_noop(total.elapsed_ms()));
    }
}

/// 保有銘柄の現在値を Yahoo chart API から取得して更新する。
/// dev/検証用に vars の YAHOO_CHART_BASE_URL でモックへ差し替えられる
async fn refresh_stock_prices(env: &Env, state: &AppState) -> Result<usize> {
    let base_url = env.var("YAHOO_CHART_BASE_URL").ok().map(|v| v.to_string());
    stock_price::refresh_stock_prices(&state.pool, base_url)
        .await
        .map_err(|e| Error::RustError(e.to_string()))
}

/// 保有ファンドの基準価額を三菱UFJアセットの投信情報APIから取得して更新する。
/// dev/検証用に vars の MUFG_API_BASE_URL でモックへ差し替えられる
async fn refresh_fund_prices(env: &Env, state: &AppState) -> Result<usize> {
    let base_url = env.var("MUFG_API_BASE_URL").ok().map(|v| v.to_string());
    fund_price::refresh_fund_prices(&state.pool, base_url)
        .await
        .map_err(|e| Error::RustError(e.to_string()))
}

/// J-Quants 上場銘柄一覧から銘柄マスタ(stock)を全置換する。
/// dev/検証用に vars の JQUANTS_MASTER_BASE_URL でモックへ差し替えられる
async fn refresh_stock_master(state: &AppState, scheduled_ms: f64) -> Result<Option<usize>> {
    let Some(client) = state.jquants_client.as_ref() else {
        return Ok(None);
    };
    stock_master::maybe_refresh(&state.pool, client, scheduled_ms)
        .await
        .map_err(|e| Error::RustError(e.to_string()))
}

async fn drain_dividend_cache(state: &AppState) -> Result<Option<usize>> {
    let Some(client) = state.jquants_client.as_ref() else {
        return Ok(None);
    };
    dividend_cache::drain_refresh_queue(&state.pool, client)
        .await
        .map(Some)
        .map_err(|e| Error::RustError(e.to_string()))
}
