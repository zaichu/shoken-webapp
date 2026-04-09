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

#[cfg(test)]
mod tests {
    use super::{
        AssetBalanceDomain, CsvDomain, DividendDomain, DomesticStockDomain, MutualfundDomain,
    };

    #[rustfmt::skip]
    fn assert_valid_rows<D: CsvDomain>(lines: &[&str], expected_valid_rows: usize) { let csv = lines.join("\n"); assert_eq!(D::preview_csv(csv.as_bytes()).unwrap().valid_rows, expected_valid_rows); }

    #[test]
    fn test_domain_preview_csv_delegates() {
        assert_valid_rows::<DividendDomain>(
            &[
                "入金日,商品,口座,銘柄コード,銘柄,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]",
                "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"8591\",\"オリックス\",\"円\",\"93.76\",\"200\",\"18,752\",\"3,808\",\"14,944\"",
            ],
            1,
        );
        assert_valid_rows::<DomesticStockDomain>(
            &[
                "約定日,受渡日,銘柄コード,銘柄名,口座,信用区分,取引,数量[株],売却/決済単価[円],売却/決済額[円],平均取得価額[円],実現損益[円]",
                "\"2026/02/09\",\"2026/02/12\",\"5020\",\"ＥＮＥＯＳホールディングス\",\"特定\",\"-\",\"売付\",\"100\",\"1,441.0\",\"144,100\",\"1,350.00\",\"9,100\"",
            ],
            1,
        );
        assert_valid_rows::<MutualfundDomain>(
            &[
                "約定日,受渡日,ファンド名,分配金,口座,取引,数量[口],為替レート［円］,解約単価［円］,解約額［円］,平均取得価額［円］,実現損益［円］",
                "\"2022/10/28\",\"2022/11/2\",\"eMAXIS Slim 米国株式(S&P500)\",\"再投資型\",\"特定\",\"解約\",\"3,721,147\",\"-\",\"19,661\",\"7,316,147\",\"18,005.20\",\"615,849\"",
            ],
            1,
        );
        assert_valid_rows::<AssetBalanceDomain>(
            &[
                "■現在の評価額合計［円］,,\"9,474,000\"",
                "■評価損益合計,前日比［円］,\"288,000\"",
                ",前月比［円］,\"-120,000\"",
                ",評価損益［円］,\"2,005,900\"",
                "■特定口座",
                "",
                "銘柄コード,銘柄名,保有数量［株］,執行中［株］,(内訳　通常数量[株]),(内訳　積立数量[株]),平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］",
                "\"1605\",\"ＩＮＰＥＸ\",\"200\",\"0\",\"200\",\"0\",\"2,355.00\",\"471,000\",\"3,685.0\",\"65.0\",\"737,000\",\"56.47\"",
                "\"7974\",\"任天堂\",\"1,000\",\"0\",\"1,000\",\"0\",\"5,997.60\",\"5,997,600\",\"8,737.0\",\"223.0\",\"8,737,000\",\"45.67\"",
                ",,,,,,特定口座合計,\"11,245,249\",,,\"14,517,240\",\"29.09\"",
            ],
            2,
        );
    }
}
