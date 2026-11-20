//! 「すべて」タブ。国内株式と投資信託を同じ一覧に並べる

use super::asset_columns::{
    UnifiedRow, day_change_rate_cell, merged_dividend_decimals, merged_table_head, sort_unified,
    unified_day_change_rate, unified_detail_cells,
};
use super::day_change::pending_day_change_cell;
use crate::features::asset_balance::format as balance_format;
use crate::features::asset_balance::fund::FundSortKey;
use crate::features::asset_balance::holdings;
use crate::features::asset_balance::portfolio::SortOrder;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::asset_balance::{AssetMarker, AssetRateAmount};
use crate::ui::badge::AccountBadge;
use crate::ui::detail_dialog::DetailDialog;
use crate::ui::labels;
use crate::ui::list_row::ListRowButton;
use crate::ui::security_link::{TableNameLink, TableSecurityCode};
use leptos::prelude::*;

#[component]
pub fn AssetMergedTable(
    rows: Vec<UnifiedRow>,
    sort_key: RwSignal<FundSortKey>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    let dividend_decimals = merged_dividend_decimals(&rows);
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
                    {merged_table_head(sort_key, sort_order)}
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
                                            {row.account.as_ref().map_or_else(|| view! { <AssetMarker text=labels::unavailable_text(labels::UnavailableKind::Unassigned).to_string() soft=true /> }
                                                    .into_any(), |account| view! {
                                                    <AccountBadge label=account.clone() />
                                                }
                                                    .into_any())}
                                        </td>
                                        <td class="col-text text-text-soft">
                                            {row.course.clone().map_or_else(
                                                || view! {
                                                    <AssetMarker text=if row.kind == labels::MUTUAL_FUND {
                                                        labels::FETCH_UNAVAILABLE.to_string()
                                                    } else {
                                                        labels::unavailable_text(
                                                            labels::UnavailableKind::OutOfScope,
                                                        )
                                                        .to_string()
                                                    } />
                                                }
                                                    .into_any(),
                                                |course| view! { {course} }.into_any(),
                                            )}
                                        </td>
                                        <td class="col-num">
                                            {row
                                                .dividend
                                                .as_ref().map_or_else(|| view! { <AssetMarker text=labels::unavailable_text(labels::UnavailableKind::OutOfScope).to_string() /> }.into_any(), |dividend| {
                                                    holdings::dividend_marker_text(dividend).map_or_else(
                                                        || view! {
                                                            <Amount text=holdings::format_dividend_per_share(
                                                                dividend,
                                                                dividend_decimals,
                                                            ) />
                                                        }.into_any(),
                                                        |marker| view! { <AssetMarker text=marker.to_string() /> }.into_any(),
                                                    )
                                                })}
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
        </div>
    }
}

#[component]
pub fn AssetMergedList(rows: Vec<UnifiedRow>) -> impl IntoView {
    view! {
        <>
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
        </>
    }
}

#[cfg(test)]
mod tests {
    use super::super::asset_columns::{
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
            ["数量", "取得総額", "評価額", "年間予想配当(税引前)", "株価"]
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
            ["保有口数", "取得総額", "評価額", labels::BASE_PRICE_AS_OF]
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
