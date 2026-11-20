#![allow(clippy::unwrap_used, clippy::expect_used)]

use backend::{
    db::{self, Bind, Db, Executor},
    models::common::SearchQueryParams,
    models::fund_balance::{FundBalance, FundBalanceSearchQueryParams},
    services::asset_balance as asset_balance_svc,
    services::dividend_cache,
    services::domain::bulk::RowLimit,
    services::domain::search::search as domain_search,
    services::fund_balance::FundBalanceDomain,
    services::fund_price::{self, FundNav},
    services::jquants::JQuantsClient,
    services::price_cron,
    services::stock_price::{self, ClosePrice},
};
use rust_decimal_macros::dec;
use shared::dividend_per_share::DividendCacheStatus;
use shared::value::SecurityCode;
use time::format_description::well_known::Rfc3339;
use time::{Date, OffsetDateTime};
mod common;

use common::*;

/// cron の価格更新が失敗銘柄で止まらず対象が循環することを確かめる(#1338)。
/// 先頭15件(LIMIT 一杯)が取得失敗でも、次回以降は未取得の正常銘柄へ到達する。
/// ホスト実行では fetch_close_price が必ず失敗するため、各銘柄へ「試行が
/// 到達したか」は security_prices.updated_at(試行時刻)の前進で観測する。
/// 最新価格(price_as_of が最新終値日)の銘柄は対象外のまま再取得抑止も維持する
const GOOD: &str = "9919";

