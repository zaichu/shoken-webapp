use std::env;

/// 本番環境かどうかを判定
/// RUST_ENV または APP_ENV のどちらかが production の場合に true
/// (fail-safe: 一方が development でも他方が production なら本番扱いにする)。
/// BACKEND_URL のスキームには依存しない(環境変数の書き間違いで
/// セキュリティ設定が緩まないよう、明示的な値のみで判定する)
pub fn is_production_env() -> bool {
    env::var("RUST_ENV").ok().as_deref() == Some("production")
        || env::var("APP_ENV").ok().as_deref() == Some("production")
}

pub fn backend_url() -> String {
    env::var("BACKEND_URL").unwrap_or_else(|_| {
        let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
        format!("http://localhost:{}", port)
    })
}

pub fn server_addr() -> String {
    let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    format!("0.0.0.0:{}", port)
}

/// CookieをSecureで発行するか判定
/// 本番環境(APP_ENV/RUST_ENV=production)では SECURE_COOKIE の値に関わらず true。
/// 非本番では SECURE_COOKIE=true/1 の明示指定のみ true。
/// BACKEND_URL のスキームには依存しない
pub fn is_secure_cookie() -> bool {
    if is_production_env() {
        return true;
    }
    env::var("SECURE_COOKIE")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

/// CSV アップロード系ルートへのレート制限（リクエスト/秒）。0 は無制限
pub fn csv_rate_limit_rps() -> u32 {
    env::var("CSV_RATE_LIMIT_RPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2)
}

/// 認証済みデータ系ルートへの IP 単位レート制限（リクエスト/秒）。0 は無制限。
/// 画面の通常操作は数リクエスト/秒程度のため、auth/stock_search と同じ 10 を既定値にする
pub fn data_rate_limit_rps() -> u32 {
    env::var("DATA_RATE_LIMIT_RPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
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
        // BACKEND_URL のスキームは判定に使わない(明示設定のみで判定する)。
        // 一方が development でも他方が production なら本番扱い(fail-safe)
        let cases = [
            (Some("production"), None, None, true),
            (None, Some("production"), None, true),
            (None, None, Some("https://api.example.com"), false),
            (None, None, Some("http://api.example.com"), false),
            (None, None, None, false),
            (Some("development"), Some("production"), None, true),
            (Some("production"), Some("development"), None, true),
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
        // (SECURE_COOKIE, APP_ENV, BACKEND_URL, expected)
        // 本番では SECURE_COOKIE の値に関わらず Secure。非本番では明示指定のみ有効。
        // BACKEND_URL のスキームは判定に使わない
        type CookieCase = (
            Option<&'static str>,
            Option<&'static str>,
            Option<&'static str>,
            bool,
        );
        let cases: [CookieCase; 7] = [
            (Some("true"), None, None, true),
            (Some("1"), None, None, true),
            (Some("false"), None, None, false),
            (None, Some("production"), None, true),
            // 本番で SECURE_COOKIE=false を明示しても無効にできない(fail-safe)
            (Some("false"), Some("production"), None, true),
            (None, None, Some("https://api.example.com"), false),
            (None, None, None, false),
        ];
        for (secure_cookie, app_env, backend_url, expected) in cases {
            with_vars(
                [
                    ("SECURE_COOKIE", secure_cookie),
                    ("APP_ENV", app_env),
                    ("RUST_ENV", None),
                    ("BACKEND_URL", backend_url),
                ],
                || {
                    assert_eq!(
                        is_secure_cookie(),
                        expected,
                        "SECURE_COOKIE={secure_cookie:?} APP_ENV={app_env:?} BACKEND_URL={backend_url:?}"
                    );
                },
            );
        }
    }

    // --- data_rate_limit_rps ---

    #[test]
    fn test_data_rate_limit_rps() {
        let _guard = ENV_MUTEX.blocking_lock();
        assert_eq!(data_rate_limit_rps(), 10);
        temp_env::with_var("DATA_RATE_LIMIT_RPS", Some("5"), || {
            assert_eq!(data_rate_limit_rps(), 5);
        });
        temp_env::with_var("DATA_RATE_LIMIT_RPS", Some("0"), || {
            assert_eq!(data_rate_limit_rps(), 0);
        });
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
