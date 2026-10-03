use crate::features::receipts::kind::CardFields;
use crate::features::receipts::{ReceiptCell, ReceiptsTab};
use crate::ui::amount::Amount;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::card::{Card, CardVariant};
use crate::ui::disclosure::{ChevronIcon, DisclosureStyle, DisclosureToggle};
use crate::ui::security_link::copy_to_clipboard;
use leptos::prelude::*;
use std::collections::HashSet;

// 配当の税引後(受取額)は損益ではないので色を付けない
pub(crate) fn is_profit_label(tab: ReceiptsTab, label: &str) -> bool {
    tab != ReceiptsTab::Dividend && matches!(label, "実現損益" | "税引後")
}

pub(crate) fn is_negative_text(value: &str) -> bool {
    let normalized: String = value
        .trim()
        .chars()
        .filter(|c| !matches!(c, '¥' | '￥' | '$' | '€' | '£' | ',') && !c.is_whitespace())
        .collect();
    let digits = normalized
        .strip_prefix('-')
        .or_else(|| normalized.strip_prefix('+'))
        .unwrap_or(normalized.as_str());
    let valid = !digits.is_empty()
        && digits
            .split('.')
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()));
    valid && normalized.parse::<f64>().is_ok_and(|n| n < 0.0)
}

pub(crate) fn is_negative_labeled_value(tab: ReceiptsTab, label: &str, value: &str) -> bool {
    is_profit_label(tab, label) && is_negative_text(value)
}

// グループ見出しに年月があるため、カード先頭の日付は年を落として MM/DD にする
pub(crate) fn short_date(formatted: &str) -> &str {
    match formatted.split_once('/') {
        Some((year, rest)) if year.len() == 4 && year.bytes().all(|b| b.is_ascii_digit()) => rest,
        _ => formatted,
    }
}

pub(crate) enum CardDetailValue {
    Text { text: String, negative: bool },
    SecurityCode(String),
    CopyName { display: String, copy: String },
}

pub(crate) struct CardDetail {
    pub(crate) label: String,
    pub(crate) value: CardDetailValue,
}

pub(crate) struct CardRowData {
    pub(crate) name: CardDetailValue,
    pub(crate) date: String,
    pub(crate) account: String,
    pub(crate) details: Vec<CardDetail>,
}

pub(crate) fn is_security_code(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'.')
}

pub(crate) fn card_detail_value(
    tab: ReceiptsTab,
    label: &str,
    index: usize,
    cells: &[ReceiptCell],
) -> CardDetailValue {
    match cells.get(index) {
        Some(ReceiptCell::SecurityCode(raw)) => CardDetailValue::SecurityCode(raw.clone()),
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
            let text = cells
                .get(index)
                .map(|cell| cell.text().to_string())
                .unwrap_or_default();
            CardDetailValue::Text {
                negative: is_negative_labeled_value(tab, label, &text),
                text,
            }
        }
    }
}

