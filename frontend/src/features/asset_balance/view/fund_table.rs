use super::day_change::{DAY_CHANGE_COLUMN_NOTE, day_change_cell, pending_day_change_cell};
use super::sort_header::sort_header;
use crate::features::asset_balance::format::{
    format_row_valuation_amount, format_signed_row_valuation_amount, format_signed_valuation_rate,
    is_negative_rate, is_negative_valuation, is_positive_rate, is_positive_valuation,
};
use crate::features::asset_balance::fund::{FundSortKey, FundView};
use crate::features::asset_balance::portfolio::SortOrder;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::AccountBadge;
use crate::ui::detail_dialog::{DetailCell, DetailDialog};
use crate::ui::format::{
    format_currency, format_iso_date, format_number_value, format_unit_price, unit_price_decimals,
};
use crate::ui::labels::{
    ACCOUNT, AVERAGE_PRICE, BASE_PRICE, BASE_PRICE_MONTH_CHANGE, DAY_CHANGE, DISTRIBUTION, FUND,
    FUND_SHARES, MARKET_VALUE, PENDING_UPDATE, TOTAL_ACQUISITION, VALUATION_PROFIT_LOSS,
};
use crate::ui::list_row::ListRowButton;
use crate::ui::security_link::TableNameLink;
use leptos::prelude::*;

