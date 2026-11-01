//! 「すべて」タブ。国内株式と投資信託を同じ一覧に並べる

use super::day_change::{day_change_cell, pending_day_change_cell};
use super::sort_header::sort_header;
use crate::features::asset_balance::format::{
    format_row_valuation_amount, format_signed_row_valuation_amount, format_signed_valuation_rate,
    is_negative_rate, is_negative_valuation, is_positive_rate, is_positive_valuation,
};
use crate::features::asset_balance::fund::{FundSortKey, FundView};
use crate::features::asset_balance::holdings::{
    HoldingDividend, HoldingView, format_dividend_per_share, holding_dividend,
};
use crate::features::asset_balance::model::{ValuationInput, holding_valuation};
use crate::features::asset_balance::portfolio::SortOrder;
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::AccountBadge;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::detail_dialog::{DetailCell, DetailDialog};
use crate::ui::format::{
    format_currency, format_iso_date, format_number_value, unit_price_decimals,
};
use crate::ui::labels::{
    ACCOUNT, ACQUISITION_YIELD, ANNUAL_DIVIDEND, AVERAGE_PRICE, BASE_PRICE, CODE_COLUMN,
    CURRENT_PRICE, DAY_CHANGE, DISTRIBUTION, DOMESTIC_STOCK, FUND_SHARES, MARKET_VALUE,
    MUTUAL_FUND, PENDING_UPDATE, PER_SHARE_DIVIDEND, PRICE_AS_OF, QUANTITY, SECURITY_AND_FUND,
    TOTAL_ACQUISITION, VALUATION_PROFIT_LOSS,
};
use crate::ui::list_row::ListRow;
use crate::ui::security_link::{TableNameLink, TableSecurityCode};
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
    let valuation = holding_valuation(&ValuationInput::from(view));
    UnifiedRow {
        name: view.name.clone(),
        kind: DOMESTIC_STOCK,
        sub: view.code.clone(),
        code: Some(view.code.clone()),
        account: view.account.clone(),
        course: None,
        quantity: format!("{}株", format_number_value(view.shares)),
        day_change: view.day_change,
        price_as_of: view.price_as_of.clone(),
        date_outlier: false,
        nav_pending: false,
        day_change_title: None,
        market: valuation.market_value,
        profit: valuation.profit_loss.amount,
        rate: valuation.profit_loss.rate,
        href: view
            .id
            .clone()
            .map(|id| format!("/assetbalance/holdings/{id}")),
        dividend: Some(holding_dividend(
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
        kind: MUTUAL_FUND,
        sub: view.course.clone().unwrap_or_default(),
        code: None,
        account: Some(view.account.clone()),
        course: view.course.clone(),
        quantity: format!("{}口", format_number_value(view.shares)),
        day_change: view.view_day_change(),
        price_as_of: view.price_as_of.clone(),
        date_outlier: false,
        nav_pending: view.nav_pending(),
        day_change_title: view.day_change_title(),
        market: Some(view.market_value),
        profit: Some(view.profit_loss),
        rate: view.profit_loss_rate,
        href: view
            .id
            .clone()
            .map(|id| format!("/assetbalance/funds/{id}")),
        dividend: None,
        current_price: Some(view.base_price),
        average_price: Some(view.average_purchase_price),
        purchase: Some(view.purchase),
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
    let dividend_decimals = unit_price_decimals(
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
                        <th scope="col" class="col-text">{CODE_COLUMN}</th>
                        <th scope="col" class="col-text">{SECURITY_AND_FUND}</th>
                        <th scope="col" class="col-text">"種別"</th>
                        <th scope="col" class="col-text">{ACCOUNT}</th>
                        <th scope="col" class="col-text">{DISTRIBUTION}</th>
                        <th scope="col" class="col-num">{PER_SHARE_DIVIDEND}</th>
                        <th scope="col" class="col-num">{DAY_CHANGE}</th>
                        {sort_header(MARKET_VALUE, FundSortKey::MarketValue, sort_key, sort_order)}
                        {sort_header(
                            VALUATION_PROFIT_LOSS,
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
                                    format_signed_row_valuation_amount(row.profit);
                                let rate_text =
                                    format_signed_valuation_rate(row.rate, 1);
                                let pnl_negative = is_negative_valuation(row.profit);
                                let pnl_positive = is_positive_valuation(row.profit);
                                let rate_negative = is_negative_rate(row.rate);
                                let rate_positive = is_positive_rate(row.rate);
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
                                            {match &row.account {
                                                Some(account) => view! {
                                                    <AccountBadge label=account.clone() />
                                                }
                                                    .into_any(),
                                                None => view! { <span class="text-text-soft">"—"</span> }
                                                    .into_any(),
                                            }}
                                        </td>
                                        <td class="col-text text-text-soft">
                                            {row.course.clone().unwrap_or_else(|| "—".to_string())}
                                        </td>
                                        <td class="col-num">
                                            {row
                                                .dividend
                                                .as_ref()
                                                .map(|dividend| {
                                                    format_dividend_per_share(
                                                        dividend,
                                                        dividend_decimals,
                                                    )
                                                })
                                                .unwrap_or_else(|| "—".to_string())}
                                        </td>
                                        <td
                                            class="col-num"
                                            title=row.day_change_title.clone()
                                        >
                                            {if row.nav_pending {
                                                pending_day_change_cell().into_any()
                                            } else {
                                                day_change_cell(row.day_change, outlier_date)
                                                    .into_any()
                                            }}
                                        </td>
                                        <td class="col-num font-bold text-text-deep">
                                            <Amount
                                                kind=AmountKind::Plain
                                                text=format_row_valuation_amount(row.market)
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
                                                            <Amount
                                                                kind=AmountKind::Delta
                                                                text=rate_text.clone()
                                                                negative=rate_negative
                                                                positive=rate_positive
                                                                class="ml-1 text-xs"
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
        </div>
    }
}

pub(crate) fn unified_detail_cells(row: &UnifiedRow) -> (Vec<DetailCell>, Vec<DetailCell>) {
    let is_fund = row.kind == MUTUAL_FUND;
    let price_label = if is_fund { BASE_PRICE } else { CURRENT_PRICE };
    let price_text = row
        .current_price
        .map(format_currency)
        .unwrap_or_else(|| "—".to_string());
    let day_text = if row.nav_pending {
        PENDING_UPDATE.to_string()
    } else {
        format_signed_row_valuation_amount(row.day_change)
    };
    let day_negative = is_negative_valuation(row.day_change);
    let day_positive = is_positive_valuation(row.day_change);
    let average_text = row
        .average_price
        .map(format_currency)
        .unwrap_or_else(|| "—".to_string());
    let purchase_text = row
        .purchase
        .map(format_currency)
        .unwrap_or_else(|| "—".to_string());
    let market_text = format_row_valuation_amount(row.market);
    let pnl_text = format_signed_row_valuation_amount(row.profit);
    let pnl_rate = format_signed_valuation_rate(row.rate, 1);
    let pnl_sub = row
        .rate
        .map(|_| format!("{pnl_text} {pnl_rate}"))
        .or_else(|| row.profit.map(|_| pnl_text.clone()));
    let pnl_negative = is_negative_valuation(row.profit);
    let pnl_positive = is_positive_valuation(row.profit);
    let as_of = row
        .price_as_of
        .clone()
        .map_or_else(|| "—".to_string(), |date| format_iso_date(&date));
    let mut left = vec![
        DetailCell::delta(
            price_label.to_string(),
            price_text,
            Some(day_text),
            day_negative,
            day_positive,
        ),
        DetailCell::plain(AVERAGE_PRICE.to_string(), average_text),
    ];
    if is_fund {
        left.push(DetailCell::plain(
            DISTRIBUTION.to_string(),
            row.course.clone().unwrap_or_else(|| "—".to_string()),
        ));
    } else {
        let per_share = row
            .dividend
            .as_ref()
            .map(|dividend| {
                crate::features::asset_balance::holdings::format_dividend_per_share(dividend, 0)
            })
            .unwrap_or_else(|| "—".to_string());
        let yield_text = row
            .dividend
            .as_ref()
            .map(crate::features::asset_balance::holdings::format_dividend_yield)
            .unwrap_or_else(|| "—".to_string());
        left.push(DetailCell::plain(ACQUISITION_YIELD.to_string(), yield_text));
        left.push(DetailCell::plain(PER_SHARE_DIVIDEND.to_string(), per_share));
    }
    let mut right = vec![
        DetailCell::plain(
            if is_fund {
                FUND_SHARES.to_string()
            } else {
                QUANTITY.to_string()
            },
            row.quantity.clone(),
        ),
        DetailCell::plain(TOTAL_ACQUISITION.to_string(), purchase_text),
        DetailCell::delta(
            MARKET_VALUE.to_string(),
            market_text,
            pnl_sub,
            pnl_negative,
            pnl_positive,
        ),
    ];
    if !is_fund {
        let annual = row
            .dividend
            .as_ref()
            .map(crate::features::asset_balance::holdings::format_dividend_annual)
            .unwrap_or_else(|| "—".to_string());
        right.push(DetailCell::plain(ANNUAL_DIVIDEND.to_string(), annual));
    }
    right.push(DetailCell::plain(
        if is_fund {
            "基準日".to_string()
        } else {
            PRICE_AS_OF.to_string()
        },
        as_of,
    ));
    (left, right)
}

#[component]
pub fn AssetMergedList(rows: Vec<UnifiedRow>) -> impl IntoView {
    view! {
        <ul class="asset-compact-list" data-testid="asset-merged-list">
            {rows
                .into_iter()
                .map(|row| {
                    let pnl_text = format_signed_row_valuation_amount(row.profit);
                    let rate_text = format_signed_valuation_rate(row.rate, 1);
                    let pnl_negative = is_negative_valuation(row.profit);
                    let pnl_positive = is_positive_valuation(row.profit);
                    let rate_negative = is_negative_rate(row.rate);
                    let rate_positive = is_positive_rate(row.rate);
                    let market_text = format_row_valuation_amount(row.market);
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
                    let inner = view! {
                        <ListRow
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
                    };
                    view! {
                        <li class="asset-compact-row" data-testid="asset-compact-row">
                            <Button
                                variant=ButtonVariant::RowButton
                                testid="asset-row-button"
                                aria_haspopup="dialog"
                                on_click=move |_| open.set(true)
                            >
                                <span class="receipt-compact-button-inner">{inner}</span>
                            </Button>
                            <Show when=move || open.get()>
                                <DetailDialog
                                    name=dialog_name.clone()
                                    code=dialog_code.clone().unwrap_or_default()
                                    account=dialog_account.clone().unwrap_or_default()
                                    left=left.clone()
                                    right=right.clone()
                                    on_close=move || open_for_dialog.set(false)
                                    testid="asset-merged-dialog"
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
    use super::{unified_detail_cells, unified_from_fund, unified_from_stock};
    use crate::api::dto::FundPriceSource;
    use crate::features::asset_balance::fund::FundView;
    use crate::features::asset_balance::holdings::HoldingView;
    use crate::features::dividend_per_share::DividendMaps;

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
        assert_eq!(left_labels, ["基準価額", "取得単価", "分配金"]);
        let right_labels: Vec<&str> = right.iter().map(|cell| cell.label.as_str()).collect();
        assert_eq!(right_labels, ["保有口数", "取得総額", "評価額", "基準日"]);
    }
}
