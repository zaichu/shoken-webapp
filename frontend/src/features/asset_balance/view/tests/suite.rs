#[path = "integration.rs"]
mod integration;

use super::summary::{PortfolioSummary, PortfolioSummaryProps};
use crate::api::ApiError;
use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::csv::{AssetBalanceCsvRow, AssetBalanceRow};
use crate::features::asset_balance::format::*;
use crate::features::asset_balance::holdings::*;
use crate::features::asset_balance::lookup::AssetBalanceLookupStore;
use crate::features::asset_balance::model::{
    KpiHolding, ValuationInput, calculate_portfolio_kpi, format_number_value,
    normalize_display_name, normalize_security_code, portfolio_valuation, total_purchase_amount,
};
use crate::features::asset_balance::portfolio::{SortKey, SortOrder};
use crate::features::asset_balance::store::csv::{ResolvedAssetBalance, resolve_asset_balance};
use crate::features::asset_balance::store::*;
use crate::features::dividend_per_share::DividendMaps;
use crate::session::{Generation, SessionStore};
use crate::support::row::Row;
use crate::testing::asset_balance::*;
use leptos::prelude::*;
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

            assert!(!session.is_current(fetch_generation));
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
    assert_eq!(format_valuation_amount(Some(60000.0)), "¥60,000");
    assert_eq!(format_valuation_amount(Some(-10000.0)), "-¥10,000");
    assert_eq!(format_valuation_amount(Some(0.0)), "¥0");
    assert_eq!(
        format_valuation_amount(Some(3632999.9999999995)),
        "¥3,633,000"
    );
    assert_eq!(format_valuation_amount(None), "—");
    assert_eq!(format_valuation_rate(Some(10.0), 1), "10.0%");
    assert_eq!(format_valuation_rate(Some(-10.0), 1), "-10.0%");
    assert_eq!(format_valuation_rate(Some(0.0), 1), "0.0%");
    assert_eq!(format_valuation_rate(None, 1), "—");
}

#[test]
fn holding_dividend_matches_component_cases() {
    let maps = DividendMaps {
        per_share: HashMap::from([("9904".to_string(), 50.0), ("9914".to_string(), 360.0)]),
        status: HashMap::from([
            ("0001".to_string(), "pending".to_string()),
            ("0002".to_string(), "error".to_string()),
            ("0003".to_string(), "zero".to_string()),
        ]),
    };
    let with_data = holding_dividend("9904", 100.0, 2500.0, &maps);
    assert_eq!(with_data.per_share, Some(50.0));
    assert_eq!(with_data.annual, Some(5000.0));
    assert_eq!(with_data.yield_value, Some(2.0));
    assert_eq!(format_dividend_yield(&with_data), "2.00%");

    let missing = holding_dividend("9999", 10.0, 100.0, &maps);
    assert_eq!(missing.per_share, None);
    assert_eq!(format_dividend_per_share(&missing), "—");

    let pending = holding_dividend("0001", 10.0, 100.0, &maps);
    assert_eq!(format_dividend_per_share(&pending), "取得中...");

    let error = holding_dividend("0002", 10.0, 100.0, &maps);
    assert_eq!(format_dividend_annual(&error), "取得失敗");

    let zero = holding_dividend("0003", 10.0, 100.0, &maps);
    assert_eq!(zero.per_share, Some(0.0));
    assert_eq!(zero.annual, Some(0.0));
    assert_eq!(format_dividend_per_share(&zero), "¥0");
    assert_eq!(format_dividend_yield(&zero), "—");

    let zero_price = holding_dividend("9904", 10.0, 0.0, &maps);
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
    assert_eq!(normalize_display_name("ＸＹＺ"), "XYZ");
    assert_eq!(normalize_display_name("架空自動車"), "架空自動車");
    assert_eq!(normalize_security_code(" 9904: 架空自動車 "), "9904");
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
    assert_eq!(format_currency(-0.0), "¥0");
    assert_eq!(format_number_value(-12345.678), "-12,345.68");
    // Intl.NumberFormat は toFixed と異なり10進の値で半分以上を切り上げる
    assert_eq!(format_number_value(1.005), "1.01");
    assert_eq!(format_number_value(-1.005), "-1.01");
    assert_eq!(format_number_value(2.675), "2.68");
    assert_eq!(format_number_value(0.995), "1");
    assert_eq!(format_currency(850000.0), "¥850,000");
    assert_eq!(format_currency(0.0), "¥0");
    assert_eq!(format_currency(-250000.0), "-¥250,000");
    assert_eq!(format_currency(2600.0), "¥2,600");
    assert_eq!(format_currency(123.456), "¥123.456");
    assert_eq!(format_fixed_percent(1.005, 2), "1.01%");
    assert_eq!(format_currency(f64::NAN), "—");
    assert_eq!(
        format_currency("0.123456789012345678".parse::<f64>().unwrap()),
        "¥0.123456789012346"
    );
    assert_eq!(format_fixed_percent(f64::NAN, 1), "—");
    assert_eq!(
        format_currency(1e29),
        "¥100,000,000,000,000,000,000,000,000,000"
    );
    assert_eq!(
        format_currency(-1e29),
        "-¥100,000,000,000,000,000,000,000,000,000"
    );
    assert_eq!(
        format_valuation_amount(Some(1e29)),
        "¥100,000,000,000,000,000,000,000,000,000"
    );
    assert_eq!(
        format_fixed_percent(1e30, 1),
        "1000000000000000019884624838656.0%"
    );
    assert_eq!(format_fixed_percent(f64::INFINITY, 1), "—");
}

