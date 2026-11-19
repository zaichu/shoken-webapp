//! 「すべて」タブ。国内株式と投資信託を同じ一覧に並べる

use super::day_change::pending_day_change_cell;
use super::sort_header::sort_header;
use crate::features::asset_balance::format as balance_format;
use crate::features::asset_balance::fund::{FundSortKey, FundView};
use crate::features::asset_balance::holdings;
use crate::features::asset_balance::holdings::{HoldingDividend, HoldingView};
use crate::features::asset_balance::model;
use crate::features::asset_balance::model::ValuationInput;
use crate::features::asset_balance::portfolio::SortOrder;
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::asset_balance::{AssetRateAmount, DayChangeNote};
use crate::ui::badge::{AccountBadge, AccountLegend};
use crate::ui::detail_dialog::{DetailCell, DetailDialog};
use crate::ui::format;
use crate::ui::labels;
use crate::ui::list_row::ListRowButton;
use crate::ui::security_link::{TableNameLink, TableSecurityCode};
use crate::urls;
use leptos::prelude::*;

/// 株とファンドをまとめる一覧行。sub はスマホの行メタ(コード・コース)、残りは表の列
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UnifiedRow {
    pub name: String,
    pub kind: &'static str,
    /// スマホの行メタ。口座は sub ではなく account から札で出す
    pub sub: String,
    pub code: Option<String>,
    pub account: Option<String>,
    pub course: Option<String>,
    /// スマホの2行目に出す株数・口数(「100株」「1.5口」。投信と株を混同しない)
    pub quantity: String,
    pub day_change: Option<f64>,
    /// 行の前日比の合計(円)。株は1株あたり×株数、投信は1万口あたり×口数/10000。
    /// 株と投信が混ざる行の騰落率(%)を出すために持つ
    pub day_change_total: Option<f64>,
    pub price_as_of: Option<String>,
    /// 基準日がその種別の代表(見出しに出す日付)と違うか。構築側で立てる
    pub date_outlier: bool,
    /// ファンドの「更新待ち」(前日比が最新の基準価額の日の変動でない)。株行は常に false
    pub nav_pending: bool,
    /// 前日比セルの title(出どころ・更新待ちの理由)。株行は None
    pub day_change_title: Option<String>,
    pub market: Option<f64>,
    pub profit: Option<f64>,
    pub rate: Option<f64>,
    pub href: Option<String>,
    /// 株式行の1株配当。投資信託の行は None
    pub dividend: Option<HoldingDividend>,
    pub current_price: Option<f64>,
    pub average_price: Option<f64>,
    pub purchase: Option<f64>,
}

pub(crate) fn unified_from_stock(view: &HoldingView, dividends: &DividendMaps) -> UnifiedRow {
    let valuation = model::holding_valuation(&ValuationInput::from(view));
    UnifiedRow {
        name: view.name.clone(),
        kind: labels::DOMESTIC_STOCK,
        sub: view.code.clone(),
        code: Some(view.code.clone()),
        account: view.account.clone(),
        course: None,
        quantity: labels::stock_shares_label(&format::format_number_value(view.shares)),
        day_change: view.day_change,
        day_change_total: view.day_change.map(|change| change * view.shares),
        price_as_of: view.price_as_of.clone(),
        date_outlier: false,
        nav_pending: false,
        day_change_title: None,
        market: valuation.market_value,
        profit: valuation.profit_loss.amount,
        rate: valuation.profit_loss.rate,
        href: Some(holdings::holding_stock_href(&view.code)),
        dividend: Some(holdings::holding_dividend(
            &view.code,
            view.shares,
            view.average_price,
            dividends,
        )),
        current_price: Some(view.current_price),
        average_price: Some(view.average_price),
        purchase: Some(view.purchase),
    }
}

