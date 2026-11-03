use super::{
    AnnotationAnchor, AnnotationRequest, CompositionBasis, CompositionGroup, CompositionProduct,
    CompositionSource, DonutGeometry, SortInput, SortKey, SortOrder, composition_color,
    composition_entries, composition_percentages, composition_rows, composition_sources,
    place_annotations, sort_holdings,
};
use crate::api::dto::FundPriceSource;
use crate::features::asset_balance::fund::FundView;
use crate::features::asset_balance::holdings::{HoldingDividend, HoldingView};
use crate::features::asset_balance::model::ValuationInput;

fn sort_input(code: &str, market: f64, purchase: f64, annual: Option<f64>) -> SortInput<'static> {
    let dividend = Box::leak(Box::new(HoldingDividend {
        per_share: annual.map(|v| v / 100.0),
        annual,
        yield_value: annual.map(|v| v / purchase * 100.0),
        status: None,
    }));
    SortInput {
        code: Box::leak(code.to_string().into_boxed_str()),
        valuation: ValuationInput {
            current_price: market,
            shares: 1.0,
            total_purchase_amount: purchase,
        },
        dividend: &*dividend,
    }
}

#[test]
fn sort_holdings_orders_by_market_value_desc() {
    let inputs = vec![
        sort_input("9904", 100.0, 80.0, Some(10.0)),
        sort_input("9914", 200.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let order = sort_holdings(inputs, SortKey::MarketValue, SortOrder::Desc);
    assert_eq!(order, vec![1, 0, 2]);
}

#[test]
fn sort_holdings_orders_by_profit_loss_asc() {
    let inputs = vec![
        sort_input("9904", 100.0, 80.0, Some(10.0)),
        sort_input("9914", 200.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let order = sort_holdings(inputs, SortKey::ProfitLoss, SortOrder::Asc);
    assert_eq!(order, vec![2, 0, 1]);
}

#[test]
fn sort_holdings_orders_by_dividend_desc() {
    let inputs = vec![
        sort_input("9904", 100.0, 80.0, Some(10.0)),
        sort_input("9914", 200.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let order = sort_holdings(inputs, SortKey::Dividend, SortOrder::Desc);
    assert_eq!(order, vec![1, 0, 2]);
}

#[test]
fn sort_holdings_orders_by_yield_asc() {
    let inputs = vec![
        sort_input("9904", 100.0, 80.0, Some(10.0)),
        sort_input("9914", 200.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let order = sort_holdings(inputs, SortKey::Yield, SortOrder::Asc);
    // 9904 と 9999 は利回り同値のためコード順
    assert_eq!(order, vec![0, 2, 1]);
}

#[test]
fn sort_holdings_places_uncomputable_at_end_regardless_of_order() {
    let inputs = vec![
        sort_input("9904", 100.0, 80.0, Some(10.0)),
        sort_input("9914", 0.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let desc = sort_holdings(inputs.clone(), SortKey::MarketValue, SortOrder::Desc);
    assert_eq!(desc, vec![0, 2, 1]);
    let asc = sort_holdings(inputs, SortKey::MarketValue, SortOrder::Asc);
    assert_eq!(asc, vec![2, 0, 1]);
}

#[test]
fn sort_holdings_breaks_ties_by_code() {
    let inputs = vec![
        sort_input("9904", 100.0, 80.0, Some(10.0)),
        sort_input("9914", 100.0, 80.0, Some(10.0)),
        sort_input("9999", 100.0, 80.0, Some(10.0)),
    ];
    let order = sort_holdings(inputs, SortKey::MarketValue, SortOrder::Desc);
    assert_eq!(order, vec![0, 1, 2]);
}

#[test]
fn composition_group_and_basis_labels_are_distinct() {
    let labels: Vec<&str> = CompositionGroup::ALL
        .iter()
        .map(|group| group.label())
        .collect();
    for (index, label) in labels.iter().enumerate() {
        assert!(!label.is_empty());
        assert!(
            !labels[..index].contains(label),
            "区分のラベルが重複している: {label}"
        );
    }
    assert_eq!(CompositionBasis::MarketValue.value(), "market_value");
    assert_eq!(CompositionBasis::Dividend.value(), "dividend");
    assert_ne!(
        CompositionBasis::MarketValue.label(),
        CompositionBasis::Dividend.label()
    );
}

fn composition_stock(
    code: &str,
    name: &str,
    account: Option<&str>,
    id: Option<&str>,
) -> HoldingView {
    HoldingView {
        id: id.map(str::to_string),
        code: code.to_string(),
        name: name.to_string(),
        account: account.map(str::to_string),
        shares: 100.0,
        average_price: 100.0,
        purchase: 100.0,
        current_price: 100.0,
        day_change: None,
        price_as_of: None,
    }
}

fn composition_fund(name: &str, account: &str, course: Option<&str>) -> FundView {
    FundView {
        id: Some(format!("fund-{name}")),
        name: name.to_string(),
        account: account.to_string(),
        course: course.map(str::to_string),
        shares: 10000.0,
        average_purchase_price: 10000.0,
        purchase: 100000.0,
        base_price: 11000.0,
        base_price_day_change: None,
        base_price_month_change: None,
        market_value: 110000.0,
        profit_loss: 10000.0,
        profit_loss_rate: None,
        total_return: None,
        price_as_of: None,
        price_source: FundPriceSource::Csv,
        is_nav_latest: true,
        expected_nav_date: None,
    }
}

fn composition_source(
    name: &str,
    sub: &str,
    account: Option<&str>,
    product: CompositionProduct,
    href: Option<&str>,
) -> CompositionSource {
    CompositionSource {
        name: name.to_string(),
        sub: sub.to_string(),
        account: account.map(str::to_string),
        product,
        href: href.map(str::to_string),
    }
}

fn mixed_sources() -> (Vec<CompositionSource>, Vec<Option<f64>>) {
    let sources = vec![
        composition_source(
            "架空自動車",
            "9904",
            Some("特定"),
            CompositionProduct::Stock,
            Some("1"),
        ),
        composition_source(
            "架空自動車",
            "9904",
            Some("NISA"),
            CompositionProduct::Stock,
            Some("2"),
        ),
        composition_source(
            "架空電機",
            "9914",
            Some("特定"),
            CompositionProduct::Stock,
            Some("3"),
        ),
        composition_source(
            "ファンドA",
            "",
            Some("特定"),
            CompositionProduct::Fund,
            Some("4"),
        ),
        composition_source(
            "ファンドB",
            "",
            Some("NISA"),
            CompositionProduct::Fund,
            Some("5"),
        ),
    ];
    let basis = vec![Some(300.0), Some(100.0), Some(200.0), Some(400.0), None];
    (sources, basis)
}

#[test]
fn composition_sources_map_stock_and_fund_rows() {
    let views = vec![composition_stock(
        "9904",
        "架空自動車",
        Some("特定"),
        Some("1"),
    )];
    let funds = vec![composition_fund("ファンドA", "特定", Some("再投資型"))];
    let sources = composition_sources(&views, &funds);
    assert_eq!(sources.len(), 2);
    assert_eq!(sources[0].sub, "9904");
    assert_eq!(sources[0].href.as_deref(), Some("/assetbalance/holdings/1"));
    assert_eq!(sources[1].sub, "再投資型");
    assert_eq!(
        sources[1].href.as_deref(),
        Some("/assetbalance/funds/fund-ファンドA")
    );
}

#[test]
fn composition_entries_group_by_product_type_not_by_code() {
    let (sources, basis) = mixed_sources();
    let product = composition_entries(&sources, CompositionGroup::Product, &basis);
    assert_eq!(product.len(), 2);
    assert_eq!(product[0].name, "国内株式");
    assert_eq!(product[0].value, Some(600.0));
    assert_eq!(product[0].holdings, 3);
    assert_eq!(product[1].name, "投資信託");
    assert_eq!(product[1].value, Some(400.0));
    assert_eq!(product[1].holdings, 2);
}

#[test]
fn composition_groups_share_the_same_total_and_amounts() {
    let (sources, basis) = mixed_sources();
    let mut totals = Vec::new();
    for group in CompositionGroup::ALL {
        let entries = composition_entries(&sources, group, &basis);
        let (total, _) =
            composition_percentages(&entries.iter().map(|entry| entry.value).collect::<Vec<_>>());
        totals.push(total);
    }
    assert_eq!(totals, vec![1000.0, 1000.0, 1000.0]);

    let product = composition_entries(&sources, CompositionGroup::Product, &basis);
    let stock_total: f64 = basis[..3].iter().map(|value| value.unwrap_or(0.0)).sum();
    assert_eq!(
        product
            .iter()
            .find(|entry| entry.name == "国内株式")
            .and_then(|entry| entry.value),
        Some(stock_total)
    );
    let account = composition_entries(&sources, CompositionGroup::Account, &basis);
    let tokutei_total: f64 = [basis[0], basis[2], basis[3]]
        .iter()
        .map(|value| value.unwrap_or(0.0))
        .sum();
    assert_eq!(
        account
            .iter()
            .find(|entry| entry.name == "特定")
            .and_then(|entry| entry.value),
        Some(tokutei_total)
    );
}

#[test]
fn composition_entries_keep_rows_for_holding_and_aggregate_for_account() {
    let (sources, _) = mixed_sources();
    let basis = vec![
        Some(300.0),
        Some(100.0),
        Some(200.0),
        Some(400.0),
        Some(50.0),
    ];

    let holding = composition_entries(&sources, CompositionGroup::Holding, &basis);
    assert_eq!(holding.len(), 5);
    assert_eq!(holding[0].name, "ファンドA");
    assert_eq!(holding[0].sub, "");
    assert_eq!(holding[0].value, Some(400.0));
    assert_eq!(holding[0].href.as_deref(), Some("4"));

    let account = composition_entries(&sources, CompositionGroup::Account, &basis);
    assert_eq!(account.len(), 2);
    assert_eq!(account[0].name, "特定");
    assert_eq!(account[0].value, Some(900.0));
    assert_eq!(account[0].holdings, 3);
    assert_eq!(account[1].name, "NISA");
}

#[test]
fn composition_entries_put_zero_and_uncomputable_last() {
    let sources = vec![
        composition_source(
            "A",
            "9904",
            Some("特定"),
            CompositionProduct::Stock,
            Some("1"),
        ),
        composition_source(
            "B",
            "9914",
            Some("特定"),
            CompositionProduct::Stock,
            Some("2"),
        ),
        composition_source(
            "C",
            "9920",
            Some("特定"),
            CompositionProduct::Stock,
            Some("3"),
        ),
    ];
    let entries = composition_entries(
        &sources,
        CompositionGroup::Holding,
        &[Some(100.0), Some(0.0), None],
    );
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].name, "A");
    assert_eq!(entries[1].value, None);
    assert_eq!(entries[2].value, None);
}

#[test]
fn composition_percentages_follow_the_existing_zero_and_total_rules() {
    let (total, percentages) = composition_percentages(&[Some(150.0), Some(50.0), Some(0.0), None]);
    assert_eq!(total, 200.0);
    assert_eq!(percentages.len(), 4);
    assert!(percentages[0].is_some_and(|value| (value - 75.0).abs() < 1e-9));
    assert!(percentages[1].is_some_and(|value| (value - 25.0).abs() < 1e-9));
    assert_eq!(percentages[2], None);
    assert_eq!(percentages[3], None);

    let (total, percentages) = composition_percentages(&[Some(0.0), None]);
    assert_eq!(total, 0.0);
    assert_eq!(percentages, vec![None, None]);
}

fn composition_stock_sources() -> (Vec<CompositionSource>, Vec<Option<f64>>) {
    let sources = vec![
        composition_source(
            "A",
            "9904",
            Some("特定"),
            CompositionProduct::Stock,
            Some("1"),
        ),
        composition_source(
            "B",
            "9914",
            Some("特定"),
            CompositionProduct::Stock,
            Some("2"),
        ),
        composition_source(
            "C",
            "9920",
            Some("特定"),
            CompositionProduct::Stock,
            Some("3"),
        ),
        composition_source(
            "D",
            "9930",
            Some("特定"),
            CompositionProduct::Stock,
            Some("4"),
        ),
        composition_source(
            "E",
            "9940",
            Some("特定"),
            CompositionProduct::Stock,
            Some("5"),
        ),
    ];
    let basis = vec![Some(50.0), Some(30.0), Some(9.0), Some(8.9), Some(2.1)];
    (sources, basis)
}

#[test]
fn composition_rows_keep_every_entry_without_other() {
    let (sources, basis) = composition_stock_sources();
    let entries = composition_entries(&sources, CompositionGroup::Holding, &basis);
    let (_, percentages) =
        composition_percentages(&entries.iter().map(|entry| entry.value).collect::<Vec<_>>());
    let rows = composition_rows(&entries, &percentages);
    assert_eq!(rows.len(), 5);
    assert!(!rows.iter().any(|row| row.name.contains("その他")));
    assert_eq!(rows[0].name, "A");
    assert!(rows[0].computable);
    assert_eq!(rows[2].color, composition_color(2));
    assert_eq!(rows[4].name, "E");
    assert!(rows[4].computable);
}

#[test]
fn composition_rows_keep_uncomputable_as_their_own_row_at_the_end() {
    let sources = vec![
        composition_source(
            "A",
            "9904",
            Some("特定"),
            CompositionProduct::Stock,
            Some("1"),
        ),
        composition_source(
            "B",
            "9914",
            Some("特定"),
            CompositionProduct::Stock,
            Some("2"),
        ),
    ];
    let entries = composition_entries(&sources, CompositionGroup::Holding, &[Some(100.0), None]);
    let (_, percentages) =
        composition_percentages(&entries.iter().map(|entry| entry.value).collect::<Vec<_>>());
    let rows = composition_rows(&entries, &percentages);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].name, "A");
    assert!(rows[0].computable);
    assert_eq!(rows[1].name, "B");
    assert!(!rows[1].computable);
    assert_eq!(rows[1].value, None);
}

fn annotation_requests<'a>(labels: &'a [(&'a str, f64)]) -> Vec<AnnotationRequest<'a>> {
    labels
        .iter()
        .map(|(label, percentage)| AnnotationRequest {
            label,
            percentage: *percentage,
            computable: true,
        })
        .collect()
}

#[test]
fn composition_annotations_keep_only_major_slices_up_to_ten() {
    let labels: Vec<(String, f64)> = (0..12)
        .map(|index| (format!("銘柄{index:02} 8.3%"), 100.0 / 12.0))
        .collect();
    let borrowed: Vec<(&str, f64)> = labels
        .iter()
        .map(|(label, percentage)| (label.as_str(), *percentage))
        .collect();
    let requests = annotation_requests(&borrowed);
    let selected = place_annotations(&requests, &DonutGeometry::WIDE, "75銘柄");
    assert_eq!(selected.len(), 10);
    assert_eq!(
        selected.iter().map(|item| item.index).collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4, 5, 7, 8, 9, 10]
    );
}

#[test]
fn composition_annotations_skip_overlapping_neighbors() {
    let requests = annotation_requests(&[("あ 40.0%", 40.0), ("い 5.0%", 5.0), ("う 5.0%", 5.0)]);
    let selected = place_annotations(&requests, &DonutGeometry::WIDE, "75銘柄");
    let indexes: Vec<usize> = selected.iter().map(|item| item.index).collect();
    assert!(indexes.contains(&0));
    assert!(indexes.contains(&1));
    assert!(!indexes.contains(&2));
}

#[test]
fn composition_annotations_require_computable_major_slices() {
    let requests = vec![
        AnnotationRequest {
            label: "あ 50.0%",
            percentage: 50.0,
            computable: true,
        },
        AnnotationRequest {
            label: "い 50.0%",
            percentage: 50.0,
            computable: false,
        },
    ];
    let selected = place_annotations(&requests, &DonutGeometry::WIDE, "75銘柄");
    assert_eq!(
        selected.iter().map(|item| item.index).collect::<Vec<_>>(),
        vec![0]
    );
}

#[test]
fn composition_annotations_spread_across_sides() {
    let requests = annotation_requests(&[
        ("ファンドA 36.4%", 100.0 * 400.0 / 1100.0),
        ("架空自動車 27.3%", 100.0 * 300.0 / 1100.0),
        ("架空電機 18.2%", 100.0 * 200.0 / 1100.0),
        ("架空自動車 9.1%", 100.0 * 100.0 / 1100.0),
        ("ファンドB 9.1%", 100.0 * 100.0 / 1100.0),
    ]);
    let selected = place_annotations(&requests, &DonutGeometry::COMPACT, "5銘柄");
    assert_eq!(selected.len(), 5);
    assert!(
        selected
            .iter()
            .any(|item| item.anchor == AnnotationAnchor::Start
                || item.anchor == AnnotationAnchor::End)
    );
    assert!(
        selected
            .iter()
            .any(|item| item.anchor == AnnotationAnchor::Middle)
    );
}

#[test]
fn composition_color_keeps_neighbors_different() {
    for position in 0..20 {
        assert_ne!(
            composition_color(position),
            composition_color(position + 1),
            "{position}"
        );
        assert!(composition_color(position) < 5, "{position}");
    }
}

#[test]
fn sort_holdings_allows_other_keys_while_dividend_pending() {
    let mut pending = sort_input("0001", 100.0, 80.0, None);
    let pending_dividend = Box::leak(Box::new(HoldingDividend {
        per_share: None,
        annual: None,
        yield_value: None,
        status: Some("pending".to_string()),
    }));
    pending.dividend = &*pending_dividend;
    let inputs = vec![
        pending,
        sort_input("9904", 200.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let order = sort_holdings(inputs, SortKey::MarketValue, SortOrder::Desc);
    assert_eq!(order, vec![1, 0, 2]);
}
