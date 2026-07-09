use crate::errors::ApiError;
use crate::models::asset_balance::{
    AssetBalance, AssetBalanceSearchQueryParams, AssetBalanceSummary, CreateAssetBalanceRequest,
};
use crate::models::common::{
    BulkCreateResponse, FacetOption, PaginatedSearchResponse, SearchFacets,
};
use crate::models::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
use crate::services::csv_import::{build_csv_preview, run_csv_upload, validate_csv_rows};
#[cfg(test)]
use crate::services::csv_pipeline::parse_csv_with_config;
use crate::services::csv_pipeline::{CsvParserConfig, CsvRow};
use crate::services::csv_util::{
    get_row_cell, normalize_security_name, parse_number, parse_optional_string_row,
};
use crate::services::shared::{
    delete_all_for_user, user_ids_for_bulk_insert, BulkTimer, DeleteTarget,
};
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, QueryBuilder};
use tracing::info;
use uuid::Uuid;

const ASSET_BALANCE_CSV_CONFIG: CsvParserConfig = CsvParserConfig {
    skip_header_rows: 6,
    exclude_row_fn: Some(is_account_summary_row),
    required_columns: &[
        "銘柄コード",
        "銘柄名",
        "保有数量［株］",
        "執行中［株］",
        "平均取得価額［円］",
        "取得総額［円］",
        "現在値［円］",
        "現在値（前日比）［円］",
        "時価評価額［円］",
        "評価損益［％］",
    ],
};

/// 保有銘柄検索条件を SQL 条件へ変換した中間表現
///
/// asset_balances には snapshot 日付がないため、date 系の条件は扱わない。
#[derive(Debug)]
struct AssetBalanceFilter {
    tokens: Vec<String>,
    security_code: Option<String>,
    security_name: Option<String>,
}

impl AssetBalanceFilter {
    fn from_params(params: &AssetBalanceSearchQueryParams) -> Self {
        let tokens = params
            .search
            .q
            .as_deref()
            .map(|q| q.split_whitespace().map(str::to_string).collect())
            .unwrap_or_default();

        Self {
            tokens,
            security_code: params.security_code.clone(),
            security_name: params.security_name.clone(),
        }
    }
}

/// ILIKE の wildcard 文字（%, _, \）をリテラル扱いにエスケープしてから前後を % で囲む
fn escape_like_pattern(token: &str) -> String {
    let escaped = token
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

/// user_id と検索条件を WHERE 句として QueryBuilder へ積む
fn push_filters(qb: &mut QueryBuilder<Postgres>, user_id: Uuid, filter: &AssetBalanceFilter) {
    qb.push(" WHERE user_id = ").push_bind(user_id);
    if let Some(v) = &filter.security_code {
        qb.push(" AND security_code = ").push_bind(v.clone());
    }
    if let Some(v) = &filter.security_name {
        qb.push(" AND security_name = ").push_bind(v.clone());
    }
    for token in &filter.tokens {
        let pattern = escape_like_pattern(token);
        qb.push(" AND (security_code ILIKE ")
            .push_bind(pattern.clone())
            .push(" ESCAPE '\\' OR security_name ILIKE ")
            .push_bind(pattern)
            .push(" ESCAPE '\\')");
    }
}

/// 認証ユーザーの保有銘柄一覧を検索（ページネーション・summary・facets 対応）
pub async fn search(
    pool: &PgPool,
    user_id: Uuid,
    params: &AssetBalanceSearchQueryParams,
) -> Result<PaginatedSearchResponse<AssetBalance, AssetBalanceSummary, SearchFacets>, ApiError> {
    info!("[asset_balance.search] リクエスト受信");
    let filter = AssetBalanceFilter::from_params(params);

    let mut count_qb: QueryBuilder<Postgres> =
        QueryBuilder::new("SELECT COUNT(*) FROM asset_balances");
    push_filters(&mut count_qb, user_id, &filter);
    let count_fut = async move {
        let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await?;
        Ok::<_, ApiError>(total)
    };

    let mut data_qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT id, user_id, security_code, security_name, shares, executing_shares, \
         average_purchase_price, total_purchase_amount, current_price, daily_change, \
         market_value, profit_loss_rate, created_at, updated_at \
         FROM asset_balances",
    );
    push_filters(&mut data_qb, user_id, &filter);
    data_qb.push(" ORDER BY security_code ASC, id ASC LIMIT ");
    data_qb.push_bind(params.per_page());
    data_qb.push(" OFFSET ");
    data_qb.push_bind(params.offset());
    let data_fut = async move {
        let data = data_qb
            .build_query_as::<AssetBalance>()
            .fetch_all(pool)
            .await?;
        Ok::<_, ApiError>(data)
    };

    let summary_fut = async {
        if params.should_include_summary() {
            fetch_summary(pool, user_id, &filter).await.map(Some)
        } else {
            Ok(None)
        }
    };

    let facets_fut = async {
        if params.should_include_facets() {
            fetch_facets(pool, user_id, &filter).await.map(Some)
        } else {
            Ok(None)
        }
    };

    // count/data/summary/facets は相互に依存しないため並行実行する
    let (total, data, summary, facets) =
        tokio::try_join!(count_fut, data_fut, summary_fut, facets_fut)?;

    Ok(PaginatedSearchResponse {
        data,
        total,
        page: params.page(),
        per_page: params.per_page(),
        summary,
        facets,
    })
}

