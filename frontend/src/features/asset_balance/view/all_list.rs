//! 「すべて」タブ。国内株式と投資信託を同じ一覧に並べる

use super::day_change::{day_change_cell, pending_day_change_cell};
use super::sort_header::sort_header;
use crate::features::asset_balance::format::{
    format_row_valuation_amount, pnl_cells, unit_price_decimals,
};
use crate::features::asset_balance::fund::{FundSortKey, FundView};
use crate::features::asset_balance::holdings::{
    HoldingDividend, HoldingView, format_dividend_per_share, holding_dividend,
};
use crate::features::asset_balance::model::{ValuationInput, holding_valuation};
use crate::features::asset_balance::portfolio::SortOrder;
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::badge::AccountBadge;
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
}

pub(crate) fn unified_from_stock(view: &HoldingView, dividends: &DividendMaps) -> UnifiedRow {
    let valuation = holding_valuation(&ValuationInput::from(view));
    UnifiedRow {
        name: view.name.clone(),
        kind: "国内株式",
        sub: view.code.clone(),
        code: Some(view.code.clone()),
        account: view.account.clone(),
        course: None,
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
    }
}

pub(crate) fn unified_from_fund(view: &FundView) -> UnifiedRow {
    UnifiedRow {
        name: view.name.clone(),
        kind: "投資信託",
        sub: view.course.clone().unwrap_or_default(),
        code: None,
        account: Some(view.account.clone()),
        course: view.course.clone(),
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
                        <th scope="col">"コード"</th>
                        <th scope="col">"銘柄・ファンド"</th>
                        <th scope="col">"種別"</th>
                        <th scope="col">"口座"</th>
                        <th scope="col">"分配金"</th>
                        <th scope="col" class="text-right">"1株配当"</th>
                        <th scope="col" class="text-right">"前日比"</th>
                        {sort_header("評価額", FundSortKey::MarketValue, sort_key, sort_order)}
                        {sort_header("評価損益", FundSortKey::ProfitLoss, sort_key, sort_order)}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        sorted
                            .get()
                            .into_iter()
                            .map(|row| {
                                let row = &row;
                                let (pnl_text, rate_text, pnl_text_class, pnl_badge_class) =
                                    pnl_cells(row.profit, row.rate);
                                let outlier_date = if row.date_outlier {
                                    row.price_as_of.clone()
                                } else {
                                    None
                                };
                                view! {
                                    <tr>
                                        <td>
                                            <TableSecurityCode
                                                code=row.code.clone().unwrap_or_default()
                                            />
                                        </td>
                                        <td class="asset-name-cell" title=row.name.clone()>
                                            <TableNameLink
                                                href=row.href.clone()
                                                name=row.name.clone()
                                            />
                                        </td>
                                        <td class="text-text-soft">{row.kind}</td>
                                        <td>
                                            {match &row.account {
                                                Some(account) => view! {
                                                    <AccountBadge label=account.clone() />
                                                }
                                                    .into_any(),
                                                None => view! { <span class="text-text-soft">"—"</span> }
                                                    .into_any(),
                                            }}
                                        </td>
                                        <td class="text-text-soft">
                                            {row.course.clone().unwrap_or_else(|| "—".to_string())}
                                        </td>
                                        <td class="text-right tabular-nums">
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
                                            class="text-right tabular-nums"
                                            title=row.day_change_title.clone()
                                        >
                                            {if row.nav_pending {
                                                pending_day_change_cell().into_any()
                                            } else {
                                                day_change_cell(row.day_change, outlier_date)
                                                    .into_any()
                                            }}
                                        </td>
                                        <td class="text-right font-bold tabular-nums text-text-deep">
                                            {format_row_valuation_amount(row.market)}
                                        </td>
                                        <td class="text-right tabular-nums">
                                            {if row.profit.is_some() {
                                                view! {
                                                    <span class=format!(
                                                        "font-bold tabular-nums {pnl_text_class}",
                                                    )>
                                                        {pnl_text}
                                                    </span>
                                                    {row.rate.is_some().then(|| {
                                                        view! {
                                                            <span class=pnl_badge_class>
                                                                {rate_text.clone()}
                                                            </span>
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

#[component]
pub fn AssetMergedList(rows: Vec<UnifiedRow>) -> impl IntoView {
    // 列の中の小数桁をそろえる(整数円の値しかない列は .00 を付けない)
    let dividend_decimals = unit_price_decimals(
        rows.iter()
            .filter_map(|row| row.dividend.as_ref()?.per_share),
    );
    view! {
        <ul class="asset-compact-list" data-testid="asset-merged-list">
            {rows
                .into_iter()
                .map(|row| {
                    let (pnl_text, rate_text, pnl_text_class, pnl_badge_class) =
                        pnl_cells(row.profit, row.rate);
                    let meta = if row.sub.is_empty() {
                        row.kind.to_string()
                    } else {
                        format!("{} ・ {}", row.kind, row.sub)
                    };
                    let inner = view! {
                        <>
                            <div class="flex items-baseline justify-between gap-2">
                                <span class="min-w-0 truncate font-medium text-text">
                                    {row.name.clone()}
                                </span>
                                <span class="shrink-0 font-bold tabular-nums text-text-deep">
                                    {format_row_valuation_amount(row.market)}
                                </span>
                            </div>
                            <div class="mt-0.5 flex items-baseline justify-between gap-2 text-xs">
                                <span class="min-w-0 truncate text-text-muted">
                                    {meta}
                                    {row
                                        .account
                                        .clone()
                                        .map(|account| {
                                            view! {
                                                " · "
                                                <AccountBadge label=account />
                                            }
                                        })}
                                </span>
                                {if row.profit.is_some() {
                                    view! {
                                        <span class="flex shrink-0 items-baseline">
                                            <span class=format!(
                                                "font-bold tabular-nums {pnl_text_class}",
                                            )>
                                                {pnl_text}
                                            </span>
                                            {row.rate.is_some().then(|| {
                                                view! {
                                                    <span class=pnl_badge_class>
                                                        {rate_text.clone()}
                                                    </span>
                                                }
                                            })}
                                        </span>
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <span class="shrink-0 text-text-muted tabular-nums">
                                            "—"
                                        </span>
                                    }
                                        .into_any()
                                }}
                            </div>
                            {row.dividend.as_ref().map(|dividend| {
                                view! {
                                    <div class="mt-0.5 flex items-baseline justify-between gap-2 text-xs">
                                        <span class="text-text-muted tabular-nums">
                                            "1株 "
                                            {format_dividend_per_share(
                                                dividend,
                                                dividend_decimals,
                                            )}
                                        </span>
                                    </div>
                                }
                            })}
                        </>
                    };
                    view! {
                        <li class="asset-compact-row" data-testid="asset-compact-row">
                            {match &row.href {
                                Some(href) => view! {
                                    <a href=href.clone() class="asset-compact-link">
                                        {inner}
                                    </a>
                                }
                                    .into_any(),
                                None => view! { <div class="asset-compact-link">{inner}</div> }
                                    .into_any(),
                            }}
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
}