#[test]
fn filtered_portfolio_keeps_all_holdings_totals_while_searching() {
    let owner = Owner::new();
    owner.with(|| {
        let mut toyota = balance_row(9904);
        toyota.security_name = "架空自動車".to_string();
        toyota.total_purchase_amount = rust_decimal_macros::dec!(100000);
        let mut syn_electronics = balance_row(9914);
        syn_electronics.security_name = "架空エレクトロニクス".to_string();
        syn_electronics.total_purchase_amount = rust_decimal_macros::dec!(200000);
        let loaded = LoadedAssetBalances {
            total: 2,
            rows: vec![toyota, syn_electronics],
            summary: None,
            truncated: false,
        };
        let lookup = RwSignal::new(AssetBalanceLookupStore::new());
        lookup.update(|store| store.seed(Generation::new(1), &loaded.rows));
        let saved_rows: Vec<AssetBalanceRow> =
            loaded.rows.iter().cloned().map(Row::Saved).collect();

        let filtered = filtered_portfolio(&saved_rows, "9904", lookup, Generation::new(1));
        assert_eq!(filtered.views.len(), 1);
        assert_eq!(filtered.views[0].code, "9904");
        // 集計は絞り込み前の全保有分を使う
        assert_eq!(filtered.all_views.len(), 2);

        // PortfolioSummary と同じ手順で画面に出る金額を計算する
        let kpi_holdings: Vec<KpiHolding> = filtered
            .all_views
            .iter()
            .map(|view| KpiHolding {
                security_code: view.code.clone(),
                shares: view.shares,
                total_purchase_amount: view.purchase,
            })
            .collect();
        let total_purchase = total_purchase_amount(&kpi_holdings);
        let dividends = RwSignal::new(DividendMaps {
            per_share: HashMap::from([("9904".to_string(), 50.0)]),
            status: HashMap::new(),
        });
        let kpi =
            Memo::new(move |_| calculate_portfolio_kpi(&kpi_holdings, &dividends.get().per_share));

        // 検索中も合計取得総額・含み損益は全保有分のまま
        assert_eq!(format_currency(total_purchase), "¥300,000");
        kpi.with(|kpi| {
            assert_eq!(kpi.total_purchase_amount, 300000.0);
            assert_eq!(kpi.total_annual_dividends, Some(5000.0));
            assert_eq!(kpi.dividend_yield, Some(5000.0 / 300000.0 * 100.0));
            assert_eq!(kpi.holdings_count, 2);
        });
        let valuation = portfolio_valuation(
            &filtered
                .all_views
                .iter()
                .map(ValuationInput::from)
                .collect::<Vec<_>>(),
        );
        assert_eq!(valuation.market_value, Some(520000.0));

        let mut first = balance_row(9904);
        first.security_name = "Decimal 検証対象 A".to_string();
        first.total_purchase_amount = rust_decimal_macros::dec!(1000.05);
        let mut second = balance_row(9914);
        second.security_name = "Decimal 検証対象 B".to_string();
        second.total_purchase_amount = rust_decimal_macros::dec!(2000.15);
        let decimal_rows = vec![first, second];
        lookup.update(|store| store.seed(Generation::new(2), &decimal_rows));
        let decimal_saved: Vec<AssetBalanceRow> =
            decimal_rows.iter().cloned().map(Row::Saved).collect();
        let decimal_filtered =
            filtered_portfolio(&decimal_saved, "検証対象", lookup, Generation::new(2));
        assert_eq!(decimal_filtered.views.len(), 2);
        let decimal_holdings: Vec<KpiHolding> = decimal_filtered
            .all_views
            .iter()
            .map(|view| KpiHolding {
                security_code: view.code.clone(),
                shares: view.shares,
                total_purchase_amount: view.purchase,
            })
            .collect();
        let decimal_total = total_purchase_amount(&decimal_holdings);
        assert_eq!(format_currency(decimal_total), "¥3,000.2");
    });
}

