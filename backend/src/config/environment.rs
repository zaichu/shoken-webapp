use std::env;

/// 開発用として明示設定しうる環境名。
/// これ以外の値(未設定・不明値を含む)は安全側に本番として扱う
const DEVELOPMENT_ENV_VALUES: &[&str] = &["local", "dev", "development", "test"];

/// 本番環境かどうかを判定(fail-safe)
/// RUST_ENV / APP_ENV に設定された値がすべて開発用の値のときだけ非本番(false)。
/// 未設定・不明値・本番値との混在はすべて本番扱いにし、環境変数の設定漏れや
/// 書き間違いでセキュリティ設定が緩まないようにする。
/// BACKEND_URL のスキームには依存しない
pub fn is_production_env() -> bool {
    let mut any_set = false;
    for value in [env::var("RUST_ENV"), env::var("APP_ENV")]
        .into_iter()
        .flatten()
    {
        if !DEVELOPMENT_ENV_VALUES.contains(&value.as_str()) {
            return true;
        }
        any_set = true;
    }
    !any_set
}

pub fn backend_url() -> String {
    env::var("BACKEND_URL").unwrap_or_else(|_| {
        let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
        format!("http://localhost:{port}")
    })
}

pub fn server_addr() -> String {
    let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    format!("0.0.0.0:{port}")
}

/// CookieをSecureで発行するか判定
/// 本番環境(環境変数が未設定の場合を含む)では SECURE_COOKIE の値に関わらず true。
/// 非本番では SECURE_COOKIE=true/1 の明示指定のみ true。
/// BACKEND_URL のスキームには依存しない
pub fn is_secure_cookie() -> bool {
    if is_production_env() {
        return true;
    }
    env::var("SECURE_COOKIE").is_ok_and(|v| v == "true" || v == "1")
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
        // fail-safe: 未設定・不明値・非開発値の混在はすべて本番扱い。
        // 設定値がすべて開発用の値のときだけ非本番。
        // BACKEND_URL のスキームは判定に使わない
        let cases = [
            (Some("production"), None, None, true),
            (None, Some("production"), None, true),
            (None, None, Some("https://api.example.com"), true),
            (None, None, Some("http://api.example.com"), true),
            (None, None, None, true),
            (Some("development"), Some("production"), None, true),
            (Some("production"), Some("development"), None, true),
            (Some("development"), Some("staging"), None, true),
            (Some("development"), None, None, false),
            (None, Some("local"), None, false),
            (None, Some("test"), None, false),
            (Some("dev"), Some("development"), None, false),
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
        // 本番(環境変数の未設定を含む)では SECURE_COOKIE の値に関わらず Secure。
        // 非本番では明示指定のみ有効。BACKEND_URL のスキームは判定に使わない
        type CookieCase = (
            Option<&'static str>,
            Option<&'static str>,
            Option<&'static str>,
            bool,
        );
        let cases: [CookieCase; 9] = [
            (Some("true"), None, None, true),
            (Some("1"), None, None, true),
            (Some("false"), None, None, true),
            (None, Some("production"), None, true),
            // 本番で SECURE_COOKIE=false を明示しても無効にできない(fail-safe)
            (Some("false"), Some("production"), None, true),
            (None, None, Some("https://api.example.com"), true),
            (None, None, None, true),
            // 開発用の値を明示した場合だけ非本番(SECURE_COOKIE 必須化は外れる)
            (None, Some("development"), None, false),
            (Some("false"), Some("development"), None, false),
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
