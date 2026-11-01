use crate::features::receipts::kind::{CardFields, CardRole};
use crate::features::receipts::{ReceiptCell, ReceiptsTab};
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::{AccountBadge, CodeBadge};
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::card::{Card, CardVariant};
use crate::ui::disclosure::{ChevronIcon, DisclosureStyle, DisclosureToggle};
use crate::ui::labels::{self, AFTER_TAX, REALIZED_PROFIT_LOSS};
use crate::ui::ledger_row::LedgerRow;
use crate::ui::security_link::{SecurityCodeLink, copy_to_clipboard};
use crate::ui::stat_row::StatRow;
use leptos::prelude::*;
use std::collections::HashSet;
use wasm_bindgen::JsCast;

pub(crate) fn is_profit_label(tab: ReceiptsTab, label: &str) -> bool {
    tab != ReceiptsTab::Dividend && matches!(label, REALIZED_PROFIT_LOSS | AFTER_TAX)
}

pub(crate) fn compact_row_label(tab: ReceiptsTab) -> &'static str {
    match tab {
        ReceiptsTab::Dividend => AFTER_TAX,
        ReceiptsTab::DomesticStock | ReceiptsTab::MutualFund => REALIZED_PROFIT_LOSS,
    }
}

/// 行の日付は年を省いた "MM/DD" にする(年はグループ見出しにある)。
/// 表のセルは "YYYY/MM/DD" なので末尾2節だけ残し、読めない値はそのまま返す
pub(crate) fn compact_date_text(date: &str) -> String {
    let mut parts = date.split('/');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(year), Some(month), Some(day), None) if year.len() == 4 => {
            format!("{month}/{day}")
        }
        _ => date.to_string(),
    }
}