/// 検索条件全体の summary を算出する（ページ内だけでなく検索条件全体の合計）
async fn fetch_summary(
    pool: &PgPool,
    user_id: Uuid,
    filter: &AssetBalanceFilter,
) -> Result<AssetBalanceSummary, ApiError> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT COALESCE(SUM(market_value), 0) AS total_market_value, \
         COALESCE(SUM(total_purchase_amount), 0) AS total_purchase_amount, \
         COALESCE(SUM(daily_change), 0) AS total_daily_change \
         FROM asset_balances",
    );
    push_filters(&mut qb, user_id, filter);
    Ok(qb
        .build_query_as::<AssetBalanceSummary>()
        .fetch_one(pool)
        .await?)
}

async fn fetch_facets(
    pool: &PgPool,
    user_id: Uuid,
    filter: &AssetBalanceFilter,
) -> Result<SearchFacets, ApiError> {
    let securities = fetch_security_facets(pool, user_id, filter).await?;

    Ok(SearchFacets {
        products: None,
        accounts: None,
        securities: Some(securities),
        funds: None,
        years: None,
        year_months: None,
    })
}

/// 銘柄コードごとに銘柄名・件数を facets として返す
async fn fetch_security_facets(
    pool: &PgPool,
    user_id: Uuid,
    filter: &AssetBalanceFilter,
) -> Result<Vec<FacetOption>, ApiError> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT security_code AS value, \
         (ARRAY_AGG(security_name ORDER BY id))[1] AS label, \
         COUNT(*) AS count \
         FROM asset_balances",
    );
    push_filters(&mut qb, user_id, filter);
    qb.push(" GROUP BY security_code ORDER BY security_code");
    Ok(qb.build_query_as::<FacetOption>().fetch_all(pool).await?)
}

/// 保有銘柄を一括登録（既存データを全削除してから挿入）
pub async fn bulk_create(
    pool: &PgPool,
    user_id: Uuid,
    items: &[CreateAssetBalanceRequest],
) -> Result<BulkCreateResponse, ApiError> {
    let total = items.len();
    let timer = BulkTimer::new("asset_balance", total);

    // 各フィールドを配列に変換
    let user_ids = user_ids_for_bulk_insert(user_id, total);
    let security_codes: Vec<&str> = items.iter().map(|i| i.security_code.as_str()).collect();
    let security_names: Vec<&str> = items.iter().map(|i| i.security_name.as_str()).collect();
    let shares: Vec<Decimal> = items.iter().map(|i| i.shares).collect();
    let executing_shares: Vec<Decimal> = items.iter().map(|i| i.executing_shares).collect();
    let average_purchase_prices: Vec<Decimal> =
        items.iter().map(|i| i.average_purchase_price).collect();
    let total_purchase_amounts: Vec<Decimal> =
        items.iter().map(|i| i.total_purchase_amount).collect();
    let current_prices: Vec<Decimal> = items.iter().map(|i| i.current_price).collect();
    let daily_changes: Vec<Decimal> = items.iter().map(|i| i.daily_change).collect();
    let market_values: Vec<Decimal> = items.iter().map(|i| i.market_value).collect();
    let profit_loss_rates: Vec<Decimal> = items.iter().map(|i| i.profit_loss_rate).collect();

    // トランザクション内で全削除 → 全件挿入（スナップショット置き換え）
    let mut tx = pool.begin().await?;

    // ユーザー単位のadvisory lockで並行bulk_createを直列化（READ COMMITTEDでのA∪B混入を防止）
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1::text))")
        .bind(user_id.to_string())
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM asset_balances WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    if !items.is_empty() {
        sqlx::query(
            r#"
            INSERT INTO asset_balances (user_id, security_code, security_name, shares, executing_shares,
                                        average_purchase_price, total_purchase_amount, current_price,
                                        daily_change, market_value, profit_loss_rate)
            SELECT * FROM UNNEST(
                $1::uuid[], $2::text[], $3::text[], $4::numeric[], $5::numeric[],
                $6::numeric[], $7::numeric[], $8::numeric[], $9::numeric[], $10::numeric[], $11::numeric[]
            )
            "#,
        )
        .bind(&user_ids)
        .bind(&security_codes)
        .bind(&security_names)
        .bind(&shares)
        .bind(&executing_shares)
        .bind(&average_purchase_prices)
        .bind(&total_purchase_amounts)
        .bind(&current_prices)
        .bind(&daily_changes)
        .bind(&market_values)
        .bind(&profit_loss_rates)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(timer.finish(total))
}

