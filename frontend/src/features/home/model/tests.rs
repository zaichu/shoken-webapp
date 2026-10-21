use super::*;
use crate::features::dividend_per_share::DividendMaps;
use rust_decimal_macros::dec;

fn stock(code: &str, name: &str, shares: i64, price: i64, purchase: i64) -> AssetBalance {
    serde_json::from_value(serde_json::json!({
        "id": format!("stock-{code}"),
        "security_code": code,
        "security_name": name,
        "shares": shares,
        "average_purchase_price": purchase / shares,
        "total_purchase_amount": purchase,
        "current_price": price,
        "daily_change": 0,
        "price_as_of": "2026-10-17",
        "prev_close": price,
        "created_at": "",
        "updated_at": "2026-10-18T00:00:00Z"
    }))
    .unwrap()
}

fn fund(name: &str, account: &str, market: i64, pnl: i64) -> FundBalance {
    serde_json::from_value(serde_json::json!({
        "id": format!("fund-{name}"),
        "fund_name": name,
        "dividends_course": null,
        "account": account,
        "shares": 1000,
        "average_purchase_price": 100,
        "total_purchase_amount": market - pnl,
        "base_price": 110,
        "base_price_day_change": null,
        "base_price_month_change": null,
        "market_value": market,
        "profit_loss": pnl,
        "profit_loss_rate": 5.5,
        "total_return": null,
        "price_as_of": "2026-10-17",
        "created_at": "",
        "updated_at": "2026-10-18T00:00:00Z"
    }))
    .unwrap()
}

fn dividend(date: &str, net: i64) -> Dividend {
    dividend_with_tax(date, net, net * 5 / 4, net / 4)
}

fn dividend_with_tax(date: &str, net: i64, before: i64, tax: i64) -> Dividend {
    serde_json::from_value(serde_json::json!({
        "id": format!("div-{date}-{net}"),
        "settlement_date": date,
        "product": "国内株式",
        "account": "特定",
        "security_code": "9999",
        "security_name": "架空工業",
        "unit_price": 1,
        "shares": 1,
        "dividends_before_tax": before,
        "taxes": tax,
        "net_amount_received": net,
        "created_at": "",
        "updated_at": "2026-10-18T00:00:00Z"
    }))
    .unwrap()
}

#[test]
fn merged_asset_summary_combines_stock_and_fund() {
    let stock = AssetBalanceSummary {
        total_market_value: dec!(1_000_000),
        total_purchase_amount: dec!(800_000),
        total_daily_change: dec!(10_000),
        priced_market_value: dec!(600_000),
        priced_purchase_amount: dec!(500_000),
        has_missing_daily_change: false,
    };
    let fund = FundBalanceSummary {
        total_market_value: dec!(300_000),
        total_purchase_amount: dec!(250_000),
        total_daily_change: dec!(2_000),
        latest_nav_count: 1,
        pending_nav_count: 0,
        nav_status: shared::domain::FundNavStatus::default(),
    };
    let merged = merged_asset_summary(Some(&stock), Some(&fund)).unwrap();
    assert_eq!(merged.total_market_value, dec!(1_300_000));
    assert_eq!(merged.total_purchase_amount, dec!(1_050_000));
    // ファンドに価格なし区別はないため、価格あり集計はファンド全件を含む
    assert_eq!(merged.priced_market_value, dec!(900_000));
    assert_eq!(merged.priced_purchase_amount, dec!(750_000));

    // ファンド欠損時は株式のみ、両方欠損なら「集計なし」
    let only_stock = merged_asset_summary(Some(&stock), None).unwrap();
    assert_eq!(only_stock.total_market_value, dec!(1_000_000));
    assert!(merged_asset_summary(None, None).is_none());
}

#[test]
fn row_summaries_skip_unpriced_stocks() {
    let rows = vec![
        stock("1111", "A", 100, 200, 20_000),
        stock("2222", "B", 100, 0, 30_000),
    ];
    let summary = stock_summary_from_rows(&rows);
    assert_eq!(summary.total_market_value, dec!(20_000));
    assert_eq!(summary.priced_market_value, dec!(20_000));
    // 価格なし行は取得総額の合計だけには残る
    assert_eq!(summary.total_purchase_amount, dec!(50_000));
    assert_eq!(summary.priced_purchase_amount, dec!(20_000));
}

