use super::data::*;
use super::format::*;
use super::holdings::*;
use super::summary::{PortfolioSummary, PortfolioSummaryProps};
use super::test_util::*;
use crate::api::ApiError;
use crate::asset_balance_domain::{
    calculate_portfolio_kpi, format_number_value, normalize_security_code, normalize_security_name,
    summarize_valuation_with_summary, total_purchase_amount, KpiHolding, SummaryOverride,
    ValuationItem,
};
use crate::asset_balance_lookup::AssetBalanceLookupStore;
use crate::asset_balance_portfolio::chart_display;
use crate::dividend_per_share::DividendMaps;
use crate::dto::AssetBalanceSummary;
use crate::security_link::is_searchable_code;
use crate::session::SessionStore;
use leptos::prelude::*;
use rust_decimal::Decimal;
use std::collections::HashMap;

#[test]
fn stale_asset_balance_result_is_rejected_after_same_or_different_user_login() {
    let owner = Owner::new();
    owner.with(|| {
        for next_user in [user("alice"), user("bob")] {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let fetch_generation = session.generation.get_untracked();

            session.mark_unauthenticated();
            session.user.set(Some(next_user));

            assert!(!should_apply_asset_balance_result(
                &session,
                fetch_generation
            ));
        }
    });
}

#[test]
fn unauthorized_asset_balance_error_uses_expected_message() {
    assert_eq!(ApiError::http(401).message(), "認証が必要です");
}

#[test]
fn percentage_formatter_formats_values() {
    assert_eq!(format_percentage_value(2.0), "2.00%");
    assert_eq!(format_percentage_value(1.3770010052107338), "1.38%");
    assert_eq!(format_fixed_percent(5.300076869056115, 1), "5.3%");
    assert_eq!(format_fixed_percent(10.0, 1), "10.0%");
    assert_eq!(format_fixed_percent(8.256880733944955, 1), "8.3%");
    assert_eq!(format_fixed_percent(60.0, 1), "60.0%");
    assert_eq!(format_valuation_amount(Some(60000.0)), "+¥ 60,000");
    assert_eq!(format_valuation_amount(Some(-10000.0)), "¥ -10,000");
    assert_eq!(format_valuation_amount(None), "—");
    assert_eq!(format_valuation_rate(Some(10.0), 1), "+10.0%");
    assert_eq!(format_valuation_rate(Some(-10.0), 1), "-10.0%");
    assert_eq!(format_valuation_rate(None, 1), "—");
}

#[test]
fn holding_dividend_matches_component_cases() {
    let maps = DividendMaps {
        per_share: HashMap::from([("7203".to_string(), 50.0), ("6758".to_string(), 360.0)]),
        status: HashMap::from([
            ("0001".to_string(), "pending".to_string()),
            ("0002".to_string(), "error".to_string()),
            ("0003".to_string(), "zero".to_string()),
        ]),
    };
    let with_data = holding_dividend("7203", 100.0, 2500.0, &maps);
    assert_eq!(with_data.per_share, Some(50.0));
    assert_eq!(with_data.annual, Some(5000.0));
    assert_eq!(with_data.yield_value, Some(2.0));
    assert_eq!(format_dividend_yield(&with_data), "2.00%");

    let missing = holding_dividend("9999", 10.0, 100.0, &maps);
    assert_eq!(missing.per_share, None);
    assert_eq!(format_dividend_per_share(&missing), "---");

    let pending = holding_dividend("0001", 10.0, 100.0, &maps);
    assert_eq!(format_dividend_per_share(&pending), "取得中...");

    let error = holding_dividend("0002", 10.0, 100.0, &maps);
    assert_eq!(format_dividend_annual(&error), "取得失敗");

    let zero = holding_dividend("0003", 10.0, 100.0, &maps);
    assert_eq!(zero.per_share, Some(0.0));
    assert_eq!(zero.annual, Some(0.0));
    assert_eq!(format_dividend_per_share(&zero), "¥ 0");
    assert_eq!(format_dividend_yield(&zero), "---");

    let zero_price = holding_dividend("7203", 10.0, 0.0, &maps);
    assert_eq!(zero_price.yield_value, None);

    let mut maps = maps;
    maps.per_share.insert("0004".to_string(), 30.0);
    maps.status
        .insert("0004".to_string(), "pending".to_string());
    let pending_with_value = holding_dividend("0004", 10.0, 100.0, &maps);
    assert_eq!(pending_with_value.per_share, None);
    assert_eq!(pending_with_value.annual, None);
    assert_eq!(pending_with_value.yield_value, None);

    for pending in [
        holding_dividend("0001", 10.0, 100.0, &maps),
        pending_with_value,
    ] {
        assert_eq!(format_dividend_per_share(&pending), "取得中...");
        assert_eq!(format_dividend_annual(&pending), "取得中...");
        assert_eq!(format_dividend_yield(&pending), "取得中...");
    }
    let error = holding_dividend("0002", 10.0, 100.0, &maps);
    assert_eq!(format_dividend_per_share(&error), "取得失敗");
    assert_eq!(format_dividend_annual(&error), "取得失敗");
    assert_eq!(format_dividend_yield(&error), "取得失敗");
}