const FRESH: &str = "9200";

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn stock_price_apply_close_price_keeps_newer_row() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    asset_balance_svc::bulk_create(
        &pool,
        user_id,
        &[make_asset_item("9931")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("bulk_create 失敗");

    let code: SecurityCode = "9931".parse().unwrap();

    // 取込直後は基準日が未確定(NULL)なので、最初の取得値がそのまま入る
    let first = ClosePrice {
        close: dec!(900),
        as_of: Date::from_calendar_date(2020, time::Month::try_from(1).unwrap(), 1).unwrap(),
        prev_close: Some(dec!(890)),
    };
    let updated = stock_price::apply_close_price(&pool, &code, &first)
        .await
        .expect("apply(first) 失敗");
    assert_eq!(updated, 1);

    // 保存済みの基準日より古い取得値は反映しない（日付の新しいほうを残す）
    let stale = ClosePrice {
        close: dec!(800),
        as_of: Date::from_calendar_date(2000, time::Month::try_from(1).unwrap(), 1).unwrap(),
        prev_close: Some(dec!(790)),
    };
    let updated = stock_price::apply_close_price(&pool, &code, &stale)
        .await
        .expect("apply(stale) 失敗");
    assert_eq!(updated, 0);

    // 新しい取得値が current_price/market_value/daily_change に反映されることを確認
    let fresh = ClosePrice {
        close: dec!(2500.5),
        as_of: Date::from_calendar_date(2020, time::Month::try_from(1).unwrap(), 2).unwrap(),
        prev_close: Some(dec!(2400)),
    };
    let updated = stock_price::apply_close_price(&pool, &code, &fresh)
        .await
        .expect("apply(fresh) 失敗");
    assert_eq!(updated, 1);

    let row = domain_search::<asset_balance_svc::AssetBalanceDomain>(
        &pool,
        user_id,
        default_asset_balance_search_params(),
    )
    .await
    .expect("list 失敗")
    .data
    .into_iter()
    .next()
    .expect("1件のはず");
    assert_eq!(row.current_price, dec!(2500.5));
    // daily_change = close - prev_close、一覧には共有テーブルの値が出る
    assert_eq!(row.daily_change, dec!(100.5));
    assert_eq!(
        row.price_as_of,
        Some(Date::from_calendar_date(2020, time::Month::try_from(1).unwrap(), 2).unwrap())
    );
}

/// 同一銘柄を持つ2人の利用者で、確定値の反映は共有テーブル1行だけを更新し、
/// 両方の一覧に同じ価格が出る
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn stock_price_applies_once_to_shared_row_and_visible_to_all_holders() {
    let (pool, _node) = start_test_pool().await;
    let user_a = create_test_user(&pool).await;
    let user_b = create_test_user(&pool).await;

    let mut shares_b = make_asset_item("9904");
    shares_b.shares = dec!(200);
    for (user_id, items) in [
        (user_a, vec![make_asset_item("9904")]),
        (user_b, vec![shares_b]),
    ] {
        asset_balance_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
            .await
            .expect("bulk_create 失敗");
    }

    // CSV 取込では共有 security_prices を seed しないため、確定値の適用前は共有行がない。
    // 一覧・集計は個人 current_price に fallback して表示される
    let seeded: Option<(db::Numeric, Option<Date>)> = db::query_as(
        "SELECT current_price, price_as_of FROM security_prices WHERE security_code = '9904'",
        vec![],
    )
    .fetch_optional(&pool)
    .await
    .expect("共有価格行の取得");
    assert!(seeded.is_none(), "CSV 取込時点では共有行を作らない");

    for (user_id, _) in [(user_a, dec!(100)), (user_b, dec!(200))] {
        let params = default_asset_balance_search_params();
        let res = domain_search::<asset_balance_svc::AssetBalanceDomain>(&pool, user_id, params)
            .await
            .expect("list 失敗");
        let row = res.data.into_iter().next().expect("1件のはず");
        assert_eq!(row.current_price, dec!(1100), "個人CSV価格が使われる");
    }

    let code: SecurityCode = "9904".parse().unwrap();
    let price = ClosePrice {
        close: dec!(2700),
        as_of: Date::from_calendar_date(2026, time::Month::try_from(10).unwrap(), 14).unwrap(),
        prev_close: Some(dec!(2650)),
    };
    let updated = stock_price::apply_close_price(&pool, &code, &price)
        .await
        .expect("apply_close_price 失敗");
    assert_eq!(updated, 1, "共有テーブルでは1銘柄1行だけ更新される");

    for (user_id, shares) in [(user_a, dec!(100)), (user_b, dec!(200))] {
        let mut params = default_asset_balance_search_params();
        params.search.include_summary = Some(true);
        let res = domain_search::<asset_balance_svc::AssetBalanceDomain>(&pool, user_id, params)
            .await
            .expect("list 失敗");
        let row = res.data.into_iter().next().expect("1件のはず");
        assert_eq!(row.current_price, dec!(2700));
        assert_eq!(row.daily_change, dec!(50));
        assert_eq!(
            row.price_as_of,
            Some(Date::from_calendar_date(2026, time::Month::try_from(10).unwrap(), 14).unwrap())
        );

        // 前日比の合計・評価額は 共有価格×その利用者の数量 で計算される
        let summary = res.summary.expect("summary 必須");
        assert_eq!(summary.total_daily_change, dec!(50) * shares);
        assert_eq!(summary.total_market_value, dec!(2700) * shares);
    }
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn dividend_cache_persistence_and_rate_slot() {
    let (pool, _node) = start_test_pool().await;

    assert!(
        dividend_cache::get_batch(&pool, None, &[])
            .await
            .unwrap()
            .is_empty()
    );

    dividend_cache::persistence::update_cache_error(
        &pool,
        &"9931".parse::<SecurityCode>().unwrap(),
        "fetch failed",
        0,
    )
    .await
    .expect("エラー記録");
    let items = dividend_cache::get_batch(&pool, None, &["9931".parse().unwrap()])
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].security_code.as_str(), "9931");
    assert_eq!(items[0].status, DividendCacheStatus::Error);
    assert_eq!(items[0].dividend_per_share, None);
    assert!(
        items[0].is_stale,
        "再試行間隔を経過した error 記録は再取得対象"
    );

    dividend_cache::persistence::update_cache_error(
        &pool,
        &"9932".parse::<SecurityCode>().unwrap(),
        "429",
        3600,
    )
    .await
    .expect("cooldown 付きエラー記録");
    let items = dividend_cache::get_batch(&pool, None, &["9932".parse().unwrap()])
        .await
        .unwrap();
    assert_eq!(items[0].status, DividendCacheStatus::Error);
    assert!(!items[0].is_stale, "cooldown 中は再取得対象外");

    // 既存行への upsert: stale_at が cooldown で未来に更新される
    dividend_cache::persistence::update_cache_error(
        &pool,
        &"9931".parse::<SecurityCode>().unwrap(),
        "429",
        3600,
    )
    .await
    .expect("既存行の上書き");
    let items = dividend_cache::get_batch(&pool, None, &["9931".parse().unwrap()])
        .await
        .unwrap();
    assert!(!items[0].is_stale);
    let row: (String,) = db::query_as(
        "SELECT error_message FROM dividend_per_share_cache WHERE security_code = '9931'",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "429");

    dividend_cache::acquire_rate_slot(&pool)
        .await
        .expect("スロット予約");
    let row: (time::OffsetDateTime,) = db::query_as(
        "SELECT next_available_at FROM market_data_provider_rate_control \
         WHERE provider = 'jquants'",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        row.0 > OffsetDateTime::now_utc(),
        "予約後の next_available_at は未来: {:?}",
        row.0
    );
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn dividend_cache_migration_backfills_all_non_pending_rows_without_stale_at() {
    let (pool, _node) = start_test_pool().await;

    db::query(
        "INSERT INTO dividend_per_share_cache
            (security_code, status, stale_at, updated_at)
         VALUES
            ('9911', 'error', NULL, '2025-01-01T00:00:00Z'),
            ('9912', 'pending', NULL, '2025-01-02T00:00:00Z'),
            ('9913', 'error', '2099-01-03T00:00:00Z', '2025-01-04T00:00:00Z'),
            ('9914', 'ok', NULL, '2025-01-05T00:00:00Z')",
        vec![],
    )
    .execute(&pool)
    .await
    .expect("既存行の投入");

    pool.batch_execute(include_str!(
        "../migrations/archive/0031_dividend_cache_error_stale_at.sql"
    ))
    .await
    .expect("NULL stale_at のバックフィル");

    let rows: Vec<(String, Option<time::OffsetDateTime>)> = db::query(
        "SELECT security_code, stale_at FROM dividend_per_share_cache
         WHERE security_code IN ('9911', '9912', '9913', '9914') ORDER BY security_code",
        vec![],
    )
    .fetch_all(&pool)
    .await
    .expect("更新結果の取得");
    assert_eq!(
        rows[0].1,
        Some(OffsetDateTime::parse("2025-01-01T00:00:00Z", &Rfc3339).unwrap())
    );
    assert_eq!(rows[1].1, None);
    assert_eq!(
        rows[2].1,
        Some(OffsetDateTime::parse("2099-01-03T00:00:00Z", &Rfc3339).unwrap())
    );
    assert_eq!(
        rows[3].1,
        Some(OffsetDateTime::parse("2025-01-05T00:00:00Z", &Rfc3339).unwrap())
    );

    let selected: Vec<(String,)> = db::query(
        "SELECT security_code FROM dividend_per_share_cache
         WHERE security_code IN ('9911', '9912', '9913', '9914')
           AND (status = 'pending' OR stale_at < NOW())
         ORDER BY security_code",
        vec![],
    )
    .fetch_all(&pool)
    .await
    .expect("再取得対象の取得");
    assert_eq!(
        selected.into_iter().map(|(code,)| code).collect::<Vec<_>>(),
        ["9911", "9912", "9914"]
    );
}

/// drain_refresh_queue の消化対象選定(pending / 再試行間隔経過済みの error は対象、
/// 間隔内(未来の stale_at)は対象外)と、取得失敗時の error 記録を検証する。
/// J-Quants への実取得経路は Worker 側にしかないため、ホスト側の取得は必ず失敗し、
/// 選定済み行が error として記録されることまでを確認する
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn dividend_cache_drain_refresh_queue() {
    let (pool, _node) = start_test_pool().await;
    let client = JQuantsClient::new("test-api-key".to_string());

    // pending 行(get_batch のエンキュー相当)と即時再取得対象の error 行を積む
    for code in ["9904", "9907"] {
        db::query(
            "INSERT INTO dividend_per_share_cache (security_code, status, provider, updated_at)
             VALUES ($1, 'pending', 'jquants', NOW())",
            vec![Bind::from(code)],
        )
        .execute(&pool)
        .await
        .expect("pending 行の投入");
    }
    // 9907 は再試行間隔 0 のエラー行に戻して「stale_at 経過済み = 即時再取得対象」の経路を確認
    dividend_cache::persistence::update_cache_error(
        &pool,
        &"9907".parse::<SecurityCode>().unwrap(),
        "previous fetch failed",
        0,
    )
    .await
    .expect("error 行の更新");
    // 429 cooldown 中の error 行は消化対象外
    dividend_cache::persistence::update_cache_error(
        &pool,
        &"9107".parse::<SecurityCode>().unwrap(),
        "429",
        3600,
    )
    .await
    .expect("cooldown 行の投入");

    let processed = dividend_cache::drain_refresh_queue(&pool, &client)
        .await
        .expect("drain 失敗");
    assert_eq!(processed, 2, "cooldown 中の銘柄を除く 2 件を消化する");

    // 選定された行は fetch 失敗により error として記録される
    for code in ["9904", "9907"] {
        let row: (String,) = db::query_as(
            "SELECT status FROM dividend_per_share_cache WHERE security_code = $1",
            vec![Bind::from(code)],
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.0, "error", "{code} は失敗記録される");
    }

    // cooldown 中の行は消化されず、記録内容も変わらない
    let row: (String, Option<String>) = db::query_as(
        "SELECT status, error_message FROM dividend_per_share_cache
         WHERE security_code = '9107'",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "error");
    assert_eq!(row.1.as_deref(), Some("429"));

    // 失敗した行は再試行間隔分だけ stale_at が未来になり、即時には再消化されない
    let row: (time::OffsetDateTime,) = db::query_as(
        "SELECT stale_at FROM dividend_per_share_cache WHERE security_code = '9904'",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        row.0 > OffsetDateTime::now_utc() + time::Duration::hours(23),
        "失敗行の stale_at は再試行間隔の先: {:?}",
        row.0
    );
    let processed = dividend_cache::drain_refresh_queue(&pool, &client)
        .await
        .expect("2回目の drain 失敗");
    assert_eq!(processed, 0, "再試行間隔が経過するまで消化しない");
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn stock_price_refresh_cycles_past_failing_codes() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    // 失敗15件 + 正常1件 + 最新済み1件。失敗組は試行時刻が古く、正常銘柄は
    // それより新しい(先頭15件に埋もれる配置)。基準日も同順にしておき、
    // 修正前後で同じ15件が先に選ばれることを担保する
    let bad_codes: Vec<String> = Vec::from_fn(15, |i| format!("90{i:02}"));
    let items: Vec<_> = bad_codes
        .iter()
        .map(|code| make_asset_item(code))
        .chain([make_asset_item(GOOD), make_asset_item(FRESH)])
        .collect();
    asset_balance_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
        .await
        .expect("seed bulk_create failed");

    let stale_as_of = Date::from_calendar_date(2020, time::Month::try_from(1).unwrap(), 1).unwrap();
    let ts = |y, m, d| {
        Date::from_calendar_date(y, m, d)
            .unwrap()
            .with_time(time::Time::MIDNIGHT)
            .assume_utc()
    };
    let bad_mark = ts(2020, time::Month::January, 1);
    let good_mark = ts(2020, time::Month::June, 1);
    let fresh_mark = ts(2019, time::Month::January, 1);
    // 最新済みの基準日は latest_close_date(now) 以上である必要があるため JST 当日
    let fresh_as_of = (OffsetDateTime::now_utc() + time::Duration::hours(9)).date();

    // 個人 CSV 価格は共有行を作らないため、確定済みの共有価格は直接 INSERT で置く
    for code in &bad_codes[..14] {
        db::query(
            "INSERT INTO security_prices (security_code, current_price, price_as_of, updated_at) \
             VALUES ($1, 1100, $2, $3)",
            vec![
                Bind::from(code.as_str()),
                Bind::from(stale_as_of),
                Bind::from(bad_mark),
            ],
        )
        .execute(&pool)
        .await
        .expect("失敗銘柄の seed に失敗");
    }
    // 15件目の失敗銘柄は共有行を作らず未取得(行なし)のままにする
    db::query(
        "INSERT INTO security_prices (security_code, current_price, price_as_of, updated_at) \
         VALUES ($1, 1100, $2, $3)",
        vec![
            Bind::from(GOOD),
            Bind::from(
                Date::from_calendar_date(2020, time::Month::try_from(6).unwrap(), 1).unwrap(),
            ),
            Bind::from(good_mark),
        ],
    )
    .execute(&pool)
    .await
    .expect("正常銘柄の seed に失敗");
    db::query(
        "INSERT INTO security_prices (security_code, current_price, price_as_of, updated_at) \
         VALUES ($1, 1100, $2, $3)",
        vec![
            Bind::from(FRESH),
            Bind::from(fresh_as_of),
            Bind::from(fresh_mark),
        ],
    )
    .execute(&pool)
    .await
    .expect("最新済み銘柄の seed に失敗");

    // 1回目: 古い15件が選ばれ全て失敗する(ホストでは fetch が必ず失敗)
    let fetched = stock_price::refresh_stock_prices(&pool, None)
        .await
        .expect("refresh 失敗");
    assert_eq!(fetched, 0, "ホストでは全件失敗するはず");

    // 失敗銘柄は前回値を残したまま試行時刻だけ進む(行なし銘柄は stub 行が作られる)
    for code in &bad_codes[..14] {
        let (as_of, updated_at) = price_row(&pool, code).await.expect("行があるはず");
        assert_eq!(as_of, Some(stale_as_of), "{code}: 前回値の基準日を維持する");
        assert!(updated_at > bad_mark, "{code}: 試行時刻が進んでいない");
    }
    let (as_of, updated_at) = price_row(&pool, &bad_codes[14])
        .await
        .expect("失敗後も未取得行として残るはず");
    assert_eq!(as_of, None, "未取得のまま(基準日を偽装しない)");
    assert!(updated_at > bad_mark, "未取得銘柄の試行時刻が進んでいない");

    // 正常銘柄はまだ選ばれていない(LIMIT 15 の外側)
    let (_, good_updated) = price_row(&pool, GOOD).await.expect("行があるはず");
    assert_eq!(good_updated, good_mark, "1回目は正常銘柄に届かない");

    // 2回目: 失敗組は試行時刻が新しくなり末尾へ回るため、正常銘柄に到達する
    let fetched = stock_price::refresh_stock_prices(&pool, None)
        .await
        .expect("refresh 失敗");
    assert_eq!(fetched, 0);
    let (_, good_updated) = price_row(&pool, GOOD).await.expect("行があるはず");
    assert!(
        good_updated > good_mark,
        "失敗銘柄に塞がれず正常銘柄の試行に到達するはず"
    );

    // 最新価格の銘柄は再取得抑止が維持され、試行時刻も動かない
    let (_, fresh_updated) = price_row(&pool, FRESH).await.expect("行があるはず");
    assert_eq!(fresh_updated, fresh_mark, "最新済み銘柄は対象外のまま");
}

/// 新しい基準日で前日終値が取れないとき、前日比は不明(0)になる(#1339/#1344)。
/// security_prices は前日終値(市場の事実)を保存し、前日比は読み取り時に
/// current_price - prev_close で導出する。同一基準日で終値だけが変わった再取得
/// でも前日比は現在値と食い違わず、古い基準日での上書きは従来どおり拒否する
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn stock_price_new_as_of_without_prev_close_drops_daily_change() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    asset_balance_svc::bulk_create(
        &pool,
        user_id,
        &[make_asset_item("9904")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("bulk_create 失敗");

    let code: SecurityCode = "9904".parse().unwrap();
    let as_of = |d| Date::from_calendar_date(2020, time::Month::January, d).unwrap();

    // 1/14 終値100・前日終値90 が確定済み
    stock_price::apply_close_price(
        &pool,
        &code,
        &ClosePrice {
            close: dec!(100),
            as_of: as_of(14),
            prev_close: Some(dec!(90)),
        },
    )
    .await
    .expect("apply 失敗");

    // 1/15 は終値120のみ取得(前日終値不明)。前日比は不明(0)
    let updated = stock_price::apply_close_price(
        &pool,
        &code,
        &ClosePrice {
            close: dec!(120),
            as_of: as_of(15),
            prev_close: None,
        },
    )
    .await
    .expect("apply 失敗");
    assert_eq!(updated, 1);
    assert_eq!(
        listed_price_and_change(&pool, user_id).await,
        (dec!(120), dec!(0)),
        "新しい基準日で前日終値不明なら前日比は不明(0)にする"
    );

    // 同一基準日の再取得で前日終値が取れれば保存され、前日比に反映される
    let updated = stock_price::apply_close_price(
        &pool,
        &code,
        &ClosePrice {
            close: dec!(120),
            as_of: as_of(15),
            prev_close: Some(dec!(110)),
        },
    )
    .await
    .expect("apply 失敗");
    assert_eq!(updated, 1);
    assert_eq!(listed_price_and_change(&pool, user_id).await.1, dec!(10));

    // 同一基準日で終値だけ変わった再取得(前日終値不明)は、既知の前日終値を
    // 維持したまま前日比を現在値から導出する: 130 - 110 = 20
    let updated = stock_price::apply_close_price(
        &pool,
        &code,
        &ClosePrice {
            close: dec!(130),
            as_of: as_of(15),
            prev_close: None,
        },
    )
    .await
    .expect("apply 失敗");
    assert_eq!(updated, 1);
    let (stored_as_of, prev_close) = db::query(
        "SELECT price_as_of, prev_close FROM security_prices WHERE security_code = $1",
        vec![Bind::from("9904")],
    )
    .fetch_one::<(Date, Option<db::Numeric>), _>(&pool)
    .await
    .expect("共有価格の取得に失敗");
    assert_eq!(stored_as_of, as_of(15));
    assert_eq!(
        prev_close.expect("前日終値は維持する").0,
        dec!(110),
        "同一基準日で前日終値不明の取得は既知の値を維持する"
    );
    assert_eq!(
        listed_price_and_change(&pool, user_id).await,
        (dec!(130), dec!(20)),
        "同一基準日で現在値が変われば前日比は current_price - prev_close で再計算される"
    );

    // 古い基準日の取得値では上書きしない(前回値を維持)
    let updated = stock_price::apply_close_price(
        &pool,
        &code,
        &ClosePrice {
            close: dec!(999),
            as_of: as_of(14),
            prev_close: Some(dec!(900)),
        },
    )
    .await
    .expect("apply 失敗");
    assert_eq!(updated, 0);
    let (stored_as_of, prev_close) = db::query(
        "SELECT price_as_of, prev_close FROM security_prices WHERE security_code = $1",
        vec![Bind::from("9904")],
    )
    .fetch_one::<(Date, Option<db::Numeric>), _>(&pool)
    .await
    .expect("共有価格の取得に失敗");
    assert_eq!(stored_as_of, as_of(15));
    assert_eq!(prev_close.expect("前日終値は維持する").0, dec!(110));
}

/// ファンド名の照合正規化: 全角/半角・空白・括弧のゆれを吸収して一致する
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn fund_name_normalize_unifies_width_space_and_paren() {
    async fn norm(pool: &Db, s: &str) -> String {
        let (v,): (String,) = db::query("SELECT fund_name_normalize($1)", vec![Bind::from(s)])
            .fetch_one(pool)
            .await
            .expect("normalize");
        v
    }

    let (pool, _node) = start_test_pool().await;
    // 協会リスト名(全角)と CSV 名(半角)が同じ照合名になる
    let listed = norm(&pool, "ｅＭＡＸＩＳ　Ｓｌｉｍ　米国株式（Ｓ＆Ｐ５００）").await;
    for variant in [
        "eMAXIS Slim 米国株式(S&P500)",
        "ｅＭＡＸＩＳ Ｓｌｉｍ 米国株式（Ｓ＆Ｐ５００）",
        "ｅＭＡＸＩＳ　Ｓｌｉｍ　米国株式　（Ｓ＆Ｐ５００）",
        "emaxis slim 米国株式(s&p500)",
        "eMAXIS Slim 米国株式（S&P500)",
    ] {
        assert_eq!(norm(&pool, variant).await, listed, "{variant}");
    }
}

/// 共有値の基準日が CSV の基準価額の基準日の3日前以降なら、基準価額まわりの値は
/// 共有値でそろう。評価額・評価損益は口数から計算し直し、1円未満は切り捨てる
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn fund_nav_view_uses_shared_values_within_three_days() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let today = (OffsetDateTime::now_utc() + time::Duration::hours(9)).date();

    db::query(
        "INSERT INTO fund_codes SELECT fund_name_normalize($1), '90000001', $1",
        vec![Bind::from("架空マップファンド")],
    )
    .execute(&pool)
    .await
    .expect("fund_codes の seed に失敗");
    insert_fund_row(
        &pool,
        user_id,
        "架空マップファンド",
        dec!(15000),
        dec!(100000),
        dec!(10000),
        Some(dec!(5)),
        dec!(150000),
        dec!(50000),
        today,
        OffsetDateTime::now_utc(),
    )
    .await;
    fund_price::apply_fund_nav(
        &pool,
        "90000001",
        &FundNav {
            nav: dec!(10001),
            prev_day_change: dec!(-40),
            base_date: today,
        },
    )
    .await
    .expect("apply 失敗");

    let row = view_fund_row(&pool, user_id).await;
    assert_eq!(row.price_source, shared::domain::FundPriceSource::Api);
    assert!(row.nav_is_latest);
    assert_eq!(row.base_price, dec!(10001));
    assert_eq!(row.base_price_day_change, Some(dec!(-40)));
    // 15000 口 * 10001 / 10000 = 15001.5 → 1円未満切り捨て
    assert_eq!(row.market_value, dec!(15001));
    assert_eq!(row.profit_loss, dec!(15001) - dec!(100000));
    assert_eq!(row.profit_loss_rate, Some(dec!(-84.9990)));
    assert_eq!(row.price_as_of, today);
}

/// 新しい CSV があるのに古い共有値が残っているとき(共有値の基準日 < E だが
/// CSV 基準日の3日前以降)、従来ルールだと共有値が選ばれてしまう。
/// CSV の取込時刻が E の23:00以降なら CSV を優先する
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn fund_nav_view_prefers_fresh_csv_over_stale_shared() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let e = fund_price::expected_nav_date(OffsetDateTime::now_utc());

    db::query(
        "INSERT INTO fund_codes SELECT fund_name_normalize($1), '90000003', $1",
        vec![Bind::from("架空CSV優先ファンド")],
    )
    .execute(&pool)
    .await
    .expect("fund_codes の seed に失敗");
    // E の23:00を過ぎてから取り込んだ CSV(中身は E の基準価額)。
    // 従来ルールでは E-1 の共有値が「CSV 基準日の3日前以降」に入ってしまう
    insert_fund_row(
        &pool,
        user_id,
        "架空CSV優先ファンド",
        dec!(10000),
        dec!(100000),
        dec!(10500),
        Some(dec!(5)),
        dec!(105000),
        dec!(5000),
        e,
        OffsetDateTime::now_utc(),
    )
    .await;
    put_fund_price(
        &pool,
        "90000003",
        "11000",
        "100",
        e - time::Duration::days(1),
    )
    .await;

    let row = view_fund_row(&pool, user_id).await;
    assert_eq!(row.price_source, shared::domain::FundPriceSource::Csv);
    assert!(row.nav_is_latest);
    assert_eq!(row.base_price, dec!(10500));
    assert_eq!(row.base_price_day_change, Some(dec!(5)));
    assert_eq!(row.market_value, dec!(105000));
    assert_eq!(row.profit_loss, dec!(5000));
    assert_eq!(row.price_as_of, e);
}

/// 共有値も CSV も古いときは従来ルールに戻す:
/// 共有値の基準日が CSV 基準日の3日より前なら CSV 値、3日前以降なら共有値。
/// どちらを選んでも nav_is_latest = false(更新待ち)
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn fund_nav_view_falls_back_to_legacy_rule_when_both_are_stale() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let e = fund_price::expected_nav_date(OffsetDateTime::now_utc());
    let far_past_ts = Date::from_calendar_date(2020, time::Month::try_from(1).unwrap(), 1)
        .unwrap()
        .with_time(time::Time::MIDNIGHT)
        .assume_utc();

    db::query(
        "INSERT INTO fund_codes SELECT fund_name_normalize($1), '90000002', $1",
        vec![Bind::from("架空古い共有ファンド")],
    )
    .execute(&pool)
    .await
    .expect("fund_codes の seed に失敗");
    insert_fund_row(
        &pool,
        user_id,
        "架空古い共有ファンド",
        dec!(10000),
        dec!(100000),
        dec!(10000),
        Some(dec!(5)),
        dec!(100000),
        dec!(0),
        e,
        far_past_ts,
    )
    .await;
    put_fund_price(
        &pool,
        "90000002",
        "11000",
        "100",
        e - time::Duration::days(4),
    )
    .await;

    let row = view_fund_row(&pool, user_id).await;
    assert_eq!(row.price_source, shared::domain::FundPriceSource::Csv);
    assert!(!row.nav_is_latest);
    assert_eq!(row.base_price, dec!(10000));
    assert_eq!(row.base_price_day_change, Some(dec!(5)));
    assert_eq!(row.market_value, dec!(100000));
    assert_eq!(row.profit_loss, dec!(0));
    assert_eq!(row.price_as_of, e);

    // ちょうど3日前の共有値は採用する(境界値)が、更新待ちのまま
    put_fund_price(
        &pool,
        "90000002",
        "11000",
        "100",
        e - time::Duration::days(3),
    )
    .await;
    let row = view_fund_row(&pool, user_id).await;
    assert_eq!(row.price_source, shared::domain::FundPriceSource::Api);
    assert!(!row.nav_is_latest);
    assert_eq!(row.base_price, dec!(11000));
}

