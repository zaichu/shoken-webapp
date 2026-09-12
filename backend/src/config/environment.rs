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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_env::ENV_MUTEX;
    use temp_env::with_vars;

    // --- is_production_env ---

    #[test]
    fn test_is_production_env() {
        let _guard = ENV_MUTEX.blocking_lock();
        // (RUST_ENV, APP_ENV, BACKEND_URL, expected)
        let cases = [
            (Some("production"), None, None, true),
            (None, Some("production"), None, true),
            (None, None, Some("https://api.example.com"), true),
            (None, None, Some("http://api.example.com"), false),
            (None, None, None, false),
        ];
        for (rust_env, app_env, backend_url, expected) in cases {
            with_vars(
                [
                    ("RUST_ENV", rust_env),
                    ("APP_ENV", app_env),
                    ("BACKEND_URL", backend_url),
                ],
                || {
                    assert_eq!(
                        is_production_env(),
                        expected,
                        "RUST_ENV={rust_env:?} APP_ENV={app_env:?} BACKEND_URL={backend_url:?}"
                    );
                },
            );
        }
    }

    // --- backend_url ---

    #[test]
    fn test_backend_url() {
        let _guard = ENV_MUTEX.blocking_lock();
        // (BACKEND_URL, PORT, expected)
        let cases: [(Option<&str>, Option<&str>, &str); 3] = [
            (
                Some("https://api.example.com"),
                None,
                "https://api.example.com",
            ),
            (None, Some("8080"), "http://localhost:8080"),
            (None, None, "http://localhost:3001"),
        ];
        for (backend_url_var, port, expected) in cases {
            with_vars([("BACKEND_URL", backend_url_var), ("PORT", port)], || {
                assert_eq!(
                    backend_url(),
                    expected,
                    "BACKEND_URL={backend_url_var:?} PORT={port:?}"
                );
            });
        }
    }

    // --- server_addr ---

    #[test]
    fn test_server_addr() {
        let _guard = ENV_MUTEX.blocking_lock();
        // (PORT, expected)
        let cases: [(Option<&str>, &str); 2] =
            [(Some("9000"), "0.0.0.0:9000"), (None, "0.0.0.0:3001")];
        for (port, expected) in cases {
            with_vars([("PORT", port)], || {
                assert_eq!(server_addr(), expected, "PORT={port:?}");
            });
        }
    }

    // --- is_secure_cookie ---

    #[test]
    fn test_is_secure_cookie() {
        let _guard = ENV_MUTEX.blocking_lock();
        // (SECURE_COOKIE, BACKEND_URL, expected)
        let cases: [(Option<&str>, Option<&str>, bool); 5] = [
            (Some("true"), None, true),
            (Some("1"), None, true),
            (Some("false"), None, false),
            (None, Some("https://api.example.com"), true),
            (None, None, false),
        ];
        for (secure_cookie, backend_url, expected) in cases {
            with_vars(
                [
                    ("SECURE_COOKIE", secure_cookie),
                    ("BACKEND_URL", backend_url),
                ],
                || {
                    assert_eq!(
                        is_secure_cookie(),
                        expected,
                        "SECURE_COOKIE={secure_cookie:?} BACKEND_URL={backend_url:?}"
                    );
                },
            );
        }
    }

    // --- csv_rate_limit_rps ---

    #[test]
    fn test_csv_rate_limit_rps() {
        let _guard = ENV_MUTEX.blocking_lock();
        // (CSV_RATE_LIMIT_RPS, expected)
        let cases: [(Option<&str>, u32); 2] = [(Some("5"), 5), (None, 2)];
        for (value, expected) in cases {
            with_vars([("CSV_RATE_LIMIT_RPS", value)], || {
                assert_eq!(
                    csv_rate_limit_rps(),
                    expected,
                    "CSV_RATE_LIMIT_RPS={value:?}"
                );
            });
        }
    }
}