#[test]
fn security_name_normalization_cases() {
    assert_eq!(normalize_security_name("ＫＤＤＩ"), "KDDI");
    assert_eq!(normalize_security_name("トヨタ自動車"), "トヨタ自動車");
    assert_eq!(normalize_security_code(" 7203: トヨタ自動車 "), "7203");
    assert!(is_searchable_code("7203"));
    assert!(is_searchable_code("BRK.B"));
    assert!(!is_searchable_code(""));
    assert!(!is_searchable_code("7203: トヨタ"));
}

#[test]
fn number_and_currency_formatters_match_intl_cases() {
    assert_eq!(format_number_value(100.0), "100");
    assert_eq!(format_number_value(-12345.0), "-12,345");
    assert_eq!(format_number_value(1.5), "1.5");
    assert_eq!(format_number_value(1.2345), "1.23");
    assert_eq!(format_number_value(-0.001), "-0");
    // 符号は丸め前の値の `num < 0` で判定するため -0 は `0` と表示する
    assert_eq!(format_number_value(-0.0), "0");
    assert_eq!(format_currency(-0.0), "¥ 0");
    assert_eq!(format_number_value(-12345.678), "-12,345.68");
    // Intl.NumberFormat は toFixed と異なり10進の値で半分以上を切り上げる
    assert_eq!(format_number_value(1.005), "1.01");
    assert_eq!(format_number_value(-1.005), "-1.01");
    assert_eq!(format_number_value(2.675), "2.68");
    assert_eq!(format_number_value(0.995), "1");
    assert_eq!(format_currency(850000.0), "¥ 850,000");
    assert_eq!(format_currency(0.0), "¥ 0");
    assert_eq!(format_currency(-250000.0), "¥ -250,000");
    assert_eq!(format_currency(2600.0), "¥ 2,600");
    assert_eq!(format_currency(123.456), "¥ 123.456");
    assert_eq!(format_currency(f64::NAN), "-");
    assert_eq!(
        format_currency("0.123456789012345678".parse::<f64>().unwrap()),
        "¥ 0.123456789012346"
    );
    assert_eq!(format_fixed_percent(f64::NAN, 1), "-");
}

#[test]
fn api_summary_is_ignored_while_searching() {
    let summary = || {
        Some(AssetBalanceSummary {
            total_market_value: Decimal::from(260_000),
            total_purchase_amount: Decimal::from(250_000),
            total_daily_change: Decimal::ZERO,
        })
    };
    assert_eq!(effective_summary(summary(), "", false), summary());
    assert_eq!(effective_summary(summary(), "7203", false), None);
    assert_eq!(effective_summary(summary(), "", true), None);
    assert_eq!(effective_summary(None, "", false), None);
}

