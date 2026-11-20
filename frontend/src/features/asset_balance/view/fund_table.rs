use super::asset_columns;
use super::day_change::{day_change_cell, pending_day_change_cell};
use crate::features::asset_balance::format as balance_format;
use crate::features::asset_balance::fund::{FundSortKey, FundView};
use crate::features::asset_balance::portfolio::SortOrder;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::asset_balance::{AssetMarker, AssetRateAmount};
use crate::ui::badge::AccountBadge;
use crate::ui::detail_dialog::DetailDialog;
use crate::ui::format;
use crate::ui::labels;
use crate::ui::list_row::ListRowButton;
use crate::ui::security_link::TableNameLink;
use crate::urls;
use leptos::prelude::*;

#[component]
pub fn FundHoldingsTable(
    views: Vec<FundView>,
    sort_key: RwSignal<FundSortKey>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    // 列の中の小数桁をそろえる(整数円の値しかない列は .00 を付けない)
    let base_price_decimals = asset_columns::fund_base_price_decimals(&views);
    view! {
        <div data-testid="asset-fund-table">
            <table class="asset-table">
                <thead>
                    {asset_columns::fund_table_head(sort_key, sort_order)}
                </thead>
                <tbody>
                    {views
                        .into_iter()
                        .map(|view| {
                            let pnl_text = balance_format::format_signed_row_valuation_amount(Some(
                                view.profit_loss,
                            ));
                            let pnl_rate_text =
                                balance_format::format_signed_valuation_rate(view.profit_loss_rate, 1);
                            let pnl_negative =
                                balance_format::is_negative_valuation(Some(view.profit_loss));
                            let pnl_positive =
                                balance_format::is_positive_valuation(Some(view.profit_loss));
                            let rate_negative =
                                balance_format::is_negative_rate(view.profit_loss_rate);
                            let rate_positive =
                                balance_format::is_positive_rate(view.profit_loss_rate);
                            let href = view.id.as_ref().map(urls::fund_holding_href);
                            let shares_text = labels::fund_units_label(
                                &format::format_number_value(view.shares),
                            );
                            let base_price_text = format::format_unit_price(
                                view.base_price,
                                base_price_decimals,
                            );
                            view! {
                                <tr>
                                    <td class="asset-name-cell col-text" title=view.name.clone()>
                                        <TableNameLink href=href name=view.name.clone() />
                                    </td>
                                    <td class="col-text">
                                        <AccountBadge label=view.account.clone() />
                                    </td>
                                    <td class="col-text text-text-soft">
                                        {view.course.clone().map_or_else(
                                            || view! { <AssetMarker text=labels::FETCH_UNAVAILABLE.to_string() /> }.into_any(),
                                            |course| view! { {course} }.into_any(),
                                        )}
                                    </td>
                                    <td class="col-num text-text-soft">
                                        <Amount kind=AmountKind::Plain text=shares_text />
                                    </td>
                                    <td class="col-num text-text-soft">
                                        <Amount kind=AmountKind::Plain text=base_price_text />
                                    </td>
                                    <td
                                        class="col-num"
                                        title=view.day_change_title()
                                    >
                                        {if view.nav_pending() {
                                            pending_day_change_cell().into_any()
                                        } else {
                                            day_change_cell(view.view_day_change(), None)
                                                .into_any()
                                        }}
                                    </td>
                                    <td class="col-num font-bold text-text-deep">
                                        <Amount
                                            kind=AmountKind::Plain
                                            text=balance_format::format_row_valuation_amount(
                                                Some(view.market_value),
                                            )
                                        />
                                    </td>
                                    <td class="col-num">
                                        <Amount
                                            kind=AmountKind::Delta
                                            text=pnl_text
                                            negative=pnl_negative
                                            positive=pnl_positive
                                            class="font-bold"
                                        />
                                        <AssetRateAmount
                                            text=pnl_rate_text
                                            negative=rate_negative
                                            positive=rate_positive
                                        />
                                    </td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}

/// スマホのファンド一覧。共通の2行の行で出す
#[component]
pub fn FundHoldingsList(views: Vec<FundView>) -> impl IntoView {
    view! {
        <ul class="asset-compact-list" data-testid="asset-fund-list">
            {views
                .into_iter()
                .map(|view| {
                    let pnl_text = balance_format::format_signed_row_valuation_amount(Some(view.profit_loss));
                    let pnl_rate_text =
                        balance_format::format_signed_valuation_rate(view.profit_loss_rate, 1);
                    let pnl_negative = balance_format::is_negative_valuation(Some(view.profit_loss));
                    let pnl_positive = balance_format::is_positive_valuation(Some(view.profit_loss));
                    let rate_negative = balance_format::is_negative_rate(view.profit_loss_rate);
                    let rate_positive = balance_format::is_positive_rate(view.profit_loss_rate);
                    let shares_text =
                        labels::fund_units_label(&format::format_number_value(view.shares));
                    let market_text = balance_format::format_row_valuation_amount(Some(view.market_value));
                    let name = view.name.clone();
                    let account = view.account.clone();
                    let dialog_name = view.name.clone();
                    let dialog_account = view.account.clone();
                    let dialog_view = view.clone();
                    let open = RwSignal::new(false);
                    let open_for_dialog = open;
                    let (left, right) = asset_columns::fund_detail_cells(&dialog_view);
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
                                    <AccountBadge label=account.clone() />
                                    {" · "}
                                    <Amount kind=AmountKind::Plain text=shares_text.clone() />
                                }
                                secondary=move || view! {
                                    <Amount
                                        kind=AmountKind::Delta
                                        text=pnl_text.clone()
                                        negative=pnl_negative
                                        positive=pnl_positive
                                    />
                                    {" "}
                                    <Amount
                                        kind=AmountKind::Delta
                                        text=pnl_rate_text.clone()
                                        negative=rate_negative
                                        positive=rate_positive
                                    />
                                }
                            />
                            <Show when=move || open.get()>
                                <DetailDialog
                                    name=dialog_name.clone()
                                    account=dialog_account.clone()
                                    left=left.clone()
                                    right=right.clone()
                                    on_close=move || open_for_dialog.set(false)
                                    testid="asset-fund-dialog"
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
    use super::super::asset_columns;
    use crate::api::dto::FundPriceSource;
    use crate::features::asset_balance::fund::FundView;
    use crate::ui::amount::AmountKind;
    use crate::ui::labels;

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
            total_return: Some(10000.0),
            price_as_of: Some("2026-01-05".to_string()),
            price_source: FundPriceSource::Api,
            is_nav_latest: true,
            expected_nav_date: None,
        }
    }

    #[test]
    fn fund_dialog_preserves_order() {
        let (left, right) = asset_columns::fund_detail_cells(&sample_fund());
        let left_labels: Vec<&str> = left.iter().map(|cell| cell.label.as_str()).collect();
        assert_eq!(
            left_labels,
            [
                labels::BASE_PRICE,
                labels::BASE_PRICE_MONTH_CHANGE,
                labels::AVERAGE_PRICE_PER_10K,
                labels::DISTRIBUTION_COURSE
            ]
        );
        let right_labels: Vec<&str> = right.iter().map(|cell| cell.label.as_str()).collect();
        assert_eq!(
            right_labels,
            [
                labels::FUND_SHARES,
                labels::TOTAL_ACQUISITION,
                labels::MARKET_VALUE,
                labels::BASE_PRICE_AS_OF
            ]
        );
    }

    #[test]
    fn fund_dialog_uses_row_basis() {
        let view = sample_fund();
        let (left, right) = asset_columns::fund_detail_cells(&view);
        assert!(left[0].value.contains("11,000") || left[0].value.contains("11000"));
        assert!(right[2].value.contains("110,000") || right[2].value.contains("110000"));
        assert!(
            right[2]
                .sub
                .as_deref()
                .is_some_and(|sub| sub.contains("10,000"))
        );
    }

    #[test]
    fn fund_dialog_month_change_uses_delta_sign() {
        let mut view = sample_fund();
        view.base_price_month_change = Some(-30.0);
        let (left, _) = asset_columns::fund_detail_cells(&view);
        let month = &left[1];
        assert_eq!(month.kind, AmountKind::Delta);
        assert!(month.negative);
        assert!(!month.positive);
        assert!(month.value.starts_with('-'));
        view.base_price_month_change = Some(20.0);
        let (left, _) = asset_columns::fund_detail_cells(&view);
        let month = &left[1];
        assert!(month.positive);
        assert!(month.value.starts_with('+'));
    }
}
