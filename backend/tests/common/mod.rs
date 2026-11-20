#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(dead_code)]

use axum::{
    body::Body,
    http::{Method, Request},
};
use backend::test_constants::POSTGRES_TAG;
use backend::{
    config::Config,
    db::{self, Bind, Db, Executor, connect_pool_lazy, run_migrations},
    models::asset_balance::{AssetBalanceSearchQueryParams, CreateAssetBalanceRequest},
    models::common::{PaginationParams, SearchQueryParams},
    models::dividend::{CreateDividendRequest, DividendSearchQueryParams},
    models::domestic_stock::{CreateDomesticStockRequest, DomesticStockSearchQueryParams},
    models::fund_balance::FundBalance,
    models::mutualfund::{CreateMutualfundRequest, MutualfundSearchQueryParams},
    models::user::GoogleUserInfo,
    services::asset_balance as asset_balance_svc,
    services::auth::{self as auth_svc, SessionToken},
    services::domain::search::search as domain_search,
    services::stock_master,
    state::{AppState, Secrets},
};
use rust_decimal_macros::dec;
use shared::value::UserId;
use std::{sync::Arc, time::Duration};
use testcontainers::{ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;
use time::format_description::well_known::Rfc3339;
use time::{Date, OffsetDateTime};
use tokio::time::{sleep, timeout};
use tower::ServiceExt;
use uuid::Uuid;

pub fn default_asset_balance_search_params() -> AssetBalanceSearchQueryParams {
    AssetBalanceSearchQueryParams::default()
}

pub fn default_dividend_search_params() -> DividendSearchQueryParams {
    DividendSearchQueryParams::default()
}

pub fn default_domestic_stock_search_params() -> DomesticStockSearchQueryParams {
    DomesticStockSearchQueryParams::default()
}

pub fn default_mutualfund_search_params() -> MutualfundSearchQueryParams {
    MutualfundSearchQueryParams::default()
}

pub fn dividend_search_params_with_pagination(
    pagination: PaginationParams,
) -> DividendSearchQueryParams {
    DividendSearchQueryParams {
        search: SearchQueryParams {
            pagination,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub async fn connect_with_retry(database_url: &str) -> Db {
    let pool = connect_pool_lazy(database_url, 4).expect("Failed to build pool");
    let mut last_error = None;
    for _ in 0..20 {
        let attempt = timeout(
            Duration::from_secs(2),
            db::query("SELECT 1", vec![]).execute(&pool),
        )
        .await;
        match attempt {
            Ok(Ok(_)) => return pool,
            Ok(Err(err)) => last_error = Some(err),
            Err(_) => {}
        }
        sleep(Duration::from_millis(500)).await;
    }

    panic!("Failed to connect to Postgres: {last_error:?}");
}

pub async fn seed_stock_row(pool: &Db) {
    let rows = vec![stock_master::StockRow {
        date: OffsetDateTime::now_utc().date() - time::Duration::days(3),
        code: "9931".to_string(),
        name: "架空水産".to_string(),
        market_category: "プライム".to_string(),
        industry_category_33: None,
        industry_category_17: None,
        size_category: None,
    }];
    stock_master::replace_master(pool, &rows)
        .await
        .expect("seed stock");
}

/// 銘柄マスタ更新枠(market_data_provider_rate_control の provider='jquants-equities-master')
/// の next_available_at。行が無ければ None
pub async fn master_refresh_gate(pool: &Db) -> Option<time::OffsetDateTime> {
    db::query_scalar::<time::OffsetDateTime>(
        "SELECT next_available_at FROM market_data_provider_rate_control \
         WHERE provider = 'jquants-equities-master'",
        vec![],
    )
    .fetch_optional(pool)
    .await
    .unwrap()
}

// epoch ミリ秒は f64 でも 2^53 未満では正確に表せるため精度は落ちない
#[expect(clippy::cast_precision_loss)]
pub fn scheduled_now_ms() -> f64 {
    OffsetDateTime::now_utc().unix_timestamp_nanos() as f64 / 1_000_000.0
}

/// テスト用の CreateAssetBalanceRequest を生成するヘルパー
pub fn make_asset_item(code: &str) -> CreateAssetBalanceRequest {
    CreateAssetBalanceRequest {
        security_code: code.parse().unwrap(),
        security_name: format!("テスト銘柄{code}"),
        account: None,
        shares: dec!(100),
        average_purchase_price: dec!(1000),
        total_purchase_amount: dec!(100000),
        current_price: dec!(1100),
        daily_change: dec!(10),
        market_value: dec!(110000),
        profit_loss_rate: dec!(10),
    }
}

pub fn make_dividend_item(security_code: &str) -> CreateDividendRequest {
    CreateDividendRequest {
        settlement_date: Date::from_calendar_date(2024, time::Month::try_from(3).unwrap(), 25)
            .unwrap(),
        product: "国内株式".to_string(),
        account: "特定".parse().unwrap(),
        security_code: security_code.to_string(),
        security_name: format!("銘柄_{security_code}"),
        unit_price: dec!(100.0),
        shares: dec!(100.0),
        dividends_before_tax: dec!(1000.0),
        taxes: dec!(203.0),
        net_amount_received: dec!(797.0),
    }
}

pub fn make_domestic_stock_item(security_code: &str) -> CreateDomesticStockRequest {
    CreateDomesticStockRequest {
        trade_date: Date::from_calendar_date(2024, time::Month::try_from(3).unwrap(), 20).unwrap(),
        settlement_date: Date::from_calendar_date(2024, time::Month::try_from(3).unwrap(), 25)
            .unwrap(),
        security_code: security_code.parse().unwrap(),
        security_name: format!("銘柄_{security_code}"),
        account: "特定".parse().unwrap(),
        shares: dec!(100.0),
        asked_price: dec!(1000.0),
        proceeds: dec!(100000.0),
        purchase_price: dec!(90000.0),
        realized_profit_and_loss: dec!(10000.0),
        taxes: dec!(2030.0),
        realized_profit_and_loss_after_tax: dec!(7970.0),
    }
}

pub fn make_mutualfund_item(fund_name: &str) -> CreateMutualfundRequest {
    CreateMutualfundRequest {
        trade_date: Date::from_calendar_date(2024, time::Month::try_from(3).unwrap(), 20).unwrap(),
        settlement_date: Date::from_calendar_date(2024, time::Month::try_from(3).unwrap(), 25)
            .unwrap(),
        fund_name: fund_name.to_string(),
        dividends: None,
        account: "特定".parse().unwrap(),
        shares: dec!(1000.0),
        exchange_rate: dec!(1.0),
        cancellation_unit_price_yen: dec!(12000.0),
        cancellation_amount_yen: dec!(12000000.0),
        average_acquisition_price_yen: dec!(10000.0),
        realized_profit_and_loss: dec!(2000000.0),
        taxes: dec!(406060.0),
        realized_profit_and_loss_after_tax: dec!(1593940.0),
    }
}

pub const fn make_mutualfund_csv() -> &'static str {
    concat!(
        "約定日,受渡日,ファンド名,分配金,口座,取引,数量[口],為替レート［円］,解約単価［円］,解約額［円］,平均取得価額［円］,実現損益［円］\n",
        "\"2022/10/28\",\"2022/11/2\",\"架空インデックス 米国株式\",\"\",\"特定\",\"解約\",\"1000\",\"1\",\"12000\",\"12000000\",\"10000\",\"600000\""
    )
}

pub const fn make_dividend_csv() -> &'static str {
    concat!(
        "入金日,商品,口座,銘柄コード,銘柄,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]\n",
        "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"8592\",\"テスト配当\",\"円\",\"90.00\",\"200\",\"18000\",\"3656\",\"14344\""
    )
}

pub const fn make_domestic_stock_csv() -> &'static str {
    concat!(
        "約定日,受渡日,銘柄コード,銘柄名,口座,信用区分,取引,数量[株],売却/決済単価[円],売却/決済額[円],平均取得価額[円],実現損益[円]\n",
        "\"2026/02/09\",\"2026/02/12\",\"9902\",\"架空エネルギー\",\"特定\",\"-\",\"売付\",\"100\",\"1500.0\",\"150000\",\"1400.00\",\"10000\""
    )
}

pub fn asset_balance_csv(rows: &[&str]) -> String {
    format!(
        "■現在の評価額合計［円］,,\"1,100,000\"\n\
         ■評価損益合計,前日比［円］,\"10,000\"\n\
         ,前月比［円］,\"5,000\"\n\
         ,評価損益［円］,\"100,000\"\n\
         ■特定口座\n\
         \n\
         銘柄コード,銘柄名,保有数量［株］,執行中［株］,(内訳　通常数量[株]),(内訳　積立数量[株]),平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］\n\
         {}",
        rows.join("\n")
    )
}

pub fn make_asset_balance_csv() -> String {
    asset_balance_csv(&[
        "\"9904\",\"架空自動車\",\"100\",\"-\",\"100\",\"0\",\"2500\",\"250000\",\"2650\",\"15\",\"265000\",\"6.0\"",
    ])
}

/// Docker が必要なテスト用の Postgres コンテナ起動ヘルパー
pub async fn start_test_pool() -> (Db, impl Drop) {
    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let pool = connect_with_retry(&database_url).await;
    run_migrations(&pool).await.expect("migrations failed");
    (pool, node)
}

pub async fn create_test_user(pool: &Db) -> UserId {
    let user_id = UserId::from(Uuid::new_v4());
    db::query(
        "INSERT INTO users (id, google_id, email) VALUES ($1, $2, $3)",
        vec![
            Bind::from(user_id),
            Bind::from(format!("test_google_{user_id}")),
            Bind::from(format!("test_{user_id}@example.com")),
        ],
    )
    .execute(pool)
    .await
    .expect("failed to insert test user");
    user_id
}

/// migrate.rs が作る _sqlx_migrations と同じ定義。履歴だけ手動で用意するときに使う
pub const SQLX_MIGRATIONS_DDL: &str = "CREATE TABLE _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
    success BOOLEAN NOT NULL,
    checksum BYTEA NOT NULL,
    execution_time BIGINT NOT NULL
)";

