use crate::db::{Bind, Db, DbError, Tx};
use crate::errors::ApiError;
use axum::http::HeaderMap;
use shared::value::UserId;

pub const MAX_EVENTS_PER_USER: i64 = 500;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClientInfo {
    pub country: Option<String>,
    pub region: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginOutcome {
    Success,
    NewDevice,
    LocationChanged,
}

impl LoginOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::NewDevice => "new_device",
            Self::LocationChanged => "location_changed",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PriorLocation<'a> {
    pub country: Option<&'a str>,
    pub region: Option<&'a str>,
}

pub fn classify_login(prior: Option<PriorLocation<'_>>, current: &ClientInfo) -> LoginOutcome {
    let Some(prior) = prior else {
        return LoginOutcome::NewDevice;
    };
    if matches!(
        (prior.country, current.country.as_deref()),
        (Some(a), Some(b)) if a != b
    ) {
        return LoginOutcome::LocationChanged;
    }
    if matches!(
        (prior.region, current.region.as_deref()),
        (Some(a), Some(b)) if a != b
    ) {
        return LoginOutcome::LocationChanged;
    }
    LoginOutcome::Success
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

fn normalize_country(value: &str) -> Option<String> {
    let upper = value.trim().to_ascii_uppercase();
    (upper.len() == 2 && upper.bytes().all(|b| b.is_ascii_alphabetic())).then_some(upper)
}

fn normalize_region(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.chars().take(64).collect())
}

pub fn parse_user_agent(ua: &str) -> (Option<String>, Option<String>) {
    let lower = ua.to_ascii_lowercase();
    let browser = if lower.contains("edg/") {
        Some("edge")
    } else if lower.contains("firefox/") || lower.contains("fxios/") {
        Some("firefox")
    } else if lower.contains("chrome/") || lower.contains("crios/") {
        Some("chrome")
    } else if lower.contains("safari/") {
        Some("safari")
    } else {
        None
    };
    let os = if lower.contains("windows nt") {
        Some("windows")
    } else if lower.contains("iphone") || lower.contains("ipad") {
        Some("ios")
    } else if lower.contains("android") {
        Some("android")
    } else if lower.contains("mac os x") || lower.contains("macintosh") {
        Some("macos")
    } else if lower.contains("linux") {
        Some("linux")
    } else {
        None
    };
    (browser.map(str::to_owned), os.map(str::to_owned))
}

pub fn client_info_from_headers(headers: &HeaderMap) -> ClientInfo {
    let (browser, os) = header_value(headers, "user-agent")
        .map(parse_user_agent)
        .unwrap_or((None, None));
    ClientInfo {
        country: header_value(headers, "cf-ipcountry").and_then(normalize_country),
        region: header_value(headers, "cf-region-code")
            .or_else(|| header_value(headers, "cf-region"))
            .and_then(normalize_region),
        browser,
        os,
    }
}

pub async fn insert_login_event_in_tx(
    tx: &mut Tx,
    user_id: UserId,
    outcome: LoginOutcome,
    device_hash: Vec<u8>,
    client: &ClientInfo,
) -> Result<(), DbError> {
    crate::db::query(
        "INSERT INTO login_events (user_id, result, device_hash, country, region, browser, os)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        vec![
            Bind::from(user_id),
            Bind::from(outcome.as_str()),
            Bind::Bytes(device_hash),
            Bind::from(client.country.clone()),
            Bind::from(client.region.clone()),
            Bind::from(client.browser.clone()),
            Bind::from(client.os.clone()),
        ],
    )
    .execute(&mut *tx)
    .await?;
    crate::db::query(
        "DELETE FROM login_events WHERE user_id = $1 AND id NOT IN (
             SELECT id FROM login_events WHERE user_id = $1 ORDER BY occurred_at DESC, id DESC LIMIT $2
         )",
        vec![Bind::from(user_id), Bind::from(MAX_EVENTS_PER_USER)],
    )
    .execute(&mut *tx)
    .await?;
    Ok(())
}