#[component]
pub fn FundHoldingsTable(
    views: Vec<FundView>,
    sort_key: RwSignal<FundSortKey>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    // 列の中の小数桁をそろえる(整数円の値しかない列は .00 を付けない)
    let base_price_decimals = unit_price_decimals(views.iter().map(|view| view.base_price));
    view! {
        <div data-testid="asset-fund-table">
            <table class="asset-table">
                <thead>
                    <tr>
                        <th scope="col" class="col-text">{FUND}</th>
                        <th scope="col" class="col-text">{ACCOUNT}</th>
                        <th scope="col" class="col-text">{DISTRIBUTION}</th>
                        {sort_header(FUND_SHARES, FundSortKey::Shares, sort_key, sort_order)}
                        {sort_header(BASE_PRICE, FundSortKey::BasePrice, sort_key, sort_order)}
                        <th
                            scope="col"
                            class="col-num"
                            title=DAY_CHANGE_COLUMN_NOTE
                            abbr=DAY_CHANGE_COLUMN_NOTE
                        >
                            {DAY_CHANGE}
                        </th>
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
                    {views
                        .into_iter()
                        .map(|view| {
                            let pnl_text = format_signed_row_valuation_amount(Some(
                                view.profit_loss,
                            ));
                            let pnl_rate_text =
                                format_signed_valuation_rate(view.profit_loss_rate, 1);
                            let pnl_negative =
                                is_negative_valuation(Some(view.profit_loss));
                            let pnl_positive =
                                is_positive_valuation(Some(view.profit_loss));
                            let rate_negative =
                                is_negative_rate(view.profit_loss_rate);
                            let rate_positive =
                                is_positive_rate(view.profit_loss_rate);
                            let href = view
                                .id
                                .clone()
                                .map(|id| format!("/assetbalance/funds/{id}"));
                            let shares_text = format!(
                                "{}口",
                                format_number_value(view.shares)
                            );
                            let base_price_text = format_unit_price(
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
                                        {view.course.clone().unwrap_or_else(|| "—".to_string())}
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
                                            text=format_row_valuation_amount(
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
                                        <Amount
                                            kind=AmountKind::Delta
                                            text=pnl_rate_text
                                            negative=rate_negative
                                            positive=rate_positive
                                            class="ml-1 text-xs"
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

pub(crate) fn fund_detail_cells(view: &FundView) -> (Vec<DetailCell>, Vec<DetailCell>) {
    let base_text = format_currency(view.base_price);
    let day_text = if view.nav_pending() {
        PENDING_UPDATE.to_string()
    } else {
        format_signed_row_valuation_amount(view.view_day_change())
    };
    let day_negative = is_negative_valuation(view.view_day_change());
    let day_positive = is_positive_valuation(view.view_day_change());
    let month_text = if view.nav_pending() {
        PENDING_UPDATE.to_string()
    } else {
        format_signed_row_valuation_amount(view.base_price_month_change)
    };
    let month_negative = is_negative_valuation(view.base_price_month_change);
    let month_positive = is_positive_valuation(view.base_price_month_change);
    let average_text = format_currency(view.average_purchase_price);
    let course_text = view.course.clone().unwrap_or_else(|| "—".to_string());
    let shares_text = format!("{}口", format_number_value(view.shares));
    let purchase_text = format_currency(view.purchase);
    let market_text = format_row_valuation_amount(Some(view.market_value));
    let pnl_text = format_signed_row_valuation_amount(Some(view.profit_loss));
    let pnl_rate = format_signed_valuation_rate(view.profit_loss_rate, 1);
    let pnl_sub = format!("{pnl_text} {pnl_rate}");
    let pnl_negative = is_negative_valuation(Some(view.profit_loss));
    let pnl_positive = is_positive_valuation(Some(view.profit_loss));
    let as_of = view
        .price_as_of
        .clone()
        .map_or_else(|| "—".to_string(), |date| format_iso_date(&date));
    let left = vec![
        DetailCell::with_sub(
            BASE_PRICE.to_string(),
            base_text,
            day_text,
            day_negative,
            day_positive,
        ),
        DetailCell {
            label: BASE_PRICE_MONTH_CHANGE.to_string(),
            value: month_text,
            sub: None,
            negative: month_negative,
            positive: month_positive,
            bold: false,
            delta: true,
        },
        DetailCell::plain(AVERAGE_PRICE.to_string(), average_text),
        DetailCell::plain(DISTRIBUTION.to_string(), course_text),
    ];
    let right = vec![
        DetailCell::plain(FUND_SHARES.to_string(), shares_text),
        DetailCell::plain(TOTAL_ACQUISITION.to_string(), purchase_text),
        DetailCell::strong_sub(
            MARKET_VALUE.to_string(),
            market_text,
            pnl_sub,
            pnl_negative,
            pnl_positive,
        ),
        DetailCell::plain("基準日".to_string(), as_of),
    ];
    (left, right)
}

/// スマホのファンド一覧。共通の2行の行で出す
#[component]
pub fn FundHoldingsList(views: Vec<FundView>) -> impl IntoView {
    view! {
        <ul class="asset-compact-list" data-testid="asset-fund-list">
            {views
                .into_iter()
                .map(|view| {
                    let pnl_text = format_signed_row_valuation_amount(Some(view.profit_loss));
                    let pnl_rate_text =
                        format_signed_valuation_rate(view.profit_loss_rate, 1);
                    let pnl_negative = is_negative_valuation(Some(view.profit_loss));
                    let pnl_positive = is_positive_valuation(Some(view.profit_loss));
                    let rate_negative = is_negative_rate(view.profit_loss_rate);
                    let rate_positive = is_positive_rate(view.profit_loss_rate);
                    let shares_text =
                        format!("{}口", format_number_value(view.shares));
                    let market_text = format_row_valuation_amount(Some(view.market_value));
                    let name = view.name.clone();
                    let account = view.account.clone();
                    let dialog_name = view.name.clone();
                    let dialog_account = view.account.clone();
                    let dialog_view = view.clone();
                    let open = RwSignal::new(false);
                    let open_for_dialog = open;
                    let (left, right) = fund_detail_cells(&dialog_view);
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
    use super::fund_detail_cells;
    use crate::api::dto::FundPriceSource;
    use crate::features::asset_balance::fund::FundView;

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
        let (left, right) = fund_detail_cells(&sample_fund());
        let left_labels: Vec<&str> = left.iter().map(|cell| cell.label.as_str()).collect();
        assert_eq!(
            left_labels,
            ["基準価額", "基準価額(前月比)", "取得単価", "分配金"]
        );
        let right_labels: Vec<&str> = right.iter().map(|cell| cell.label.as_str()).collect();
        assert_eq!(right_labels, ["保有口数", "取得総額", "評価額", "基準日"]);
    }

    #[test]
    fn fund_dialog_uses_row_basis() {
        let view = sample_fund();
        let (left, right) = fund_detail_cells(&view);
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
        let (left, _) = fund_detail_cells(&view);
        let month = &left[1];
        assert!(month.delta);
        assert!(month.negative);
        assert!(!month.positive);
        assert!(month.value.starts_with('-'));
        view.base_price_month_change = Some(20.0);
        let (left, _) = fund_detail_cells(&view);
        let month = &left[1];
        assert!(month.positive);
        assert!(month.value.starts_with('+'));
    }
}
