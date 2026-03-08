use crate::errors::ApiError;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

/// ユーザーに紐づく全レコードを削除する共通実装。
/// テーブル名は呼び出し元がリテラルで指定するため SQL インジェクションの危険はない。
pub async fn delete_all_for_user(
    pool: &PgPool,
    user_id: Uuid,
    table_name: &'static str,
    domain: &'static str,
) -> Result<u64, ApiError> {
    info!("[{}.delete_all] リクエスト受信", domain);
    let result = sqlx::query(&format!("DELETE FROM {} WHERE user_id = $1", table_name))
        .bind(user_id)
        .execute(pool)
        .await?;
    let deleted = result.rows_affected();
    info!("[{}.delete_all] 完了: {}件削除", domain, deleted);
    Ok(deleted)
}
