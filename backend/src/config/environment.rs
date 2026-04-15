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
    use temp_env::with_vars;

    // --- is_production_env ---

    #[test]
    fn test_is_production_env_rust_env() {
        with_vars(
            [
                ("RUST_ENV", Some("production")),
                ("APP_ENV", None::<&str>),
                ("BACKEND_URL", None::<&str>),
            ],
            || {
                assert!(is_production_env());
            },
        );
    }

    #[test]
    fn test_is_production_env_app_env() {
        with_vars(
            [
                ("RUST_ENV", None::<&str>),
                ("APP_ENV", Some("production")),
                ("BACKEND_URL", None::<&str>),
            ],
            || {
                assert!(is_production_env());
            },
        );
    }

    #[test]
    fn test_is_production_env_backend_url_https() {
        with_vars(
            [
                ("RUST_ENV", None::<&str>),
                ("APP_ENV", None::<&str>),
                ("BACKEND_URL", Some("https://api.example.com")),
            ],
            || {
                assert!(is_production_env());
            },
        );
    }

    #[test]
    fn test_is_production_env_backend_url_http() {
        with_vars(
            [
                ("RUST_ENV", None::<&str>),
                ("APP_ENV", None::<&str>),
                ("BACKEND_URL", Some("http://api.example.com")),
            ],
            || {
                assert!(!is_production_env());
            },
        );
    }

    #[test]
    fn test_is_production_env_none() {
        with_vars(
            [
                ("RUST_ENV", None::<&str>),
                ("APP_ENV", None::<&str>),
                ("BACKEND_URL", None::<&str>),
            ],
            || {
                assert!(!is_production_env());
            },
        );
    }

    // --- backend_url ---

    #[test]
    fn test_backend_url_explicit() {
        with_vars(
            [
                ("BACKEND_URL", Some("https://api.example.com")),
                ("PORT", None::<&str>),
            ],
            || {
                assert_eq!(backend_url(), "https://api.example.com");
            },
        );
    }

    #[test]
    fn test_backend_url_port() {
        with_vars(
            [("BACKEND_URL", None::<&str>), ("PORT", Some("8080"))],
            || {
                assert_eq!(backend_url(), "http://localhost:8080");
            },
        );
    }

    #[test]
    fn test_backend_url_default() {
        with_vars(
            [("BACKEND_URL", None::<&str>), ("PORT", None::<&str>)],
            || {
                assert_eq!(backend_url(), "http://localhost:3001");
            },
        );
    }

    // --- server_addr ---

    #[test]
    fn test_server_addr_port() {
        with_vars([("PORT", Some("9000"))], || {
            assert_eq!(server_addr(), "0.0.0.0:9000");
        });
    }

    #[test]
    fn test_server_addr_default() {
        with_vars([("PORT", None::<&str>)], || {
            assert_eq!(server_addr(), "0.0.0.0:3001");
        });
    }

    // --- is_secure_cookie ---

    #[test]
    fn test_is_secure_cookie_true() {
        with_vars(
            [
                ("SECURE_COOKIE", Some("true")),
                ("BACKEND_URL", None::<&str>),
            ],
            || {
                assert!(is_secure_cookie());
            },
        );
    }

    #[test]
    fn test_is_secure_cookie_one() {
        with_vars(
            [("SECURE_COOKIE", Some("1")), ("BACKEND_URL", None::<&str>)],
            || {
                assert!(is_secure_cookie());
            },
        );
    }

    #[test]
    fn test_is_secure_cookie_false() {
        with_vars(
            [
                ("SECURE_COOKIE", Some("false")),
                ("BACKEND_URL", None::<&str>),
            ],
            || {
                assert!(!is_secure_cookie());
            },
        );
    }

    #[test]
    fn test_is_secure_cookie_backend_url_https() {
        with_vars(
            [
                ("SECURE_COOKIE", None::<&str>),
                ("BACKEND_URL", Some("https://api.example.com")),
            ],
            || {
                assert!(is_secure_cookie());
            },
        );
    }

    #[test]
    fn test_is_secure_cookie_none() {
        with_vars(
            [
                ("SECURE_COOKIE", None::<&str>),
                ("BACKEND_URL", None::<&str>),
            ],
            || {
                assert!(!is_secure_cookie());
            },
        );
    }

    // --- csv_rate_limit_rps ---

    #[test]
    fn test_csv_rate_limit_rps_explicit() {
        with_vars([("CSV_RATE_LIMIT_RPS", Some("5"))], || {
            assert_eq!(csv_rate_limit_rps(), 5);
        });
    }

    #[test]
    fn test_csv_rate_limit_rps_default() {
        with_vars([("CSV_RATE_LIMIT_RPS", None::<&str>)], || {
            assert_eq!(csv_rate_limit_rps(), 2);
        });
    }
}
