use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::ApiError,
    models::csv_import::{CsvPreviewResponse, CsvUploadResponse},
};

/// CSV アップロードをサポートするドメインのトレイト
///
/// 新規ドメインを追加する場合は本トレイトを実装し、
/// ハンドラーから `handle_preview_csv<D>` / `handle_upload_csv<D>` を呼ぶだけでよい。
#[async_trait]
pub trait CsvDomain: Send + Sync + 'static {
    /// CSV バイト列をパースして DB 書き込みなしのプレビューを返す
    fn preview_csv(bytes: &[u8]) -> Result<CsvPreviewResponse, ApiError>;

    /// CSV バイト列をパースして DB に一括登録する
    async fn upload_csv(
        pool: &PgPool,
        user_id: Uuid,
        bytes: &[u8],
    ) -> Result<CsvUploadResponse, ApiError>;
}

/// 配当金ドメイン
pub struct DividendDomain;

/// 国内株式ドメイン
pub struct DomesticStockDomain;

/// 投資信託ドメイン
pub struct MutualfundDomain;

/// 資産残高（保有銘柄）ドメイン
pub struct AssetBalanceDomain;

#[async_trait]
impl CsvDomain for DividendDomain {
    fn preview_csv(bytes: &[u8]) -> Result<CsvPreviewResponse, ApiError> {
        crate::services::dividend::preview_csv(bytes)
    }

    async fn upload_csv(
        pool: &PgPool,
        user_id: Uuid,
        bytes: &[u8],
    ) -> Result<CsvUploadResponse, ApiError> {
        crate::services::dividend::upload_csv(pool, user_id, bytes).await
    }
}

#[async_trait]
impl CsvDomain for DomesticStockDomain {
    fn preview_csv(bytes: &[u8]) -> Result<CsvPreviewResponse, ApiError> {
        crate::services::domestic_stock::preview_csv(bytes)
    }

    async fn upload_csv(
        pool: &PgPool,
        user_id: Uuid,
        bytes: &[u8],
    ) -> Result<CsvUploadResponse, ApiError> {
        crate::services::domestic_stock::upload_csv(pool, user_id, bytes).await
    }
}

#[async_trait]
impl CsvDomain for MutualfundDomain {
    fn preview_csv(bytes: &[u8]) -> Result<CsvPreviewResponse, ApiError> {
        crate::services::mutualfund::preview_csv(bytes)
    }

    async fn upload_csv(
        pool: &PgPool,
        user_id: Uuid,
        bytes: &[u8],
    ) -> Result<CsvUploadResponse, ApiError> {
        crate::services::mutualfund::upload_csv(pool, user_id, bytes).await
    }
}

#[async_trait]
impl CsvDomain for AssetBalanceDomain {
    fn preview_csv(bytes: &[u8]) -> Result<CsvPreviewResponse, ApiError> {
        crate::services::asset_balance::preview_csv(bytes)
    }

    async fn upload_csv(
        pool: &PgPool,
        user_id: Uuid,
        bytes: &[u8],
    ) -> Result<CsvUploadResponse, ApiError> {
        crate::services::asset_balance::upload_csv(pool, user_id, bytes).await
    }
}