/// 対応表にないファンドは共有値があっても CSV 値のまま(出どころは混ぜない)
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn fund_nav_view_keeps_csv_for_unmapped_fund() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let today = (OffsetDateTime::now_utc() + time::Duration::hours(9)).date();
    insert_fund_row(
        &pool,
        user_id,
        "対応表にないファンド",
        dec!(10000),
        dec!(100000),
        dec!(10000),
        Some(dec!(5)),
        dec!(100000),
        dec!(0),
        today,
        OffsetDateTime::now_utc(),
    )
    .await;
    put_fund_price(&pool, "90000099", "11000", "100", today).await;

    let row = view_fund_row(&pool, user_id).await;
    assert_eq!(row.price_source, shared::domain::FundPriceSource::Csv);
    assert_eq!(row.base_price, dec!(10000));
}

/// 「更新待ち」の判定: 共有値は基準日が E 未満、CSV 値は取込時刻が E の
/// 23:00 より前なら nav_is_latest = false。実行時刻に依らず時刻を固定して確かめる
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn fund_nav_view_marks_latest_and_pending_by_expected_date() {
    use shared::domain::FundPriceSource::*;

    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let e = fund_price::expected_nav_date(OffsetDateTime::now_utc());
    let far_past = Date::from_calendar_date(2020, time::Month::try_from(1).unwrap(), 1).unwrap();
    let far_past_ts = far_past.with_time(time::Time::MIDNIGHT).assume_utc();

    db::query(
        "INSERT INTO fund_codes (normalized_name, association_fund_cd, fund_name) VALUES \
         (fund_name_normalize('最新APIファンド'), '90000011', '最新APIファンド'), \
         (fund_name_normalize('待ちAPIファンド'), '90000012', '待ちAPIファンド')",
        vec![],
    )
    .execute(&pool)
    .await
    .expect("fund_codes の seed に失敗");

    // 共有値: 基準日が E なら最新、E-1 なら更新待ち。
    // CSV 側の基準日・取込時刻を遠い過去にしておくと、E-1 の共有値でも
    // 3日ルールで必ず共有側になる
    for (name, cd, as_of) in [
        ("最新APIファンド", "90000011", e),
        ("待ちAPIファンド", "90000012", e - time::Duration::days(1)),
    ] {
        insert_fund_row(
            &pool,
            user_id,
            name,
            dec!(10000),
            dec!(100000),
            dec!(10000),
            Some(dec!(5)),
            dec!(100000),
            dec!(0),
            far_past,
            far_past_ts,
        )
        .await;
        put_fund_price(&pool, cd, "11000", "100", as_of).await;
    }
    // CSV 値: 取込時刻が E の23:00以降なら最新、それより前なら更新待ち
    insert_fund_row(
        &pool,
        user_id,
        "最新CSVファンド",
        dec!(10000),
        dec!(100000),
        dec!(10000),
        Some(dec!(5)),
        dec!(100000),
        dec!(0),
        Date::from_calendar_date(2026, time::Month::try_from(1).unwrap(), 1).unwrap(),
        OffsetDateTime::now_utc() + time::Duration::days(1),
    )
    .await;
    insert_fund_row(
        &pool,
        user_id,
        "待ちCSVファンド",
        dec!(10000),
        dec!(100000),
        dec!(10000),
        Some(dec!(5)),
        dec!(100000),
        dec!(0),
        Date::from_calendar_date(2026, time::Month::try_from(1).unwrap(), 1).unwrap(),
        far_past_ts,
    )
    .await;

    let rows: Vec<FundBalance> = db::query(
        "SELECT * FROM fund_balances_view WHERE user_id = $1",
        vec![Bind::from(user_id)],
    )
    .fetch_all::<FundBalance, _>(&pool)
    .await
    .expect("view の取得に失敗");
    let is = |name: &str| {
        let row = rows.iter().find(|r| r.fund_name == name).expect(name);
        (row.price_source, row.nav_is_latest)
    };
    assert_eq!(is("最新APIファンド"), (Api, true));
    assert_eq!(is("待ちAPIファンド"), (Api, false));
    assert_eq!(is("最新CSVファンド"), (Csv, true));
    assert_eq!(is("待ちCSVファンド"), (Csv, false));
}