/// backend/migrations にある migration の版(昇順)。新しい DB に当たる版の正本
pub fn local_migration_versions() -> Vec<i64> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let mut versions: Vec<i64> = std::fs::read_dir(&dir)
        .expect("migrations の読み取りに失敗")
        .map(|entry| entry.expect("migrations の要素の取得に失敗").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sql"))
        .map(|path| {
            path.file_name().unwrap().to_string_lossy()[..4]
                .parse()
                .expect("migration version")
        })
        .collect();
    versions.sort_unstable();
    versions
}

/// 土台の最後の版。ランナーの `BASELINE_LAST_VERSION` を正本として読む
pub fn baseline_last_version() -> i64 {
    include_str!("../../src/db/migrate.rs")
        .lines()
        .find_map(|line| line.strip_prefix("const BASELINE_LAST_VERSION: i64 = "))
        .and_then(|rest| rest.strip_suffix(';'))
        .expect("BASELINE_LAST_VERSION の定義")
        .parse()
        .expect("BASELINE_LAST_VERSION は数値")
}

pub fn legacy_last_version() -> i64 {
    legacy_migration_files()
        .last()
        .map(|(version, _, _)| *version)
        .expect("archive に旧 migration がある")
}

/// 旧 migration(backend/migrations/archive)を版の順に (版, 説明, SQL) で読む
pub fn legacy_migration_files() -> Vec<(i64, String, String)> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations/archive");
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .expect("archive の読み取りに失敗")
        .map(|entry| entry.expect("archive の要素の取得に失敗").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sql"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| {
            let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
            let (version, description) = stem.split_once('_').expect("migration 名の形式");
            (
                version.parse().expect("migration version"),
                description.replace('_', " "),
                std::fs::read_to_string(path).expect("archive の読み取りに失敗"),
            )
        })
        .collect()
}

