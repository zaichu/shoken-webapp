pub mod admin;
pub mod client_info;
pub mod devices;
pub mod history;

pub use client_info::{ClientInfo, client_info_from_headers, parse_user_agent};
pub use devices::{
    DeviceEntry, DeviceListResponse, device_opaque_id, list_devices, resolve_device_hash,
    revoke_device,
};
pub use history::{
    DEFAULT_HISTORY_LIMIT, LoginHistoryEntry, LoginHistoryListResponse, MAX_EVENTS_PER_USER,
    MAX_HISTORY_LIMIT, clamp_history_limit, clamp_history_offset, insert_login_event_in_tx,
    list_login_history, prune_login_events,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginOutcome {
    Success,
    NewDevice,
    LocationChanged,
}

impl LoginOutcome {
    pub const fn as_str(self) -> &'static str {
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::db::Bind;
    use shared::value::UserId;

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
        assert_eq!(
            list_devices(&pool, user_id, &hash)
                .await
                .expect("解除後の取得失敗"),
            [] as [DeviceEntry; 0]
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
