#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::{body::to_bytes, http::StatusCode};
use backend::services::csv::pipeline::CsvImport;
use backend::{
    db::{self, Bind},
    models::csv_import::CsvUploadResponse,
    models::dividend::CreateDividendRequest,
    models::domestic_stock::CreateDomesticStockRequest,
    services::asset_balance as asset_balance_svc,
    services::dividend as dividend_svc,
    services::domain::bulk::{self, RowLimit},
    services::domain::search::search as domain_search,
    services::domestic_stock as domestic_stock_svc,
    services::mutualfund as mutualfund_svc,
    services::stock_price::{self, ClosePrice},
};
use rust_decimal_macros::dec;
use shared::value::{SecurityCode, UserId};
use time::format_description::well_known::Rfc3339;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

mod common;

use common::*;

/// 置換型の資産CSVは、行エラー・有効行0件のとき保存前に拒否され、
/// 既存残高と共有価格が変わらないことを確かめる(#1336)。
/// 追記型CSVの部分成功は従来通り維持する
const VALID_ROW_9931: &str = "\"9931\",\"架空水産\",\"100\",\"0\",\"100\",\"0\",\"500\",\"50000\",\"600\",\"5\",\"60000\",\"20.0\"";

const VALID_ROW_9901: &str = "\"9901\",\"架空資源開発\",\"200\",\"0\",\"200\",\"0\",\"2000\",\"400000\",\"3000\",\"50\",\"600000\",\"50.00\"";

const BAD_SHARES_ROW: &str = "\"9905\",\"架空ゲーム\",\"N/A\",\"0\",\"1,000\",\"0\",\"5,000.00\",\"5,000,000\",\"7,000.0\",\"100.0\",\"7,000,000\",\"40.00\"";

const BAD_PRICE_ROW: &str = "\"9901\",\"架空資源開発\",\"200\",\"0\",\"200\",\"0\",\"2,000.00\",\"400,000\",\"不明\",\"50.0\",\"600,000\",\"50.00\"";

const SUMMARY_ONLY_ROW: &str = ",,,,,,特定口座合計,\"5,400,000\",,,\"7,600,000\",\"40.74\"";