/// 旧 migration を `to` の版まで当て、ランナーと同じ checksum(本文の SHA-384)で履歴に記録する。
/// 本番(旧 migration を全て適用済み)と同じ状態を作る
pub async fn apply_legacy_migrations(pool: &Db, to: i64) {
    pool.batch_execute(SQLX_MIGRATIONS_DDL)
        .await
        .expect("履歴テーブルの作成に失敗");
    for (version, description, sql) in legacy_migration_files() {
        if version > to {
            break;
        }
        let mut tx = pool.begin().await.expect("トランザクション開始に失敗");
        (&tx)
            .batch_execute(&sql)
            .await
            .unwrap_or_else(|e| panic!("旧 migration {version} の適用に失敗: {e}"));
        db::query(
            "INSERT INTO _sqlx_migrations
                (version, description, success, checksum, execution_time)
            VALUES ($1, $2, true, digest($3, 'sha384'), 1)",
            vec![
                Bind::from(version),
                Bind::from(description),
                Bind::from(sql),
            ],
        )
        .execute(&mut tx)
        .await
        .expect("履歴行の作成に失敗");
        tx.commit().await.expect("コミットに失敗");
    }
}

/// 指定の版を任意の checksum で適用済みにした状態を再現する
pub async fn seed_migration_history(pool: &Db, versions: &[i64]) {
    pool.batch_execute(SQLX_MIGRATIONS_DDL)
        .await
        .expect("履歴テーブルの作成に失敗");
    for version in versions.iter().copied() {
        db::query(
            "INSERT INTO _sqlx_migrations
                (version, description, success, checksum, execution_time)
            VALUES ($1, 'legacy', true, '\\x00'::bytea, 0)",
            vec![Bind::from(version)],
        )
        .execute(pool)
        .await
        .expect("履歴行の作成に失敗");
    }
}

