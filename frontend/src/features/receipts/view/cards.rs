use crate::features::receipts::kind::{CardFields, CardRole};
use crate::features::receipts::{ReceiptCell, ReceiptsTab};
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::{AccountBadge, CodeBadge};
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::card::{Card, CardVariant};
use crate::ui::detail_dialog::{DetailCell, DetailDialog};
use crate::ui::disclosure::{ChevronIcon, DisclosureStyle, DisclosureToggle};
use crate::ui::format;
use crate::ui::labels;
use crate::ui::list_row::{GroupSummary, ListGroupHeader, ListRowButtonWithoutSecondary};
use crate::ui::receipts::{
    ReceiptGroupBreakdown, ReceiptGroupCount, ReceiptGroupDetails, ReceiptGroupHeadLabel,
    ReceiptGroupLabel, ReceiptGroupSummaryMeta, ReceiptGroupSummaryValue, ReceiptItemDetails,
    ReceiptItemHead, ReceiptItemMeta, ReceiptItemTitleName, ReceiptItemTitleRow,
    ReceiptMobileGroupList,
};
use crate::ui::security_link;
use crate::ui::security_link::SecurityCodeLink;
use crate::ui::stat_row::StatRow;
use leptos::prelude::*;
use std::collections::HashSet;

// 税引前(配当金・実現損益・ダイアログの税引前)と税引後はお金の出入りとして Delta。
pub(crate) fn is_profit_label(_tab: ReceiptsTab, label: &str) -> bool {
    matches!(
        label,
        labels::REALIZED_PROFIT_LOSS
            | labels::AFTER_TAX
            | labels::DIVIDEND_AMOUNT
            | labels::BEFORE_TAX
    )
}

pub(crate) fn is_tax_label(label: &str) -> bool {
    label == labels::TAX
}

pub(crate) fn tax_signed_text(raw: &str) -> String {
    if is_positive_text(raw) {
        format!("-{raw}")
    } else if is_negative_text(raw) {
        format!("+{}", raw.trim_start_matches('-'))
    } else {
        raw.to_string()
    }
}

fn stripped_amount_text(value: &str) -> String {
    let without_symbols: String = value
        .trim()
        .chars()
        .filter(|c| !matches!(c, '¥' | '￥' | '$' | '€' | '£' | ',') && !c.is_whitespace())
        .collect();
    // 表示単位(円/千円/万円)を除く。長い接尾辞から剥がす
    for suffix in [labels::THOUSAND_YEN, labels::TEN_THOUSAND_YEN, labels::YEN] {
        if let Some(stripped) = without_symbols.strip_suffix(suffix) {
            return stripped.to_string();
        }
    }
    without_symbols
}

