use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
use crate::services::csv::pipeline::{CsvParserConfig, CsvTable, parse_csv_with_config};
use crate::services::csv::util::{CsvRowView, RowNumber};
use crate::services::domain::bulk::RowLimit;
use shared::value::UserId;
use sqlx::PgPool;
use std::future::Future;

pub fn validate_csv_rows<T, F>(table: &CsvTable, transform_row: F) -> (Vec<T>, Vec<CsvRowError>)
where
    F: Fn(&CsvRowView<'_>, RowNumber) -> Result<T, CsvRowError>,
{
    let mut items = Vec::new();
    let mut errors = Vec::new();

    for (index, row) in table.rows().enumerate() {
        let row_num = RowNumber::new(index + 1);
        match transform_row(&row, row_num) {
            Ok(item) => items.push(item),
            Err(error) => errors.push(error),
        }
    }

    (items, errors)
}

pub fn build_preview_response<T>(items: &[T], errors: Vec<CsvRowError>) -> CsvPreviewResponse
where
    T: serde::Serialize,
{
    let rows = items
        .iter()
        .map(|item| serde_json::to_value(item).unwrap_or(serde_json::Value::Null))
        .collect();

    CsvPreviewResponse {
        total_rows: items.len() + errors.len(),
        valid_rows: items.len(),
        errors,
        rows,
    }
}

/// bulk_create 結果と行エラーから CsvUploadResponse を構築
pub fn finish_csv_upload(
    result: &BulkCreateResponse,
    errors: Vec<CsvRowError>,
) -> CsvUploadResponse {
    CsvUploadResponse {
        inserted: result.inserted,
        skipped: result.skipped,
        errors,
    }
}

/// CSV bytes をパース → 行 transform → preview response 化を共通化する
pub fn build_csv_preview<T, F>(
    bytes: &[u8],
    config: &CsvParserConfig,
    transform_rows: F,
) -> Result<CsvPreviewResponse, ApiError>
where
    T: serde::Serialize,
    F: FnOnce(&CsvTable) -> (Vec<T>, Vec<CsvRowError>),
{
    let table = parse_csv_with_config(bytes, config)?;
    let (items, errors) = transform_rows(&table);
    Ok(build_preview_response(&items, errors))
}

/// CSV bytes をパース → 行 transform → bulk_create → upload response 化を共通化する
pub async fn run_csv_upload<T, F, BulkFn, BulkFut>(
    bytes: &[u8],
    config: &CsvParserConfig,
    transform_rows: F,
    bulk_create: BulkFn,
) -> Result<CsvUploadResponse, ApiError>
where
    F: FnOnce(&CsvTable) -> (Vec<T>, Vec<CsvRowError>),
    BulkFn: FnOnce(Vec<T>) -> BulkFut,
    BulkFut: Future<Output = Result<BulkCreateResponse, ApiError>>,
{
    let table = parse_csv_with_config(bytes, config)?;
    let (items, errors) = transform_rows(&table);
    let result = bulk_create(items).await?;
    Ok(finish_csv_upload(&result, errors))
}
/// CSV から取り込めるドメイン。ドメインごとに持つのはパース設定・行の読み取り・一括登録だけ
pub trait CsvImport: Send + Sync + 'static {
    type Row: serde::Serialize + Send + Sync;
    const CSV_CONFIG: CsvParserConfig;

    fn transform_rows(table: &CsvTable) -> (Vec<Self::Row>, Vec<CsvRowError>);

    fn bulk_create(
        pool: &PgPool,
        user_id: UserId,
        items: &[Self::Row],
        limit: RowLimit,
    ) -> impl Future<Output = Result<BulkCreateResponse, ApiError>> + Send;

    /// CSV バイト列をパースして DB 書き込みなしのプレビューを返す
    fn preview_csv(bytes: &[u8]) -> Result<CsvPreviewResponse, ApiError> {
        build_csv_preview(bytes, &Self::CSV_CONFIG, Self::transform_rows)
    }

    /// CSV バイト列をパースして DB に一括登録する
    fn upload_csv(
        pool: &PgPool,
        user_id: UserId,
        bytes: &[u8],
        user_row_limit: RowLimit,
    ) -> impl Future<Output = Result<CsvUploadResponse, ApiError>> + Send {
        async move {
            run_csv_upload(
                bytes,
                &Self::CSV_CONFIG,
                Self::transform_rows,
                |items| async move { Self::bulk_create(pool, user_id, &items, user_row_limit).await },
            )
            .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::csv::util::CsvCells;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn test_finish_csv_upload() {
        let response = finish_csv_upload(
            &crate::models::common::BulkCreateResponse {
                inserted: 3,
                skipped: 1,
            },
            vec![CsvRowError {
                row: 5,
                message: "エラー".to_string(),
            }],
        );
        assert_eq!(
            (
                response.inserted,
                response.skipped,
                response.errors.len(),
                response.errors.first().map(|error| error.row)
            ),
            (3, 1, 1, Some(5))
        );
    }

    const KEY_CONFIG: CsvParserConfig = CsvParserConfig {
        skip_header_rows: 0,
        exclude_row_fn: None,
    };

    #[test]
    fn test_validate_csv_rows() {
        let table = parse_csv_with_config("key\na\nb".as_bytes(), &KEY_CONFIG).unwrap();
        let (items, errors): (Vec<String>, _) =
            validate_csv_rows(&table, |row, _row_num| Ok(row.cell("key").to_string()));
        assert_eq!((items, errors.is_empty()), (strings(&["a", "b"]), true));

        let table = parse_csv_with_config("key\nok\nbad".as_bytes(), &KEY_CONFIG).unwrap();
        let (items, errors): (Vec<String>, _) = validate_csv_rows(&table, |row, row_num| {
            let value = row.cell("key").to_string();
            if value == "bad" {
                Err(CsvRowError {
                    row: row_num.get(),
                    message: "invalid".to_string(),
                })
            } else {
                Ok(value)
            }
        });
        assert_eq!(items, strings(&["ok"]));
        assert_eq!(
            (errors.len(), errors.first().map(|error| error.row)),
            (1, Some(2))
        );

        let table = parse_csv_with_config("key".as_bytes(), &KEY_CONFIG).unwrap();
        let (items, errors): (Vec<String>, _) =
            validate_csv_rows(&table, |_, _| Ok("".to_string()));
        assert_eq!((items.is_empty(), errors.is_empty()), (true, true));
    }

    const PREVIEW_TEST_CONFIG: CsvParserConfig = CsvParserConfig {
        skip_header_rows: 0,
        exclude_row_fn: None,
    };

    fn collect_names(table: &CsvTable) -> (Vec<String>, Vec<CsvRowError>) {
        let mut items = Vec::new();
        let mut errors = Vec::new();
        for (index, row) in table.rows().enumerate() {
            let name = row.cell("name").to_string();
            if name.is_empty() {
                errors.push(CsvRowError {
                    row: index + 1,
                    message: "name is empty".to_string(),
                });
            } else {
                items.push(name);
            }
        }
        (items, errors)
    }

    #[test]
    fn test_build_csv_preview() {
        let preview = build_csv_preview(
            "name,amount\nfoo,100\n,200\nbar,300\n".as_bytes(),
            &PREVIEW_TEST_CONFIG,
            collect_names,
        )
        .unwrap();
        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.errors.len(),
                preview.errors.first().map(|error| error.row),
                preview.rows.len(),
            ),
            (3, 2, 1, Some(2), 2)
        );

        assert!(matches!(
            build_csv_preview(b"", &PREVIEW_TEST_CONFIG, collect_names),
            Err(ApiError::Csv(_))
        ));
    }

    #[tokio::test]
    async fn test_run_csv_upload() {
        let response = run_csv_upload(
            "name,amount\nfoo,1\n,2\nbar,3\n".as_bytes(),
            &PREVIEW_TEST_CONFIG,
            collect_names,
            |items| async move {
                Ok(BulkCreateResponse {
                    inserted: items.len(),
                    skipped: 0,
                })
            },
        )
        .await
        .unwrap();
        assert_eq!(
            (
                response.inserted,
                response.skipped,
                response.errors.len(),
                response.errors.first().map(|error| error.row),
            ),
            (2, 0, 1, Some(2))
        );

        let bulk_invoked = std::cell::Cell::new(false);
        let result = run_csv_upload(
            b"",
            &PREVIEW_TEST_CONFIG,
            collect_names,
            |_items: Vec<String>| {
                bulk_invoked.set(true);
                async {
                    Ok(BulkCreateResponse {
                        inserted: 0,
                        skipped: 0,
                    })
                }
            },
        )
        .await;
        assert!(matches!(result, Err(ApiError::Csv(_))));
        assert!(
            !bulk_invoked.get(),
            "bulk_create must not run when parse fails"
        );
    }

    #[test]
    fn test_build_preview_response() {
        let items = vec!["alpha", "beta"];
        let errors = vec![CsvRowError {
            row: 3,
            message: "error".to_string(),
        }];
        let response = build_preview_response(&items, errors);
        assert_eq!(
            (
                response.total_rows,
                response.valid_rows,
                response.errors.len(),
                response.rows.len()
            ),
            (3, 2, 1, 2)
        );
        assert_eq!(response.errors[0].row, 3);

        let empty: Vec<String> = vec![];
        let response = build_preview_response(&empty, vec![]);
        assert_eq!((response.total_rows, response.valid_rows), (0, 0));
    }

    use crate::services::asset_balance::AssetBalanceDomain;
    use crate::services::dividend::DividendDomain;
    use crate::services::domestic_stock::DomesticStockDomain;
    use crate::services::mutualfund::MutualfundDomain;

    fn assert_valid_rows<D: CsvImport>(lines: &[&str], expected_valid_rows: usize) {
        let csv = lines.join("\n");
        assert_eq!(
            D::preview_csv(csv.as_bytes()).unwrap().valid_rows,
            expected_valid_rows
        );
    }
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
