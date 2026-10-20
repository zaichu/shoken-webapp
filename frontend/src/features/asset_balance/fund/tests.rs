use super::*;

fn dec(value: &str) -> Decimal {
    value.parse().unwrap()
}

fn csv_row(
    name: &str,
    account: &str,
    course: Option<&str>,
    market: &str,
    profit: &str,
) -> FundBalanceRow {
    FundBalanceRow::Preview(FundBalanceCsvRow {
        fund_name: name.to_string(),
        account: account.to_string(),
        dividends_course: course.map(str::to_string),
        shares: dec("10000"),
        average_purchase_price: dec("10000"),
        total_purchase_amount: dec("1000"),
        base_price: dec("11000"),
        market_value: dec(market),
        profit_loss: dec(profit),
        ..Default::default()
    })
}

#[test]
fn detect_kind_reads_fund_header_after_preamble() {
    let head = "資産残高のお知らせ\n\n架空証券\n\n\"投資信託種別\",\"口座区分\",\"ファンド\"\n";
    assert_eq!(detect_asset_csv_kind(head.as_bytes()), AssetKind::Fund);
}

#[test]
fn detect_kind_reads_stock_header() {
    let head = "前置き\n\"銘柄コード\",\"銘柄名\"\n\"1234\",\"テスト\"\n";
    assert_eq!(detect_asset_csv_kind(head.as_bytes()), AssetKind::Stock);
}

#[test]
fn detect_kind_defaults_to_stock() {
    assert_eq!(detect_asset_csv_kind(b""), AssetKind::Stock);
    assert_eq!(detect_asset_csv_kind("なにか".as_bytes()), AssetKind::Stock);
}

#[test]
fn detect_kind_handles_shift_jis() {
    // 「投資信託種別」を CP932 で書いた行(実ファイルの先頭と同じ並び)
    let sjis: &[u8] = b"\x93\x8a\x8e\x91\x90\x4d\x91\xf5\x8e\xed\x95\xca,\"a\"\n";
    assert_eq!(detect_asset_csv_kind(sjis), AssetKind::Fund);
}

fn rows() -> Vec<FundBalanceRow> {
    vec![
        csv_row("Aファンド", "特定", Some("受取型"), "100", "10"),
        csv_row("Bファンド", "NISA", Some("再投資型"), "300", "-5"),
        csv_row("Cインデックス", "旧NISA", None, "200", "30"),
    ]
}

#[test]
fn filter_by_text_matches_name_and_account() {
    let rows = rows();
    let search = AssetSearch {
        text: "インデックス".to_string(),
        ..Default::default()
    };
    assert_eq!(filter_fund_rows(&rows, &search).len(), 1);
    let search = AssetSearch {
        text: "nisa".to_string(),
        ..Default::default()
    };
    assert_eq!(filter_fund_rows(&rows, &search).len(), 2);
}

#[test]
fn filter_by_accounts_and_courses() {
    let rows = rows();
    let search = AssetSearch {
        accounts: vec!["NISA".to_string(), "旧NISA".to_string()],
        ..Default::default()
    };
    assert_eq!(filter_fund_rows(&rows, &search).len(), 2);

    let search = AssetSearch {
        courses: vec!["再投資型".to_string()],
        ..Default::default()
    };
    let filtered = filter_fund_rows(&rows, &search);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].fund_name(), "Bファンド");

    // 組み合わせは AND
    let search = AssetSearch {
        accounts: vec!["旧NISA".to_string()],
        courses: vec!["再投資型".to_string()],
        ..Default::default()
    };
    assert!(filter_fund_rows(&rows, &search).is_empty());
}

#[test]
fn sort_by_market_value_desc_then_name() {
    let rows = rows();
    let views: Vec<FundView> = rows.iter().map(fund_view).collect();
    let order = sort_fund_views(&views, FundSortKey::MarketValue, SortOrder::Desc);
    let names: Vec<_> = order.iter().map(|&i| views[i].name.as_str()).collect();
    assert_eq!(names, ["Bファンド", "Cインデックス", "Aファンド"]);

    let order = sort_fund_views(&views, FundSortKey::MarketValue, SortOrder::Asc);
    let names: Vec<_> = order.iter().map(|&i| views[i].name.as_str()).collect();
    assert_eq!(names, ["Aファンド", "Cインデックス", "Bファンド"]);
}

