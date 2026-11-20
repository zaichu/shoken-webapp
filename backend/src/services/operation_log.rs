use crate::db::{Bind, Db, DbError, Tx};
use shared::value::UserId;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct OperationLogEntry {
    #[serde(with = "time::serde::rfc3339")]
    pub occurred_at: OffsetDateTime,
    pub operation: String,
    pub target: String,
    pub count: i64,
}

impl crate::db::FromRow for OperationLogEntry {
    fn from_row(row: &crate::db::Row) -> Result<Self, DbError> {
        Ok(Self {
            occurred_at: row.try_get("occurred_at")?,
            operation: row.try_get("operation")?,
            target: row.try_get("target")?,
            count: row.try_get("count")?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct OperationLogListResponse {
    pub data: Vec<OperationLogEntry>,
}

pub async fn record(
    tx: &mut Tx,
    user_id: UserId,
    operation: &str,
    target: &str,
    count: u64,
) -> Result<(), DbError> {
    crate::db::query(
        "INSERT INTO operation_logs (user_id, operation, target, count) VALUES ($1, $2, $3, $4)",
        vec![
            Bind::from(user_id),
            Bind::from(operation.to_string()),
            Bind::from(target.to_string()),
            Bind::from(i64::try_from(count).unwrap_or(i64::MAX)),
        ],
    )
    .execute(&mut *tx)
    .await?;
    Ok(())
}

pub async fn list(
    pool: &Db,
    user_id: UserId,
    limit: i64,
    offset: i64,
) -> Result<Vec<OperationLogEntry>, DbError> {
    crate::db::query_as::<OperationLogEntry>(
        "SELECT occurred_at, operation, target, count FROM operation_logs
         WHERE user_id = $1 ORDER BY occurred_at DESC, id DESC LIMIT $2 OFFSET $3",
        vec![Bind::from(user_id), Bind::from(limit), Bind::from(offset)],
    )
    .fetch_all_with_user(pool, user_id)
    .await
}

pub async fn prune(pool: &Db) -> Result<u64, DbError> {
    let mut tx = pool.begin().await?;
    let deleted = crate::db::query_scalar::<i64>(
        "SELECT public.prune_operation_logs_older_than_one_year_v1()",
        vec![],
    )
    .fetch_one(&mut tx)
    .await?;
    tx.commit().await?;
    u64::try_from(deleted).map_err(|_| DbError::Other("操作記録削除件数が不正です".to_string()))
}