fn valid_amount_digits(normalized: &str) -> bool {
    let digits = normalized
        .strip_prefix('-')
        .or_else(|| normalized.strip_prefix('+'))
        .unwrap_or(normalized);
    !digits.is_empty()
        && digits
            .split('.')
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

pub(crate) fn is_negative_text(value: &str) -> bool {
    let normalized = stripped_amount_text(value);
    if !valid_amount_digits(&normalized) {
        return false;
    }
    // 「-¥0」のような真のゼロは負としないが、小数部を持つ -0.0 は
    // 丸め前の符号を残した表示なので負とする
    normalized.parse::<f64>().is_ok_and(|n| {
        n < 0.0 || (n == 0.0 && normalized.starts_with('-') && normalized.contains('.'))
    })
}

pub(crate) fn is_negative_labeled_value(tab: ReceiptsTab, label: &str, value: &str) -> bool {
    if is_tax_label(label) {
        return is_positive_text(value);
    }
    is_profit_label(tab, label) && is_negative_text(value)
}

pub(crate) fn is_positive_text(value: &str) -> bool {
    let normalized = stripped_amount_text(value);
    if !valid_amount_digits(&normalized) {
        return false;
    }
    normalized.parse::<f64>().is_ok_and(|n| n > 0.0)
}

pub(crate) fn is_positive_labeled_value(tab: ReceiptsTab, label: &str, value: &str) -> bool {
    if is_tax_label(label) {
        return is_negative_text(value);
    }
    is_profit_label(tab, label) && is_positive_text(value)
}

/// 損益と税額の正値に符号を付ける。税額は出ていくお金として反転する。
/// 0・欠損・既に符号付き・対象外はそのまま返す。
pub(crate) fn signed_text_for_profit(tab: ReceiptsTab, label: &str, value: &str) -> String {
    if is_tax_label(label) {
        return tax_signed_text(value);
    }
    if is_profit_label(tab, label)
        && is_positive_text(value)
        && !value.trim_start().starts_with('+')
    {
        format!("+{value}")
    } else {
        value.to_string()
    }
}

pub(crate) fn amount_kind_for_label(tab: ReceiptsTab, label: &str, value: &str) -> AmountKind {
    if is_tax_label(label) {
        if is_positive_text(value) || is_negative_text(value) {
            return AmountKind::Delta;
        }
        return AmountKind::Plain;
    }
    if is_profit_label(tab, label) {
        return AmountKind::Delta;
    }
    AmountKind::Plain
}

#[derive(Clone)]
pub(crate) enum CardDetailValue {
    Text {
        text: String,
        negative: bool,
        positive: bool,
        is_profit: bool,
    },
    CopyName {
        display: String,
        copy: String,
    },
}

#[derive(Clone)]
pub(crate) struct CardDetail {
    pub(crate) label: String,
    pub(crate) value: CardDetailValue,
}

#[derive(Clone)]
pub(crate) struct CardRowData {
    pub(crate) name: CardDetailValue,
    pub(crate) code: Option<String>,
    pub(crate) date: String,
    pub(crate) account: String,
    pub(crate) details: Vec<CardDetail>,
}

pub(crate) fn card_detail_value(
    tab: ReceiptsTab,
    label: &str,
    index: usize,
    cells: &[ReceiptCell],
) -> CardDetailValue {
    if let Some(ReceiptCell::InstrumentName { name, code }) = cells.get(index) {
        let trimmed = name.trim();
        let display = if trimmed.is_empty() {
            "—".to_string()
        } else {
            trimmed.to_string()
        };
        let copy = code
            .as_deref()
            .map(crate::features::receipts::model::normalize_security_code)
            .filter(|code| !code.is_empty())
            .map_or_else(|| display.clone(), |code| format!("{display}({code})"));
        CardDetailValue::CopyName { display, copy }
    } else {
        let raw = cells
            .get(index)
            .map_or_default(|cell| cell.text().to_string());
        let text = signed_text_for_profit(tab, label, &raw);
        CardDetailValue::Text {
            negative: is_negative_labeled_value(tab, label, &raw),
            positive: is_positive_labeled_value(tab, label, &raw),
            is_profit: is_profit_label(tab, label),
            text,
        }
    }
}

// 見出し(銘柄コード・銘柄名・日付・口座)を除き、
// 役割のない列を表の列順で全部出す
pub(crate) fn card_row_data(
    tab: ReceiptsTab,
    cells: &[ReceiptCell],
    order: &[usize],
    fields: CardFields,
) -> CardRowData {
    let columns = tab.columns();
    let text = |index: usize| {
        cells
            .get(index)
            .map_or_default(|cell| cell.text().to_string())
    };
    CardRowData {
        name: card_detail_value(tab, columns[fields.name].header, fields.name, cells),
        // コード列を持つタブでは空コードも欠損値として見出しに出す
        code: fields
            .code
            .map(|index| crate::features::receipts::model::normalize_security_code(&text(index))),
        date: text(fields.date),
        account: text(fields.account),
        details: order
            .iter()
            .filter(|&&i| columns[i].card_role == CardRole::None)
            .map(|&i| CardDetail {
                label: columns[i].header.to_string(),
                value: card_detail_value(tab, columns[i].header, i, cells),
            })
            .collect(),
    }
}

pub(crate) fn card_detail_view(value: &CardDetailValue) -> (AnyView, Option<String>) {
    match value {
        CardDetailValue::Text {
            text,
            negative,
            positive,
            is_profit,
        } => (
            view! {
                <Amount
                    kind=if *is_profit {
                        AmountKind::Delta
                    } else {
                        AmountKind::Plain
                    }
                    text=text.clone()
                    negative=*negative
                    positive=*positive
                />
            }
            .into_any(),
            Some(text.clone()),
        ),
        CardDetailValue::CopyName { display, copy } => {
            let copy_text = copy.clone();
            let aria_label = labels::copy_aria_label(copy);
            let display = display.clone();
            (
                view! {
                    <Button
                        variant=ButtonVariant::CopyName
                        class="group min-h-11 min-w-11 items-center"
                        aria_label=aria_label
                        on_click=move |_| security_link::copy_to_clipboard(copy_text.clone())
                    >
                        <span class="min-w-0 break-words">{move || display.clone()}</span>
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="12"
                            height="12"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            aria-hidden="true"
                            class="copy-icon"
                        >
                            <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                        </svg>
                    </Button>
                }
                .into_any(),
                None,
            )
        }
    }
}

fn compact_value(details: &[CardDetail], label: &str) -> (String, bool, bool) {
    details
        .iter()
        .find(|detail| detail.label == label)
        .map_or_default(|detail| match &detail.value {
            CardDetailValue::Text {
                text,
                negative,
                positive,
                ..
            } => (text.clone(), *negative, *positive),
            CardDetailValue::CopyName { display, .. } => (display.clone(), false, false),
        })
}

pub(crate) fn before_tax_subtext(
    tab: ReceiptsTab,
    details: &[CardDetail],
) -> Option<(String, bool, bool)> {
    if tab == ReceiptsTab::Dividend {
        return None;
    }
    let (value, negative, positive) = compact_value(details, labels::REALIZED_PROFIT_LOSS);
    (!value.is_empty()).then_some((value, negative, positive))
}

fn detail_text(details: &[CardDetail], label: &str) -> (String, bool, bool, bool) {
    details
        .iter()
        .find(|detail| detail.label == label)
        .map_or_default(|detail| match &detail.value {
            CardDetailValue::Text {
                text,
                negative,
                positive,
                is_profit,
            } => (text.clone(), *negative, *positive, *is_profit),
            CardDetailValue::CopyName { display, .. } => (display.clone(), false, false, false),
        })
}

pub(crate) fn receipt_detail_cells(
    tab: ReceiptsTab,
    card: &CardRowData,
) -> (Vec<DetailCell>, Vec<DetailCell>) {
    let date_text = format::format_iso_date(&card.date.replace('/', "-"));
    let date_label = match tab {
        ReceiptsTab::Dividend => labels::PAYMENT_DATE,
        _ => labels::TRADE_DATE,
    };
    if tab == ReceiptsTab::Dividend {
        let (qty, _, _, _) = detail_text(&card.details, labels::QUANTITY);
        let (per_share, _, _, _) = detail_text(&card.details, labels::PER_SHARE_DIVIDEND);
        let (gross, gross_neg, gross_pos, _) = detail_text(&card.details, labels::DIVIDEND_AMOUNT);
        let (tax, tax_neg, tax_pos, _) = detail_text(&card.details, labels::TAX);
        let (net, net_neg, net_pos, _) = detail_text(&card.details, labels::AFTER_TAX);
        let left = vec![
            DetailCell::plain(date_label.to_string(), date_text),
            DetailCell::plain(labels::QUANTITY.to_string(), qty),
            DetailCell::plain(labels::PER_SHARE_DIVIDEND.to_string(), per_share),
        ];
        let right = vec![
            DetailCell {
                label: labels::BEFORE_TAX.to_string(),
                value: gross,
                sub: None,
                sub_rate: None,
                sub_rate_negative: false,
                sub_rate_positive: false,
                negative: gross_neg,
                positive: gross_pos,
                bold: false,
                kind: AmountKind::Delta,
            },
            DetailCell {
                label: labels::TAX.to_string(),
                value: tax,
                sub: None,
                sub_rate: None,
                sub_rate_negative: false,
                sub_rate_positive: false,
                negative: tax_neg,
                positive: tax_pos,
                bold: false,
                kind: if tax_neg || tax_pos {
                    AmountKind::Delta
                } else {
                    AmountKind::Plain
                },
            },
            DetailCell::strong(
                labels::AFTER_TAX.to_string(),
                net,
                net_neg,
                net_pos,
                AmountKind::Delta,
            ),
        ];
        (left, right)
    } else {
        let amount_label = match tab {
            ReceiptsTab::DomesticStock => labels::SELLING_AMOUNT,
            _ => labels::REDEMPTION_AMOUNT,
        };
        let (qty, _, _, _) = detail_text(&card.details, labels::QUANTITY);
        let (amount, _, _, _) = detail_text(&card.details, amount_label);
        let (cost, _, _, _) = detail_text(&card.details, labels::ACQUISITION_PRICE);
        let (pnl, pnl_neg, pnl_pos, _) = detail_text(&card.details, labels::REALIZED_PROFIT_LOSS);
        let (tax, tax_neg, tax_pos, _) = detail_text(&card.details, labels::TAX);
        let (net, net_neg, net_pos, _) = detail_text(&card.details, labels::AFTER_TAX);
        let left = vec![
            DetailCell::plain(date_label.to_string(), date_text),
            DetailCell::plain(labels::QUANTITY.to_string(), qty),
            DetailCell::plain(amount_label.to_string(), amount),
            DetailCell::plain(labels::ACQUISITION_PRICE.to_string(), cost),
        ];
        let right = vec![
            DetailCell {
                label: labels::BEFORE_TAX.to_string(),
                value: pnl,
                sub: None,
                sub_rate: None,
                sub_rate_negative: false,
                sub_rate_positive: false,
                negative: pnl_neg,
                positive: pnl_pos,
                bold: false,
                kind: AmountKind::Delta,
            },
            DetailCell {
                label: labels::TAX.to_string(),
                value: tax,
                sub: None,
                sub_rate: None,
                sub_rate_negative: false,
                sub_rate_positive: false,
                negative: tax_neg,
                positive: tax_pos,
                bold: false,
                kind: if tax_neg || tax_pos {
                    AmountKind::Delta
                } else {
                    AmountKind::Plain
                },
            },
            DetailCell::strong(
                labels::AFTER_TAX.to_string(),
                net,
                net_neg,
                net_pos,
                AmountKind::Delta,
            ),
        ];
        (left, right)
    }
}

#[component]
fn CompactReceiptRow(tab: ReceiptsTab, card: CardRowData) -> impl IntoView {
    let CardRowData {
        name,
        code,
        date,
        account,
        details,
        ..
    } = card;
    let (hero_value, hero_negative, hero_positive) = compact_value(&details, labels::AFTER_TAX);
    let before_tax_sub = before_tax_subtext(tab, &details);
    let hero_is_profit = is_profit_label(tab, labels::AFTER_TAX);
    let hero_kind = if hero_is_profit {
        AmountKind::Delta
    } else {
        AmountKind::Plain
    };
    let name_text = match &name {
        CardDetailValue::Text { text, .. } => text.clone(),
        CardDetailValue::CopyName { display, .. } => display.clone(),
    };
    let row_name = name_text.clone();
    let sheet_title = name_text.clone();
    let short_date = format::format_short_date(&date.replace('/', "-"));
    let sheet_open = RwSignal::new(false);
    // 配当タブの主役は税引後の受取額だが行だけでは税区分が分からないため添える。
    // 国内株式・投資信託は補助に税引前が出るため付けない
    let hero_value_for_row = if tab == ReceiptsTab::Dividend && !hero_value.is_empty() {
        format!("{}({})", hero_value, labels::AFTER_TAX)
    } else {
        hero_value.clone()
    };
    let account_for_meta = account.clone();
    let code_for_meta = code.clone();
    let short_date_for_meta = short_date.clone();
    let sheet_card = CardRowData {
        name,
        code,
        date,
        account: account_for_meta.clone(),
        details,
    };

    let dialog_open = sheet_open;
    let dialog_card = sheet_card.clone();
    let (dialog_left, dialog_right) = receipt_detail_cells(tab, &dialog_card);
    let dialog_name = sheet_title.clone();
    let dialog_code = code_for_meta.clone();
    let dialog_account = account_for_meta.clone();

    view! {
        <li class="list-row-item" data-testid="receipt-card">
            <ListRowButtonWithoutSecondary
                testid="receipt-row-button"
                on_click=move |_| sheet_open.set(true)
                name=move || row_name.clone()
                primary=move || view! {
                    <span class="receipt-compact-value">
                        <Amount
                            kind=hero_kind
                            text=hero_value_for_row.clone()
                            negative=hero_negative
                            positive=hero_positive
                        />
                        {before_tax_sub.clone().map(|(text, negative, positive)| view! {
                            <span class="receipt-before-tax-sub">
                                {labels::BEFORE_TAX}" "
                                <Amount
                                    kind=AmountKind::Delta
                                    text=text
                                    negative=negative
                                    positive=positive
                                />
                            </span>
                        })}
                    </span>
                }
                meta=move || view! {
                    {short_date_for_meta.clone()}
                    {code_for_meta
                        .clone()
                        .and_then(|code| {
                            let trimmed = code.trim().to_string();
                            (!trimmed.is_empty()).then_some(trimmed)
                        })
                        .map(|trimmed| {
                            view! { {" · "} <span class="receipt-compact-code">{trimmed}</span> }
                        })}
                    {" · "}
                    <AccountBadge label=account_for_meta.clone() />
                }
            />
            <Show when=move || dialog_open.get()>
                <DetailDialog
                    name=dialog_name.clone()
                    code=dialog_code.clone().unwrap_or_default()
                    account=dialog_account.clone()
                    left=dialog_left.clone()
                    right=dialog_right.clone()
                    on_close=move || dialog_open.set(false)
                    testid="receipt-detail-dialog"
                />
            </Show>
        </li>
    }
}

/// 月・日見出しの右に出す合計。行の主役・帯と同じ税引後の合計を使う
pub(crate) fn group_headline_value<'a>(summary: &'a [(&'static str, String)]) -> &'a str {
    summary
        .iter()
        .find(|(label, _)| *label == labels::AFTER_TAX)
        .map(|(_, value)| value.as_str())
        .or_else(|| summary.last().map(|(_, value)| value.as_str()))
        .unwrap_or("")
}

/// 月見出しの合計。集計が無い単発の日はその行の税引後を使う
pub(crate) fn single_row_headline(cards: &[CardRowData]) -> Vec<(&'static str, String)> {
    let [card] = cards else {
        return Vec::new();
    };
    let (text, _, _) = compact_value(&card.details, labels::AFTER_TAX);
    if text.is_empty() {
        Vec::new()
    } else {
        vec![(labels::AFTER_TAX, text)]
    }
}

/// 月見出しの右に出す小計の表示。金額は符号付きにし、受取額・損益は
/// Delta の種別で出す(行の主役・帯と同じ税引後の合計を使う)
pub(crate) fn group_headline_summary(
    tab: ReceiptsTab,
    summary: &[(&'static str, String)],
) -> Option<GroupSummary> {
    let raw = group_headline_value(summary);
    if raw.is_empty() {
        return None;
    }
    let text = if is_positive_text(raw) && !raw.trim_start().starts_with('+') {
        format!("+{raw}")
    } else {
        raw.to_string()
    };
    let label = match tab {
        ReceiptsTab::Dividend => labels::RECEIPT_SHORT,
        _ => labels::PROFIT_SHORT,
    };
    Some(GroupSummary {
        label: label.to_string(),
        value: text,
        kind: if is_profit_label(tab, labels::AFTER_TAX) {
            AmountKind::Delta
        } else {
            AmountKind::Plain
        },
        negative: is_negative_labeled_value(tab, labels::AFTER_TAX, raw),
        positive: is_positive_labeled_value(tab, labels::AFTER_TAX, raw),
    })
}

#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned by the view"
)]
pub(crate) fn CompactReceiptGroup(
    tab: ReceiptsTab,
    label: String,
    count: usize,
    summary: Vec<(&'static str, String)>,
    cards: Vec<CardRowData>,
) -> impl IntoView {
    // 単発の日は集計が無いのでその行の税引後を見出しに出す
    let summary: Vec<(&'static str, String)> = if summary.is_empty() {
        single_row_headline(&cards)
    } else {
        summary
    };
    let headline = group_headline_summary(tab, &summary);
    view! {
        <section data-testid="receipt-card-group">
            <ListGroupHeader label=labels::group_count(&label, count) summary=headline />
            <ul class="receipt-compact-list">
                {cards
                    .into_iter()
                    .map(|card| view! {
                        <CompactReceiptRow tab=tab card=card />
                    })
                    .collect_view()}
            </ul>
        </section>
    }
}

#[component]
fn ReceiptItemCard(card: CardRowData) -> impl IntoView {
    let CardRowData {
        name,
        code,
        date,
        account,
        details,
        ..
    } = card;
    let (name_view, name_title) = card_detail_view(&name);
    view! {
        <Card variant=CardVariant::Item class="border-l-4 border-l-accent" testid="receipt-card">
            <ReceiptItemHead>
                <ReceiptItemTitleRow>
                    {code.map(|code| {
                        view! {
                            <CodeBadge flush=true testid="receipt-card-code">
                                <SecurityCodeLink
                                    value=code
                                    class="min-h-11 min-w-11 items-center justify-center no-underline hover:underline".to_string()
                                />
                            </CodeBadge>
                        }
                    })}
                    <ReceiptItemTitleName title=name_title>
                        {name_view}
                    </ReceiptItemTitleName>
                </ReceiptItemTitleRow>
                <ReceiptItemMeta>
                    <span class="shrink-0">{date}</span>
                    {(!account.is_empty()).then(|| view! { <AccountBadge label=account /> })}
                </ReceiptItemMeta>
            </ReceiptItemHead>
            <ReceiptItemDetails>
                {details
                    .iter()
                    .map(|detail| {
                        let (content, _) = card_detail_view(&detail.value);
                        view! {
                            <StatRow label=detail.label.clone() numeric=true>
                                {content}
                            </StatRow>
                        }
                    })
                    .collect_view()}
            </ReceiptItemDetails>
        </Card>
    }
}

#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned by the view"
)]
pub(crate) fn MobileCardGroup(
    tab: ReceiptsTab,
    label: String,
    count: usize,
    summary: Vec<(&'static str, String)>,
    cards: Vec<CardRowData>,
    id_prefix: String,
    expanded_key: String,
    expanded_ids: RwSignal<HashSet<String>>,
) -> impl IntoView {
    let expanded_id = expanded_key;
    let toggle_id = expanded_id.clone();
    let expanded = Memo::new(move |_| expanded_ids.with(|set| set.contains(&expanded_id)));
    let button_id = format!("{id_prefix}-group-button");
    let details_id = format!("{id_prefix}-group-details");
    let card_list = view! {
        <ReceiptMobileGroupList>
            {cards
                .into_iter()
                .map(|card| view! { <ReceiptItemCard card=card /> })
                .collect_view()}
        </ReceiptMobileGroupList>
    };
    if summary.is_empty() {
        return view! {
            <section data-testid="receipt-card-group">
                <Card variant=CardVariant::GroupLabel>
                    <ReceiptGroupLabel>{label}</ReceiptGroupLabel>
                    <ReceiptGroupCount text=labels::count(count) />
                </Card>
                {card_list}
            </section>
        }
        .into_any();
    }
    let (primary_label, primary_raw) = summary.last().cloned().unwrap_or_default();
    let primary_negative = is_negative_labeled_value(tab, primary_label, &primary_raw);
    let primary_positive = is_positive_labeled_value(tab, primary_label, &primary_raw);
    let primary_value = signed_text_for_profit(tab, primary_label, &primary_raw);
    let primary_kind = amount_kind_for_label(tab, primary_label, &primary_raw);
    let aria_label = format!(
        "{} {} {} {}",
        label,
        labels::count(count),
        primary_label,
        primary_value
    );
    view! {
        <section data-testid="receipt-card-group">
            <Card variant=CardVariant::Group>
                <DisclosureToggle
                    id=button_id.clone()
                    style=DisclosureStyle::GroupCard
                    expanded=Signal::derive(move || expanded.get())
                    controls=details_id.clone()
                    aria_label=aria_label
                    on_toggle=move || {
                        expanded_ids.update(|set| {
                            if !set.remove(&toggle_id) {
                                set.insert(toggle_id.clone());
                            }
                        });
                    }
                >
                    <ReceiptGroupHeadLabel>
                        {label}
                        <ReceiptGroupCount text=labels::count(count) />
                    </ReceiptGroupHeadLabel>
                    <ReceiptGroupSummaryMeta>
                        <span class="text-cell text-text-muted">{primary_label}</span>
                        <ReceiptGroupSummaryValue
                            kind=primary_kind
                            text=primary_value.clone()
                            negative=primary_negative
                            positive=primary_positive
                        />
                        <ChevronIcon
                            expanded=Signal::derive(move || expanded.get())
                            class="h-4 w-4 shrink-0 self-center text-text-muted"
                        />
                    </ReceiptGroupSummaryMeta>
                </DisclosureToggle>
                <ReceiptGroupDetails
                    id=details_id
                    labelledby=button_id
                    expanded=expanded
                >
                    {move || {
                        let summary = summary.clone();
                        expanded
                            .get()
                            .then(|| {
                                view! {
                                    <ReceiptGroupBreakdown>
                                        {summary
                                            .iter()
                                            .map(|(label, value)| {
                                                let negative =
                                                    is_negative_labeled_value(tab, label, value);
                                                let positive =
                                                    is_positive_labeled_value(tab, label, value);
                                                let text =
                                                    signed_text_for_profit(tab, label, value);
                                                let kind =
                                                    amount_kind_for_label(tab, label, value);
                                                view! {
                                                    <StatRow
                                                        label=label.to_string()
                                                        numeric=true
                                                    >
                                                        <Amount
                                                            kind=kind
                                                            text=text
                                                            negative=negative
                                                            positive=positive
                                                        />
                                                    </StatRow>
                                                }
                                            })
                                            .collect_view()}
                                    </ReceiptGroupBreakdown>
                                }
                            })
                    }}
                </ReceiptGroupDetails>
            </Card>
            {card_list}
        </section>
    }
    .into_any()
}
