use super::asset_columns::holding_detail_cells;
use crate::features::asset_balance::format as balance_format;
use crate::features::asset_balance::holdings::HoldingView;
use crate::features::asset_balance::model;
use crate::features::asset_balance::model::ValuationInput;
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::AccountBadge;
use crate::ui::detail_dialog::DetailDialog;
use crate::ui::format;
use crate::ui::labels;
use crate::ui::list_row::ListRowButton;
use leptos::prelude::*;

#[component]
pub fn HoldingsList(views: Vec<HoldingView>, dividends: RwSignal<DividendMaps>) -> impl IntoView {
    view! {
        <>
            <ul class="asset-compact-list" data-testid="asset-holdings-list">
            {views
                .into_iter()
                .map(|view| {
                    let valuation = model::holding_valuation(&ValuationInput::from(&view));
                    let market_text = balance_format::format_row_valuation_amount(valuation.market_value);
                    let pnl_text = balance_format::format_signed_row_valuation_amount(
                        valuation.profit_loss.amount,
                    );
                    let pnl_rate_text = balance_format::format_signed_valuation_rate(
                        valuation.profit_loss.rate,
                        1,
                    );
                    let pnl_negative = balance_format::is_negative_valuation(valuation.profit_loss.amount);
                    let pnl_positive = balance_format::is_positive_valuation(valuation.profit_loss.amount);
                    let rate_negative = balance_format::is_negative_rate(valuation.profit_loss.rate);
                    let rate_positive = balance_format::is_positive_rate(valuation.profit_loss.rate);
                    let shares_text =
                        labels::stock_shares_label(&format::format_number_value(view.shares));
                    let code = view.code.clone();
                    let name = view.name.clone();
                    let account = view.account.clone();
                    let dialog_name = view.name.clone();
                    let dialog_code = view.code.clone();
                    let dialog_account = view.account.clone();
                    let dialog_view = view.clone();
                    let open = RwSignal::new(false);
                    let open_for_dialog = open;
                    let (left, right) = holding_detail_cells(
                        &dialog_view,
                        &dividends.get(),
                    );
                    view! {
                        <li class="list-row-item" data-testid="asset-compact-row">
                            <ListRowButton
                                testid="asset-row-button"
                                on_click=move |_| open.set(true)
                                name=move || name.clone()
                                primary=move || view! {
                                    <Amount
                                        kind=AmountKind::Plain
                                        text=market_text.clone()
                                    />
                                }
                                meta=move || view! {
                                    {code.clone()}
                                    {account
                                        .clone()
                                        .map(|account| {
                                            view! {
                                                " · "
                                                <AccountBadge label=account />
                                            }
                                        })}
                                    {" · "}
                                    <Amount kind=AmountKind::Plain text=shares_text.clone() />
                                }
                                secondary=move || view! {
                                    {if valuation.profit_loss.amount.is_some() {
                                        view! {
                                            <Amount
                                                kind=AmountKind::Delta
                                                text=pnl_text.clone()
                                                negative=pnl_negative
                                                positive=pnl_positive
                                            />
                                            {valuation
                                                .profit_loss
                                                .rate
                                                .is_some()
                                                .then(|| {
                                                    view! {
                                                        " "
                                                        <Amount
                                                            kind=AmountKind::Delta
                                                            text=pnl_rate_text.clone()
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
                                    code=dialog_code.clone()
                                    account=dialog_account.clone().unwrap_or_default()
                                    left=left.clone()
                                    right=right.clone()
                                    on_close=move || open_for_dialog.set(false)
                                    testid="asset-holding-dialog"
                                    link_code=dialog_code.clone()
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
    use super::super::asset_columns::holding_detail_cells;
    use crate::features::asset_balance::holdings::HoldingView;
    use crate::features::dividend_per_share::DividendMaps;
    use crate::ui::amount::AmountKind;
    use crate::ui::labels;

    fn sample_view() -> HoldingView {
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

    #[test]
    fn holding_dialog_preserves_order() {
        let (left, right) = holding_detail_cells(&sample_view(), &DividendMaps::default());
        let left_labels: Vec<&str> = left.iter().map(|cell| cell.label.as_str()).collect();
        assert_eq!(
            left_labels,
            [
                labels::CURRENT_PRICE,
                labels::AVERAGE_PRICE,
                labels::ACQUISITION_YIELD,
                labels::PER_SHARE_DIVIDEND
            ]
        );
        let right_labels: Vec<&str> = right.iter().map(|cell| cell.label.as_str()).collect();
        assert_eq!(
            right_labels,
            [
                labels::QUANTITY,
                labels::TOTAL_ACQUISITION,
                labels::MARKET_VALUE,
                labels::ANNUAL_EXPECTED_DIVIDEND,
                labels::STOCK_PRICE
            ]
        );
    }

    #[test]
    fn holding_dialog_uses_row_basis() {
        let view = sample_view();
        let (left, right) = holding_detail_cells(&view, &DividendMaps::default());
        assert!(left[0].value.contains("2,600") || left[0].value.contains("2600"));
        assert!(right[1].value.contains("250,000") || right[1].value.contains("250000"));
        assert!(right[2].value.contains("260,000") || right[2].value.contains("260000"));
    }

    #[test]
    fn only_deltas_carry_sign_and_color() {
        let mut view = sample_view();
        view.day_change = Some(-30.0);
        view.current_price = 2000.0;
        view.purchase = 250000.0;
        let (left, right) = holding_detail_cells(&view, &DividendMaps::default());
        let price = &left[0];
        assert_eq!(price.kind, AmountKind::Plain);
        assert!(!price.bold);
        assert_eq!(price.sub.as_deref(), Some("-¥30"));
        assert!(price.negative);
        let market = &right[2];
        assert_eq!(market.kind, AmountKind::Plain);
        assert!(market.bold);
        assert!(
            market
                .sub
                .as_deref()
                .is_some_and(|sub| sub.contains("-¥50,000"))
        );
        assert!(market.negative);
    }
}