#[test]
fn totals_weight_daily_change_by_shares() {
    let mut row_a = FundBalanceCsvRow {
        shares: dec("20000"),
        base_price_day_change: Some(dec("100")),
        market_value: dec("220"),
        total_purchase_amount: dec("200"),
        profit_loss: dec("20"),
        ..Default::default()
    };
    // 前日比を持たない行は合計に混ぜない
    let row_b = FundBalanceCsvRow {
        shares: dec("10000"),
        base_price_day_change: None,
        market_value: dec("110"),
        total_purchase_amount: dec("100"),
        profit_loss: dec("10"),
        ..Default::default()
    };
    row_a.fund_name = "A".to_string();
    let rows = [
        FundBalanceRow::Preview(row_a),
        FundBalanceRow::Preview(row_b),
    ];
    let views: Vec<FundView> = rows.iter().map(fund_view).collect();
    let totals = fund_totals(&views);
    assert_eq!(totals.market, 330.0);
    assert_eq!(totals.purchase, 300.0);
    assert_eq!(totals.profit, 30.0);
    // 20000口 × 100円 / 10000 = 200円
    assert_eq!(totals.daily_change, 200.0);
    assert!(totals.has_daily_change);
}

#[test]
fn fund_accounts_and_courses_dedup() {
    let rows = vec![
        csv_row("A", "特定", Some("受取型"), "1", "0"),
        csv_row("B", "特定", None, "1", "0"),
        csv_row("C", "NISA", Some("受取型"), "1", "0"),
    ];
    assert_eq!(fund_accounts(&rows), ["特定", "NISA"]);
    assert_eq!(fund_courses(&rows), ["受取型"]);
}

fn saved_fund(
    name: &str,
    price_source: &str,
    nav_is_latest: bool,
    day_change: Option<i64>,
    price_as_of: &str,
    expected_nav_date: Option<&str>,
) -> FundBalanceRow {
    FundBalanceRow::Saved(
        serde_json::from_value(serde_json::json!({
            "id": format!("fund-{name}"),
            "fund_name": name,
            "dividends_course": null,
            "account": "特定",
            "shares": 10000,
            "average_purchase_price": 10000,
            "total_purchase_amount": 1000,
            "base_price": 11000,
            "base_price_day_change": day_change,
            "base_price_month_change": null,
            "market_value": 1100,
            "profit_loss": 100,
            "profit_loss_rate": 10.0,
            "total_return": null,
            "price_as_of": price_as_of,
            "price_source": price_source,
            "nav_is_latest": nav_is_latest,
            "expected_nav_date": expected_nav_date,
            "created_at": "2026-10-17T00:00:00Z",
            "updated_at": "2026-10-18T00:00:00Z",
        }))
        .unwrap(),
    )
}

#[test]
fn view_day_change_is_hidden_when_nav_is_pending() {
    let row = saved_fund(
        "A",
        "csv",
        false,
        Some(50),
        "2026-10-10",
        Some("2026-10-17"),
    );
    let view = fund_view(&row);
    assert!(view.nav_pending());
    assert_eq!(view.view_day_change(), None);
    let title = view.day_change_title().unwrap();
    assert!(title.contains("更新待ち"));

    let latest = saved_fund("B", "api", true, Some(50), "2026-10-17", None);
    let view = fund_view(&latest);
    assert!(!view.nav_pending());
    assert_eq!(view.view_day_change(), Some(50.0));
    let title = view.day_change_title().unwrap();
    assert!(title.contains("自動更新"));
    assert!(title.contains("10/17"));
}

#[test]
fn totals_count_latest_and_pending_and_exclude_pending_change() {
    let rows = [
        saved_fund("A", "api", true, Some(100), "2026-10-17", None),
        saved_fund(
            "B",
            "csv",
            false,
            Some(50),
            "2026-10-10",
            Some("2026-10-17"),
        ),
        saved_fund("C", "csv", false, None, "2026-10-10", None),
    ];
    let views: Vec<FundView> = rows.iter().map(fund_view).collect();
    let totals = fund_totals(&views);
    // 更新待ちの B・C は合計に入らない。100円 × 10000口 / 10000 = 100円
    assert_eq!(totals.daily_change, 100.0);
    assert_eq!(totals.latest_nav_count, 1);
    assert_eq!(totals.pending_nav_count, 2);
    assert!(totals.has_daily_change);
}