/// _sqlx_migrations の適用済み version 一覧(昇順)
pub async fn migration_versions(pool: &Db) -> Vec<i64> {
    db::query(
        "SELECT version FROM _sqlx_migrations ORDER BY version",
        vec![],
    )
    .fetch_all::<(i64,), _>(pool)
    .await
    .expect("_sqlx_migrations の取得に失敗")
    .into_iter()
    .map(|(version,)| version)
    .collect()
}

/// public の表・ビュー・関数・ポリシーと、その所有者・権限・RLS・オプションの指紋
pub async fn schema_fingerprint(pool: &Db) -> String {
    db::query_scalar::<String>(
        "SELECT md5(string_agg(item, E'\n' ORDER BY item)) FROM (
            SELECT c.relname || ':' || c.relkind::text || ':' || c.relowner::regrole::text || ':'
                || coalesce(c.relacl::text, '') || ':' || c.relrowsecurity || ':'
                || coalesce(array_to_string(c.reloptions, ','), '') AS item
              FROM pg_class c
             WHERE c.relnamespace = 'public'::regnamespace AND c.relname NOT LIKE '\\_sqlx\\_migrations%'
            UNION ALL
            SELECT a.attrelid::regclass::text || '.' || a.attname || ':' || format_type(a.atttypid, a.atttypmod)
                || ':' || coalesce(a.attacl::text, '')
              FROM pg_attribute a JOIN pg_class c ON c.oid = a.attrelid
             WHERE c.relnamespace = 'public'::regnamespace AND c.relname NOT LIKE '\\_sqlx\\_migrations%'
               AND a.attnum > 0 AND NOT a.attisdropped
            UNION ALL
            SELECT p.oid::regprocedure::text || ':' || p.proowner::regrole::text || ':'
                || coalesce(p.proacl::text, '') || ':' || md5(p.prosrc)
              FROM pg_proc p WHERE p.pronamespace = 'public'::regnamespace
            UNION ALL
            SELECT pol.polrelid::regclass::text || ':' || pol.polname || ':'
                || coalesce(pg_get_expr(pol.polqual, pol.polrelid), '') || ':'
                || coalesce(pg_get_expr(pol.polwithcheck, pol.polrelid), '')
              FROM pg_policy pol
        ) s",
        vec![],
    )
    .fetch_one(pool)
    .await
    .expect("スキーマの指紋の取得に失敗")
}