/// 確定値が入った銘柄は CSV 再取込で上書きされない。
/// 未確認(price_as_of NULL)の共有行だけが CSV の値に更新される
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn asset_balance_csv_reimport_keeps_confirmed_price() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    asset_balance_svc::bulk_create(
        &pool,
        user_id,
        &[make_asset_item("9904")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("初回 bulk_create 失敗");

    let code: SecurityCode = "9904".parse().unwrap();
    let confirmed = ClosePrice {
        close: dec!(2700),
        as_of: Date::from_calendar_date(2026, time::Month::try_from(10).unwrap(), 14).unwrap(),
        prev_close: Some(dec!(2650)),
    };
    stock_price::apply_close_price(&pool, &code, &confirmed)
        .await
        .expect("確定値の適用");

    // 別価格を載せた再取込でも確定値は残り、確定値のない銘柄だけ CSV 値が入る
    let mut reimported = make_asset_item("9904");
    reimported.current_price = dec!(9999);
    reimported.daily_change = dec!(-999);
    let mut fresh = make_asset_item("9914");
    fresh.current_price = dec!(1234.5);
    asset_balance_svc::bulk_create(
        &pool,
        user_id,
        &[reimported, fresh],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("再取込");

    let rows = domain_search::<asset_balance_svc::AssetBalanceDomain>(
        &pool,
        user_id,
        default_asset_balance_search_params(),
    )
    .await
    .expect("list 失敗")
    .data;
    assert_eq!(rows.len(), 2);
    let by_code = |code: &str| {
        rows.iter()
            .find(|r| r.security_code.as_str() == code)
            .expect("行がある")
    };
    let confirmed_row = by_code("9904");
    assert_eq!(confirmed_row.current_price, dec!(2700), "確定値は消えない");
    assert_eq!(confirmed_row.daily_change, dec!(50));
    assert_eq!(
        confirmed_row.price_as_of,
        Some(Date::from_calendar_date(2026, time::Month::try_from(10).unwrap(), 14).unwrap())
    );
    let seeded_row = by_code("9914");
    assert_eq!(seeded_row.current_price, dec!(1234.5));
    assert_eq!(seeded_row.price_as_of, None, "CSV 取込値は未確認");

    // 個人CSV価格は利用者ごとに独立し、別利用者の取込では変わらない
    let other_user = create_test_user(&pool).await;
    let mut other_import = make_asset_item("9914");
    other_import.current_price = dec!(2000);
    asset_balance_svc::bulk_create(&pool, other_user, &[other_import], RowLimit::new(i64::MAX))
        .await
        .expect("別利用者の取込");

    let rows = domain_search::<asset_balance_svc::AssetBalanceDomain>(
        &pool,
        user_id,
        default_asset_balance_search_params(),
    )
    .await
    .expect("list 失敗")
    .data;
    let updated_row = rows
        .iter()
        .find(|r| r.security_code.as_str() == "9914")
        .expect("行がある");
    // 個人CSV価格は利用者ごとに独立し、別利用者の取込では変わらない
    assert_eq!(updated_row.current_price, dec!(1234.5));
    assert_eq!(updated_row.price_as_of, None);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn dividend_csv_keeps_same_detail_in_different_accounts() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    // 同一入金日・同一銘柄・同一数量・同一額で、口座または商品だけ違う明細は別データ
    let csv = concat!(
        "入金日,商品,口座,銘柄コード,銘柄,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]\n",
        "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"9908\",\"架空リース\",\"円\",\"90.00\",\"200\",\"18000\",\"3656\",\"14344\"\n",
        "\"2025/12/09\",\"国内株式\",\"NISA\",\"9908\",\"架空リース\",\"円\",\"90.00\",\"200\",\"18000\",\"3656\",\"14344\"\n",
        "\"2025/12/09\",\"投資信託\",\"特定・一般\",\"9908\",\"架空リース\",\"円\",\"90.00\",\"200\",\"18000\",\"3656\",\"14344\""
    );

    let first = dividend_svc::DividendDomain::upload_csv(
        &pool,
        user_id,
        csv.as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("dividend upload_csv failed");
    assert_eq!(first.inserted, 3);
    assert_eq!(first.skipped, 0);
    assert_eq!(first.errors, [] as [shared::csv_import::CsvRowError; 0]);

    // 同じ CSV の再取込では増えない
    let second = dividend_svc::DividendDomain::upload_csv(
        &pool,
        user_id,
        csv.as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("dividend re-upload_csv failed");
    assert_eq!(second.inserted, 0);
    assert_eq!(second.skipped, 3);
    assert_eq!(second.errors, [] as [shared::csv_import::CsvRowError; 0]);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn dividend_csv_rejects_foreign_currency_rows() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    // 円限定の現仕様: 外貨行は行エラーとなり、円の行だけが保存される
    let csv = concat!(
        "入金日,商品,口座,銘柄コード,銘柄,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]\n",
        "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"9908\",\"架空リース\",\"円\",\"90.00\",\"200\",\"18000\",\"3656\",\"14344\"\n",
        "\"2025/12/09\",\"外国株式\",\"特定・一般\",\"AAPL\",\"アップル\",\"米ドル\",\"200.00\",\"10\",\"1000\",\"100\",\"900\""
    );

    let uploaded = dividend_svc::DividendDomain::upload_csv(
        &pool,
        user_id,
        csv.as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("dividend upload_csv failed");
    assert_eq!(uploaded.inserted, 1);
    assert_eq!(uploaded.errors.len(), 1);
    assert_eq!(uploaded.errors[0].row, 2);

    let mut params = default_dividend_search_params();
    params.search.include_summary = Some(true);
    let res = domain_search::<dividend_svc::DividendDomain>(&pool, user_id, params)
        .await
        .expect("dividend list failed");
    assert_eq!(res.data.len(), 1);
    assert_eq!(res.data[0].security_code, "9908");

    let summary = res.summary.expect("summary required");
    assert_eq!(summary.total_dividends_before_tax, dec!(18000));
    assert_eq!(summary.total_net_amount_received, dec!(14344));
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn mutualfund_csv_keeps_same_trade_in_different_accounts() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    // 同一約定日・同一ファンド・同一数量・同一解約額で、口座だけ違う明細は別データ
    let csv = concat!(
        "約定日,受渡日,ファンド名,分配金,口座,取引,数量[口],為替レート［円］,解約単価［円］,解約額［円］,平均取得価額［円］,実現損益［円］\n",
        "\"2022/10/28\",\"2022/11/2\",\"架空インデックス 米国株式\",\"再投資型\",\"特定\",\"解約\",\"1000\",\"1\",\"12000\",\"12000000\",\"10000\",\"600000\"\n",
        "\"2022/10/28\",\"2022/11/2\",\"架空インデックス 米国株式\",\"再投資型\",\"NISA\",\"解約\",\"1000\",\"1\",\"12000\",\"12000000\",\"10000\",\"600000\""
    );

    let first = mutualfund_svc::MutualfundDomain::upload_csv(
        &pool,
        user_id,
        csv.as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("mutualfund upload_csv failed");
    assert_eq!(first.inserted, 2);
    assert_eq!(first.skipped, 0);
    assert_eq!(first.errors, [] as [shared::csv_import::CsvRowError; 0]);

    // 同じ CSV の再取込では増えない
    let second = mutualfund_svc::MutualfundDomain::upload_csv(
        &pool,
        user_id,
        csv.as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("mutualfund re-upload_csv failed");
    assert_eq!(second.inserted, 0);
    assert_eq!(second.skipped, 2);
    assert_eq!(second.errors, [] as [shared::csv_import::CsvRowError; 0]);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn asset_balance_csv_upload_rejects_invalid_or_empty() {
    use backend::services::asset_balance::AssetBalanceDomain;
    use backend::services::dividend::DividendDomain;

    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let seed = vec![make_asset_item("9904"), make_asset_item("9905")];
    asset_balance_svc::bulk_create(&pool, user_id, &seed, RowLimit::new(i64::MAX))
        .await
        .expect("seed bulk_create failed");

    // 正常CSVは従来通り全置換される
    let ok = AssetBalanceDomain::upload_csv(
        &pool,
        user_id,
        asset_balance_csv(&[VALID_ROW_9931, VALID_ROW_9901]).as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("正常CSVは取り込める");
    assert_eq!((ok.inserted, ok.errors.is_empty()), (2, true));
    assert_eq!(
        asset_balance_rows(&pool, user_id).await,
        vec![
            ("9901".to_string(), "200".to_string()),
            ("9931".to_string(), "100".to_string()),
        ]
    );

    // 拒否ケースの比較基準になる状態へ戻す
    asset_balance_svc::bulk_create(&pool, user_id, &seed, RowLimit::new(i64::MAX))
        .await
        .expect("re-seed bulk_create failed");
    let balances_before = asset_balance_rows(&pool, user_id).await;
    let prices_before = security_price_rows(&pool).await;

    for (label, csv) in [
        (
            "1行不正",
            asset_balance_csv(&[VALID_ROW_9931, BAD_SHARES_ROW]),
        ),
        (
            "全行不正",
            asset_balance_csv(&[BAD_SHARES_ROW, BAD_PRICE_ROW]),
        ),
        ("有効行0件", asset_balance_csv(&[SUMMARY_ONLY_ROW])),
        ("データ行なし", asset_balance_csv(&[])),
    ] {
        let err =
            AssetBalanceDomain::upload_csv(&pool, user_id, csv.as_bytes(), RowLimit::new(i64::MAX))
                .await
                .expect_err(label);
        assert!(
            matches!(err, backend::errors::ApiError::Validation(_)),
            "{label}: {err:?}"
        );
        assert_eq!(
            asset_balance_rows(&pool, user_id).await,
            balances_before,
            "{label}"
        );
        assert_eq!(security_price_rows(&pool).await, prices_before, "{label}");
    }

    // 追記型(配当)は行エラーがあっても有効行だけ取り込む従来仕様を維持する
    let dividend_csv = concat!(
        "入金日,商品,口座,銘柄コード,銘柄,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]\n",
        "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"8592\",\"テスト配当\",\"円\",\"90.00\",\"200\",\"18000\",\"3656\",\"14344\"\n",
        "\"N/A\",\"国内株式\",\"特定\",\"8593\",\"不正行\",\"円\",\"1\",\"1\",\"1\",\"1\",\"1\""
    );
    let res = DividendDomain::upload_csv(
        &pool,
        user_id,
        dividend_csv.as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("追記型CSVは部分成功する");
    assert_eq!((res.inserted, res.errors.len()), (1, 1));

    let (dividend_count,): (i64,) = db::query(
        "SELECT COUNT(*) FROM dividends WHERE user_id = $1",
        vec![Bind::from(user_id)],
    )
    .fetch_one::<(i64,), _>(&pool)
    .await
    .expect("dividends の件数取得に失敗");
    assert_eq!(dividend_count, 1);
}

/// 数量 0 の行を含む CSV は、その行だけ行エラーになり残りは取り込まれる。
/// DB の CHECK 違反で取込全体が HTTP 500 にならないことと、プレビューでも行エラーになることを確かめる。
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn csv_upload_zero_shares_is_row_error_and_keeps_valid_rows() {
    use backend::services::dividend::DividendDomain;
    use backend::services::domestic_stock::DomesticStockDomain;
    use backend::services::mutualfund::MutualfundDomain;

    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let dividend_csv = concat!(
        "入金日,商品,口座,銘柄コード,銘柄,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]\n",
        "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"8592\",\"テスト配当\",\"円\",\"90.00\",\"200\",\"18000\",\"3656\",\"14344\"\n",
        "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"8593\",\"テスト配当ゼロ\",\"円\",\"90.00\",\"0\",\"18000\",\"3656\",\"14344\""
    );
    let preview =
        DividendDomain::preview_csv(dividend_csv.as_bytes()).expect("配当プレビューは通る");
    assert_eq!(
        (
            preview.total_rows,
            preview.valid_rows,
            preview.errors.len(),
            preview
                .errors
                .first()
                .is_some_and(|error| error.message.contains("0より大きい値")),
        ),
        (2, 1, 1, true)
    );
    let res = DividendDomain::upload_csv(
        &pool,
        user_id,
        dividend_csv.as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("配当は0行があっても部分成功する");
    assert_eq!((res.inserted, res.errors.len()), (1, 1));
    let (count,): (i64,) = db::query(
        "SELECT COUNT(*) FROM dividends WHERE user_id = $1",
        vec![Bind::from(user_id)],
    )
    .fetch_one::<(i64,), _>(&pool)
    .await
    .expect("dividends の件数取得に失敗");
    assert_eq!(count, 1);

    let domestic_csv = concat!(
        "約定日,受渡日,銘柄コード,銘柄名,口座,信用区分,取引,数量[株],売却/決済単価[円],売却/決済額[円],平均取得価額[円],実現損益[円]\n",
        "\"2026/02/09\",\"2026/02/12\",\"9902\",\"架空エネルギー\",\"特定\",\"-\",\"売付\",\"100\",\"1500.0\",\"150000\",\"1400.00\",\"10000\"\n",
        "\"2026/02/09\",\"2026/02/12\",\"9903\",\"架空ゼロ\",\"特定\",\"-\",\"売付\",\"0\",\"1500.0\",\"150000\",\"1400.00\",\"10000\""
    );
    let preview = DomesticStockDomain::preview_csv(domestic_csv.as_bytes())
        .expect("国内株式プレビューは通る");
    assert_eq!(
        (
            preview.total_rows,
            preview.valid_rows,
            preview.errors.len(),
            preview
                .errors
                .first()
                .is_some_and(|error| error.message.contains("0より大きい値")),
        ),
        (2, 1, 1, true)
    );
    let res = DomesticStockDomain::upload_csv(
        &pool,
        user_id,
        domestic_csv.as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("国内株式は0行があっても部分成功する");
    assert_eq!((res.inserted, res.errors.len()), (1, 1));
    let (count,): (i64,) = db::query(
        "SELECT COUNT(*) FROM domestic_stocks WHERE user_id = $1",
        vec![Bind::from(user_id)],
    )
    .fetch_one::<(i64,), _>(&pool)
    .await
    .expect("domestic_stocks の件数取得に失敗");
    assert_eq!(count, 1);

    let mutualfund_csv = concat!(
        "約定日,受渡日,ファンド名,分配金,口座,取引,数量[口],為替レート［円］,解約単価［円］,解約額［円］,平均取得価額［円］,実現損益［円］\n",
        "\"2022/10/28\",\"2022/11/2\",\"架空インデックス 米国株式\",\"再投資型\",\"特定\",\"解約\",\"1000\",\"1\",\"12000\",\"12000000\",\"10000\",\"600000\"\n",
        "\"2022/10/28\",\"2022/11/2\",\"架空ゼロファンド\",\"再投資型\",\"特定\",\"解約\",\"0\",\"1\",\"12000\",\"12000000\",\"10000\",\"600000\""
    );
    let preview =
        MutualfundDomain::preview_csv(mutualfund_csv.as_bytes()).expect("投信プレビューは通る");
    assert_eq!(
        (
            preview.total_rows,
            preview.valid_rows,
            preview.errors.len(),
            preview
                .errors
                .first()
                .is_some_and(|error| error.message.contains("0より大きい値")),
        ),
        (2, 1, 1, true)
    );
    let res = MutualfundDomain::upload_csv(
        &pool,
        user_id,
        mutualfund_csv.as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("投信は0行があっても部分成功する");
    assert_eq!((res.inserted, res.errors.len()), (1, 1));
    let (count,): (i64,) = db::query(
        "SELECT COUNT(*) FROM mutualfunds WHERE user_id = $1",
        vec![Bind::from(user_id)],
    )
    .fetch_one::<(i64,), _>(&pool)
    .await
    .expect("mutualfunds の件数取得に失敗");
    assert_eq!(count, 1);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn csv_kind_stats_queries_all_five_tables() {
    let (pool, _node) = start_test_pool().await;
    let user_id = UserId::from(Uuid::new_v4());

    let stats = backend::services::csv::stats::collect_csv_kind_stats(&pool, user_id)
        .await
        .expect("5種類すべてのテーブルを検索できる");

    let kinds: Vec<&str> = stats.kinds.iter().map(|stat| stat.kind.as_str()).collect();
    assert_eq!(
        kinds,
        [
            "assetbalance",
            "fundbalance",
            "dividend",
            "domesticstock",
            "mutualfund"
        ]
    );
    assert!(stats.kinds.iter().all(|stat| stat.count == 0));
    assert!(
        stats
            .kinds
            .iter()
            .all(|stat| stat.latest_imported_at.is_none())
    );
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn csv_import_success_updates_history_when_no_rows_are_inserted() {
    let (pool, _node) = start_test_pool().await;
    let (app, user_id, session) = csv_test_app(&pool).await;
    let old_timestamp = OffsetDateTime::now_utc() - time::Duration::days(3);
    let fund_csv = concat!(
        "投資信託種別,口座区分,ファンド,分配金コース,保有数量[口],平均取得価額[円],取得総額[円],基準価額[円],時価評価額[円],評価損益[円]\n",
        "投資信託,特定,架空長期積立ファンド,再投資型,10000,10000,10000,11000,11000,1000"
    );

    for (kind, path, csv, replaces) in [
        (
            "dividend",
            "/api/v1/dividend-imports",
            make_dividend_csv().to_string(),
            false,
        ),
        (
            "domesticstock",
            "/api/v1/domestic-stock-imports",
            make_domestic_stock_csv().to_string(),
            false,
        ),
        (
            "mutualfund",
            "/api/v1/mutual-fund-imports",
            make_mutualfund_csv().to_string(),
            false,
        ),
        (
            "assetbalance",
            "/api/v1/asset-balance-imports",
            make_asset_balance_csv(),
            true,
        ),
        (
            "fundbalance",
            "/api/v1/fund-balance-imports",
            fund_csv.to_string(),
            true,
        ),
    ] {
        let first = post_csv(&app, session, path, &csv).await;
        assert_eq!(first.status(), StatusCode::CREATED, "{kind}");
        let recorded: Option<time::OffsetDateTime> = db::query_scalar(
            "SELECT last_imported_at FROM csv_import_history WHERE user_id = $1 AND kind = $2",
            vec![Bind::from(user_id), Bind::from(kind)],
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(recorded.is_some(), "{kind} の初回成功時刻が記録される");

        db::query(
            "UPDATE csv_import_history SET last_imported_at = $3 WHERE user_id = $1 AND kind = $2",
            vec![
                Bind::from(user_id),
                Bind::from(kind),
                Bind::from(old_timestamp),
            ],
        )
        .execute(&pool)
        .await
        .unwrap();
        let response = post_csv(&app, session, path, &csv).await;
        assert_eq!(response.status(), StatusCode::CREATED, "{kind}");
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let result: CsvUploadResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            (result.inserted, result.skipped),
            if replaces { (1, 0) } else { (0, 1) },
            "{kind}"
        );
        assert!(result.errors.is_empty(), "{kind}: {:?}", result.errors);

        let last_imported_at: time::OffsetDateTime = db::query_scalar(
            "SELECT last_imported_at FROM csv_import_history WHERE user_id = $1 AND kind = $2",
            vec![Bind::from(user_id), Bind::from(kind)],
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(last_imported_at > old_timestamp, "{kind}");
        let stats = backend::services::csv::stats::collect_csv_kind_stats(&pool, user_id)
            .await
            .unwrap();
        assert_eq!(csv_kind_timestamp(&stats, kind), last_imported_at, "{kind}");
        assert_eq!(
            stats
                .kinds
                .iter()
                .find(|stat| stat.kind == kind)
                .unwrap()
                .count,
            1
        );
    }
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn csv_import_failure_does_not_update_history() {
    let (pool, _node) = start_test_pool().await;
    let (app, user_id, session) = csv_test_app(&pool).await;
    let old_timestamp = OffsetDateTime::parse("2026-10-24T02:15:00Z", &Rfc3339).unwrap();

    for (kind, path, csv, expected_status) in [
        (
            "assetbalance",
            "/api/v1/asset-balance-imports",
            asset_balance_csv(&[]),
            StatusCode::BAD_REQUEST,
        ),
        (
            "fundbalance",
            "/api/v1/fund-balance-imports",
            "投資信託種別,口座区分,ファンド\n".to_string(),
            StatusCode::BAD_REQUEST,
        ),
        (
            "domesticstock",
            "/api/v1/domestic-stock-imports",
            String::new(),
            StatusCode::BAD_REQUEST,
        ),
        (
            "mutualfund",
            "/api/v1/mutual-fund-imports",
            String::new(),
            StatusCode::BAD_REQUEST,
        ),
        (
            "dividend",
            "/api/v1/dividend-imports",
            make_dividend_csv().replace("18000", "不正値"),
            StatusCode::CREATED,
        ),
    ] {
        db::query(
            "INSERT INTO csv_import_history (user_id, kind, last_imported_at) VALUES ($1, $2, $3)",
            vec![
                Bind::from(user_id),
                Bind::from(kind),
                Bind::from(old_timestamp),
            ],
        )
        .execute(&pool)
        .await
        .unwrap();
        let response = post_csv(&app, session, path, &csv).await;
        assert_eq!(response.status(), expected_status, "{kind}");
        if expected_status == StatusCode::CREATED {
            let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
            let result: CsvUploadResponse = serde_json::from_slice(&body).unwrap();
            assert_eq!((result.inserted, result.skipped), (0, 0));
            assert_ne!(result.errors, [] as [shared::csv_import::CsvRowError; 0]);
        }
        let last_imported_at: time::OffsetDateTime = db::query_scalar(
            "SELECT last_imported_at FROM csv_import_history WHERE user_id = $1 AND kind = $2",
            vec![Bind::from(user_id), Bind::from(kind)],
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(last_imported_at, old_timestamp, "{kind}");
    }
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn csv_kind_stats_falls_back_to_latest_created_at_without_history() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let expected = OffsetDateTime::parse("2026-10-24T02:15:00Z", &Rfc3339).unwrap();

    dividend_svc::bulk_create(
        &pool,
        user_id,
        &[make_dividend_item("9908"), make_dividend_item("9909")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("配当データを作成できる");
    db::query(
        "UPDATE dividends SET created_at = CASE WHEN security_code = '9908' THEN $2 ELSE $2 - INTERVAL '1 day' END WHERE user_id = $1",
        vec![Bind::from(user_id), Bind::from(expected)],
    )
    .execute(&pool)
    .await
    .expect("created_at を固定できる");

    let stats = backend::services::csv::stats::collect_csv_kind_stats(&pool, user_id)
        .await
        .expect("統計を取得できる");
    assert_eq!(csv_kind_timestamp(&stats, "dividend"), expected);
    assert_eq!(
        stats
            .kinds
            .iter()
            .find(|stat| stat.kind == "dividend")
            .unwrap()
            .count,
        2
    );
    let recorded = expected - time::Duration::days(2);
    db::query(
        "INSERT INTO csv_import_history (user_id, kind, last_imported_at) VALUES ($1, $2, $3)",
        vec![
            Bind::from(user_id),
            Bind::from("dividend"),
            Bind::from(recorded),
        ],
    )
    .execute(&pool)
    .await
    .unwrap();
    let stats = backend::services::csv::stats::collect_csv_kind_stats(&pool, user_id)
        .await
        .unwrap();
    assert_eq!(csv_kind_timestamp(&stats, "dividend"), recorded);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn csv_import_history_is_scoped_by_user() {
    let (pool, _node) = start_test_pool().await;
    let (app, first_user, session) = csv_test_app(&pool).await;
    let second_user = create_test_user(&pool).await;
    let first_timestamp = OffsetDateTime::parse("2026-10-24T02:15:00Z", &Rfc3339).unwrap();
    let second_timestamp = OffsetDateTime::parse("2026-10-27T04:30:00Z", &Rfc3339).unwrap();

    for (user_id, timestamp) in [
        (first_user, first_timestamp),
        (second_user, second_timestamp),
    ] {
        dividend_svc::bulk_create(
            &pool,
            user_id,
            &[make_dividend_item("9908")],
            RowLimit::new(i64::MAX),
        )
        .await
        .expect("ユーザー別の配当データを作成できる");
        db::query(
            "INSERT INTO csv_import_history (user_id, kind, last_imported_at) VALUES ($1, $2, $3) ON CONFLICT (user_id, kind) DO UPDATE SET last_imported_at = EXCLUDED.last_imported_at",
            vec![
                Bind::from(user_id),
                Bind::from("dividend"),
                Bind::from(timestamp),
            ],
        )
        .execute(&pool)
        .await
        .expect("ユーザー別の履歴を作成できる");
    }

    let first_stats = backend::services::csv::stats::collect_csv_kind_stats(&pool, first_user)
        .await
        .expect("1人目の統計を取得できる");
    let second_stats = backend::services::csv::stats::collect_csv_kind_stats(&pool, second_user)
        .await
        .expect("2人目の統計を取得できる");

    assert_eq!(
        csv_kind_timestamp(&first_stats, "dividend"),
        first_timestamp
    );
    assert_eq!(
        csv_kind_timestamp(&second_stats, "dividend"),
        second_timestamp
    );
    let response = post_csv(
        &app,
        session,
        "/api/v1/dividend-imports",
        make_dividend_csv(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let first_stats = backend::services::csv::stats::collect_csv_kind_stats(&pool, first_user)
        .await
        .unwrap();
    let second_stats = backend::services::csv::stats::collect_csv_kind_stats(&pool, second_user)
        .await
        .unwrap();
    assert!(csv_kind_timestamp(&first_stats, "dividend") > first_timestamp);
    assert_eq!(
        csv_kind_timestamp(&second_stats, "dividend"),
        second_timestamp
    );
    assert!(
        first_stats
            .kinds
            .iter()
            .filter(|stat| stat.kind != "dividend")
            .all(|stat| stat.latest_imported_at.is_none())
    );
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn csv_kind_stats_ignores_history_after_delete_all() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    dividend_svc::bulk_create(
        &pool,
        user_id,
        &[make_dividend_item("9908")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("配当データを作成できる");
    let recorded = OffsetDateTime::parse("2026-10-24T02:15:00Z", &Rfc3339).unwrap();
    db::query(
        "INSERT INTO csv_import_history (user_id, kind, last_imported_at) VALUES ($1, $2, $3)",
        vec![
            Bind::from(user_id),
            Bind::from("dividend"),
            Bind::from(recorded),
        ],
    )
    .execute(&pool)
    .await
    .unwrap();
    let stats = backend::services::csv::stats::collect_csv_kind_stats(&pool, user_id)
        .await
        .unwrap();
    assert_eq!(csv_kind_timestamp(&stats, "dividend"), recorded);

    bulk::delete_all::<dividend_svc::DividendDomain>(&pool, user_id)
        .await
        .expect("全削除できる");
    let stats = backend::services::csv::stats::collect_csv_kind_stats(&pool, user_id)
        .await
        .unwrap();
    let dividend = stats
        .kinds
        .iter()
        .find(|stat| stat.kind == "dividend")
        .unwrap();
    assert_eq!(dividend.count, 0);
    assert!(dividend.latest_imported_at.is_none());
}

/// CSV 取り込み SQL の性能計測。架空データ1万行で UNNEST 取り込みと
/// facets(ARRAY_AGG 含む)の所要時間を報告する。行・件数・並び順は変えない
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn csv_import_perf_10k_reports_timing() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let base = Date::from_calendar_date(2024, time::Month::try_from(1).unwrap(), 1).unwrap();
    let items: Vec<CreateDividendRequest> = (0..10_000)
        .map(|i| {
            let code = format!("C{:04}", i % 100);
            CreateDividendRequest {
                settlement_date: base + time::Duration::days(i / 100),
                product: "国内株式".to_string(),
                account: "特定".parse().unwrap(),
                security_code: code.clone(),
                security_name: format!("架空銘柄{:03}_{:05}", i % 200, i),
                unit_price: dec!(100.0),
                shares: dec!(100.0) + rust_decimal::Decimal::from(i),
                dividends_before_tax: dec!(1000.0) + rust_decimal::Decimal::from(i),
                taxes: dec!(203.0),
                net_amount_received: dec!(797.0) + rust_decimal::Decimal::from(i),
            }
        })
        .collect();

    let start = std::time::Instant::now();
    let created = dividend_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
        .await
        .expect("10k bulk_create");
    let import_ms = start.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(created.inserted, 10_000);
    assert_eq!(created.skipped, 0);

    let mut params = default_dividend_search_params();
    params.search.include_facets = Some(true);
    let start = std::time::Instant::now();
    let res = domain_search::<dividend_svc::DividendDomain>(&pool, user_id, params)
        .await
        .expect("10k facets search");
    let facets_ms = start.elapsed().as_secs_f64() * 1000.0;
    let facets = res.facets.expect("facets 必須");
    assert_eq!(res.total, 10_000);
    assert_eq!(facets.securities.expect("securities").len(), 100);

    eprintln!("perf: import_10k_ms={import_ms:.1} facets_ms={facets_ms:.1}");

    let ds_items: Vec<CreateDomesticStockRequest> = (0..10_000)
        .map(|i| CreateDomesticStockRequest {
            trade_date: base + time::Duration::days(i),
            settlement_date: base + time::Duration::days(i),
            security_code: format!("D{:04}", i % 100).parse().unwrap(),
            security_name: format!("架空銘柄D{:03}_{:05}", i % 200, i),
            account: "特定".parse().unwrap(),
            shares: dec!(100.0) + rust_decimal::Decimal::from(i),
            asked_price: dec!(1000.0),
            proceeds: dec!(100000.0) + rust_decimal::Decimal::from(i),
            purchase_price: dec!(90000.0),
            realized_profit_and_loss: dec!(10000.0) + rust_decimal::Decimal::from(i),
            taxes: dec!(2030.0),
            realized_profit_and_loss_after_tax: dec!(7970.0) + rust_decimal::Decimal::from(i),
        })
        .collect();

    let start = std::time::Instant::now();
    let ds_created =
        domestic_stock_svc::bulk_create(&pool, user_id, &ds_items, RowLimit::new(i64::MAX))
            .await
            .expect("10k domestic bulk_create");
    let ds_import_ms = start.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(ds_created.inserted, 10_000);

    let mut ds_params = default_domestic_stock_search_params();
    ds_params.search.include_facets = Some(true);
    let start = std::time::Instant::now();
    let ds_res =
        domain_search::<domestic_stock_svc::DomesticStockDomain>(&pool, user_id, ds_params)
            .await
            .expect("10k domestic facets search");
    let ds_facets_ms = start.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(ds_res.total, 10_000);

    eprintln!(
        "perf: domestic_import_10k_ms={ds_import_ms:.1} domestic_facets_ms={ds_facets_ms:.1}"
    );

    let plans: Vec<(String,)> = db::query(
        "EXPLAIN (ANALYZE, BUFFERS) SELECT slot, value, value AS label, count FROM (SELECT CASE WHEN GROUPING(product) = 0 THEN 0 WHEN GROUPING(account) = 0 THEN 1 WHEN GROUPING(security_code) = 0 THEN 2 ELSE 5 END AS slot, COALESCE(product, account, security_code) AS value, COUNT(*) AS count FROM dividends WHERE user_id = $1 \
         GROUP BY GROUPING SETS ((product), (account), (security_code), ())) AS aggregates ORDER BY slot, value",
        vec![Bind::from(user_id)],
    )
    .fetch_all(&pool)
    .await
    .expect("EXPLAIN aggregates");
    for (line,) in &plans {
        eprintln!("plan(aggregates): {line}");
    }

    let plans: Vec<(String,)> = db::query(
        "EXPLAIN (ANALYZE, BUFFERS) SELECT DISTINCT ON (security_code) security_code, security_name FROM dividends WHERE user_id = $1 ORDER BY security_code, settlement_date DESC, id DESC",
        vec![Bind::from(user_id)],
    )
    .fetch_all(&pool)
    .await
    .expect("EXPLAIN latest names");
    for (line,) in &plans {
        eprintln!("plan(latest_names): {line}");
    }

    let plans: Vec<(String,)> = db::query(
        "EXPLAIN (ANALYZE, BUFFERS) SELECT product AS value, product AS label, COUNT(*) AS count \
         FROM dividends WHERE user_id = $1 GROUP BY product ORDER BY product ASC",
        vec![Bind::from(user_id)],
    )
    .fetch_all(&pool)
    .await
    .expect("EXPLAIN products");
    for (line,) in &plans {
        eprintln!("plan(products): {line}");
    }

    let plans: Vec<(String,)> = db::query(
        "EXPLAIN (ANALYZE, BUFFERS) INSERT INTO dividends (user_id, settlement_date, product, \
         account, security_code, security_name, unit_price, shares, dividends_before_tax, taxes, \
         net_amount_received, content_hash, occurrence_index) SELECT $1::uuid, $2::date, $3::text, $4::text, $5::text, $6::text, \
         $7::numeric, $8::numeric, $9::numeric, $10::numeric, $11::numeric, 'dummyhash', 99 \
         ON CONFLICT (user_id, content_hash, occurrence_index) DO NOTHING",
        vec![
            Bind::from(user_id),
            Bind::from(base),
            Bind::from("国内株式"),
            Bind::from("特定"),
            Bind::from("C0000"),
            Bind::from("架空銘柄000_explain"),
            Bind::from(dec!(100.0)),
            Bind::from(dec!(100.0)),
            Bind::from(dec!(1000.0)),
            Bind::from(dec!(203.0)),
            Bind::from(dec!(797.0)),
        ],
    )
    .fetch_all(&pool)
    .await
    .expect("EXPLAIN insert");
    for (line,) in &plans {
        eprintln!("plan(insert): {line}");
    }
}
