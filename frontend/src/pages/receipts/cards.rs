use crate::components::security_link::copy_to_clipboard;
use crate::receipts::{ReceiptCell, ReceiptItem, ReceiptsTab};
use leptos::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Copy)]
pub(crate) struct CardFields {
    pub(crate) name: usize,
    pub(crate) primary: usize,
    pub(crate) date: usize,
    pub(crate) account: usize,
}

pub(crate) fn card_fields(tab: ReceiptsTab) -> CardFields {
    match tab {
        ReceiptsTab::Dividend => CardFields {
            name: 4,
            primary: 9,
            date: 0,
            account: 2,
        },
        ReceiptsTab::DomesticStock => CardFields {
            name: 2,
            primary: 10,
            date: 0,
            account: 3,
        },
        ReceiptsTab::MutualFund => CardFields {
            name: 1,
            primary: 9,
            date: 0,
            account: 2,
        },
    }
}

pub(crate) fn summary_labels(tab: ReceiptsTab) -> [&'static str; 3] {
    match tab {
        ReceiptsTab::Dividend => ["配当金", "税額", "税引後"],
        ReceiptsTab::DomesticStock => ["損益", "税額", "税引後"],
        ReceiptsTab::MutualFund => ["実現損益", "税額", "税引損益"],
    }
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
    pub(crate) key: String,
    pub(crate) name: String,
    pub(crate) amount: String,
    pub(crate) amount_negative: bool,
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

pub(crate) fn card_detail_value(index: usize, cells: &[ReceiptCell]) -> CardDetailValue {
    match cells.get(index) {
        Some(ReceiptCell::SecurityCode(raw)) => CardDetailValue::SecurityCode(raw.clone()),
        Some(ReceiptCell::InstrumentName { name, code }) => {
            let trimmed = name.trim();
            let display = if trimmed.is_empty() {
                "-".to_string()
            } else {
                trimmed.to_string()
            };
            let copy = code
                .as_deref()
                .map(crate::receipts_domain::normalize_security_code)
                .filter(|code| !code.is_empty())
                .map_or_else(|| display.clone(), |code| format!("{display}({code})"));
            CardDetailValue::CopyName { display, copy }
        }
        _ => {
            let text = cells
                .get(index)
                .map(|cell| cell_text(cell).to_string())
                .unwrap_or_default();
            CardDetailValue::Text {
                negative: is_negative_text(&text),
                text,
            }
        }
    }
}

pub(crate) fn cell_text(cell: &ReceiptCell) -> &str {
    match cell {
        ReceiptCell::Text(value) | ReceiptCell::SecurityCode(value) => value,
        ReceiptCell::InstrumentName { name, .. } => name,
    }
}

// id を持たない行(CSVプレビュー等)は、同一内容の行と区別するため一覧内の位置も含めて識別する。
// 表示は数量等を丸めるため、行の同一性は丸め前の raw_key で判定する
pub(crate) fn card_key(slug: &str, id: &str, raw_key: &str, ordinal: usize) -> String {
    if id.is_empty() {
        format!("{slug}:p:{ordinal}:{raw_key}")
    } else {
        format!("{slug}:r:{id}")
    }
}

pub(crate) fn card_ordinal(
    ordinals: &mut HashMap<String, VecDeque<usize>>,
    id: &str,
    raw_key: &str,
) -> usize {
    if id.is_empty() {
        ordinals
            .get_mut(raw_key)
            .and_then(|queue| queue.pop_front())
            .unwrap_or_default()
    } else {
        0
    }
}

// 絞り込みや並べ替えで表示位置が変わっても同じ行を同じカードキーへ対応させるため、
// id を持たない行の通し番号は絞り込み前の全行内での位置から引く。
pub(crate) fn idless_row_ordinals(all_rows: &[ReceiptItem]) -> HashMap<String, VecDeque<usize>> {
    let mut ordinals: HashMap<String, VecDeque<usize>> = HashMap::new();
    for (index, item) in all_rows.iter().enumerate() {
        if item.id().is_empty() {
            ordinals.entry(item.raw_key()).or_default().push_back(index);
        }
    }
    ordinals
}

pub(crate) fn card_row_data(
    key: String,
    cells: &[ReceiptCell],
    headers: &[&'static str],
    order: &[usize],
    fields: CardFields,
) -> CardRowData {
    let text = |index: usize| {
        cells
            .get(index)
            .map(|cell| cell_text(cell).to_string())
            .unwrap_or_default()
    };
    let amount = text(fields.primary);
    CardRowData {
        key,
        name: text(fields.name),
        amount_negative: is_negative_text(&amount),
        amount,
        date: short_date(&text(fields.date)).to_string(),
        account: text(fields.account),
        details: order
            .iter()
            .map(|&i| CardDetail {
                label: headers[i].to_string(),
                value: card_detail_value(i, cells),
            })
            .collect(),
    }
}

pub(crate) fn card_detail_view(value: &CardDetailValue) -> (AnyView, Option<String>) {
    match value {
        CardDetailValue::Text { text, .. } => {
            (view! { {text.clone()} }.into_any(), Some(text.clone()))
        }
        CardDetailValue::SecurityCode(raw) => {
            let code = crate::receipts_domain::normalize_security_code(raw);
            if code.is_empty() {
                (view! { <span>"-"</span> }.into_any(), None)
            } else if !is_security_code(&code) {
                (view! { <span>{code}</span> }.into_any(), None)
            } else {
                let href = format!("/search?code={code}");
                (
                    view! {
                        <a
                            href=href
                            class="security-code-link font-bold"
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
            (
                view! {
                    <button
                        type="button"
                        aria-label=format!("{copy} をコピー")
                        on:click=move |_| copy_to_clipboard(copy_text.clone())
                        class="copyable-name group"
                    >
                        <span>{display.clone()}</span>
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
                    </button>
                }
                .into_any(),
                None,
            )
        }
    }
}

#[component]
fn ReceiptItemCard(
    card: CardRowData,
    id_prefix: String,
    expanded_ids: RwSignal<HashSet<String>>,
) -> impl IntoView {
    let expanded_id = card.key.clone();
    let toggle_id = expanded_id.clone();
    let expanded = Memo::new(move |_| expanded_ids.with(|set| set.contains(&expanded_id)));
    let button_id = format!("{id_prefix}-button");
    let details_id = format!("{id_prefix}-details");
    let CardRowData {
        name,
        amount,
        amount_negative,
        date,
        account,
        details,
        ..
    } = card;
    let aria_label = format!("{name} {amount}");
    let amount_class = if amount_negative {
        "min-w-[11ch] shrink-0 whitespace-nowrap text-right font-mono text-base font-semibold tabular-nums text-red-800"
    } else {
        "min-w-[11ch] shrink-0 whitespace-nowrap text-right font-mono text-base font-semibold tabular-nums text-slate-950"
    };
    view! {
        <div data-testid="receipt-card" class="rounded-lg border border-slate-300 bg-white">
            <button
                id=button_id.clone()
                type="button"
                aria-label=aria_label
                aria-expanded=move || if expanded.get() { "true" } else { "false" }
                aria-controls=details_id.clone()
                on:click=move |_| {
                    expanded_ids.update(|set| {
                        if !set.remove(&toggle_id) {
                            set.insert(toggle_id.clone());
                        }
                    })
                }
                class="receipt-card-trigger"
            >
                <span class="flex w-full items-baseline gap-2">
                    <span class="min-w-0 flex-1 truncate text-base font-semibold text-slate-950">
                        {name}
                    </span>
                    <span class=amount_class>{amount}</span>
                </span>
                <span
                    class="flex w-full items-center gap-2 text-xs text-slate-600"
                    aria-hidden="true"
                >
                    <span class="shrink-0">{date}</span>
                    <span class="min-w-0 flex-1 truncate">{account}</span>
                    <svg
                        class=move || {
                            if expanded.get() {
                                "h-4 w-4 shrink-0 rotate-180"
                            } else {
                                "h-4 w-4 shrink-0"
                            }
                        }
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        aria-hidden="true"
                    >
                        <path
                            d="m6 9 6 6 6-6"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        />
                    </svg>
                </span>
            </button>
            <div
                id=details_id
                role="region"
                aria-labelledby=button_id
                hidden=move || !expanded.get()
                class="border-t border-slate-300 px-3 py-2"
            >
                {move || {
                    expanded
                        .get()
                        .then(|| {
                            view! {
                                <dl>
                                    {details
                                        .iter()
                                        .map(|detail| {
                                            let negative = matches!(
                                                &detail.value,
                                                CardDetailValue::Text { negative: true, .. }
                                            );
                                            let value_class = if negative {
                                                "min-w-0 break-words text-right text-sm font-semibold tabular-nums text-red-800"
                                            } else {
                                                "min-w-0 break-words text-right text-sm font-semibold tabular-nums text-slate-800"
                                            };
                                            let (content, title) = card_detail_view(&detail.value);
                                            view! {
                                                <div class="flex items-start justify-between gap-3 border-b border-slate-100 py-1.5 last:border-b-0">
                                                    <dt class="shrink-0 pt-0.5 text-xs text-slate-600">
                                                        {detail.label.clone()}
                                                    </dt>
                                                    <dd class=value_class title=title>
                                                        {content}
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
        </div>
    }
}

#[component]
pub(crate) fn MobileCardGroup(
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
                .enumerate()
                .map(|(index, card)| {
                    view! {
                        <ReceiptItemCard
                            card=card
                            id_prefix=format!("{id_prefix}-card-{index}")
                            expanded_ids=expanded_ids
                        />
                    }
                })
                .collect_view()}
        </div>
    };
    if summary.is_empty() {
        return view! {
            <section data-testid="receipt-card-group">
                <div class="flex min-h-[44px] items-center rounded-lg bg-slate-700 px-3 py-2">
                    <span class="text-sm font-semibold text-white">{label}</span>
                    <span class="ml-2 inline-flex items-center rounded bg-white/20 px-2 py-0.5 text-xs font-medium text-white">
                        {format!("{count}件")}
                    </span>
                </div>
                {card_list}
            </section>
        }
        .into_any();
    }
    let (primary_label, primary_value) = summary.last().cloned().unwrap_or_default();
    let aria_label = format!("{label} {count}件 {primary_label} {primary_value}");
    view! {
        <section data-testid="receipt-card-group">
            <div class="overflow-hidden rounded-lg border border-slate-700">
                <button
                    id=button_id.clone()
                    type="button"
                    aria-label=aria_label
                    aria-expanded=move || if expanded.get() { "true" } else { "false" }
                    aria-controls=details_id.clone()
                    on:click=move |_| {
                        expanded_ids.update(|set| {
                            if !set.remove(&toggle_id) {
                                set.insert(toggle_id.clone());
                            }
                        })
                    }
                    class="group-card-trigger"
                >
                    <span class="min-w-0 flex-1 truncate text-sm font-semibold text-white">
                        {label}
                        <span class="ml-2 inline-flex items-center rounded bg-white/20 px-2 py-0.5 text-xs font-medium text-white">
                            {format!("{count}件")}
                        </span>
                    </span>
                    <span
                        class="flex shrink-0 items-baseline gap-1 whitespace-nowrap"
                        aria-hidden="true"
                    >
                        <span class="text-xs text-slate-200">{primary_label}</span>
                        <span class="font-mono text-sm font-semibold tabular-nums text-white">
                            {primary_value}
                        </span>
                        <svg
                            class=move || {
                                if expanded.get() {
                                    "h-4 w-4 shrink-0 self-center text-slate-200 rotate-180"
                                } else {
                                    "h-4 w-4 shrink-0 self-center text-slate-200"
                                }
                            }
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            aria-hidden="true"
                        >
                            <path
                                d="m6 9 6 6 6-6"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            />
                        </svg>
                    </span>
                </button>
                <div
                    id=details_id
                    role="region"
                    aria-labelledby=button_id
                    hidden=move || !expanded.get()
                    class="border-t border-slate-200 bg-white px-3 py-1"
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
                                                let negative = is_negative_text(value);
                                                let value_class = if negative {
                                                    "min-w-0 break-words text-right text-sm font-semibold tabular-nums text-red-800"
                                                } else {
                                                    "min-w-0 break-words text-right text-sm font-semibold tabular-nums text-slate-800"
                                                };
                                                view! {
                                                    <div class="flex items-start justify-between gap-3 border-b border-slate-100 py-1.5 last:border-b-0">
                                                        <dt class="shrink-0 pt-0.5 text-xs text-slate-600">
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
            </div>
            {card_list}
        </section>
    }
    .into_any()
}