/// (銘柄コード, 数量) の一覧。置換ガードの前後比較用
pub async fn asset_balance_rows(pool: &Db, user_id: UserId) -> Vec<(String, String)> {
    db::query(
        "SELECT security_code, shares::text FROM asset_balances \
         WHERE user_id = $1 ORDER BY security_code",
        vec![Bind::from(user_id)],
    )
    .fetch_all::<(String, String), _>(pool)
    .await
    .expect("asset_balances の取得に失敗")
}

/// (銘柄コード, 価格内容) の一覧。共有価格が巻き込まれていないかの比較用
pub async fn security_price_rows(pool: &Db) -> Vec<(String, String)> {
    db::query(
        "SELECT security_code, current_price::text || '/' || COALESCE(prev_close::text, '-') \
         || '/' || COALESCE(price_as_of::text, '-') \
         FROM security_prices ORDER BY security_code",
        vec![],
    )
    .fetch_all::<(String, String), _>(pool)
    .await
    .expect("security_prices の取得に失敗")
}

/// (基準日, 試行時刻) の取得用ヘルパー。価格更新cronの選択・循環確認に使う
pub async fn price_row(pool: &Db, code: &str) -> Option<(Option<Date>, time::OffsetDateTime)> {
    db::query(
        "SELECT price_as_of, updated_at FROM security_prices WHERE security_code = $1",
        vec![Bind::from(code)],
    )
    .fetch_optional(pool)
    .await
    .expect("security_prices の取得に失敗")
}

/// 一覧の (現在値, 前日比) を返す。集計の前日比合計が行と一致することもあわせて確かめる
pub async fn listed_price_and_change(
    pool: &Db,
    user_id: UserId,
) -> (rust_decimal::Decimal, rust_decimal::Decimal) {
    let mut params = default_asset_balance_search_params();
    params.search.include_summary = Some(true);
    let res = domain_search::<asset_balance_svc::AssetBalanceDomain>(pool, user_id, params)
        .await
        .expect("list 失敗");
    let row = res
        .data
        .iter()
        .find(|r| r.security_code.as_str() == "9904")
        .expect("9904 が一覧にあるはず");
    let summary = res.summary.expect("summary 必須");
    assert_eq!(
        summary.total_daily_change,
        row.daily_change * row.shares,
        "一覧と集計の前日比は一致する"
    );
    (row.current_price, row.daily_change)
}

/// fund_balances に1行だけ足す。price_as_of(CSV の基準価額が指す日)と created_at を固定できる
/// ため、「更新待ち」や共有値の鮮度判定を実行時刻に依らず確かめられる
#[expect(clippy::too_many_arguments)]
pub async fn insert_fund_row(
    pool: &Db,
    user_id: UserId,
    name: &str,
    shares: rust_decimal::Decimal,
    total_purchase_amount: rust_decimal::Decimal,
    base_price: rust_decimal::Decimal,
    day_change: Option<rust_decimal::Decimal>,
    market_value: rust_decimal::Decimal,
    profit_loss: rust_decimal::Decimal,
    price_as_of: Date,
    created_at: time::OffsetDateTime,
) {
    db::query(
        "INSERT INTO fund_balances (user_id, fund_name, account, dividends_course, shares, \
         average_purchase_price, total_purchase_amount, base_price, base_price_day_change, \
         market_value, profit_loss, profit_loss_rate, price_as_of, created_at) \
         VALUES ($1, $2, '特定', '再投資型', $3, 10000, $4, $5, $6, $7, $8, 0, $9, $10)",
        vec![
            Bind::from(user_id),
            Bind::from(name),
            Bind::from(shares),
            Bind::from(total_purchase_amount),
            Bind::from(base_price),
            Bind::opt(day_change),
            Bind::from(market_value),
            Bind::from(profit_loss),
            Bind::from(price_as_of),
            Bind::from(created_at),
        ],
    )
    .execute(pool)
    .await
    .expect("fund_balances の seed に失敗");
}