pub(crate) fn unified_from_fund(view: &FundView) -> UnifiedRow {
    UnifiedRow {
        name: view.name.clone(),
        kind: labels::MUTUAL_FUND,
        sub: view.course.clone().unwrap_or_default(),
        code: None,
        account: Some(view.account.clone()),
        course: view.course.clone(),
        quantity: labels::fund_units_label(&format::format_number_value(view.shares)),
        day_change: view.view_day_change(),
        day_change_total: view
            .view_day_change()
            .map(|change| change * view.shares / 10_000.0),
        price_as_of: view.price_as_of.clone(),
        date_outlier: false,
        nav_pending: view.nav_pending(),
        day_change_title: view.day_change_title(),
        market: Some(view.market_value),
        profit: Some(view.profit_loss),
        rate: view.profit_loss_rate,
        href: view.id.as_ref().map(urls::fund_holding_href),
        dividend: None,
        current_price: Some(view.base_price),
        average_price: Some(view.average_purchase_price),
        purchase: Some(view.purchase),
    }
}

/// 「すべて」タブの前日比の騰落率(%)。前日の評価額に対する比。
/// 前日の値がない・合計が0以下・更新待ちの行は None
/// (「—」・「更新待ち」は呼び出し側の今の決まりのまま)
pub(crate) fn unified_day_change_rate(row: &UnifiedRow) -> Option<f64> {
    if row.nav_pending {
        return None;
    }
    let (Some(market), Some(daily)) = (row.market, row.day_change_total) else {
        return None;
    };
    if market - daily <= 0.0 {
        return None;
    }
    Some(daily / (market - daily) * 100.0)
}

/// 「すべて」タブの前日比セル。株と投信が混ざるため金額ではなく騰落率(%)で出す。
/// 値がない行は「—」。基準日が他の行と違うときだけ横に日付を小さく出す
pub(crate) fn day_change_rate_cell(rate: Option<f64>, date: Option<String>) -> impl IntoView {
    let negative = balance_format::is_negative_rate(rate);
    let positive = balance_format::is_positive_rate(rate);
    let text = balance_format::format_signed_valuation_rate(rate, 1);
    let date_text = date.map(|date| format::format_price_date(&date));
    view! {
        <Amount
            kind=AmountKind::Delta
            text=text
            negative=negative
            positive=positive
            class="font-bold"
        />
        {date_text.map(|date| {
            view! { <DayChangeNote text=format!("({date})") /> }
        })}
    }
}
/// 「すべて」タブの並べ替え。算出不能は末尾、同値は名前順
pub(crate) fn sort_unified(rows: &[UnifiedRow], key: FundSortKey, order: SortOrder) -> Vec<usize> {
    let value = |row: &UnifiedRow| match key {
        FundSortKey::MarketValue => row.market,
        _ => row.profit,
    };
    let mut indexed: Vec<usize> = (0..rows.len()).collect();
    indexed.sort_by(|&a, &b| {
        let ordering = match (value(&rows[a]), value(&rows[b])) {
            (Some(a), Some(b)) => {
                let ordering = b.total_cmp(&a);
                match order {
                    SortOrder::Desc => ordering,
                    SortOrder::Asc => ordering.reverse(),
                }
            }
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
        };
        if ordering == std::cmp::Ordering::Equal {
            rows[a].name.cmp(&rows[b].name)
        } else {
            ordering
        }
    });
    indexed
}