/// summary は「最新」の行だけを前日比の合計に入れ、件数も返す
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn fund_summary_sums_only_latest_rows() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let today = (OffsetDateTime::now_utc() + time::Duration::hours(9)).date();
    let far_past_ts = Date::from_calendar_date(2020, time::Month::try_from(1).unwrap(), 1)
        .unwrap()
        .with_time(time::Time::MIDNIGHT)
        .assume_utc();

    db::query(
        "INSERT INTO fund_codes SELECT fund_name_normalize($1), '90000021', $1",
        vec![Bind::from("集計APIファンド")],
    )
    .execute(&pool)
    .await
    .expect("fund_codes の seed に失敗");
    // 最新(api・前日比あり): shares=20000, 前日比=10 → 20000*10/10000 = +20円
    insert_fund_row(
        &pool,
        user_id,
        "集計APIファンド",
        dec!(20000),
        dec!(200000),
        dec!(10000),
        Some(dec!(5)),
        dec!(200000),
        dec!(0),
        today,
        OffsetDateTime::now_utc(),
    )
    .await;
    put_fund_price(&pool, "90000021", "11000", "10", today).await;
    // 更新待ち(csv・前日比あり): 合計には入らない
    insert_fund_row(
        &pool,
        user_id,
        "集計待ちファンド",
        dec!(10000),
        dec!(100000),
        dec!(10000),
        Some(dec!(99)),
        dec!(100000),
        dec!(0),
        today,
        far_past_ts,
    )
    .await;
    // 最新(api・前日比なし): 合計に入らず件数にも数えない
    db::query(
        "INSERT INTO fund_codes SELECT fund_name_normalize($1), '90000022', $1",
        vec![Bind::from("集計変動なしファンド")],
    )
    .execute(&pool)
    .await
    .expect("fund_codes の seed に失敗");
    insert_fund_row(
        &pool,
        user_id,
        "集計変動なしファンド",
        dec!(10000),
        dec!(100000),
        dec!(10000),
        None,
        dec!(100000),
        dec!(0),
        Date::from_calendar_date(2020, time::Month::try_from(1).unwrap(), 1).unwrap(),
        OffsetDateTime::now_utc(),
    )
    .await;
    db::query(
        "INSERT INTO fund_prices (association_fund_cd, nav, prev_day_change, price_as_of) \
         VALUES ('90000022', 11000, NULL, $1)",
        vec![Bind::from(today)],
    )
    .execute(&pool)
    .await
    .expect("fund_prices の seed に失敗");

    let res = domain_search::<FundBalanceDomain>(
        &pool,
        user_id,
        FundBalanceSearchQueryParams {
            search: SearchQueryParams {
                include_summary: Some(true),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .await
    .expect("search 失敗");
    let summary = res.summary.expect("summary が返る");
    assert_eq!(summary.total_daily_change, dec!(20));
    assert_eq!(summary.latest_nav_count, 1);
    assert_eq!(summary.pending_nav_count, 1);
}

/// 自動更新の状況は、保有ファンドのうち協会コードが引けるものだけを対象にする。
/// 取得できた/失敗した件数と、直近の失敗理由を出す
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn fund_nav_status_counts_fetched_failed_and_pending() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let today = (OffsetDateTime::now_utc() + time::Duration::hours(9)).date();

    // 取得できた(共有値があり失敗記録がない)
    db::query(
        "INSERT INTO fund_codes SELECT fund_name_normalize($1), '90000031', $1",
        vec![Bind::from("状況取得済ファンド")],
    )
    .execute(&pool)
    .await
    .expect("fund_codes の seed に失敗");
    insert_fund_row(
        &pool,
        user_id,
        "状況取得済ファンド",
        dec!(10000),
        dec!(100000),
        dec!(10000),
        Some(dec!(5)),
        dec!(100000),
        dec!(0),
        today,
        OffsetDateTime::now_utc(),
    )
    .await;
    put_fund_price(&pool, "90000031", "11000", "10", today).await;

    // 取得に失敗した(共有値がなく失敗理由が残っている)
    db::query(
        "INSERT INTO fund_codes SELECT fund_name_normalize($1), '90000032', $1",
        vec![Bind::from("状況失敗ファンド")],
    )
    .execute(&pool)
    .await
    .expect("fund_codes の seed に失敗");
    insert_fund_row(
        &pool,
        user_id,
        "状況失敗ファンド",
        dec!(10000),
        dec!(100000),
        dec!(10000),
        None,
        dec!(100000),
        dec!(0),
        today,
        OffsetDateTime::now_utc(),
    )
    .await;
    db::query(
        "INSERT INTO fund_prices (association_fund_cd, nav, fetch_error, fetch_failed_at) \
         VALUES ('90000032', NULL, 'HTTP 403', NOW())",
        vec![],
    )
    .execute(&pool)
    .await
    .expect("fund_prices の seed に失敗");

    // まだ取得されていない(共有値の行が無い)
    db::query(
        "INSERT INTO fund_codes SELECT fund_name_normalize($1), '90000033', $1",
        vec![Bind::from("状況未取得ファンド")],
    )
    .execute(&pool)
    .await
    .expect("fund_codes の seed に失敗");
    insert_fund_row(
        &pool,
        user_id,
        "状況未取得ファンド",
        dec!(10000),
        dec!(100000),
        dec!(10000),
        None,
        dec!(100000),
        dec!(0),
        today,
        OffsetDateTime::now_utc(),
    )
    .await;

    let res = domain_search::<FundBalanceDomain>(
        &pool,
        user_id,
        FundBalanceSearchQueryParams {
            search: SearchQueryParams {
                include_summary: Some(true),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .await
    .expect("search 失敗");
    let summary = res.summary.expect("summary が返る");
    assert_eq!(summary.nav_status.fetched_count, 1);
    assert_eq!(summary.nav_status.failed_count, 1);
    assert_eq!(summary.nav_status.fund_count, 3);
    assert_eq!(summary.nav_status.auto_update_target_count, 3);
    assert_eq!(
        summary.nav_status.failure_reason.as_deref(),
        Some("HTTP 403")
    );
    // 最終更新の時刻は JST の "YYYY-MM-DD HH:MM"
    let updated_at = summary.nav_status.updated_at.expect("更新時刻がある");
    assert_eq!(updated_at.len(), 16, "{updated_at}");
    assert!(updated_at.contains(':'), "{updated_at}");
}

/// 失敗がないときは失敗理由を持たない。JST の分表示は UTC+9 で切り捨てる
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn fund_nav_status_without_failure_has_no_reason() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let today = (OffsetDateTime::now_utc() + time::Duration::hours(9)).date();

    db::query(
        "INSERT INTO fund_codes SELECT fund_name_normalize($1), '90000041', $1",
        vec![Bind::from("失敗なしファンド")],
    )
    .execute(&pool)
    .await
    .expect("fund_codes の seed に失敗");
    insert_fund_row(
        &pool,
        user_id,
        "失敗なしファンド",
        dec!(10000),
        dec!(100000),
        dec!(10000),
        None,
        dec!(100000),
        dec!(0),
        today,
        OffsetDateTime::now_utc(),
    )
    .await;
    put_fund_price(&pool, "90000041", "11000", "100", today).await;
    let fixed = OffsetDateTime::parse("2026-10-26T14:05:00Z", &Rfc3339).unwrap();
    db::query(
        "UPDATE fund_prices SET updated_at = $1 WHERE association_fund_cd = $2",
        vec![Bind::from(fixed), Bind::from("90000041")],
    )
    .execute(&pool)
    .await
    .expect("updated_at の固定に失敗");

    let res = domain_search::<FundBalanceDomain>(
        &pool,
        user_id,
        FundBalanceSearchQueryParams {
            search: SearchQueryParams {
                include_summary: Some(true),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .await
    .expect("search 失敗");
    let summary = res.summary.expect("summary が返る");
    assert_eq!(summary.nav_status.fetched_count, 1);
    assert_eq!(summary.nav_status.failed_count, 0);
    assert_eq!(summary.nav_status.failure_reason, None);
    assert_eq!(summary.nav_status.fund_count, 1);
    assert_eq!(summary.nav_status.auto_update_target_count, 1);
    assert_eq!(
        summary.nav_status.updated_at.as_deref(),
        Some("2026-10-26 23:05")
    );
}

/// ホストでは投信情報APIに触れないため取得は必ず失敗し、前回値を残したまま
/// 失敗理由だけが残る。対象の選定と試行時刻の前進(price_cron へ寄せた部分)を確かめる
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn fund_price_refresh_records_failure_and_marks_attempted() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let today = (OffsetDateTime::now_utc() + time::Duration::hours(9)).date();

    db::query(
        "INSERT INTO fund_codes SELECT fund_name_normalize($1), '90000051', $1",
        vec![Bind::from("更新試行ファンド")],
    )
    .execute(&pool)
    .await
    .expect("fund_codes の seed に失敗");
    insert_fund_row(
        &pool,
        user_id,
        "更新試行ファンド",
        dec!(10000),
        dec!(100000),
        dec!(10000),
        None,
        dec!(100000),
        dec!(0),
        today,
        OffsetDateTime::now_utc(),
    )
    .await;

    let selected = price_cron::select_targets(
        &pool,
        "SELECT association_fund_cd FROM public.scheduled_fund_price_targets_v2($1, $2)",
        "SELECT public.scheduled_fund_price_mark_attempted_v2($1)",
        today,
        30,
    )
    .await
    .expect("対象の選定に失敗");
    assert_eq!(selected, ["90000051".to_string()]);
    let marked: (Option<String>,) = db::query(
        "SELECT fetch_error FROM fund_prices WHERE association_fund_cd = $1",
        vec![Bind::from("90000051")],
    )
    .fetch_one::<(Option<String>,), _>(&pool)
    .await
    .expect("試行時刻の前進の確認に失敗");
    assert_eq!(marked.0, None);

    let fetched = fund_price::refresh_fund_prices(&pool, None)
        .await
        .expect("refresh 失敗");
    assert_eq!(fetched, 0);

    let row: (Option<String>,) = db::query(
        "SELECT fetch_error FROM fund_prices WHERE association_fund_cd = $1",
        vec![Bind::from("90000051")],
    )
    .fetch_one::<(Option<String>,), _>(&pool)
    .await
    .expect("失敗記録の取得に失敗");
    assert_eq!(row.0.as_deref(), Some("取得エラー"));
}

/// 失敗が複数あるときは fetch_failed_at の新しい理由を返す(NULL は最後)。
/// 同じファンドを口座違いで持っても件数は重ねず、別ユーザーの失敗は混ざらない
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn fund_nav_status_failure_reason_prefers_latest_and_isolates_users() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let other_id = create_test_user(&pool).await;
    let today = (OffsetDateTime::now_utc() + time::Duration::hours(9)).date();
    let at = |s: &str| OffsetDateTime::parse(s, &Rfc3339).unwrap();

    for (name, cd) in [
        ("口座またぎ失敗ファンド", "90000042"),
        ("新しい失敗ファンド", "90000043"),
        ("時刻なし失敗ファンド", "90000044"),
        ("別ユーザー失敗ファンド", "90000045"),
    ] {
        db::query(
            "INSERT INTO fund_codes SELECT fund_name_normalize($1), $2, $1",
            vec![Bind::from(name), Bind::from(cd)],
        )
        .execute(&pool)
        .await
        .expect("fund_codes の seed に失敗");
    }
    for (name, account) in [
        ("口座またぎ失敗ファンド", "特定"),
        ("口座またぎ失敗ファンド", "一般"),
        ("新しい失敗ファンド", "特定"),
        ("時刻なし失敗ファンド", "特定"),
    ] {
        db::query(
            "INSERT INTO fund_balances (user_id, fund_name, account, dividends_course, shares, \
             average_purchase_price, total_purchase_amount, base_price, base_price_day_change, \
             market_value, profit_loss, profit_loss_rate, price_as_of, created_at) \
             VALUES ($1, $2, $3, '再投資型', 10000, 10000, 100000, 10000, NULL, 100000, 0, 0, $4, $5)",
            vec![
                Bind::from(user_id),
                Bind::from(name),
                Bind::from(account),
                Bind::from(today),
                Bind::from(OffsetDateTime::now_utc()),
            ],
        )
        .execute(&pool)
        .await
        .expect("fund_balances の seed に失敗");
    }
    insert_fund_row(
        &pool,
        other_id,
        "別ユーザー失敗ファンド",
        dec!(10000),
        dec!(100000),
        dec!(10000),
        None,
        dec!(100000),
        dec!(0),
        today,
        OffsetDateTime::now_utc(),
    )
    .await;
    for (cd, reason, failed_at) in [
        ("90000042", "古い失敗", Some(at("2026-10-20T00:00:00Z"))),
        ("90000043", "新しい失敗", Some(at("2026-10-21T00:00:00Z"))),
        ("90000044", "時刻なし失敗", None),
        (
            "90000045",
            "別ユーザーの失敗",
            Some(at("2026-10-22T00:00:00Z")),
        ),
    ] {
        db::query(
            "INSERT INTO fund_prices (association_fund_cd, nav, fetch_error, fetch_failed_at) \
             VALUES ($1, NULL, $2, $3)",
            vec![Bind::from(cd), Bind::from(reason), Bind::opt(failed_at)],
        )
        .execute(&pool)
        .await
        .expect("fund_prices の seed に失敗");
    }

    let res = domain_search::<FundBalanceDomain>(
        &pool,
        user_id,
        FundBalanceSearchQueryParams {
            search: SearchQueryParams {
                include_summary: Some(true),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .await
    .expect("search 失敗");
    let summary = res.summary.expect("summary が返る");
    assert_eq!(summary.nav_status.fetched_count, 0);
    assert_eq!(summary.nav_status.failed_count, 3);
    assert_eq!(summary.nav_status.fund_count, 3);
    assert_eq!(summary.nav_status.auto_update_target_count, 3);
    assert_eq!(
        summary.nav_status.failure_reason.as_deref(),
        Some("新しい失敗")
    );

    let res = domain_search::<FundBalanceDomain>(
        &pool,
        other_id,
        FundBalanceSearchQueryParams {
            search: SearchQueryParams {
                include_summary: Some(true),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .await
    .expect("search 失敗");
    let summary = res.summary.expect("summary が返る");
    assert_eq!(summary.nav_status.failed_count, 1);
    assert_eq!(
        summary.nav_status.failure_reason.as_deref(),
        Some("別ユーザーの失敗")
    );
}

/// 古い基準日の取得値では共有値を上書きしない(前回値を維持)
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn fund_nav_apply_keeps_newer_row() {
    let (pool, _node) = start_test_pool().await;
    let as_of = |d: u32| {
        Date::from_calendar_date(2026, time::Month::January, u8::try_from(d).unwrap()).unwrap()
    };

    let updated = fund_price::apply_fund_nav(
        &pool,
        "90000031",
        &FundNav {
            nav: dec!(11000),
            prev_day_change: dec!(100),
            base_date: as_of(15),
        },
    )
    .await
    .expect("apply 失敗");
    assert_eq!(updated, 1);

    let updated = fund_price::apply_fund_nav(
        &pool,
        "90000031",
        &FundNav {
            nav: dec!(999),
            prev_day_change: dec!(-1),
            base_date: as_of(14),
        },
    )
    .await
    .expect("apply 失敗");
    assert_eq!(updated, 0);
    let (stored_as_of, nav): (Date, db::Numeric) = db::query(
        "SELECT price_as_of, nav FROM fund_prices WHERE association_fund_cd = $1",
        vec![Bind::from("90000031")],
    )
    .fetch_one(&pool)
    .await
    .expect("fund_prices の取得に失敗");
    assert_eq!(stored_as_of, as_of(15));
    assert_eq!(nav.0, dec!(11000));
}

/// SQL の fund_expected_nav_date と Rust 側 expected_nav_date が同じ規則で動くこと
/// (表示判定は SQL、cron 側は Rust の2箇所にあるため分岐を揃える)
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn fund_expected_nav_date_matches_rust_mirror() {
    let (pool, _node) = start_test_pool().await;
    for ts in [
        "2026-10-20T09:00:00+09:00",
        "2026-10-20T22:59:00+09:00",
        "2026-10-20T23:01:00+09:00",
        "2026-10-23T23:01:00+09:00",
        "2026-10-24T09:00:00+09:00",
        "2026-10-25T23:30:00+09:00",
        "2026-10-26T09:00:00+09:00",
        "2026-10-26T23:01:00+09:00",
    ] {
        let now = OffsetDateTime::parse(ts, &Rfc3339).unwrap();
        let rust_now = now.to_offset(time::UtcOffset::from_hms(9, 0, 0).unwrap());
        let (sql_e,): (Date,) = db::query(
            "SELECT fund_expected_nav_date($1::timestamptz)",
            vec![Bind::from(now)],
        )
        .fetch_one(&pool)
        .await
        .expect("fund_expected_nav_date");
        assert_eq!(
            sql_e,
            fund_price::expected_nav_date(rust_now),
            "{ts} で SQL/Rust の E が不一致"
        );
    }
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn fund_balance_import_uses_expected_nav_date() {
    let (pool, _node) = start_test_pool().await;
    for (ts, expected) in [
        ("2026-11-21T10:00:00+09:00", "2026-11-20"),
        ("2026-11-22T12:00:00+09:00", "2026-11-20"),
        ("2026-10-20T09:00:00+09:00", "2026-10-19"),
        ("2026-10-20T23:01:00+09:00", "2026-10-20"),
        ("2026-10-26T09:00:00+09:00", "2026-10-23"),
        ("2026-11-03T10:00:00+09:00", "2026-11-02"),
    ] {
        let now = OffsetDateTime::parse(ts, &Rfc3339).unwrap();
        let want = Date::parse(
            expected,
            time::macros::format_description!("[year]-[month]-[day]"),
        )
        .unwrap();
        let (got,): (Date,) = db::query(
            "SELECT fund_expected_nav_date($1::timestamptz)",
            vec![Bind::from(now)],
        )
        .fetch_one(&pool)
        .await
        .expect("fund_expected_nav_date");
        assert_eq!(got, want, "{ts}");
    }

    let user_id = create_test_user(&pool).await;
    backend::services::fund_balance::bulk_create(
        &pool,
        user_id,
        &[backend::models::fund_balance::CreateFundBalanceRequest {
            fund_name: "架空基準日ファンド".to_string(),
            account: shared::value::Account::try_from("特定").unwrap(),
            dividends_course: None,
            shares: dec!(10000),
            average_purchase_price: dec!(10000),
            total_purchase_amount: dec!(100000),
            base_price: dec!(10500),
            base_price_day_change: Some(dec!(5)),
            base_price_month_change: None,
            market_value: dec!(105000),
            profit_loss: dec!(5000),
            profit_loss_rate: None,
            total_return: None,
        }],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("bulk_create");
    let (matches,): (bool,) = db::query(
        "SELECT price_as_of = fund_expected_nav_date(created_at) FROM fund_balances WHERE user_id = $1",
        vec![Bind::from(user_id)],
    )
    .fetch_one(&pool)
    .await
    .expect("price_as_of");
    assert!(matches);
}
