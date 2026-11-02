use crate::db::{Bind, Db, DbError, Tx};
pub mod admin;
use crate::errors::ApiError;
use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared::value::UserId;
use utoipa::ToSchema;

pub const MAX_EVENTS_PER_USER: i64 = 500;
pub const MAX_HISTORY_LIMIT: i64 = 100;
pub const DEFAULT_HISTORY_LIMIT: i64 = 50;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct LoginHistoryEntry {
    pub occurred_at: DateTime<Utc>,
    pub result: String,
    pub country: Option<String>,
    pub region: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
}

impl crate::db::FromRow for LoginHistoryEntry {
    fn from_row(row: &crate::db::Row) -> Result<Self, DbError> {
        Ok(Self {
            occurred_at: row.try_get("occurred_at")?,
            result: row.try_get("result")?,
            country: row.try_get("country")?,
            region: row.try_get("region")?,
            browser: row.try_get("browser")?,
            os: row.try_get("os")?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct LoginHistoryListResponse {
    pub data: Vec<LoginHistoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DeviceEntry {
    pub id: String,
    pub last_used_at: DateTime<Utc>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub is_current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DeviceListResponse {
    pub data: Vec<DeviceEntry>,
}

pub fn device_opaque_id(user_id: &UserId, device_hash: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(user_id.to_string().as_bytes());
    hasher.update(device_hash);
    hex::encode(hasher.finalize())
}

pub fn clamp_history_limit(limit: Option<i64>) -> i64 {
    match limit {
        Some(n) if n > 0 => n.min(MAX_HISTORY_LIMIT),
        _ => DEFAULT_HISTORY_LIMIT,
    }
}

pub fn clamp_history_offset(offset: Option<i64>) -> i64 {
    match offset {
        Some(n) if n > 0 => n,
        _ => 0,
    }
}

pub async fn list_login_history(
    pool: &Db,
    user_id: UserId,
    limit: i64,
    offset: i64,
) -> Result<Vec<LoginHistoryEntry>, DbError> {
    crate::db::query_as::<LoginHistoryEntry>(
        "SELECT occurred_at, result, country, region, browser, os FROM login_events
         WHERE user_id = $1 ORDER BY occurred_at DESC, id DESC LIMIT $2 OFFSET $3",
        vec![Bind::from(user_id), Bind::from(limit), Bind::from(offset)],
    )
    .fetch_all(pool)
    .await
}

struct DeviceRow {
    device_hash: Vec<u8>,
    last_used_at: DateTime<Utc>,
    country: Option<String>,
    region: Option<String>,
    browser: Option<String>,
    os: Option<String>,
}

impl crate::db::FromRow for DeviceRow {
    fn from_row(row: &crate::db::Row) -> Result<Self, DbError> {
        Ok(Self {
            device_hash: row.try_get("device_hash")?,
            last_used_at: row.try_get("last_used_at")?,
            country: row.try_get("country")?,
            region: row.try_get("region")?,
            browser: row.try_get("browser")?,
            os: row.try_get("os")?,
        })
    }
}

pub async fn list_devices(
    pool: &Db,
    user_id: UserId,
    current_device_hash: &[u8],
) -> Result<Vec<DeviceEntry>, DbError> {
    let rows = crate::db::query_as::<DeviceRow>(
        "SELECT DISTINCT ON (s.device_hash) s.device_hash,
                COALESCE(le.occurred_at, s.created_at) AS last_used_at,
                le.country, le.region, le.browser, le.os
         FROM sessions s
         LEFT JOIN LATERAL (
             SELECT occurred_at, country, region, browser, os
             FROM login_events
             WHERE user_id = s.user_id AND device_hash = s.device_hash
             ORDER BY occurred_at DESC
             LIMIT 1
         ) le ON TRUE
         WHERE s.user_id = $1 AND s.expires_at > NOW() AND s.device_hash IS NOT NULL
         ORDER BY s.device_hash, last_used_at DESC",
        vec![Bind::from(user_id)],
    )
    .fetch_all(pool)
    .await?;
    let mut devices: Vec<DeviceEntry> = rows
        .into_iter()
        .map(|row| DeviceEntry {
            id: device_opaque_id(&user_id, &row.device_hash),
            last_used_at: row.last_used_at,
            country: row.country,
            region: row.region,
            browser: row.browser,
            os: row.os,
            is_current: row.device_hash == current_device_hash,
        })
        .collect();
    devices.sort_by_key(|a| std::cmp::Reverse(a.last_used_at));
    Ok(devices)
}

pub async fn resolve_device_hash(
    pool: &Db,
    user_id: UserId,
    opaque_id: &str,
) -> Result<Option<Vec<u8>>, DbError> {
    struct HashRow {
        device_hash: Vec<u8>,
    }
    impl crate::db::FromRow for HashRow {
        fn from_row(row: &crate::db::Row) -> Result<Self, DbError> {
            Ok(Self {
                device_hash: row.try_get("device_hash")?,
            })
        }
    }
    let rows = crate::db::query_as::<HashRow>(
        "SELECT DISTINCT device_hash FROM sessions
         WHERE user_id = $1 AND expires_at > NOW() AND device_hash IS NOT NULL",
        vec![Bind::from(user_id)],
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| row.device_hash)
        .find(|hash| device_opaque_id(&user_id, hash) == opaque_id))
}

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

    #[test]
    fn opaque_id_is_stable_user_scoped_and_hides_hash() {
        let user_a = UserId::from(uuid::Uuid::new_v4());
        let user_b = UserId::from(uuid::Uuid::new_v4());
        let hash = vec![9u8; 32];
        let other_hash = vec![8u8; 32];
        assert_eq!(
            device_opaque_id(&user_a, &hash),
            device_opaque_id(&user_a, &hash)
        );
        assert_ne!(
            device_opaque_id(&user_a, &hash),
            device_opaque_id(&user_a, &other_hash)
        );
        assert_ne!(
            device_opaque_id(&user_a, &hash),
            device_opaque_id(&user_b, &hash)
        );
        let id = device_opaque_id(&user_a, &hash);
        let raw_hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
        assert!(!id.contains(&raw_hex));
    }

    #[test]
    fn history_limit_is_clamped() {
        assert_eq!(clamp_history_limit(None), DEFAULT_HISTORY_LIMIT);
        assert_eq!(clamp_history_limit(Some(0)), DEFAULT_HISTORY_LIMIT);
        assert_eq!(clamp_history_limit(Some(-5)), DEFAULT_HISTORY_LIMIT);
        assert_eq!(clamp_history_limit(Some(10)), 10);
        assert_eq!(clamp_history_limit(Some(10_000)), MAX_HISTORY_LIMIT);
    }

    #[test]
    fn history_offset_starts_at_zero_and_rejects_negative() {
        assert_eq!(clamp_history_offset(None), 0);
        assert_eq!(clamp_history_offset(Some(0)), 0);
        assert_eq!(clamp_history_offset(Some(-3)), 0);
        assert_eq!(clamp_history_offset(Some(10)), 10);
        assert_eq!(clamp_history_offset(Some(500)), 500);
    }

    #[test]
    fn history_and_device_responses_carry_no_secret_fields() {
        let entry = LoginHistoryEntry {
            occurred_at: chrono::Utc::now(),
            result: "success".to_string(),
            country: Some("JP".to_string()),
            region: Some("Tokyo".to_string()),
            browser: Some("chrome".to_string()),
            os: Some("windows".to_string()),
        };
        let history_json =
            serde_json::to_value(LoginHistoryListResponse { data: vec![entry] }).unwrap();
        let history_text = history_json.to_string();
        for forbidden in ["device_hash", "device_id", "token", "ip"] {
            assert!(!history_text.contains(forbidden), "{forbidden}");
        }
        let device = DeviceEntry {
            id: "opaque".to_string(),
            last_used_at: chrono::Utc::now(),
            country: None,
            region: None,
            browser: None,
            os: None,
            is_current: true,
        };
        let device_text =
            serde_json::to_string(&DeviceListResponse { data: vec![device] }).unwrap();
        for forbidden in ["device_hash", "token", "ip"] {
            assert!(!device_text.contains(forbidden), "{forbidden}");
        }
    }

    #[tokio::test]
    #[ignore = "requires Docker to run Postgres container"]
    async fn history_and_devices_are_scoped_to_owner() {
        let (pool, _node) = crate::test_db::start_test_pool().await;
        let owner: UserId = crate::db::query_scalar(
            "INSERT INTO users (google_id, email) VALUES ('history-owner', 'history-owner@example.com') RETURNING id",
            vec![],
        )
        .fetch_one(&pool)
        .await
        .expect("利用者の作成失敗");
        let other: UserId = crate::db::query_scalar(
            "INSERT INTO users (google_id, email) VALUES ('history-other', 'history-other@example.com') RETURNING id",
            vec![],
        )
        .fetch_one(&pool)
        .await
        .expect("別利用者の作成失敗");
        let owner_hash = vec![11u8; 32];
        let other_hash = vec![12u8; 32];
        let expired_hash = vec![13u8; 32];
        for (user, hash) in [(owner, &owner_hash), (other, &other_hash)] {
            crate::db::query(
                "INSERT INTO login_events (user_id, result, device_hash, country) VALUES ($1, 'success', $2, 'JP')",
                vec![Bind::from(user), Bind::Bytes(hash.clone())],
            )
            .execute(&pool)
            .await
            .expect("履歴の投入失敗");
            crate::db::query(
                "INSERT INTO sessions (user_id, token_hash, device_hash) VALUES ($1, $2, $3)",
                vec![
                    Bind::from(user),
                    Bind::Bytes(vec![rand_byte(), rand_byte()]),
                    Bind::Bytes(hash.clone()),
                ],
            )
            .execute(&pool)
            .await
            .expect("セッションの投入失敗");
        }
        for hash in [&expired_hash] {
            crate::db::query(
                "INSERT INTO login_events (user_id, result, device_hash, country) VALUES ($1, 'success', $2, 'JP')",
                vec![Bind::from(owner), Bind::Bytes(hash.clone())],
            )
            .execute(&pool)
            .await
            .expect("履歴の投入失敗");
            crate::db::query(
                "INSERT INTO sessions (user_id, token_hash, device_hash, expires_at)
                 VALUES ($1, $2, $3, NOW() - INTERVAL '1 hour')",
                vec![
                    Bind::from(owner),
                    Bind::Bytes(vec![rand_byte(), rand_byte()]),
                    Bind::Bytes(hash.clone()),
                ],
            )
            .execute(&pool)
            .await
            .expect("期限切れセッションの投入失敗");
        }
        let owner_history = list_login_history(&pool, owner, 50, 0)
            .await
            .expect("履歴の取得失敗");
        assert_eq!(owner_history.len(), 2);
        let owner_devices = list_devices(&pool, owner, &owner_hash)
            .await
            .expect("端末の取得失敗");
        assert_eq!(owner_devices.len(), 1);
        assert!(owner_devices[0].is_current);
        assert_eq!(owner_devices[0].id, device_opaque_id(&owner, &owner_hash));
        let other_devices = list_devices(&pool, other, &other_hash)
            .await
            .expect("別利用者の端末取得失敗");
        assert_ne!(owner_devices[0].id, other_devices[0].id);
        assert_eq!(
            resolve_device_hash(&pool, owner, &owner_devices[0].id)
                .await
                .expect("端末IDの解決失敗"),
            Some(owner_hash.clone())
        );
        assert_eq!(
            resolve_device_hash(&pool, other, &owner_devices[0].id)
                .await
                .expect("別利用者の解決失敗"),
            None
        );
        assert_eq!(
            resolve_device_hash(&pool, owner, &device_opaque_id(&owner, &expired_hash))
                .await
                .expect("期限切れ端末の解決失敗"),
            None
        );
    }

    #[tokio::test]
    #[ignore = "requires Docker to run Postgres container"]
    async fn revoked_device_leaves_device_list() {
        let (pool, _node) = crate::test_db::start_test_pool().await;
        let user_id: UserId = crate::db::query_scalar(
            "INSERT INTO users (google_id, email) VALUES ('revoked-list-user', 'revoked-list-user@example.com') RETURNING id",
            vec![],
        )
        .fetch_one(&pool)
        .await
        .expect("利用者の作成失敗");
        let hash = vec![31u8; 32];
        crate::db::query(
            "INSERT INTO login_events (user_id, result, device_hash, country) VALUES ($1, 'success', $2, 'JP')",
            vec![Bind::from(user_id), Bind::Bytes(hash.clone())],
        )
        .execute(&pool)
        .await
        .expect("履歴の投入失敗");
        crate::db::query(
            "INSERT INTO sessions (user_id, token_hash, device_hash) VALUES ($1, $2, $3)",
            vec![
                Bind::from(user_id),
                Bind::Bytes(vec![rand_byte(), rand_byte()]),
                Bind::Bytes(hash.clone()),
            ],
        )
        .execute(&pool)
        .await
        .expect("セッションの投入失敗");
        let opaque = device_opaque_id(&user_id, &hash);
        assert_eq!(
            list_devices(&pool, user_id, &hash)
                .await
                .expect("端末の取得失敗")
                .len(),
            1
        );
        crate::services::auth::delete_device_sessions(&pool, user_id, &hash)
            .await
            .expect("端末の解除失敗");
        assert!(
            list_devices(&pool, user_id, &hash)
                .await
                .expect("解除後の取得失敗")
                .is_empty()
        );
        assert_eq!(
            resolve_device_hash(&pool, user_id, &opaque)
                .await
                .expect("解除後の解決失敗"),
            None
        );
    }

    #[tokio::test]
    #[ignore = "requires Docker to run Postgres container"]
    async fn revoke_one_device_keeps_other_sessions_and_logout_all_clears() {
        let (pool, _node) = crate::test_db::start_test_pool().await;
        let user_id: UserId = crate::db::query_scalar(
            "INSERT INTO users (google_id, email) VALUES ('revoke-user', 'revoke-user@example.com') RETURNING id",
            vec![],
        )
        .fetch_one(&pool)
        .await
        .expect("利用者の作成失敗");
        let hash_a = vec![21u8; 32];
        let hash_b = vec![22u8; 32];
        for hash in [&hash_a, &hash_b] {
            crate::db::query(
                "INSERT INTO sessions (user_id, token_hash, device_hash) VALUES ($1, $2, $3)",
                vec![
                    Bind::from(user_id),
                    Bind::Bytes(vec![rand_byte()]),
                    Bind::Bytes(hash.clone()),
                ],
            )
            .execute(&pool)
            .await
            .expect("セッションの投入失敗");
        }
        crate::db::query(
            "INSERT INTO sessions (user_id, token_hash) VALUES ($1, $2)",
            vec![Bind::from(user_id), Bind::Bytes(vec![rand_byte()])],
        )
        .execute(&pool)
        .await
        .expect("移行前のセッションの投入失敗");
        let removed = crate::services::auth::delete_device_sessions(&pool, user_id, &hash_a)
            .await
            .expect("端末の解除失敗");
        assert_eq!(removed, 2);
        let remaining: i64 = crate::db::query_scalar(
            "SELECT COUNT(*) FROM sessions WHERE user_id = $1",
            vec![Bind::from(user_id)],
        )
        .fetch_one(&pool)
        .await
        .expect("件数取得失敗");
        assert_eq!(remaining, 1);
        let remaining_hash: Vec<u8> = crate::db::query_scalar(
            "SELECT device_hash FROM sessions WHERE user_id = $1",
            vec![Bind::from(user_id)],
        )
        .fetch_one(&pool)
        .await
        .expect("残セッションの取得失敗");
        assert_eq!(remaining_hash, hash_b);
        let cleared = crate::services::auth::delete_all_sessions(&pool, user_id)
            .await
            .expect("全ログアウト失敗");
        assert_eq!(cleared, 1);
        let remaining: i64 = crate::db::query_scalar(
            "SELECT COUNT(*) FROM sessions WHERE user_id = $1",
            vec![Bind::from(user_id)],
        )
        .fetch_one(&pool)
        .await
        .expect("件数取得失敗");
        assert_eq!(remaining, 0);
    }

    fn rand_byte() -> u8 {
        use std::sync::atomic::{AtomicU8, Ordering};
        static NEXT: AtomicU8 = AtomicU8::new(1);
        NEXT.fetch_add(1, Ordering::Relaxed)
    }
}
