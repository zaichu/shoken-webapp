use crate::{
    db::{Bind, Db, DbError, Tx},
    errors::ApiError,
};
use serde::{Deserialize, Serialize};
use shared::value::UserId;
use time::OffsetDateTime;
use utoipa::ToSchema;

use super::client_info::ClientInfo;

pub const MAX_EVENTS_PER_USER: i64 = 500;
pub const MAX_HISTORY_LIMIT: i64 = 100;
pub const DEFAULT_HISTORY_LIMIT: i64 = 50;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct LoginHistoryEntry {
    #[serde(with = "time::serde::rfc3339")]
    pub occurred_at: OffsetDateTime,
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

pub fn clamp_history_limit(limit: Option<i64>) -> i64 {
    match limit {
        Some(n) if n > 0 => n.min(MAX_HISTORY_LIMIT),
        _ => DEFAULT_HISTORY_LIMIT,
    }
}

pub const fn clamp_history_offset(offset: Option<i64>) -> i64 {
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
    .fetch_all_with_user(pool, user_id)
    .await
}

async fn insert_login_event_row(
    tx: &mut Tx,
    user_id: UserId,
    outcome: &str,
    device_hash: Vec<u8>,
    client: &ClientInfo,
) -> Result<(), DbError> {
    crate::db::query(
        "INSERT INTO login_events (user_id, result, device_hash, country, region, browser, os)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        vec![
            Bind::from(user_id),
            Bind::from(outcome),
            Bind::Bytes(device_hash),
            Bind::from(client.country.clone()),
            Bind::from(client.region.clone()),
            Bind::from(client.browser.clone()),
            Bind::from(client.os.clone()),
        ],
    )
    .execute(&mut *tx)
    .await?;
    Ok(())
}

async fn trim_login_events_to_cap(tx: &mut Tx, user_id: UserId) -> Result<(), DbError> {
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

pub async fn insert_login_event_in_tx(
    tx: &mut Tx,
    user_id: UserId,
    outcome: super::LoginOutcome,
    device_hash: Vec<u8>,
    client: &ClientInfo,
) -> Result<(), DbError> {
    insert_login_event_row(tx, user_id, outcome.as_str(), device_hash, client).await?;
    trim_login_events_to_cap(tx, user_id).await?;
    Ok(())
}

pub async fn prune_login_events(pool: &Db) -> Result<u64, ApiError> {
    let mut tx = pool.begin().await?;
    let deleted = crate::db::query_scalar::<i64>(
        "SELECT public.prune_login_events_older_than_90_days_v2()",
        vec![],
    )
    .fetch_one(&mut tx)
    .await?;
    tx.commit().await?;
    u64::try_from(deleted).map_err(|_| ApiError::Internal("履歴削除件数が不正です"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn history_limit_is_clamped() {
        assert_eq!(clamp_history_limit(None), DEFAULT_HISTORY_LIMIT);
        assert_eq!(clamp_history_limit(Some(0)), DEFAULT_HISTORY_LIMIT);
        assert_eq!(clamp_history_limit(Some(-5)), DEFAULT_HISTORY_LIMIT);
        assert_eq!(clamp_history_limit(Some(10)), 10);
        assert_eq!(
            clamp_history_limit(Some(MAX_HISTORY_LIMIT)),
            MAX_HISTORY_LIMIT
        );
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
    fn history_response_carries_no_secret_fields() {
        let entry = LoginHistoryEntry {
            occurred_at: time::OffsetDateTime::now_utc(),
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
    }

    #[test]
    fn history_response_keeps_rfc3339_timestamp_string() {
        let entry = LoginHistoryEntry {
            occurred_at: OffsetDateTime::from_unix_timestamp(1_705_281_825).unwrap(),
            result: "success".to_string(),
            country: None,
            region: None,
            browser: None,
            os: None,
        };

        let json = serde_json::to_value(entry).unwrap();
        assert_eq!(json["occurred_at"], "2024-01-15T01:23:45Z");
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
