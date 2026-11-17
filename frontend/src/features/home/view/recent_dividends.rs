use super::super::store::{self, HomeOverviewData};
use super::card_parts::{CardError, CardHead, CardSkeleton};
use crate::api::dto::Dividend;
use crate::features::asset_balance;
use crate::session;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::AccountBadge;
use crate::ui::card::{Card, CardVariant};
use crate::ui::detail_dialog::{DetailCell, DetailDialog};
use crate::ui::empty_state::EmptyState;
use crate::ui::format;
use crate::ui::labels;
use crate::ui::list_row::ListRowButtonWithoutSecondary;
use leptos::prelude::*;

pub(super) fn home_dividend_dialog_cells(
    full_date: &str,
    shares: String,
    per_share: String,
    gross: (String, bool, bool),
    tax: (String, bool, bool),
    net: (String, bool, bool),
) -> (Vec<DetailCell>, Vec<DetailCell>) {
    let left = vec![
        DetailCell::plain(
            labels::PAYMENT_DATE.to_string(),
            format::format_iso_date(full_date),
        ),
        DetailCell::plain(labels::QUANTITY.to_string(), shares),
        DetailCell::plain(labels::PER_SHARE_DIVIDEND.to_string(), per_share),
    ];
    let right = vec![
        DetailCell {
            label: labels::BEFORE_TAX.to_string(),
            value: gross.0,
            sub: None,
            negative: gross.1,
            positive: gross.2,
            bold: false,
            kind: AmountKind::Delta,
        },
        DetailCell {
            label: labels::TAX.to_string(),
            value: tax.0,
            sub: None,
            negative: tax.1,
            positive: tax.2,
            bold: false,
            kind: if tax.1 || tax.2 {
                AmountKind::NeutralDelta
            } else {
                AmountKind::Plain
            },
        },
        DetailCell::strong(
            labels::AFTER_TAX.to_string(),
            net.0,
            net.1,
            net.2,
            AmountKind::Delta,
        ),
    ];
    (left, right)
}

pub(super) fn signed_gross_text(value: f64, text: &str) -> String {
    if value > 0.0 && !text.trim_start().starts_with('+') {
        format!("+{text}")
    } else {
        text.to_string()
    }
}

pub(super) fn signed_tax_text(value: f64, text: &str) -> String {
    if value > 0.0 {
        format!("-{text}")
    } else if value < 0.0 {
        format!("+{}", text.trim_start_matches('-'))
    } else {
        text.to_string()
    }
}

/// 明細行に並べる表示文言。符号の付け方はダイアログと行でそろえる
pub(super) struct HomeDividendRowTexts {
    pub(super) code: String,
    pub(super) date: String,
    pub(super) full_date: String,
    pub(super) name: String,
    pub(super) account: String,
    pub(super) net_signed: String,
    pub(super) net_negative: bool,
    pub(super) net_positive: bool,
    pub(super) gross_signed: String,
    pub(super) gross_negative: bool,
    pub(super) gross_positive: bool,
    pub(super) tax_signed: String,
    pub(super) tax_negative: bool,
    pub(super) tax_positive: bool,
    pub(super) per_share: String,
    pub(super) shares: String,
}

pub(super) fn home_dividend_row_texts(row: &Dividend) -> HomeDividendRowTexts {
    let code = if row.security_code.is_empty() {
        labels::MISSING.to_string()
    } else {
        row.security_code.clone()
    };
    let net_value = asset_balance::dec_to_f64(&row.net_amount_received);
    let net = format::format_currency(net_value);
    let net_signed = if net_value > 0.0 {
        format!("+{net}")
    } else {
        net.clone()
    };
    let gross_value = asset_balance::dec_to_f64(&row.dividends_before_tax);
    let gross_signed = signed_gross_text(gross_value, &format::format_currency(gross_value));
    let tax_value = asset_balance::dec_to_f64(&row.taxes);
    let tax_signed = signed_tax_text(tax_value, &format::format_currency(tax_value));
    HomeDividendRowTexts {
        code,
        date: format::format_short_date(&row.settlement_date),
        full_date: row.settlement_date.clone(),
        name: row.security_name.clone(),
        account: String::from(row.account.clone()),
        net_signed,
        net_negative: net_value < 0.0,
        net_positive: net_value > 0.0,
        gross_signed,
        gross_negative: gross_value < 0.0,
        gross_positive: gross_value > 0.0,
        tax_signed,
        tax_negative: tax_value > 0.0,
        tax_positive: tax_value < 0.0,
        per_share: format::format_currency(asset_balance::dec_to_f64(&row.unit_price)),
        shares: format::format_number_value(asset_balance::dec_to_f64(&row.shares)),
    }
}