/// CSV bytes をパースしてプレビュー情報を返す（DB 書き込みなし）
/// 現在の取込対象形式では、先頭6行はメタデータのためスキップ
pub fn preview_csv(bytes: &[u8]) -> Result<CsvPreviewResponse, ApiError> {
    build_csv_preview(
        bytes,
        &ASSET_BALANCE_CSV_CONFIG,
        transform_asset_balance_rows,
    )
}

/// CSV bytes をパースして保有銘柄を一括登録
pub async fn upload_csv(
    pool: &PgPool,
    user_id: Uuid,
    bytes: &[u8],
) -> Result<CsvUploadResponse, ApiError> {
    run_csv_upload(
        bytes,
        &ASSET_BALANCE_CSV_CONFIG,
        transform_asset_balance_rows,
        |items| async move { bulk_create(pool, user_id, &items).await },
    )
    .await
}

/// 認証ユーザーの保有銘柄を全削除
pub async fn delete_all(pool: &PgPool, user_id: Uuid) -> Result<u64, ApiError> {
    delete_all_for_user(pool, user_id, DeleteTarget::AssetBalances).await
}

fn transform_asset_balance_rows(
    rows: &[CsvRow],
) -> (Vec<CreateAssetBalanceRequest>, Vec<CsvRowError>) {
    let (items, errors) = validate_csv_rows(rows, transform_asset_balance_row);
    let items = items
        .into_iter()
        .filter(|i| !i.security_code.is_empty())
        .collect();
    (items, errors)
}

/// 保有銘柄 CSV に混ざる「特定口座合計」などの口座集計行を除外する
///
/// 先頭フィールド（銘柄コード）が空の行かつ「口座合計」を含む行のみ除外する。
/// 銘柄名に「口座合計」を含む銘柄を誤除外しないよう、先頭が空であることを条件とする。
fn is_account_summary_row(row: &CsvRow) -> bool {
    get_row_cell(row, "銘柄コード").trim().is_empty()
        && row.values().any(|value| value.contains("口座合計"))
}