#[test]
fn filtered_portfolio_keeps_preview_row_instead_of_lookup_row() {
    let owner = Owner::new();
    owner.with(|| {
        // lookup に同じ銘柄コード・別金額の保存済み行を置く。
        // Row::Saved なら lookup で置き換わるが、Preview は CSV の値をそのまま見せる
        let mut saved = balance_row(9904);
        saved.security_name = "保存済み架空".to_string();
        saved.total_purchase_amount = rust_decimal_macros::dec!(999999);
        let lookup = RwSignal::new(AssetBalanceLookupStore::new());
        lookup.update(|store| store.seed(Generation::new(1), std::slice::from_ref(&saved)));

        let preview = AssetBalanceCsvRow {
            security_code: "9904".to_string(),
            security_name: "CSV架空".to_string(),
            total_purchase_amount: rust_decimal_macros::dec!(100),
            ..AssetBalanceCsvRow::default()
        };
        let rows = vec![AssetBalanceRow::Preview(preview)];

        // Preview 行は lookup で置き換えず CSV の値をそのまま見せる
        let filtered = filtered_portfolio(&rows, "", lookup, Generation::new(1));
        assert_eq!(filtered.views.len(), 1);
        assert_eq!(filtered.views[0].code, "9904");
        assert_eq!(filtered.views[0].name, "CSV架空");
        assert_eq!(filtered.views[0].purchase, 100.0);
    });
}

#[test]
fn summary_renders_with_empty_views() {
    let owner = Owner::new();
    owner.with(|| {
        let dividends = RwSignal::new(DividendMaps::default());
        let views: Vec<HoldingView> = Vec::new();
        let _ = PortfolioSummary(
            PortfolioSummaryProps::builder()
                .filtered(FilteredPortfolio {
                    all_views: views.clone(),
                    views,
                })
                .fallback_totals(None)
                .on_clear_filter(|| {})
                .dividends(dividends)
                .build(),
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
            per_share: HashMap::from([("9904".to_string(), 50.0)]),
            status: HashMap::new(),
        };
        apply_dividend_maps(&balances, &dividends, Generation::new(7), fresh.clone());
        assert!(balances.get_untracked().is_none());
        assert!(dividends.get_untracked().per_share.is_empty());
        balances.set(Some((
            Generation::new(7),
            Ok(LoadedAssetBalances {
                rows: vec![],
                total: 0,
                summary: None,
                truncated: false,
            }),
        )));
        apply_dividend_maps(&balances, &dividends, Generation::new(8), fresh.clone());
        assert!(dividends.get_untracked().per_share.is_empty());
        apply_dividend_maps(&balances, &dividends, Generation::new(7), fresh);
        assert_eq!(dividends.get_untracked().per_share.get("9904"), Some(&50.0));
    });
}