#[test]
fn filtered_portfolio_shows_filtered_row_totals_while_searching() {
    let owner = Owner::new();
    owner.with(|| {
        // 行合計と一致しない API summary を使い、画面の金額が表示中の行から来ることを確認する
        let mut toyota = balance_row(7203);
        toyota.security_name = "トヨタ自動車".to_string();
        toyota.total_purchase_amount = rust_decimal_macros::dec!(100000);
        toyota.market_value = rust_decimal_macros::dec!(110000);
        let mut sony = balance_row(6758);
        sony.security_name = "ソニーグループ".to_string();
        sony.total_purchase_amount = rust_decimal_macros::dec!(200000);
        sony.market_value = rust_decimal_macros::dec!(180000);
        let loaded = LoadedAssetBalances {
            total: 2,
            rows: vec![toyota, sony],
            summary: Some(AssetBalanceSummary {
                total_market_value: rust_decimal_macros::dec!(999999),
                total_purchase_amount: rust_decimal_macros::dec!(888888),
                total_daily_change: rust_decimal_macros::dec!(0),
            }),
            facets: None,
            truncated: false,
        };
        let lookup = RwSignal::new(AssetBalanceLookupStore::new());
        lookup.update(|store| store.seed(1, &loaded.rows));

        let filtered = filtered_portfolio(
            &loaded.rows,
            loaded.summary.clone(),
            "7203",
            lookup,
            1,
            false,
        );
        assert_eq!(filtered.views.len(), 1);
        assert_eq!(filtered.views[0].code, "7203");
        assert!(filtered.summary.is_none());

        // PortfolioSummary と同じ手順で画面に出る金額を計算する
        let valuation_items: Vec<ValuationItem> = filtered
            .views
            .iter()
            .map(|view| ValuationItem {
                market_value: serde_json::json!(view.market),
                total_purchase_amount: serde_json::json!(view.purchase),
            })
            .collect();
        let summary_override = filtered.summary.as_ref().map(|summary| SummaryOverride {
            total_purchase_amount: serde_json::json!(dec_to_f64(&summary.total_purchase_amount,)),
            total_market_value: serde_json::json!(dec_to_f64(&summary.total_market_value)),
        });
        let valuation =
            summarize_valuation_with_summary(&valuation_items, summary_override.as_ref());
        let kpi_holdings: Vec<KpiHolding> = filtered
            .views
            .iter()
            .map(|view| KpiHolding {
                security_code: view.code.clone(),
                shares: view.shares,
                total_purchase_amount: view.purchase,
            })
            .collect();
        let summary_total = filtered
            .summary
            .as_ref()
            .map(|summary| dec_to_f64(&summary.total_purchase_amount));
        let total_purchase = total_purchase_amount(&kpi_holdings, summary_total);
        let dividends = RwSignal::new(DividendMaps {
            per_share: HashMap::from([("7203".to_string(), 50.0)]),
            status: HashMap::new(),
        });
        let kpi = Memo::new(move |_| {
            calculate_portfolio_kpi(&kpi_holdings, &dividends.get().per_share, summary_total)
        });

        assert_eq!(valuation.market_value, Some(110000.0));
        assert_eq!(valuation.amount, Some(10000.0));
        assert_eq!(valuation.rate, Some(10.0));
        assert_eq!(format_currency(total_purchase), "¥ 100,000");
        kpi.with(|kpi| {
            assert_eq!(kpi.total_purchase_amount, 100000.0);
            assert_eq!(kpi.total_annual_dividends, Some(5000.0));
            assert_eq!(kpi.dividend_yield, Some(5.0));
            assert_eq!(kpi.holdings_count, 1);
        });

        let unfiltered =
            filtered_portfolio(&loaded.rows, loaded.summary.clone(), "", lookup, 1, false);
        assert_eq!(unfiltered.views.len(), 2);
        assert_eq!(
            unfiltered
                .summary
                .map(|summary| summary.total_purchase_amount),
            Some(rust_decimal_macros::dec!(888888))
        );
    });
}