// 見出しの銘柄名・日付・口座とは重複させず、残りの列を表の列順で全部出す
pub(crate) fn card_row_data(
    tab: ReceiptsTab,
    cells: &[ReceiptCell],
    headers: &[&'static str],
    order: &[usize],
    fields: CardFields,
    // 見出しが年月でない(銘柄名や口座で絞った)グループでは年を落とすと日付が分からなくなる
    full_date: bool,
) -> CardRowData {
    let text = |index: usize| {
        cells
            .get(index)
            .map(|cell| cell.text().to_string())
            .unwrap_or_default()
    };
    let header_fields = [fields.name, fields.date, fields.account];
    CardRowData {
        name: card_detail_value(tab, headers[fields.name], fields.name, cells),
        date: {
            let date = text(fields.date);
            if full_date {
                date
            } else {
                short_date(&date).to_string()
            }
        },
        account: text(fields.account),
        details: order
            .iter()
            .filter(|index| !header_fields.contains(index))
            .map(|&i| CardDetail {
                label: headers[i].to_string(),
                value: card_detail_value(tab, headers[i], i, cells),
            })
            .collect(),
    }
}

pub(crate) fn card_detail_view(value: &CardDetailValue) -> (AnyView, Option<String>) {
    match value {
        CardDetailValue::Text { text, negative } => (
            view! {
                <Amount text=text.clone() negative=*negative class="font-semibold" />
            }
            .into_any(),
            Some(text.clone()),
        ),
        CardDetailValue::SecurityCode(raw) => {
            let code = crate::features::receipts::model::normalize_security_code(raw);
            if code.is_empty() {
                (view! { <span>"—"</span> }.into_any(), None)
            } else if !is_security_code(&code) {
                (view! { <span>{code}</span> }.into_any(), None)
            } else {
                let href = format!("/search?code={code}");
                (
                    view! {
                        <a
                            href=href
                            class="security-code-link inline-flex min-h-11 min-w-11 items-center justify-end font-bold"
                            data-search=code.clone()
                        >
                            {code.clone()}
                        </a>
                    }
                    .into_any(),
                    None,
                )
            }
        }
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

#[component]
fn ReceiptItemCard(card: CardRowData) -> impl IntoView {
    let CardRowData {
        name,
        date,
        account,
        details,
    } = card;
    let (name_view, name_title) = card_detail_view(&name);
    view! {
        <Card variant=CardVariant::Item testid="receipt-card">
            <div class="flex min-w-0 flex-col gap-1.5 px-3 py-3">
                <div
                    class="min-w-0 break-words text-base font-semibold text-ink"
                    title=name_title
                >
                    {name_view}
                </div>
                <div class="flex items-center gap-2 text-xs text-text-muted">
                    <span class="shrink-0">{date}</span>
                    <span class="min-w-0 flex-1 break-words">{account}</span>
                </div>
            </div>
            <dl class="grid grid-cols-2 gap-x-4 gap-y-2 border-t border-border-strong px-3 py-2.5">
                {details
                    .iter()
                    .map(|detail| {
                        let (content, title) = card_detail_view(&detail.value);
                        view! {
                            <div class="min-w-0">
                                <dt class="text-xs text-text-muted">{detail.label.clone()}</dt>
                                <dd
                                    class="min-w-0 break-words text-right text-sm font-semibold text-text"
                                    title=title
                                >
                                    {content}
                                </dd>
                            </div>
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
                        {format!("{count}件")}
                    </span>
                </Card>
                {card_list}
            </section>
        }
        .into_any();
    }
    let (primary_label, primary_value) = summary.last().cloned().unwrap_or_default();
    let primary_negative = is_negative_labeled_value(tab, primary_label, &primary_value);
    let primary_value_class = if primary_negative {
        "text-sm font-semibold tabular-nums text-negative-vivid"
    } else {
        "text-sm font-semibold tabular-nums text-text"
    };
    let aria_label = format!("{label} {count}件 {primary_label} {primary_value}");
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
                            {format!("{count}件")}
                        </span>
                    </span>
                    <span
                        class="flex shrink-0 items-baseline gap-1 whitespace-nowrap"
                        aria-hidden="true"
                    >
                        <span class="text-xs text-text-muted">{primary_label}</span>
                        <span class=primary_value_class>
                            {primary_value}
                        </span>
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
                                    <dl>
                                        {summary
                                            .iter()
                                            .map(|(label, value)| {
                                                let negative =
                                                    is_negative_labeled_value(tab, label, value);
                                                let value_class = if negative {
                                                    "min-w-0 break-words text-right text-sm font-semibold tabular-nums text-negative"
                                                } else {
                                                    "min-w-0 break-words text-right text-sm font-semibold tabular-nums text-text"
                                                };
                                                view! {
                                                    <div class="flex items-start justify-between gap-3 border-b border-border-faint py-1.5 last:border-b-0">
                                                        <dt class="shrink-0 pt-0.5 text-xs text-text-muted">
                                                            {*label}
                                                        </dt>
                                                        <dd
                                                            class=value_class
                                                            data-negative=negative.then_some("true")
                                                        >
                                                            {value.clone()}
                                                        </dd>
                                                    </div>
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