#[test]
fn dividend_update_does_not_notify_balance_view() {
    // 配当更新で balances の購読者(ページ側の動的 view)を再実行させないこと。
    // 再実行されると子コンポーネントが作り直され、開閉状態が失われる。
    let owner = Owner::new();
    owner.with(|| {
        let balances: RwSignal<BalanceSlot> = RwSignal::new(Some((
            Generation::new(7),
            Ok(LoadedAssetBalances {
                rows: vec![],
                total: 0,
                summary: None,
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
            Generation::new(7),
            DividendMaps {
                per_share: HashMap::from([("9904".to_string(), 50.0)]),
                status: HashMap::new(),
            },
        );

        assert!(balance_view.get());
        assert_eq!(runs.get_untracked(), 1);
        assert_eq!(dividends.get_untracked().per_share.get("9904"), Some(&50.0));
    });
}

#[test]
fn valuation_formatters_treat_nan_as_missing() {
    assert_eq!(format_valuation_amount(Some(f64::NAN)), "—");
    assert_eq!(format_valuation_rate(Some(f64::NAN), 1), "—");
}

// 表示単位は集計(サマリー)だけに使い、明細の行は1円単位のまま出す
#[test]
fn row_valuation_stays_in_yen_while_summary_follows_the_amount_unit() {
    use crate::settings::{AmountUnit, amount_unit_signal};
    let unit = amount_unit_signal();
    unit.set(AmountUnit::Man);
    assert_eq!(format_row_valuation_amount(Some(60000.0)), "¥60,000");
    assert_eq!(format_row_valuation_amount(Some(400.0)), "¥400");
    assert_eq!(format_row_valuation_amount(Some(-400.0)), "-¥400");
    assert_eq!(format_row_valuation_amount(None), "—");
    assert_eq!(format_row_valuation_amount(Some(f64::NAN)), "—");
    assert_eq!(format_signed_row_valuation_amount(Some(400.0)), "+¥400");
    assert_eq!(format_signed_row_valuation_amount(Some(-400.0)), "-¥400");
    // 集計は単位に従う
    assert_eq!(format_valuation_amount(Some(60000.0)), "6万円");
    assert_eq!(format_signed_valuation_amount(Some(60000.0)), "+6万円");
    unit.set(AmountUnit::Yen);
}

#[test]
fn negative_valuation_is_decided_by_rounded_amount() {
    assert!(!is_negative_valuation(Some(-0.4)));
    assert!(!is_negative_valuation(Some(0.0)));
    assert!(!is_negative_valuation(None));
    assert!(!is_negative_valuation(Some(f64::NAN)));
    assert!(is_negative_valuation(Some(-0.5)));
    assert!(is_negative_valuation(Some(-1.0)));
    assert_eq!(format_valuation_amount(Some(-0.4)), "¥0");
    assert_eq!(format_valuation_amount(Some(-0.5)), "-¥1");
}

#[test]
fn positive_valuation_is_decided_by_rounded_amount() {
    assert!(is_positive_valuation(Some(0.5)));
    assert!(!is_positive_valuation(Some(0.4)));
    assert!(!is_positive_valuation(Some(0.0)));
    assert!(!is_positive_valuation(Some(-1.0)));
    assert!(!is_positive_valuation(None));
    assert!(!is_positive_valuation(Some(f64::NAN)));
}

#[test]
fn price_date_range_picks_min_and_max_dates() {
    assert_eq!(
        price_date_range(["2025-10-10", "2025-10-08", ""].into_iter()),
        Some(("10/08".to_string(), "10/10".to_string()))
    );
    assert_eq!(
        price_date_range(["2025-10-10"].into_iter()),
        Some(("10/10".to_string(), "10/10".to_string()))
    );
    assert_eq!(price_date_range(std::iter::empty()), None);
    assert_eq!(format_price_date("2025-10-10"), "10/10");
    // 想定外の形式はそのまま返す
    assert_eq!(format_price_date("不明"), "不明");
    assert_eq!(format_price_date("24-10-10"), "24-10-10");
    assert_eq!(format_price_date("-10-10"), "-10-10");
    assert_eq!(format_price_date("2025-"), "2025-");
}

#[test]
fn sort_key_value_round_trips() {
    for key in SortKey::ALL {
        assert_eq!(SortKey::from_value(key.value()), Some(key));
        assert!(!key.label().is_empty());
    }
    assert_eq!(SortKey::from_value("unknown"), None);
}

#[test]
fn sort_labels_distinguish_each_option() {
    // 同じラベルが並ぶとドロップダウンと順序ボタンで項目を区別できない
    let labels: Vec<&str> = SortKey::ALL.iter().map(|key| key.label()).collect();
    for (index, label) in labels.iter().enumerate() {
        assert!(!labels[..index].contains(label), "{label}");
    }
    assert_ne!(SortOrder::Asc.label(), SortOrder::Desc.label());
    assert!(!SortOrder::Asc.label().is_empty());
    assert!(!SortOrder::Desc.label().is_empty());
}

#[test]
fn filtered_portfolio_keeps_saved_record_id_for_detail_links() {
    let owner = Owner::new();
    owner.with(|| {
        // lookup が同じ銘柄コードの別レコードを返しても、詳細リンクの ID は
        // 一覧に並んだ保存済み行のものを使う
        let saved = balance_row(9904);
        let mut other_record = balance_row(9904);
        other_record.id = "other-record".to_string().into();
        other_record.security_name = "差し替わり行".to_string();
        let lookup = RwSignal::new(AssetBalanceLookupStore::new());
        lookup.update(|store| store.seed(Generation::new(1), std::slice::from_ref(&other_record)));

        let saved_id = saved.id.to_string();
        let rows = vec![
            AssetBalanceRow::Saved(saved),
            AssetBalanceRow::Preview(AssetBalanceCsvRow::default()),
        ];
        let filtered = filtered_portfolio(&rows, "", lookup, Generation::new(1));

        assert_eq!(filtered.views[0].id, Some(saved_id));
        assert_eq!(
            filtered.views[0].name, "差し替わり行",
            "表示値は lookup の行を使う"
        );
        assert_eq!(filtered.views[1].id, None, "プレビュー行はリンクしない");
    });
}

#[test]
fn resolve_asset_balance_carries_api_total() {
    let owner = Owner::new();
    owner.with(|| {
        let generation = Generation::new(1);
        let loaded = LoadedAssetBalances {
            rows: vec![balance_row(9904)],
            total: 150000,
            summary: None,
            truncated: true,
        };
        let slot = Some((generation, Ok(loaded)));
        let resolved = resolve_asset_balance(generation, &slot).expect("resolved");
        assert_eq!(
            resolved.total, 150000,
            "行が欠けるときも API の総件数を保持する"
        );
    });
}

#[test]
fn composition_basis_matches_abs_sum_percentages() {
    let owner = Owner::new();
    owner.with(|| {
        let lookup = RwSignal::new(AssetBalanceLookupStore::new());
        let mut positive = balance_row(9904);
        positive.total_purchase_amount = rust_decimal_macros::dec!(100);
        let mut negative = balance_row(9914);
        negative.total_purchase_amount = rust_decimal_macros::dec!(-50);
        let rows: Vec<AssetBalanceRow> = vec![positive, negative]
            .into_iter()
            .map(Row::Saved)
            .collect();
        let views = filtered_portfolio(&rows, "", lookup, Generation::new(1)).views;

        let basis = super::composition_page::composition_basis(&views);
        assert_eq!(basis, 150.0, "母数は絶対値合計");
        assert!(basis > 0.0, "符号付き合計が0以下でも算出不可にしない");
    });
}

#[test]
fn list_subtitle_uses_api_totals_when_truncated() {
    let owner = Owner::new();
    owner.with(|| {
        // 読み込み済み行は価格あり1件だけだが、API 集計は価格なし行を含む
        let resolved = ResolvedAssetBalance {
            rows: vec![Row::Saved(balance_row(9904))],
            total: 150000,
            warning: None,
            fallback_totals: Some(AssetBalanceSummary {
                total_market_value: rust_decimal_macros::dec!(260000),
                total_purchase_amount: rust_decimal_macros::dec!(100),
                total_daily_change: rust_decimal_macros::dec!(0),
                priced_market_value: rust_decimal_macros::dec!(260000),
                priced_purchase_amount: rust_decimal_macros::dec!(60),
            }),
        };
        let subtitle = super::summary::list_subtitle(&resolved);
        assert!(
            subtitle.starts_with("150000銘柄"),
            "件数は読み込み済み行数ではなく API の総件数: {subtitle}"
        );
        assert!(
            subtitle.contains("価格なし銘柄"),
            "行外の価格なし銘柄も注記に出る: {subtitle}"
        );

        // 通常時は行から件数と価格なし判定を出す
        let mut unpriced = balance_row(9914);
        unpriced.current_price = rust_decimal_macros::dec!(0);
        let resolved = ResolvedAssetBalance {
            rows: vec![Row::Saved(balance_row(9904)), Row::Saved(unpriced)],
            total: 2,
            warning: None,
            fallback_totals: None,
        };
        let subtitle = super::summary::list_subtitle(&resolved);
        assert!(subtitle.starts_with("2銘柄"));
        assert!(subtitle.contains("価格なし銘柄"));
        // 価格あり行の基準日を MM/DD にして出す
        assert!(subtitle.contains("01/16"), "{subtitle}");
    });
}

#[test]
fn list_subtitle_excludes_unpriced_rows_from_price_dates() {
    let owner = Owner::new();
    owner.with(|| {
        // 価格が出ていない行の日付は「その時点の株価」と結びつかないので範囲に混ぜない
        let mut unpriced = balance_row(9914);
        unpriced.current_price = rust_decimal_macros::dec!(0);
        unpriced.price_as_of = Some("2024-01-01".to_string());
        let resolved = ResolvedAssetBalance {
            rows: vec![Row::Saved(balance_row(9904)), Row::Saved(unpriced)],
            total: 2,
            warning: None,
            fallback_totals: None,
        };
        let subtitle = super::summary::list_subtitle(&resolved);
        assert!(subtitle.contains("01/16"), "{subtitle}");
        assert!(!subtitle.contains("01/01"), "{subtitle}");
    });
}

#[test]
fn list_subtitle_shows_single_date_when_all_rows_share_as_of() {
    let owner = Owner::new();
    owner.with(|| {
        // 基準日が揃っているときは範囲ではなく単一の日付を出す
        let resolved = ResolvedAssetBalance {
            rows: vec![Row::Saved(balance_row(9904)), Row::Saved(balance_row(9914))],
            total: 2,
            warning: None,
            fallback_totals: None,
        };
        let subtitle = super::summary::list_subtitle(&resolved);
        assert!(!subtitle.contains("〜"), "{subtitle}");
    });
}

#[test]
fn asset_balance_result_applies_within_the_same_generation() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        assert!(session.is_current(generation));
    });
}
