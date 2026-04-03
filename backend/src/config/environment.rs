use std::env;

/// 本番環境かどうかを判定
/// RUST_ENV=production または APP_ENV=production の場合に true
/// 明示的なフラグがない場合のみ BACKEND_URL の https:// スキームで判定
pub fn is_production_env() -> bool {
    if let Ok(v) = env::var("RUST_ENV") {
        return v == "production";
    }
    if let Ok(v) = env::var("APP_ENV") {
        return v == "production";
    }
    env::var("BACKEND_URL")
        .map(|url| url.starts_with("https://"))
        .unwrap_or(false)
}

/// バックエンドのベースURLを取得
pub fn backend_url() -> String {
    env::var("BACKEND_URL").unwrap_or_else(|_| {
        let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
        format!("http://localhost:{}", port)
    })
}

/// サーバーのバインドアドレスを取得
pub fn server_addr() -> String {
    let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    format!("0.0.0.0:{}", port)
}

/// CookieをSecureで発行するか判定
/// BACKEND_URL が https:// で始まる場合、または SECURE_COOKIE=true の場合に true
pub fn is_secure_cookie() -> bool {
    if let Ok(secure) = env::var("SECURE_COOKIE") {
        return secure == "true" || secure == "1";
    }

    env::var("BACKEND_URL")
        .map(|url| url.starts_with("https://"))
        .unwrap_or(false)
}

/// CSV アップロード系ルートへのレート制限（リクエスト/秒）。0 は無制限
pub fn csv_rate_limit_rps() -> u32 {
    env::var("CSV_RATE_LIMIT_RPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2)
}