fn stripped_amount_text(value: &str) -> String {
    let without_symbols: String = value
        .trim()
        .chars()
        .filter(|c| !matches!(c, '¥' | '￥' | '$' | '€' | '£' | ',') && !c.is_whitespace())
        .collect();
    // 表示単位(円/千円/万円)を除く。長い接尾辞から剥がす
    for suffix in ["千円", "万円", "円"] {
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
    match normalized.parse::<f64>() {
        // 「-¥0」のような真のゼロは負としないが、小数部を持つ -0.0 は
        // 丸め前の符号を残した表示なので負とする
        Ok(n) => n < 0.0 || (n == 0.0 && normalized.starts_with('-') && normalized.contains('.')),
        Err(_) => false,
    }
}

pub(crate) fn is_negative_labeled_value(tab: ReceiptsTab, label: &str, value: &str) -> bool {
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
    is_profit_label(tab, label) && is_positive_text(value)
}

/// 損益の正値に「+」を付ける。色だけに頼らず符号で正負を示す。
/// 0・欠損・既に符号付き・損益でない項目はそのまま返す。
pub(crate) fn signed_text_for_profit(tab: ReceiptsTab, label: &str, value: &str) -> String {
    if is_profit_label(tab, label)
        && is_positive_text(value)
        && !value.trim_start().starts_with('+')
    {
        format!("+{value}")
    } else {
        value.to_string()
    }
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
    /// 開閉状態を照合する行キー(表示丸め前の値)
    pub(crate) key: String,
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
    match cells.get(index) {
        Some(ReceiptCell::InstrumentName { name, code }) => {
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
        }
        _ => {
            let raw = cells
                .get(index)
                .map(|cell| cell.text().to_string())
                .unwrap_or_default();
            let text = signed_text_for_profit(tab, label, &raw);
            CardDetailValue::Text {
                negative: is_negative_labeled_value(tab, label, &raw),
                positive: is_positive_labeled_value(tab, label, &raw),
                is_profit: is_profit_label(tab, label),
                text,
            }
        }
    }
}

// 見出し(銘柄コード・銘柄名・日付・口座)を除き、
// 役割のない列を表の列順で全部出す
pub(crate) fn card_row_data(
    tab: ReceiptsTab,
    key: String,
    cells: &[ReceiptCell],
    order: &[usize],
    fields: CardFields,
) -> CardRowData {
    let columns = tab.columns();
    let text = |index: usize| {
        cells
            .get(index)
            .map(|cell| cell.text().to_string())
            .unwrap_or_default()
    };
    CardRowData {
        key,
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
                    class="font-semibold"
                />
            }
            .into_any(),
            Some(text.clone()),
        ),
        CardDetailValue::CopyName { display, copy } => {
            let copy_text = copy.clone();
            let aria_label = format!("{copy} をコピー");
            let display = display.clone();
            (
                view! {
                    <Button
                        variant=ButtonVariant::CopyName
                        class="group min-h-11 min-w-11 items-center"
                        aria_label=aria_label
                        on_click=move |_| copy_to_clipboard(copy_text.clone())
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
        .map(|detail| match &detail.value {
            CardDetailValue::Text {
                text,
                negative,
                positive,
                ..
            } => (text.clone(), *negative, *positive),
            CardDetailValue::CopyName { display, .. } => (display.clone(), false, false),
        })
        .unwrap_or_default()
}

fn compact_name(value: CardDetailValue) -> AnyView {
    match value {
        CardDetailValue::CopyName { display, copy } => {
            let aria_label = format!("{copy} をコピー");
            view! {
                <Button
                    variant=ButtonVariant::CopyName
                    class="receipt-compact-name"
                    aria_label=aria_label
                    on_click=move |_| copy_to_clipboard(copy.clone())
                >
                    <span>{display}</span>
                </Button>
            }
            .into_any()
        }
        CardDetailValue::Text { text, .. } => {
            view! { <span class="receipt-compact-name-static">{text}</span> }.into_any()
        }
    }
}

#[component]
fn CompactReceiptRow(
    tab: ReceiptsTab,
    card: CardRowData,
    detail_id: String,
    expanded_ids: RwSignal<HashSet<String>>,
) -> impl IntoView {
    let CardRowData {
        key,
        name,
        code,
        date,
        account,
        details,
    } = card;
    let row_label = compact_row_label(tab);
    let (row_value, row_negative, row_positive) = compact_value(&details, row_label);
    let row_kind = if is_profit_label(tab, row_label) {
        AmountKind::Delta
    } else {
        AmountKind::Plain
    };
    let rest: Vec<CardDetail> = details
        .iter()
        .filter(|detail| detail.label.as_str() != row_label)
        .cloned()
        .collect();
    let name_text = match &name {
        CardDetailValue::Text { text, .. } => text.clone(),
        CardDetailValue::CopyName { display, .. } => display.clone(),
    };
    let name = compact_name(name);
    let code_text = code.filter(|code| !code.is_empty()).unwrap_or_default();

    let toggle_key = format!("r:{key}");
    let key_for_expanded = toggle_key.clone();
    let expanded = Memo::new(move |_| expanded_ids.with(|set| set.contains(&key_for_expanded)));
    let toggle_ids = expanded_ids;
    let toggle = move |key: &str| {
        toggle_ids.update(|set| {
            if !set.remove(key) {
                set.insert(key.to_string());
            }
        });
    };
    let key_for_toggle = toggle_key.clone();
    let on_row_click = move |ev: leptos::ev::MouseEvent| {
        let interactive = ev
            .target()
            .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            .and_then(|element| element.closest("a,button").ok().flatten())
            .is_some();
        if !interactive {
            toggle(&key_for_toggle);
        }
    };
    let key_for_button = toggle_key;
    let account_label = account.clone();

    view! {
        <li
            class="receipt-compact-row"
            data-testid="receipt-card"
            on:click=on_row_click
        >
            <div class="receipt-compact-line">
                <LedgerRow
                    date=compact_date_text(&date)
                    name=name
                    code=(!code_text.is_empty())
                        .then(|| view! {
                            <SecurityCodeLink value=code_text.clone() />
                        })
                    amount=view! {
                        <Amount
                            kind=row_kind
                            text=row_value
                            negative=row_negative
                            positive=row_positive
                        />
                    }
                />
                {(!rest.is_empty()).then(|| view! {
                    <DisclosureToggle
                        style=DisclosureStyle::RowToggle
                        expanded=Signal::derive(move || expanded.get())
                        controls=detail_id.clone()
                        aria_label=format!("{name_text}の詳細")
                        on_toggle=move || toggle(&key_for_button)
                    >
                        <ChevronIcon
                            expanded=Signal::derive(move || expanded.get())
                            class="h-3.5 w-3.5"
                        />
                    </DisclosureToggle>
                })}
            </div>
            <div
                id=detail_id
                class="receipt-compact-details"
                hidden=move || !expanded.get()
            >
                <dl class="space-y-1">
                    <StatRow label="口座".to_string() numeric=true>
                        <AccountBadge label=account_label.clone() />
                    </StatRow>
                    {rest
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
                </dl>
            </div>
        </li>
    }
}

#[component]
pub(crate) fn CompactReceiptGroup(
    tab: ReceiptsTab,
    label: String,
    count: usize,
    summary: Vec<(&'static str, String)>,
    cards: Vec<CardRowData>,
    id_prefix: String,
    expanded_ids: RwSignal<HashSet<String>>,
) -> impl IntoView {
    let (summary_label, summary_value) = summary.last().cloned().unwrap_or_default();
    let summary_negative = is_negative_labeled_value(tab, summary_label, &summary_value);
    let summary_positive = is_positive_labeled_value(tab, summary_label, &summary_value);
    let summary_text = signed_text_for_profit(tab, summary_label, &summary_value);
    let summary_kind = if is_profit_label(tab, summary_label) {
        AmountKind::Delta
    } else {
        AmountKind::Plain
    };
    view! {
        <section data-testid="receipt-card-group">
            <div class="receipt-compact-group">
                <span class="font-bold">{labels::group_count(&label, count)}</span>
                {(!summary_value.is_empty()).then(|| view! {
                    <span class="whitespace-nowrap">
                        {summary_label}
                        " "
                        <Amount
                            kind=summary_kind
                            text=summary_text
                            negative=summary_negative
                            positive=summary_positive
                        />
                    </span>
                })}
            </div>
            <ul class="receipt-compact-list">
                {cards
                    .into_iter()
                    .enumerate()
                    .map(|(index, card)| view! {
                        <CompactReceiptRow
                            tab=tab
                            card=card
                            detail_id=format!("{id_prefix}-detail-{index}")
                            expanded_ids=expanded_ids
                        />
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
            <div class="flex min-w-0 flex-col gap-1.5 px-3 py-3">
                <div class="flex min-w-0 items-center gap-2">
                    {code.map(|code| {
                        view! {
                            <CodeBadge flush=true testid="receipt-card-code">
                                <SecurityCodeLink
                                    value=code
                                    class="min-h-11 min-w-11 items-center justify-center font-semibold no-underline hover:underline".to_string()
                                />
                            </CodeBadge>
                        }
                    })}
                    <div
                        class="min-w-0 break-words text-base font-semibold text-ink"
                        title=name_title
                    >
                        {name_view}
                    </div>
                </div>
                <div class="flex items-center gap-2 text-xs text-text-muted">
                    <span class="shrink-0">{date}</span>
                    {(!account.is_empty()).then(|| view! { <AccountBadge label=account /> })}
                </div>
            </div>
            <dl class="space-y-1.5 border-t border-ink/10 px-3 py-2.5">
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
            </dl>
        </Card>
    }
}

#[component]
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
        <div class="mt-2 space-y-2">
            {cards
                .into_iter()
                .map(|card| view! { <ReceiptItemCard card=card /> })
                .collect_view()}
        </div>
    };
    if summary.is_empty() {
        return view! {
            <section data-testid="receipt-card-group">
                <Card variant=CardVariant::GroupLabel>
                    <span class="text-sm font-semibold text-text-soft">{label}</span>
                    <span class="ml-2 text-xs font-medium text-text-muted">
                        {labels::count(count)}
                    </span>
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
    let primary_kind = if is_profit_label(tab, primary_label) {
        AmountKind::Delta
    } else {
        AmountKind::Plain
    };
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
                        })
                    }
                >
                    <span class="min-w-0 flex-1 truncate text-sm font-semibold text-text-soft">
                        {label}
                        <span class="ml-2 text-xs font-medium text-text-muted">
                            {labels::count(count)}
                        </span>
                    </span>
                    <span
                        class="flex shrink-0 items-baseline gap-1 whitespace-nowrap"
                        aria-hidden="true"
                    >
                        <span class="text-xs text-text-muted">{primary_label}</span>
                        <Amount
                            kind=primary_kind
                            text=primary_value.clone()
                            negative=primary_negative
                            positive=primary_positive
                            class="text-sm font-semibold"
                        />
                        <ChevronIcon
                            expanded=Signal::derive(move || expanded.get())
                            class="h-4 w-4 shrink-0 self-center text-text-muted"
                        />
                    </span>
                </DisclosureToggle>
                <div
                    id=details_id
                    role="region"
                    aria-labelledby=button_id
                    hidden=move || !expanded.get()
                    class="border-t border-border-subtle bg-surface px-3 py-1"
                >
                    {move || {
                        expanded
                            .get()
                            .then(|| {
                                view! {
                                    <dl class="space-y-1.5">
                                        {summary
                                            .iter()
                                            .map(|(label, value)| {
                                                let negative =
                                                    is_negative_labeled_value(tab, label, value);
                                                let positive =
                                                    is_positive_labeled_value(tab, label, value);
                                                let text =
                                                    signed_text_for_profit(tab, label, value);
                                                let kind = if is_profit_label(tab, label) {
                                                    AmountKind::Delta
                                                } else {
                                                    AmountKind::Plain
                                                };
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
                                    </dl>
                                }
                            })
                    }}
                </div>
            </Card>
            {card_list}
        </section>
    }
    .into_any()
}
