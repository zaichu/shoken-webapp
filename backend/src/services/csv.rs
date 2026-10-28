pub mod import;
pub mod pipeline;
pub mod stats;
pub mod util;

use crate::db::{self, Bind, Db};
use crate::errors::ApiError;
use crate::models::csv_import::CsvUploadResponse;
use shared::value::UserId;

pub async fn record_import_success(
    pool: &Db,
    user_id: UserId,
    kind: &str,
    response: &CsvUploadResponse,
) -> Result<(), ApiError> {
    if response.inserted == 0 && response.skipped == 0 && !response.errors.is_empty() {
        return Ok(());
    }
    db::query(
        "INSERT INTO csv_import_history (user_id, kind, last_imported_at)
         VALUES ($1, $2, NOW())
         ON CONFLICT (user_id, kind) DO UPDATE SET last_imported_at = EXCLUDED.last_imported_at",
        vec![Bind::from(user_id), Bind::from(kind)],
    )
    .execute(pool)
    .await?;
    Ok(())
}