#[test]
fn show_all_resets_when_summary_stops_drawing_the_chart() {
    let owner = Owner::new();
    owner.with(|| {
        let dividends = RwSignal::new(DividendMaps::default());
        let show_all = RwSignal::new(true);
        let views: Vec<HoldingView> = (0..21).map(|id| holding_view(&balance_row(id))).collect();
        let summary_view = |views: Vec<HoldingView>| {
            PortfolioSummary(
                PortfolioSummaryProps::builder()
                    .views(views)
                    .total_count(21)
                    .is_filtered(true)
                    .on_clear_filter(|| {})
                    .summary(None)
                    .dividends(dividends)
                    .show_all(show_all)
                    .build(),
            )
        };

        let _ = summary_view(views.clone());
        assert!(show_all.get_untracked());

        let _ = summary_view(Vec::new());
        assert!(!show_all.get_untracked());

        show_all.set(true);
        let mut zero = balance_row(9999);
        zero.total_purchase_amount = rust_decimal_macros::dec!(0);
        zero.market_value = rust_decimal_macros::dec!(0);
        let _ = summary_view(vec![holding_view(&zero)]);
        assert!(!show_all.get_untracked());

        let _ = summary_view(views);
        assert!(!show_all.get_untracked());
        let display = chart_display(21, &[Some(100.0 / 21.0); 21], show_all.get_untracked());
        assert_eq!(display.visible_count, 20);
        assert_eq!(
            display.toggle_label.as_deref(),
            Some("残り1銘柄を表示（全21）")
        );
    });
}

#[test]
fn dividend_maps_apply_only_to_current_generation() {
    let owner = Owner::new();
    owner.with(|| {
        let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
        let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps::default());
        let fresh = DividendMaps {
            per_share: HashMap::from([("7203".to_string(), 50.0)]),
            status: HashMap::new(),
        };
        apply_dividend_maps(&balances, &dividends, 7, fresh.clone());
        assert!(balances.get_untracked().is_none());
        assert!(dividends.get_untracked().per_share.is_empty());
        balances.set(Some((
            7,
            Ok(LoadedAssetBalances {
                rows: vec![],
                total: 0,
                summary: None,
                facets: None,
                truncated: false,
            }),
        )));
        apply_dividend_maps(&balances, &dividends, 8, fresh.clone());
        assert!(dividends.get_untracked().per_share.is_empty());
        apply_dividend_maps(&balances, &dividends, 7, fresh);
        assert_eq!(dividends.get_untracked().per_share.get("7203"), Some(&50.0));
    });
}

#[test]
fn dividend_update_does_not_notify_balance_view() {
    // 配当更新で balances の購読者(ページ側の動的 view)を再実行させないこと。
    // 再実行されると子コンポーネントが作り直され、開閉状態が失われる。
    let owner = Owner::new();
    owner.with(|| {
        let balances: RwSignal<BalanceSlot> = RwSignal::new(Some((
            7,
            Ok(LoadedAssetBalances {
                rows: vec![],
                total: 0,
                summary: None,
                facets: None,
                truncated: false,
            }),
        )));
        let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps::default());
        let runs = RwSignal::new(0u32);
        let balance_view = Memo::new(move |_| {
            runs.update(|count| *count += 1);
            balances.get().is_some()
        });
        assert!(balance_view.get());

        apply_dividend_maps(
            &balances,
            &dividends,
            7,
            DividendMaps {
                per_share: HashMap::from([("7203".to_string(), 50.0)]),
                status: HashMap::new(),
            },
        );

        assert!(balance_view.get());
        assert_eq!(runs.get_untracked(), 1);
        assert_eq!(dividends.get_untracked().per_share.get("7203"), Some(&50.0));
    });
}

#[test]
fn valuation_formatters_treat_nan_as_missing() {
    assert_eq!(format_valuation_amount(Some(f64::NAN)), "—");
    assert_eq!(format_valuation_rate(Some(f64::NAN), 1), "—");
}

#[test]
fn asset_balance_result_applies_within_the_same_generation() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        assert!(should_apply_asset_balance_result(&session, generation));
    });
}