pub async fn view_fund_row(pool: &Db, user_id: UserId) -> FundBalance {
    db::query(
        "SELECT * FROM fund_balances_view WHERE user_id = $1",
        vec![Bind::from(user_id)],
    )
    .fetch_one::<FundBalance, _>(pool)
    .await
    .expect("fund_balances_view の取得に失敗")
}

pub async fn put_fund_price(pool: &Db, cd: &str, nav: &str, change: &str, as_of: Date) {
    db::query(
        "INSERT INTO fund_prices (association_fund_cd, nav, prev_day_change, price_as_of) \
         VALUES ($1, $2::numeric, $3::numeric, $4) \
         ON CONFLICT (association_fund_cd) DO UPDATE SET nav = EXCLUDED.nav, \
         prev_day_change = EXCLUDED.prev_day_change, price_as_of = EXCLUDED.price_as_of",
        vec![
            Bind::from(cd),
            Bind::from(nav),
            Bind::from(change),
            Bind::from(as_of),
        ],
    )
    .execute(pool)
    .await
    .expect("fund_prices の seed に失敗");
}

pub async fn csv_test_app(pool: &Db) -> (axum::Router, UserId, SessionToken) {
    let session = auth_svc::upsert_user_and_rotate_session(
        pool,
        &GoogleUserInfo {
            sub: Uuid::new_v4().to_string(),
            email: format!("csv-{}@example.com", Uuid::new_v4()),
            name: Some("架空の取込利用者".to_string()),
            picture_url: None,
        },
    )
    .await
    .unwrap();
    let user_id = auth_svc::select_user_id_by_session(pool, session)
        .await
        .unwrap()
        .unwrap();
    let state = AppState {
        pool: pool.clone(),
        secrets: Arc::new(Secrets {
            jquants_api_key: None,
            google_client_id: None,
            google_client_secret: None,
            frontend_url: "http://localhost:8080".to_string(),
            gmail_user: None,
            gmail_app_password: None,
        }),
        config: Arc::new(Config::default()),
        google_oauth: None,
        jquants_client: None,
    };
    (
        backend::handlers::v1::csv_upload_routes().with_state(state),
        user_id,
        session,
    )
}

pub async fn post_csv(
    app: &axum::Router,
    session: SessionToken,
    path: &str,
    csv: &str,
) -> axum::response::Response {
    let body = format!(
        "--csv-boundary\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\nContent-Type: text/csv\r\n\r\n{csv}\r\n--csv-boundary--\r\n"
    );
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(path)
                .header("cookie", format!("session_token={session}"))
                .header("content-type", "multipart/form-data; boundary=csv-boundary")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap()
}

pub fn csv_kind_timestamp(
    stats: &shared::csv_kind_stats::CsvKindStatsResponse,
    kind: &str,
) -> time::OffsetDateTime {
    stats
        .kinds
        .iter()
        .find(|stat| stat.kind == kind)
        .and_then(|stat| stat.latest_imported_at.as_deref())
        .and_then(|timestamp| OffsetDateTime::parse(timestamp, &Rfc3339).ok())
        .expect("取込時刻が存在する")
}