/// 保有銘柄 CSV の1行をパースして CreateAssetBalanceRequest に変換
///
/// 数値パース失敗は CsvRowError として返す。
/// ただし以下の列は "-" / 空欄が仕様上ありうるため 0.0 フォールバックを維持する:
///   - 執行中: 執行中の注文がなければ "-" または空欄
///   - 現在値（前日比）: 変動なし時は 0 または "-"
///   - 評価損益（%）: NISA 等で表示されない場合に "-"
fn transform_asset_balance_row(
    row: &CsvRow,
    row_num: usize,
) -> Result<CreateAssetBalanceRequest, CsvRowError> {
    let num = |col: &str| {
        let raw = parse_optional_string_row(row, col);
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed == "-" {
            return Err(CsvRowError {
                row: row_num,
                message: format!("必須列 '{}' が空または値なし", col),
            });
        }
        parse_number(trimmed).map_err(|e| CsvRowError {
            row: row_num,
            message: format!("{}: {}", col, e),
        })
    };

    let security_code = parse_optional_string_row(row, "銘柄コード").replace('"', "");
    Ok(CreateAssetBalanceRequest {
        security_code,
        security_name: normalize_security_name(&parse_optional_string_row(row, "銘柄名")),
        shares: num("保有数量［株］")?,
        // 執行中は "-" / 空欄が仕様上ありうるため 0.0 フォールバック
        executing_shares: parse_number(&parse_optional_string_row(row, "執行中［株］"))
            .unwrap_or(Decimal::ZERO),
        average_purchase_price: num("平均取得価額［円］")?,
        total_purchase_amount: num("取得総額［円］")?,
        current_price: num("現在値［円］")?,
        // 前日比は変動なし時に 0 または "-" が仕様上ありうるため 0.0 フォールバック
        daily_change: parse_number(&parse_optional_string_row(row, "現在値（前日比）［円］"))
            .unwrap_or(Decimal::ZERO),
        market_value: num("時価評価額［円］")?,
        // 評価損益は NISA 等で表示されない場合に "-" が仕様上ありうるため 0.0 フォールバック
        profit_loss_rate: parse_number(&parse_optional_string_row(row, "評価損益［％］"))
            .unwrap_or(Decimal::ZERO),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    type ExpectedRow<'a> = (
        &'a str,
        Decimal,
        Decimal,
        Decimal,
        Decimal,
        Decimal,
        Decimal,
    );

    const HEADER: &str =
        "銘柄コード,銘柄名,保有数量［株］,執行中［株］,平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］";
    const ASSET_BALANCE_CSV_HEADER: &str =
        "銘柄コード,銘柄名,保有数量［株］,執行中［株］,(内訳　通常数量[株]),(内訳　積立数量[株]),平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］";

    const TEST_ROW_1: &str = "1234,テスト株式会社,100,0,100,0,1500,150000,1600,10,160000,6.67";
    const TEST_ROW_2: &str = "5678,サンプル株式会社,200,0,200,0,1800,360000,1900,15,380000,5.56";
    const INPEX_ROW: &str =
        "\"1605\",\"ＩＮＰＥＸ\",\"200\",\"0\",\"200\",\"0\",\"2,355.00\",\"471,000\",\"3,685.0\",\"65.0\",\"737,000\",\"56.47\"";
    const NINTENDO_ROW: &str =
        "\"7974\",\"任天堂\",\"1,000\",\"0\",\"1,000\",\"0\",\"5,997.60\",\"5,997,600\",\"8,737.0\",\"223.0\",\"8,737,000\",\"45.67\"";
    const ACCOUNT_SUMMARY_ROW: &str =
        ",,,,,,特定口座合計,\"11,245,249\",,,\"14,517,240\",\"29.09\"";

    fn parse_row_csv(row: &str) -> (Vec<CreateAssetBalanceRequest>, Vec<CsvRowError>) {
        let rows = parse_csv_with_config(
            format!("{HEADER}\n{row}\n").as_bytes(),
            &CsvParserConfig {
                skip_header_rows: 0,
                exclude_row_fn: None,
                required_columns: ASSET_BALANCE_CSV_CONFIG.required_columns,
            },
        )
        .unwrap();

        transform_asset_balance_rows(&rows)
    }

    fn make_asset_balance_csv(rows: &[&str]) -> String {
        format!(
            "■現在の評価額合計［円］,,\"9,474,000\"\n\
             ■評価損益合計,前日比［円］,\"288,000\"\n\
             ,前月比［円］,\"-120,000\"\n\
             ,評価損益［円］,\"2,005,900\"\n\
             ■特定口座\n\n\
             {ASSET_BALANCE_CSV_HEADER}\n{}",
            rows.join("\n")
        )
    }

    fn assert_row_ok(row: &str, expected: ExpectedRow<'_>) {
        let (items, errors) = parse_row_csv(row);

        assert!(errors.is_empty(), "unexpected errors for {row}: {errors:?}");
        assert_eq!(items.len(), 1, "row should produce one item: {row}");

        let item = &items[0];
        assert_eq!(
            (
                item.security_code.as_str(),
                item.shares,
                item.executing_shares,
                item.average_purchase_price,
                item.current_price,
                item.daily_change,
                item.profit_loss_rate,
            ),
            expected
        );
    }

    fn assert_row_error(row: &str, expected_message: &str) {
        let (items, errors) = parse_row_csv(row);

        assert!(items.is_empty(), "invalid row must not be parsed: {row}");
        assert_eq!(errors.len(), 1, "row should produce one error: {row}");
        assert!(errors[0].message.contains(expected_message), "{errors:?}");
    }

    #[test]
    fn test_parse_asset_balance_row() {
        for (row, expected) in [
            (
                "1234,テスト株式会社,100,-,1500,150000,1600,10,160000,6.67",
                (
                    "1234",
                    dec!(100),
                    Decimal::ZERO,
                    dec!(1500),
                    dec!(1600),
                    dec!(10),
                    dec!(6.67),
                ),
            ),
            (
                "5678,ファンド,50,-,2000,100000,2100,-,105000,-",
                (
                    "5678",
                    dec!(50),
                    Decimal::ZERO,
                    dec!(2000),
                    dec!(2100),
                    Decimal::ZERO,
                    Decimal::ZERO,
                ),
            ),
        ] {
            assert_row_ok(row, expected);
        }

        for (row, expected_message) in [
            ("1234,テスト,N/A,-,1500,150000,1600,0,160000,0", "保有数量"),
            ("1234,テスト,-,-,1500,150000,1600,0,160000,0", "保有数量"),
            ("1234,テスト,100,-,1500,150000,N/A,0,160000,0", "現在値"),
        ] {
            assert_row_error(row, expected_message);
        }

        let preview = preview_csv(
            make_asset_balance_csv(&[INPEX_ROW, NINTENDO_ROW, ACCOUNT_SUMMARY_ROW]).as_bytes(),
        )
        .unwrap();
        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.rows.len(),
                preview.errors.is_empty(),
            ),
            (2, 2, 2, true),
            "unexpected errors: {:?}",
            preview.errors
        );

        assert!(matches!(
            preview_csv(b""),
            Err(ApiError::ValidationError(_))
        ));

        for (rows, expected_codes) in [
            (
                &[TEST_ROW_1, ",,,,,,,,,,,", TEST_ROW_2][..],
                &["1234", "5678"][..],
            ),
            (
                &[INPEX_ROW, NINTENDO_ROW, ACCOUNT_SUMMARY_ROW][..],
                &["1605", "7974"][..],
            ),
        ] {
            let rows = parse_csv_with_config(
                make_asset_balance_csv(rows).as_bytes(),
                &ASSET_BALANCE_CSV_CONFIG,
            )
            .unwrap();
            let (items, errors) = transform_asset_balance_rows(&rows);

            assert!(errors.is_empty(), "unexpected errors: {errors:?}");
            assert_eq!(
                items
                    .iter()
                    .map(|item| item.security_code.as_str())
                    .collect::<Vec<_>>(),
                expected_codes
            );
        }
    }

    #[test]
    fn test_asset_balance_search_query_params_delegate_include_flags() {
        let mut params = AssetBalanceSearchQueryParams::default();
        assert!(!params.should_include_summary());
        assert!(!params.should_include_facets());

        params.search.include_summary = Some(true);
        params.search.include_facets = Some(true);
        assert!(params.should_include_summary());
        assert!(params.should_include_facets());
    }

    #[test]
    fn test_push_filters_combines_q_tokens_as_and_of_or_across_all_columns() {
        let mut params = AssetBalanceSearchQueryParams::default();
        params.search.q = Some("AA BB".to_string());
        let filter = AssetBalanceFilter::from_params(&params);

        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT 1 FROM asset_balances");
        push_filters(&mut qb, Uuid::nil(), &filter);
        let sql = qb.sql();
        let sql = sql.as_str();

        // token ごとに 1 つの AND (...) ブロックが生成され、ブロック内は2カラムの OR になる
        assert_eq!(sql.matches(" AND (").count(), 2);
        assert_eq!(sql.matches("security_code ILIKE").count(), 2);
        assert_eq!(sql.matches("security_name ILIKE").count(), 2);
    }

    #[test]
    fn test_escape_like_pattern_escapes_wildcard_characters() {
        assert_eq!(escape_like_pattern("abc"), "%abc%");
        assert_eq!(escape_like_pattern("50%"), "%50\\%%");
        assert_eq!(escape_like_pattern("A_B"), "%A\\_B%");
        assert_eq!(escape_like_pattern("a\\b"), "%a\\\\b%");
        assert_eq!(escape_like_pattern("100%_off\\"), "%100\\%\\_off\\\\%");
    }

    #[test]
    fn test_push_filters_binds_asset_balance_specific_field_filters() {
        let params = AssetBalanceSearchQueryParams {
            security_code: Some("1234".to_string()),
            security_name: Some("テスト株式会社".to_string()),
            ..Default::default()
        };
        let filter = AssetBalanceFilter::from_params(&params);

        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT 1 FROM asset_balances");
        push_filters(&mut qb, Uuid::nil(), &filter);
        let sql = qb.sql();
        let sql = sql.as_str();

        assert!(sql.contains("AND security_code = "));
        assert!(sql.contains("AND security_name = "));
    }
}
