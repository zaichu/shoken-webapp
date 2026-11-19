use crate::db::{Bind, Db, DbError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared::value::UserId;
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DeviceEntry {
    pub id: String,
    #[serde(with = "time::serde::rfc3339")]
    pub last_used_at: OffsetDateTime,
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

struct DeviceRow {
    device_hash: Vec<u8>,
    last_used_at: OffsetDateTime,
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

const DEVICE_LIST_SQL: &str = "SELECT DISTINCT ON (s.device_hash) s.device_hash,
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
         ORDER BY s.device_hash, last_used_at DESC";

const ACTIVE_DEVICE_HASHES_SQL: &str = "SELECT DISTINCT device_hash FROM sessions
         WHERE user_id = $1 AND expires_at > NOW() AND device_hash IS NOT NULL";

pub fn device_opaque_id(user_id: &UserId, device_hash: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(user_id.to_string().as_bytes());
    hasher.update(device_hash);
    hex::encode(hasher.finalize())
}

async fn fetch_device_rows(pool: &Db, user_id: UserId) -> Result<Vec<DeviceRow>, DbError> {
    crate::db::query_as::<DeviceRow>(DEVICE_LIST_SQL, vec![Bind::from(user_id)])
        .fetch_all_with_user(pool, user_id)
        .await
}

fn to_device_entries(
    user_id: &UserId,
    rows: Vec<DeviceRow>,
    current_device_hash: &[u8],
) -> Vec<DeviceEntry> {
    let mut devices: Vec<DeviceEntry> = rows
        .into_iter()
        .map(|row| DeviceEntry {
            id: device_opaque_id(user_id, &row.device_hash),
            last_used_at: row.last_used_at,
            country: row.country,
            region: row.region,
            browser: row.browser,
            os: row.os,
            is_current: row.device_hash == current_device_hash,
        })
        .collect();
    devices.sort_by_key(|a| std::cmp::Reverse(a.last_used_at));
    devices
}

pub async fn list_devices(
    pool: &Db,
    user_id: UserId,
    current_device_hash: &[u8],
) -> Result<Vec<DeviceEntry>, DbError> {
    let rows = fetch_device_rows(pool, user_id).await?;
    Ok(to_device_entries(&user_id, rows, current_device_hash))
}

async fn fetch_device_hashes(pool: &Db, user_id: UserId) -> Result<Vec<Vec<u8>>, DbError> {
    let rows = crate::db::query_as::<HashRow>(ACTIVE_DEVICE_HASHES_SQL, vec![Bind::from(user_id)])
        .fetch_all_with_user(pool, user_id)
        .await?;
    Ok(rows.into_iter().map(|row| row.device_hash).collect())
}

fn find_hash_for_opaque_id(
    user_id: &UserId,
    hashes: Vec<Vec<u8>>,
    opaque_id: &str,
) -> Option<Vec<u8>> {
    hashes
        .into_iter()
        .find(|hash| device_opaque_id(user_id, hash) == opaque_id)
}

pub async fn resolve_device_hash(
    pool: &Db,
    user_id: UserId,
    opaque_id: &str,
) -> Result<Option<Vec<u8>>, DbError> {
    let hashes = fetch_device_hashes(pool, user_id).await?;
    Ok(find_hash_for_opaque_id(&user_id, hashes, opaque_id))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn row(hash_byte: u8, last_used_at: OffsetDateTime) -> DeviceRow {
        DeviceRow {
            device_hash: vec![hash_byte; 32],
            last_used_at,
            country: Some("JP".to_string()),
            region: None,
            browser: None,
            os: None,
        }
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
        let raw_hex = hex::encode(hash);
        assert!(!id.contains(&raw_hex));
    }

    #[test]
    fn device_entries_mark_current_and_sort_newest_first() {
        let user_id = UserId::from(uuid::Uuid::new_v4());
        let older = OffsetDateTime::from_unix_timestamp(1767225600).unwrap();
        let newer = OffsetDateTime::from_unix_timestamp(1767312000).unwrap();
        let entries = to_device_entries(&user_id, vec![row(1, older), row(2, newer)], &[1u8; 32]);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].last_used_at, newer);
        assert!(!entries[0].is_current);
        assert_eq!(entries[1].last_used_at, older);
        assert!(entries[1].is_current);
        assert_eq!(entries[1].id, device_opaque_id(&user_id, &[1u8; 32]));
    }

    #[test]
    fn opaque_match_is_user_scoped_and_rejects_unknown() {
        let owner = UserId::from(uuid::Uuid::new_v4());
        let other = UserId::from(uuid::Uuid::new_v4());
        let hash = vec![11u8; 32];
        let opaque = device_opaque_id(&owner, &hash);
        assert_eq!(
            find_hash_for_opaque_id(&owner, vec![hash.clone()], &opaque),
            Some(hash.clone())
        );
        assert_eq!(
            find_hash_for_opaque_id(&other, vec![hash.clone()], &opaque),
            None
        );
        assert_eq!(
            find_hash_for_opaque_id(&owner, vec![hash], "no-such-id"),
            None
        );
        assert_eq!(find_hash_for_opaque_id(&owner, vec![], &opaque), None);
    }

    #[test]
    fn device_response_carries_no_secret_fields() {
        let device = DeviceEntry {
            id: "opaque".to_string(),
            last_used_at: time::OffsetDateTime::now_utc(),
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
}