pub async fn prune_login_events(pool: &Db) -> Result<u64, ApiError> {
    let deleted = crate::db::query(
        "DELETE FROM login_events WHERE occurred_at < NOW() - INTERVAL '90 days'",
        vec![],
    )
    .execute(pool)
    .await?;
    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(country: &str, region: &str) -> ClientInfo {
        ClientInfo {
            country: Some(country.to_string()),
            region: Some(region.to_string()),
            browser: Some("chrome".to_string()),
            os: Some("windows".to_string()),
        }
    }

    fn prior<'a>(country: &'a str, region: &'a str) -> PriorLocation<'a> {
        PriorLocation {
            country: Some(country),
            region: Some(region),
        }
    }

    #[test]
    fn unseen_device_counts_as_new() {
        assert_eq!(
            classify_login(None, &info("JP", "Tokyo")),
            LoginOutcome::NewDevice
        );
    }

    #[test]
    fn known_device_same_place_counts_as_success() {
        assert_eq!(
            classify_login(Some(prior("JP", "Tokyo")), &info("JP", "Tokyo")),
            LoginOutcome::Success
        );
    }

    #[test]
    fn country_change_counts_as_location_change() {
        assert_eq!(
            classify_login(Some(prior("JP", "Tokyo")), &info("US", "Tokyo")),
            LoginOutcome::LocationChanged
        );
    }

    #[test]
    fn region_change_counts_as_location_change() {
        assert_eq!(
            classify_login(Some(prior("JP", "Tokyo")), &info("JP", "Osaka")),
            LoginOutcome::LocationChanged
        );
    }

    #[test]
    fn missing_place_on_either_side_counts_as_success() {
        let no_place = ClientInfo::default();
        assert_eq!(
            classify_login(Some(prior("JP", "Tokyo")), &no_place),
            LoginOutcome::Success
        );
        assert_eq!(
            classify_login(
                Some(PriorLocation {
                    country: None,
                    region: None,
                }),
                &info("JP", "Tokyo")
            ),
            LoginOutcome::Success
        );
    }

    #[test]
    fn outcome_strings_match_schema_check() {
        assert_eq!(
            [
                LoginOutcome::Success.as_str(),
                LoginOutcome::NewDevice.as_str(),
                LoginOutcome::LocationChanged.as_str(),
            ],
            ["success", "new_device", "location_changed"]
        );
    }

    #[test]
    fn user_agent_yields_kinds_only() {
        for (ua, browser, os) in [
            (
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0 Safari/537.36",
                Some("chrome"),
                Some("windows"),
            ),
            (
                "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
                Some("safari"),
                Some("ios"),
            ),
            (
                "Mozilla/5.0 (X11; Linux x86_64; rv:132.0) Gecko/20100101 Firefox/132.0",
                Some("firefox"),
                Some("linux"),
            ),
            (
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0 Safari/537.36 Edg/131.0",
                Some("edge"),
                Some("windows"),
            ),
            ("", None, None),
            ("curl/8.0", None, None),
        ] {
            assert_eq!(
                parse_user_agent(ua),
                (browser.map(str::to_owned), os.map(str::to_owned)),
                "{ua}"
            );
        }
    }

    #[test]
    fn country_accepts_two_letter_codes_only() {
        assert_eq!(normalize_country("jp"), Some("JP".to_string()));
        assert_eq!(normalize_country("JP "), Some("JP".to_string()));
        for bad in ["", "JPN", "J1", "1"] {
            assert_eq!(normalize_country(bad), None, "{bad}");
        }
    }

    #[test]
    fn headers_never_carry_raw_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("cf-connecting-ip", "203.0.113.10".parse().unwrap());
        headers.insert("x-forwarded-for", "203.0.113.10".parse().unwrap());
        headers.insert("cf-ipcountry", "JP".parse().unwrap());
        headers.insert(
            "user-agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/131.0"
                .parse()
                .unwrap(),
        );
        let info = client_info_from_headers(&headers);
        assert_eq!(info.country.as_deref(), Some("JP"));
        assert_eq!(info.browser.as_deref(), Some("chrome"));
        assert!(
            !format!("{info:?}").contains("203.0.113"),
            "IP が記録対象に含まれないこと"
        );
    }

    #[test]
    fn region_prefers_code_header_and_ignores_blank() {
        let mut headers = HeaderMap::new();
        headers.insert("cf-region-code", "Tokyo".parse().unwrap());
        headers.insert("cf-region", "Osaka".parse().unwrap());
        assert_eq!(
            client_info_from_headers(&headers).region.as_deref(),
            Some("Tokyo")
        );
        let mut headers = HeaderMap::new();
        headers.insert("cf-region", "Osaka".parse().unwrap());
        assert_eq!(
            client_info_from_headers(&headers).region.as_deref(),
            Some("Osaka")
        );
        let mut headers = HeaderMap::new();
        headers.insert("cf-region", "   ".parse().unwrap());
        assert_eq!(client_info_from_headers(&headers).region, None);
    }

    #[tokio::test]
    #[ignore = "requires Docker to run Postgres container"]
    async fn old_rows_are_pruned_and_ip_column_is_absent() {
        let (pool, _node) = crate::test_db::start_test_pool().await;
        let user_id: UserId = crate::db::query_scalar(
            "INSERT INTO users (google_id, email) VALUES ('login-event-test', 'login-event@example.com') RETURNING id",
            vec![],
        )
        .fetch_one(&pool)
        .await
        .expect("ユーザー作成失敗");
        crate::db::query(
            "INSERT INTO login_events (user_id, result, device_hash, occurred_at)
             VALUES ($1, 'success', $2, NOW() - INTERVAL '91 days'), ($1, 'new_device', $2, NOW())",
            vec![Bind::from(user_id), Bind::Bytes(vec![7; 32])],
        )
        .execute(&pool)
        .await
        .expect("履歴の投入失敗");
        assert_eq!(prune_login_events(&pool).await.expect("削除失敗"), 1);
        let remaining: i64 = crate::db::query_scalar(
            "SELECT COUNT(*) FROM login_events WHERE user_id = $1",
            vec![Bind::from(user_id)],
        )
        .fetch_one(&pool)
        .await
        .expect("件数取得失敗");
        assert_eq!(remaining, 1);
        let ip_like: i64 = crate::db::query_scalar(
            "SELECT COUNT(*) FROM information_schema.columns
             WHERE table_name = 'login_events' AND column_name ILIKE '%ip%'",
            vec![],
        )
        .fetch_one(&pool)
        .await
        .expect("列の確認失敗");
        assert_eq!(ip_like, 0);
    }
}