#[test]
fn monthly_totals_and_year_comparison() {
    let rows = vec![
        dividend("2025-01-15", 1000),
        dividend("2025-01-20", 2000),
        dividend("2025-03-10", 3000),
        dividend("2025-12-25", 5000),
    ];
    let totals = monthly_net_totals(&rows);
    assert_eq!(totals[0], dec!(3000));
    assert_eq!(totals[2], dec!(3000));
    assert_eq!(totals[11], dec!(5000));
    // 前年同期(1月〜2月)と残り期間(3月〜12月)の分け方
    assert_eq!(sum_month_range(&totals, 1, 2), dec!(3000));
    assert_eq!(sum_month_range(&totals, 3, 12), dec!(8000));
    assert_eq!(sum_month_range(&totals, 12, 3), dec!(0));
}

#[test]
fn dividend_card_compares_with_same_period() {
    // 10月時点: 今年 1-10月の入金 vs 前年 1-10月の入金。前年の11-12月は「残り期間」
    let current = vec![dividend("2026-03-10", 12000), dividend("2026-09-15", 8000)];
    let previous = vec![
        dividend("2025-02-10", 5000),
        dividend("2025-06-10", 3000),
        dividend("2025-11-10", 9000),
        dividend("2025-12-10", 1000),
    ];
    let card = build_dividend_card(&current, Some(&previous), Some(40_000.0), 10);
    assert_eq!(card.received, dec!(20000));
    assert_eq!(card.yoy_diff, Some(dec!(12000)));
    assert_eq!(card.previous_remaining, Some(dec!(10000)));
    // 予想(税引後) = 40000 × (1 - 実効税率 0.2)
    assert_eq!(card.forecast_net, Some(32_000.0));
    assert_eq!(card.progress, Some(0.625));
    assert_eq!(card.remaining, Some(12_000.0));
    assert_eq!(card.monthly.len(), 12);
    assert_eq!(card.monthly[8].current, dec!(8000));
}

#[test]
fn dividend_card_applies_effective_tax_rate() {
    // 税引前 1000 に対し税 200(=20%)の実績 → 予想は 80% に換算
    let current = vec![dividend_with_tax("2026-01-10", 800, 1000, 200)];
    let card = build_dividend_card(&current, Some(&[]), Some(10_000.0), 2);
    assert_eq!(card.forecast_net, Some(8_000.0));
    // 実績が無い年は法定税率(20.315%)に倒す
    let card = build_dividend_card(&[], Some(&[]), Some(10_000.0), 2);
    assert_eq!(card.forecast_net, Some(10_000.0 * (1.0 - 0.20315)));
}

#[test]
fn effective_tax_rate_blends_accounts() {
    let taxed = dividend_with_tax("2026-01-10", 800, 1000, 200);
    let nisa = dividend_with_tax("2026-01-11", 500, 500, 0);
    let rate = effective_tax_rate(&[taxed], &[nisa]).unwrap();
    assert_eq!(rate, dec!(200) / dec!(1500));
    assert!(effective_tax_rate(&[], &[]).is_none());
}

#[test]
fn dividend_card_handles_empty_and_zero() {
    let card = build_dividend_card(&[], Some(&[]), None, 5);
    assert_eq!(card.received, dec!(0));
    assert!(card.forecast_net.is_none());
    assert!(card.progress.is_none());
    assert!(card.remaining.is_none());
    assert_eq!(card.yoy_diff, Some(dec!(0)));
    // 12月では前年の残り期間が無い
    let card = build_dividend_card(&[], Some(&[dividend("2025-12-01", 3000)]), None, 12);
    assert_eq!(card.previous_remaining, Some(dec!(0)));
    assert_eq!(card.yoy_diff, Some(dec!(-3000)));
}

#[test]
fn dividend_card_hides_yoy_when_previous_fetch_failed() {
    // 前年の取得失敗は「前年0円」とは別物。前年比較の数値は出さず、
    // 月別グラフの前年系列だけ 0 に倒す
    let card = build_dividend_card(&[dividend("2026-03-10", 12000)], None, None, 10);
    assert!(card.yoy_diff.is_none());
    assert!(card.previous_remaining.is_none());
    assert!(card.monthly.iter().all(|pair| pair.previous.is_zero()));
    assert_eq!(card.monthly[2].current, dec!(12000));
}

#[test]
fn product_slices_sort_desc_and_drop_empty() {
    let slices = product_slices(1_000_000.0, 500_000.0);
    assert_eq!(slices.len(), 2);
    assert_eq!(slices[0].label, "国内株式");
    assert_eq!(slices[1].label, "投資信託");
    let slices = product_slices(0.0, 500_000.0);
    assert_eq!(slices.len(), 1);
    assert_eq!(slices[0].label, "投資信託");
    assert!(product_slices(0.0, 0.0).is_empty());
}