#[component]
pub fn AssetMergedTable(
    rows: Vec<UnifiedRow>,
    sort_key: RwSignal<FundSortKey>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    // 列の中の小数桁をそろえる(整数円の値しかない列は .00 を付けない)
    let dividend_decimals = format::unit_price_decimals(
        rows.iter()
            .filter_map(|row| row.dividend.as_ref()?.per_share),
    );
    let sorted = Memo::new(move |_| {
        sort_unified(&rows, sort_key.get(), sort_order.get())
            .into_iter()
            .map(|index| rows[index].clone())
            .collect::<Vec<_>>()
    });
    view! {
        <div data-testid="asset-merged-table">
            <table class="asset-table">
                <thead>
                    <tr>
                        <th scope="col" class="col-text">{labels::CODE_COLUMN}</th>
                        <th scope="col" class="col-text">{labels::SECURITY_AND_FUND}</th>
                        <th scope="col" class="col-text">{labels::KIND}</th>
                        <th scope="col" class="col-text">{labels::ACCOUNT}</th>
                        <th scope="col" class="col-text">{labels::DISTRIBUTION_COURSE}</th>
                        <th scope="col" class="col-num">{labels::PER_SHARE_DIVIDEND}</th>
                        <th scope="col" class="col-num">{labels::DAY_CHANGE_RATE}</th>
                        {sort_header(labels::MARKET_VALUE, FundSortKey::MarketValue, sort_key, sort_order)}
                        {sort_header(
                            labels::VALUATION_PROFIT_LOSS,
                            FundSortKey::ProfitLoss,
                            sort_key,
                            sort_order,
                        )}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        sorted
                            .get()
                            .into_iter()
                            .map(|row| {
                                let row = &row;
                                let pnl_text =
                                    balance_format::format_signed_row_valuation_amount(row.profit);
                                let rate_text =
                                    balance_format::format_signed_valuation_rate(row.rate, 1);
                                let pnl_negative = balance_format::is_negative_valuation(row.profit);
                                let pnl_positive = balance_format::is_positive_valuation(row.profit);
                                let rate_negative = balance_format::is_negative_rate(row.rate);
                                let rate_positive = balance_format::is_positive_rate(row.rate);
                                let outlier_date = if row.date_outlier {
                                    row.price_as_of.clone()
                                } else {
                                    None
                                };
                                view! {
                                    <tr>
                                        <td class="col-text">
                                            <TableSecurityCode
                                                code=row.code.clone().unwrap_or_default()
                                            />
                                        </td>
                                        <td class="asset-name-cell col-text" title=row.name.clone()>
                                            <TableNameLink
                                                href=row.href.clone()
                                                name=row.name.clone()
                                            />
                                        </td>
                                        <td class="col-text text-text-soft">{row.kind}</td>
                                        <td class="col-text">
                                            {row.account.as_ref().map_or_else(|| view! { <span class="text-text-soft">{labels::unavailable_text(labels::UnavailableKind::Unassigned)}</span> }
                                                    .into_any(), |account| view! {
                                                    <AccountBadge label=account.clone() />
                                                }
                                                    .into_any())}
                                        </td>
                                        <td class="col-text text-text-soft">
                                            {row.course.clone().unwrap_or_else(|| {
                                                if row.kind == labels::MUTUAL_FUND {
                                                    labels::FETCH_UNAVAILABLE.to_string()
                                                } else {
                                                    labels::unavailable_text(
                                                        labels::UnavailableKind::OutOfScope,
                                                    )
                                                    .to_string()
                                                }
                                            })}
                                        </td>
                                        <td class="col-num">
                                            <Amount text=row
                                                .dividend
                                                .as_ref().map_or_else(|| labels::unavailable_text(labels::UnavailableKind::OutOfScope).to_string(), |dividend| {
                                                    holdings::format_dividend_per_share(
                                                        dividend,
                                                        dividend_decimals,
                                                    )
                                                }) />
                                        </td>
                                        <td
                                            class="col-num"
                                            title=row.day_change_title.clone()
                                        >
                                            {if row.nav_pending {
                                                pending_day_change_cell().into_any()
                                            } else {
                                                day_change_rate_cell(
                                                        unified_day_change_rate(row),
                                                        outlier_date,
                                                    )
                                                    .into_any()
                                            }}
                                        </td>
                                        <td class="col-num font-bold text-text-deep">
                                            <Amount
                                                kind=AmountKind::Plain
                                                text=balance_format::format_row_valuation_amount(row.market)
                                            />
                                        </td>
                                        <td class="col-num">
                                            {if row.profit.is_some() {
                                                view! {
                                                    <Amount
                                                        kind=AmountKind::Delta
                                                        text=pnl_text
                                                        negative=pnl_negative
                                                        positive=pnl_positive
                                                        class="font-bold"
                                                    />
                                                    {row.rate.is_some().then(|| {
                                                        view! {
                                                            <AssetRateAmount
                                                                text=rate_text.clone()
                                                                negative=rate_negative
                                                                positive=rate_positive
                                                            />
                                                        }
                                                    })}
                                                }
                                                    .into_any()
                                            } else {
                                                view! { "—" }.into_any()
                                            }}
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
            <AccountLegend />
        </div>
    }
}

pub(crate) fn unified_detail_cells(row: &UnifiedRow) -> (Vec<DetailCell>, Vec<DetailCell>) {
    let is_fund = row.kind == labels::MUTUAL_FUND;
    let price_label = if is_fund {
        labels::BASE_PRICE
    } else {
        labels::CURRENT_PRICE
    };
    let price_text = row
        .current_price
        .map_or_else(|| "—".to_string(), format::format_currency);
    let (day_text, day_negative, day_positive) = if row.nav_pending {
        (labels::PENDING_UPDATE.to_string(), false, false)
    } else {
        let rate = unified_day_change_rate(row);
        (
            balance_format::format_signed_valuation_rate(rate, 1),
            balance_format::is_negative_rate(rate),
            balance_format::is_positive_rate(rate),
        )
    };
    let average_text = row
        .average_price
        .map_or_else(|| "—".to_string(), format::format_currency);
    let purchase_text = row
        .purchase
        .map_or_else(|| "—".to_string(), format::format_currency);
    let market_text = balance_format::format_row_valuation_amount(row.market);
    let pnl_text = balance_format::format_signed_row_valuation_amount(row.profit);
    let pnl_rate = balance_format::format_signed_valuation_rate(row.rate, 1);
    let pnl_negative = balance_format::is_negative_valuation(row.profit);
    let pnl_positive = balance_format::is_positive_valuation(row.profit);
    let as_of = row.price_as_of.clone().map_or_else(
        || labels::FETCH_UNAVAILABLE.to_string(),
        |date| format::format_iso_date(&date),
    );
    let mut left = vec![
        DetailCell::with_sub(
            price_label.to_string(),
            price_text,
            day_text,
            day_negative,
            day_positive,
        ),
        DetailCell::plain(
            if is_fund {
                labels::AVERAGE_PRICE_PER_10K.to_string()
            } else {
                labels::AVERAGE_PRICE.to_string()
            },
            average_text,
        ),
    ];
    if is_fund {
        left.push(DetailCell::plain(
            labels::DISTRIBUTION_COURSE.to_string(),
            row.course
                .clone()
                .unwrap_or_else(|| labels::FETCH_UNAVAILABLE.to_string()),
        ));
    } else {
        let per_share_value = row
            .dividend
            .as_ref()
            .and_then(|dividend| dividend.per_share);
        let yield_value = row
            .dividend
            .as_ref()
            .and_then(|dividend| dividend.yield_value);
        let per_share = row.dividend.as_ref().map_or_else(
            || labels::FETCH_UNAVAILABLE.to_string(),
            |dividend| {
                crate::features::asset_balance::holdings::format_dividend_per_share(
                    dividend,
                    format::unit_price_decimals(dividend.per_share.into_iter()),
                )
            },
        );
        let yield_text = row.dividend.as_ref().map_or_else(
            || labels::FETCH_UNAVAILABLE.to_string(),
            crate::features::asset_balance::holdings::format_dividend_yield,
        );
        left.push(DetailCell {
            label: labels::ACQUISITION_YIELD.to_string(),
            value: yield_text,
            sub: None,
            sub_rate: None,
            sub_rate_negative: false,
            sub_rate_positive: false,
            negative: balance_format::is_negative_rate(yield_value),
            positive: balance_format::is_positive_rate(yield_value),
            bold: false,
            kind: AmountKind::Delta,
        });
        left.push(DetailCell {
            label: labels::PER_SHARE_DIVIDEND.to_string(),
            value: per_share,
            sub: None,
            sub_rate: None,
            sub_rate_negative: false,
            sub_rate_positive: false,
            negative: balance_format::is_negative_rate(per_share_value),
            positive: balance_format::is_positive_rate(per_share_value),
            bold: false,
            kind: AmountKind::Delta,
        });
    }
    let mut market_cell = match row.profit {
        Some(_) => DetailCell::strong_sub(
            labels::MARKET_VALUE.to_string(),
            market_text,
            pnl_text,
            pnl_negative,
            pnl_positive,
        ),
        None => DetailCell::plain(labels::MARKET_VALUE.to_string(), market_text),
    };
    market_cell.sub_rate = row.rate.map(|_| pnl_rate);
    market_cell.sub_rate_negative = balance_format::is_negative_rate(row.rate);
    market_cell.sub_rate_positive = balance_format::is_positive_rate(row.rate);
    let mut right = vec![
        DetailCell::plain(
            if is_fund {
                labels::FUND_SHARES.to_string()
            } else {
                labels::QUANTITY.to_string()
            },
            row.quantity.clone(),
        ),
        DetailCell::plain(labels::TOTAL_ACQUISITION.to_string(), purchase_text),
        market_cell,
    ];
    if !is_fund {
        let annual = row.dividend.as_ref().map_or_else(
            || labels::FETCH_UNAVAILABLE.to_string(),
            crate::features::asset_balance::holdings::format_dividend_annual,
        );
        right.push(DetailCell::plain(
            labels::ANNUAL_DIVIDEND.to_string(),
            annual,
        ));
    }
    right.push(DetailCell::plain(labels::PRICE_AS_OF.to_string(), as_of));
    (left, right)
}

#[component]
pub fn AssetMergedList(rows: Vec<UnifiedRow>) -> impl IntoView {
    view! {
        <ul class="asset-compact-list" data-testid="asset-merged-list">
            {rows
                .into_iter()
                .map(|row| {
                    let pnl_text = balance_format::format_signed_row_valuation_amount(row.profit);
                    let rate_text = balance_format::format_signed_valuation_rate(row.rate, 1);
                    let pnl_negative = balance_format::is_negative_valuation(row.profit);
                    let pnl_positive = balance_format::is_positive_valuation(row.profit);
                    let rate_negative = balance_format::is_negative_rate(row.rate);
                    let rate_positive = balance_format::is_positive_rate(row.rate);
                    let market_text = balance_format::format_row_valuation_amount(row.market);
                    let name = row.name.clone();
                    let code = row.code.clone();
                    let account = row.account.clone();
                    let quantity = row.quantity.clone();
                    let dialog_name = row.name.clone();
                    let dialog_code = row.code.clone();
                    let dialog_account = row.account.clone();
                    let dialog_row = row.clone();
                    let open = RwSignal::new(false);
                    let open_for_dialog = open;
                    let (left, right) = unified_detail_cells(&dialog_row);
                    view! {
                        <li class="list-row-item" data-testid="asset-compact-row">
                            <ListRowButton
                                testid="asset-row-button"
                                on_click=move |_| open.set(true)
                                name=move || name.clone()
                                primary=move || view! {
                                    <Amount kind=AmountKind::Plain text=market_text.clone() />
                                }
                                meta=move || view! {
                                    {code
                                        .clone()
                                        .map(|code| {
                                            view! {
                                                {code}
                                                {" · "}
                                            }
                                        })}
                                    {account
                                        .clone()
                                        .map(|account| {
                                            view! {
                                                <AccountBadge label=account />
                                                {" · "}
                                            }
                                        })}
                                    <Amount kind=AmountKind::Plain text=quantity.clone() />
                                }
                                secondary=move || view! {
                                    {if row.profit.is_some() {
                                        view! {
                                            <Amount
                                                kind=AmountKind::Delta
                                                text=pnl_text.clone()
                                                negative=pnl_negative
                                                positive=pnl_positive
                                            />
                                            {row
                                                .rate
                                                .is_some()
                                                .then(|| {
                                                    view! {
                                                        " "
                                                        <Amount
                                                            kind=AmountKind::Delta
                                                            text=rate_text.clone()
                                                            negative=rate_negative
                                                            positive=rate_positive
                                                        />
                                                    }
                                                })}
                                        }
                                            .into_any()
                                    } else {
                                        view! { "—" }.into_any()
                                    }}
                                }
                            />
                            <Show when=move || open.get()>
                                <DetailDialog
                                    name=dialog_name.clone()
                                    code=dialog_code.clone().unwrap_or_default()
                                    account=dialog_account.clone().unwrap_or_default()
                                    left=left.clone()
                                    right=right.clone()
                                    on_close=move || open_for_dialog.set(false)
                                    testid="asset-merged-dialog"
                                    link_code=dialog_row.code.clone().unwrap_or_default()
                                />
                            </Show>
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
}

#[cfg(test)]
mod tests {
    use super::{
        UnifiedRow, unified_day_change_rate, unified_detail_cells, unified_from_fund,
        unified_from_stock,
    };
    use crate::api::dto::FundPriceSource;
    use crate::features::asset_balance::fund::FundView;
    use crate::features::asset_balance::holdings::HoldingView;
    use crate::features::dividend_per_share::DividendMaps;
    use crate::ui::labels;

    fn sample_stock() -> HoldingView {
        HoldingView {
            id: Some("1".to_string()),
            code: "9904".to_string(),
            name: "架空自動車".to_string(),
            account: Some("特定".to_string()),
            shares: 100.0,
            average_price: 2500.0,
            purchase: 250000.0,
            current_price: 2600.0,
            day_change: Some(50.0),
            price_as_of: Some("2026-01-01".to_string()),
        }
    }

    fn sample_fund() -> FundView {
        FundView {
            id: Some("fund-1".to_string()),
            name: "架空ファンド".to_string(),
            account: "特定".to_string(),
            course: Some("受取型".to_string()),
            shares: 10000.0,
            average_purchase_price: 10000.0,
            purchase: 100000.0,
            base_price: 11000.0,
            base_price_day_change: Some(50.0),
            base_price_month_change: None,
            market_value: 110000.0,
            profit_loss: 10000.0,
            profit_loss_rate: Some(10.0),
            total_return: None,
            price_as_of: Some("2026-01-05".to_string()),
            price_source: FundPriceSource::Api,
            is_nav_latest: true,
            expected_nav_date: None,
        }
    }

    #[test]
    fn merged_dialog_uses_row_basis_and_order() {
        let stock = unified_from_stock(&sample_stock(), &DividendMaps::default());
        let (left, right) = unified_detail_cells(&stock);
        let left_labels: Vec<&str> = left.iter().map(|cell| cell.label.as_str()).collect();
        assert_eq!(
            left_labels,
            ["現在値", "取得単価", "取得額利回り", "1株配当"]
        );
        let right_labels: Vec<&str> = right.iter().map(|cell| cell.label.as_str()).collect();
        assert_eq!(
            right_labels,
            ["数量", "取得総額", "評価額", "年間配当", "価格基準日"]
        );
        assert!(right[2].value.contains("260,000") || right[2].value.contains("260000"));

        let fund = unified_from_fund(&sample_fund());
        let (left, right) = unified_detail_cells(&fund);
        let left_labels: Vec<&str> = left.iter().map(|cell| cell.label.as_str()).collect();
        assert_eq!(
            left_labels,
            [
                "基準価額(1万口あたり)",
                "取得単価(1万口あたり)",
                "分配金コース"
            ]
        );
        let right_labels: Vec<&str> = right.iter().map(|cell| cell.label.as_str()).collect();
        assert_eq!(
            right_labels,
            ["保有口数", "取得総額", "評価額", labels::PRICE_AS_OF]
        );
    }

    #[test]
    fn merged_day_change_rate_uses_previous_day_valuation() {
        // 株: 50円×100株=5,000円、前日評価額 255,000円
        let stock = unified_from_stock(&sample_stock(), &DividendMaps::default());
        let rate = unified_day_change_rate(&stock).expect("株の騰落率");
        assert!((rate - 5000.0 / 255000.0 * 100.0).abs() < 1e-9, "{rate}");

        // 投信: 50円×10000口/10000=50円、前日評価額 109,950円
        let fund = unified_from_fund(&sample_fund());
        let rate = unified_day_change_rate(&fund).expect("投信の騰落率");
        assert!((rate - 50.0 / 109950.0 * 100.0).abs() < 1e-9, "{rate}");
    }

    #[test]
    fn merged_day_change_rate_returns_none_without_previous_value() {
        // 前日比がない行は「—」のまま
        let mut stock = sample_stock();
        stock.day_change = None;
        let row = unified_from_stock(&stock, &DividendMaps::default());
        assert_eq!(unified_day_change_rate(&row), None);

        // 評価額がない行も出さない
        let row = UnifiedRow {
            market: None,
            ..unified_from_stock(&sample_stock(), &DividendMaps::default())
        };
        assert_eq!(unified_day_change_rate(&row), None);

        // 前日の評価額が0以下の行も出さない
        let row = UnifiedRow {
            market: Some(100.0),
            day_change_total: Some(100.0),
            ..unified_from_stock(&sample_stock(), &DividendMaps::default())
        };
        assert_eq!(unified_day_change_rate(&row), None);

        // 更新待ちの投信は古い日の変動を見せない
        let mut fund = sample_fund();
        fund.is_nav_latest = false;
        let row = unified_from_fund(&fund);
        assert!(row.nav_pending);
        assert_eq!(unified_day_change_rate(&row), None);
    }
}