#[component]
fn HomeDividendRow(row: Dividend) -> impl IntoView {
    let texts = home_dividend_row_texts(&row);
    let code = texts.code.clone();
    let date = texts.date.clone();
    let full_date = texts.full_date.clone();
    let sheet_title = texts.name.clone();
    let row_name = texts.name.clone();
    let account = texts.account.clone();
    let account_for_meta = texts.account.clone();
    let net_for_row = texts.net_signed.clone();
    let net_negative = texts.net_negative;
    let net_positive = texts.net_positive;
    let code_for_meta = code.clone();
    let date_for_meta = date.clone();
    let sheet_open = RwSignal::new(false);
    let dialog_open = sheet_open;
    let dialog_name = sheet_title.clone();
    let dialog_code = if code == labels::MISSING {
        String::new()
    } else {
        code.clone()
    };
    let dialog_account = account.clone();
    let (dialog_left, dialog_right) = home_dividend_dialog_cells(
        &full_date,
        texts.shares.clone(),
        texts.per_share.clone(),
        (
            texts.gross_signed.clone(),
            texts.gross_negative,
            texts.gross_positive,
        ),
        (
            texts.tax_signed.clone(),
            texts.tax_negative,
            texts.tax_positive,
        ),
        (texts.net_signed.clone(), net_negative, net_positive),
    );
    view! {
        <li class="list-row-item" data-testid="home-dividend-row">
            <ListRowButtonWithoutSecondary
                testid="home-dividend-button"
                on_click=move |_| sheet_open.set(true)
                name=move || row_name.clone()
                primary=move || view! {
                    <Amount
                        kind=AmountKind::Delta
                        text=net_for_row.clone()
                        negative=net_negative
                        positive=net_positive
                    />
                }
                meta=move || view! {
                    {date_for_meta.clone()}
                    {" · "}
                    {code_for_meta.clone()}
                    {" · "}
                    <AccountBadge label=account_for_meta.clone() />
                }
            />
            <Show when=move || dialog_open.get()>
                <DetailDialog
                    name=dialog_name.clone()
                    code=dialog_code.clone()
                    account=dialog_account.clone()
                    left=dialog_left.clone()
                    right=dialog_right.clone()
                    on_close=move || dialog_open.set(false)
                    testid="home-dividend-dialog"
                />
            </Show>
        </li>
    }
}

#[component]
pub(super) fn RecentDividendsCard(data: HomeOverviewData) -> impl IntoView {
    let session = session::use_session();
    let card = move || {
        let generation = session.generation.get();
        store::current_summary(data.recent, generation)
    };
    view! {
        <Card variant=CardVariant::Panel class="overflow-clip" testid="home-recent-card">
            <CardHead title=labels::HOME_RECENT_DIVIDENDS href="/receipts" link_label=labels::HOME_SEE_ALL />
            {move || match card() {
                None => view! { <CardSkeleton /> }.into_any(),
                Some(None) => view! { <CardError message=labels::HOME_RECENT_FETCH_FAILED.to_string() /> }.into_any(),
                Some(Some(rows)) if rows.is_empty() => {
                    view! {
                        <div class="px-4 py-4">
                            <EmptyState title=labels::HOME_NO_RECENT />
                        </div>
                    }
                        .into_any()
                }
                Some(Some(rows)) => view! {
                    <ul class="asset-compact-list" data-testid="home-recent-list">
                        {rows
                            .iter()
                            .cloned()
                            .map(|row| view! { <HomeDividendRow row=row /> })
                            .collect_view()}
                    </ul>
                }
                    .into_any(),
            }}
        </Card>
    }
}