#[test]
fn account_slices_group_funds_by_account() {
    let funds = vec![
        fund("F1", "特定", 600_000, 10_000),
        fund("F2", "NISA", 200_000, 5_000),
        fund("F3", "特定", 200_000, -5_000),
        fund("F4", "NISA成長投資枠", 100_000, 1_000),
    ];
    let slices = account_slices(&funds);
    // 特定は合算され 800,000。株式は口座を持たないため口座別には出ない
    assert_eq!(slices.len(), 3);
    assert_eq!(slices[0].label, "特定");
    assert_eq!(slices[0].amount, 800_000.0);
    // 長い口座名は取引明細と同じ表示名に短縮してから合算する
    assert_eq!(slices[2].label, "NISA成長");
    assert!(slices.iter().all(|s| s.label != "国内株式"));
    // ファンドが無ければ口座内訳は作らない
    assert!(account_slices(&[]).is_empty());
}

#[test]
fn pnl_extremes_pick_top_gains_and_losses() {
    let stocks = vec![
        stock("1", "Gain1", 100, 300, 10_000),  // +20,000
        stock("2", "Gain2", 100, 250, 10_000),  // +15,000
        stock("3", "Gain3", 100, 200, 10_000),  // +10,000
        stock("4", "Gain4", 100, 150, 10_000),  // +5,000(落選)
        stock("5", "Loss1", 100, 50, 10_000),   // -5,000
        stock("6", "Loss2", 100, 20, 10_000),   // -8,000
        stock("7", "Unpriced", 100, 0, 10_000), // 価格なしは除外
    ];
    let funds = vec![fund("含み損F", "特定", 8_000, -2_000)];
    let holdings = holding_pnls(&stocks, &funds);
    assert_eq!(holdings.len(), 7, "価格なし銘柄は含めない");
    let (gains, losses) = pnl_extremes(&holdings, 3, 2);
    assert_eq!(gains.len(), 3);
    assert_eq!(gains[0].name, "Gain1");
    assert_eq!(gains[2].name, "Gain3");
    assert_eq!(losses.len(), 2);
    assert_eq!(losses[0].profit_loss, -8_000.0);
    assert_eq!(losses[1].profit_loss, -5_000.0);
    // ファンド行も比較対象に入る
    let (_, losses) = pnl_extremes(&holdings, 3, 3);
    assert_eq!(losses.len(), 3);
    assert_eq!(losses[2].name, "含み損F");
    // 空でも壊れない
    let (gains, losses) = pnl_extremes(&[], 3, 2);
    assert!(gains.is_empty() && losses.is_empty());
}

#[test]
fn annual_forecast_uses_per_share_and_shares() {
    let stocks = vec![
        stock("1111", "A", 100, 0, 10_000),
        stock("2222", "B", 200, 0, 40_000),
    ];
    let mut maps = DividendMaps::default();
    maps.per_share.insert("1111".to_string(), 50.0);
    maps.per_share.insert("2222".to_string(), 0.0);
    maps.status.insert("1111".to_string(), "ok".to_string());
    maps.status.insert("2222".to_string(), "zero".to_string());
    let kpi = annual_dividend_forecast(&stocks, &maps);
    assert_eq!(kpi.total_annual_dividends, Some(5_000.0));
    assert_eq!(kpi.dividend_yield, Some(10.0));
    assert!(!has_pending_estimate(&maps));
    maps.status
        .insert("3333".to_string(), "pending".to_string());
    assert!(has_pending_estimate(&maps));
    // 見積もりが空なら算出不可
    assert!(
        annual_dividend_forecast(&stocks, &DividendMaps::default())
            .total_annual_dividends
            .is_none()
    );
}

#[test]
fn remaining_label_writes_overage_as_exceeding() {
    // 受取が予想を超えたら負の残りではなく「上回り」に書き分ける
    assert_eq!(remaining_label(Some(-17_500.0)), "予想を ¥17,500 上回り");
    assert_eq!(remaining_label(Some(12_000.0)), "残り ¥12,000");
    assert_eq!(remaining_label(None), "残り —");
}

#[test]
fn bar_height_has_visibility_floor() {
    assert_eq!(bar_height(dec!(0), dec!(100)), 0.0);
    assert_eq!(bar_height(dec!(100), dec!(0)), 0.0);
    assert_eq!(bar_height(dec!(50), dec!(100)), 50.0);
    // 1% 未満でも棒が見えるよう 3% に底上げ
    assert_eq!(bar_height(dec!(1), dec!(100)), 3.0);
}
