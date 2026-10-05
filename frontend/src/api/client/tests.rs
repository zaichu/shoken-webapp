use super::*;

#[test]
fn retryable_errors() {
    assert!(ApiError::Network.is_retryable());
    assert!(ApiError::Timeout.is_retryable());
    assert!(ApiError::http(500).is_retryable());
    assert!(ApiError::http(503).is_retryable());
    assert!(!ApiError::http(400).is_retryable());
    assert!(!ApiError::http(401).is_retryable());
    assert!(!ApiError::http(403).is_retryable());
    assert!(!ApiError::http(404).is_retryable());
    assert!(!ApiError::Parse.is_retryable());
}

#[test]
fn unauthorized_detection() {
    assert!(ApiError::http(401).is_unauthorized());
    assert!(!ApiError::http(403).is_unauthorized());
    assert!(!ApiError::Network.is_unauthorized());
}

#[test]
fn user_message_maps_status_to_text() {
    assert_eq!(ApiError::http(401).user_message(), "ログインが必要です");
    assert_eq!(
        ApiError::http(404).user_message(),
        "指定されたリソースが見つかりません"
    );
    assert_eq!(
        ApiError::http(500).user_message(),
        "サーバーエラーが発生しました。しばらくしてから再度お試しください"
    );
    assert_eq!(
        ApiError::Network.user_message(),
        "ネットワーク接続を確認してください"
    );
    assert_eq!(
        ApiError::http(400).user_message(),
        "入力内容を確認してください"
    );
    assert_eq!(
        ApiError::http(403).user_message(),
        "このリソースへのアクセス権限がありません"
    );
    assert_eq!(
        ApiError::http(418).user_message(),
        "エラーが発生しました (ステータス: 418)"
    );
}

#[test]
fn display_uses_user_message() {
    for error in [
        ApiError::Network,
        ApiError::Timeout,
        ApiError::Parse,
        ApiError::http(404),
    ] {
        assert_eq!(format!("{error}"), error.user_message());
    }
}

#[test]
fn base_url_comes_from_build_env_or_empty() {
    assert_eq!(
        ApiClient::base_url(),
        option_env!("SHOKEN_WEBAPI_URL").unwrap_or("")
    );
}

#[test]
fn with_max_retries_overrides_only_retry_count() {
    let client = ApiClient::auth_client().with_max_retries(7);
    assert_eq!(client.max_retries, 7);
    assert_eq!(client.timeout_ms, AUTH_TIMEOUT_MS);
    assert_eq!(client.retry_delay_ms, AUTH_RETRY_DELAY_MS);
}

#[test]
fn with_timeout_ms_overrides_only_timeout() {
    let client = ApiClient::auth_client().with_timeout_ms(7_000);
    assert_eq!(client.timeout_ms, 7_000);
    assert_eq!(client.max_retries, AUTH_MAX_RETRIES);
    assert_eq!(client.retry_delay_ms, AUTH_RETRY_DELAY_MS);
}

#[test]
fn message_maps_variant_and_status_to_text() {
    for (error, expected) in [
        (ApiError::Network, "ネットワークエラーが発生しました"),
        (ApiError::Timeout, "リクエストがタイムアウトしました"),
        (ApiError::Parse, "応答の解析に失敗しました"),
        (ApiError::http(400), "リクエストが不正です"),
        (
            ApiError::Http {
                status: 400,
                server_message: Some("CSVファイル（.csv）のみアップロードできます".to_string()),
            },
            "CSVファイル（.csv）のみアップロードできます",
        ),
        (ApiError::http(401), "認証が必要です"),
        (ApiError::http(403), "アクセスが拒否されました"),
        (ApiError::http(404), "リソースが見つかりません"),
        (ApiError::http(500), "サーバーエラーが発生しました"),
        (ApiError::http(503), "サーバーエラーが発生しました"),
        (
            ApiError::http(501),
            "エラーが発生しました (ステータス: 501)",
        ),
        (
            ApiError::http(418),
            "エラーが発生しました (ステータス: 418)",
        ),
    ] {
        assert_eq!(error.message(), expected);
    }
}

#[test]
fn client_profiles_have_expected_defaults() {
    let default = ApiClient::default_client();
    assert_eq!(default.timeout_ms, 30_000);
    assert_eq!(default.max_retries, 3);
    let read = ApiClient::read_client();
    assert_eq!(read.timeout_ms, 10_000);
    assert_eq!(read.max_retries, 1);
    let auth = ApiClient::auth_client();
    assert_eq!(auth.timeout_ms, 5_000);
    assert_eq!(auth.max_retries, 1);
    assert_eq!(auth.retry_delay_ms, 500);
    assert_eq!(auth.retry_multiplier, 1);
}

#[test]
fn url_building() {
    let client = ApiClient::new(String::new(), 1, 0, 1, 1);
    assert_eq!(client.url("/api/v1/session", &[]), "/api/v1/session");
    assert_eq!(
        client.url("/api/v1/stocks", &[("query", "7974")]),
        "/api/v1/stocks?query=7974"
    );
    let prefixed = ApiClient::new("https://example.test".to_string(), 1, 0, 1, 1);
    assert_eq!(
        prefixed.url("/api/v1/session", &[]),
        "https://example.test/api/v1/session"
    );
}
