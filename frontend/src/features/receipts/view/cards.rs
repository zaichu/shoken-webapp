use crate::features::receipts::kind::{CardFields, CardRole};
use crate::features::receipts::{ReceiptCell, ReceiptsTab};
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::{AccountBadge, CodeBadge};
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::card::{Card, CardVariant};
use crate::ui::disclosure::{ChevronIcon, DisclosureStyle, DisclosureToggle};
use crate::ui::format::format_short_date;
use crate::ui::labels::{
    self, AFTER_TAX, BEFORE_TAX, DIVIDEND_AMOUNT, PROFIT_SHORT, REALIZED_PROFIT_LOSS,
    RECEIPT_SHORT, REDEMPTION_AMOUNT, SELLING_AMOUNT,
};
use crate::ui::list_row::ListRow;
use crate::ui::security_link::{SecurityCodeLink, copy_to_clipboard};
use crate::ui::stat_row::StatRow;
use leptos::prelude::*;
use std::collections::HashSet;
use wasm_bindgen::JsCast;

// 配当の税引後(受取額)は損益ではないので色を付けない
pub(crate) fn is_profit_label(tab: ReceiptsTab, label: &str) -> bool {
    tab != ReceiptsTab::Dividend && matches!(label, REALIZED_PROFIT_LOSS | AFTER_TAX)
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
    let aux_detail = match tab {
        ReceiptsTab::Dividend => DIVIDEND_AMOUNT,
        ReceiptsTab::DomesticStock => SELLING_AMOUNT,
        ReceiptsTab::MutualFund => REDEMPTION_AMOUNT,
    };
    let (aux_value, _, _) = compact_value(&details, aux_detail);
    // 行の主役は全タブで税引後(月見出し・帯と同じ基準。行を足すと帯に合う)。
    // 税引前の実現損益は展開した詳細に残す
    let (hero_value, hero_negative, hero_positive) = compact_value(&details, AFTER_TAX);
    let hero_is_profit = tab != ReceiptsTab::Dividend;
    let hero_label = AFTER_TAX;
    let hero_kind = if hero_is_profit {
        AmountKind::Delta
    } else {
        AmountKind::Plain
    };
    // 行に出している項目だけ詳細から除き、税引後など残りは詳細に残す
    let shown = [aux_detail, hero_label];
    let rest: Vec<CardDetail> = details
        .iter()
        .filter(|detail| !shown.contains(&detail.label.as_str()))
        .cloned()
        .collect();
    // トグルの読み上げ名は行ごとに識別できるよう銘柄名を入れる
    let name_text = match &name {
        CardDetailValue::Text { text, .. } => text.clone(),
        CardDetailValue::CopyName { display, .. } => display.clone(),
    };
    let row_name = name_text.clone();

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
    // リンクやボタンの上以外を押しても詳細が開くようにする
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
    // 行の日付セルは "YYYY/MM/DD" 済みなので区切りを戻してから月/日に直す
    let short_date = format_short_date(&date.replace('/', "-"));
    let has_details = !rest.is_empty();
    // 開閉ボタンは行の2行目に入れる。expanded の信号で追従するため作り直しは要らない
    let toggle_view: AnyView = view! {
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
    }
    .into_any();
    // 補助の金額には何の金額か分かるようラベルを付ける
    let aux_label: &'static str = match tab {
        ReceiptsTab::Dividend => BEFORE_TAX,
        ReceiptsTab::DomesticStock => SELLING_AMOUNT,
        ReceiptsTab::MutualFund => REDEMPTION_AMOUNT,
    };
    let secondary_view: AnyView = view! {
        <span class="flex items-center gap-4">
            <span>
                {aux_label}
                " "
                <Amount kind=AmountKind::Plain text=aux_value.clone() />
            </span>
            {has_details.then_some(toggle_view)}
        </span>
    }
    .into_any();

    view! {
        <li
            class="receipt-compact-row"
            data-testid="receipt-card"
            on:click=on_row_click
        >
            <div class="px-4 py-2">
                <ListRow
                    name=move || row_name.clone()
                    primary=move || view! {
                        <Amount
                            kind=hero_kind
                            text=hero_value.clone()
                            negative=hero_negative
                            positive=hero_positive
                        />
                    }
                    meta=move || view! {
                        {short_date.clone()}
                        {code
                            .clone()
                            .map(|code| {
                                view! {
                                    {" · "}
                                    <SecurityCodeLink
                                        value=code
                                        class="receipt-compact-code".to_string()
                                    />
                                }
                            })}
                        {" · "}
                        <AccountBadge label=account.clone() />
                    }
                    secondary=secondary_view
                />
            </div>
            <div
                id=detail_id
                class="receipt-compact-details"
                hidden=move || !expanded.get()
            >
                <dl class="space-y-1">
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

/// 月・日見出しの右に出す合計。行の主役・帯と同じ税引後の合計を使う
pub(crate) fn group_headline_value<'a>(summary: &'a [(&'static str, String)]) -> &'a str {
    summary
        .iter()
        .find(|(label, _)| *label == AFTER_TAX)
        .map(|(_, value)| value.as_str())
        .or_else(|| summary.last().map(|(_, value)| value.as_str()))
        .unwrap_or("")
}

/// 月見出しの合計。集計が無い単発の日はその行の税引後を使う
pub(crate) fn single_row_headline(cards: &[CardRowData]) -> Vec<(&'static str, String)> {
    let [card] = cards else {
        return Vec::new();
    };
    let (text, _, _) = compact_value(&card.details, AFTER_TAX);
    if text.is_empty() {
        Vec::new()
    } else {
        vec![(AFTER_TAX, text)]
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
    // 単発の日は集計が無いのでその行の税引後を見出しに出す
    let summary: Vec<(&'static str, String)> = if summary.is_empty() {
        single_row_headline(&cards)
    } else {
        summary
    };
    let (display_label, display_value, display_negative, display_positive, display_kind) = match tab
    {
        ReceiptsTab::Dividend => {
            let value = group_headline_value(&summary).to_string();
            (RECEIPT_SHORT, value, false, false, AmountKind::Plain)
        }
        _ => {
            let value = group_headline_value(&summary).to_string();
            let negative = is_negative_text(&value);
            let positive = is_positive_text(&value);
            let text = if positive && !value.trim_start().starts_with('+') {
                format!("+{value}")
            } else {
                value
            };
            (PROFIT_SHORT, text, negative, positive, AmountKind::Delta)
        }
    };
    view! {
        <section data-testid="receipt-card-group">
            <div class="receipt-compact-group">
                <span class="font-bold">{labels::group_count(&label, count)}</span>
                {(!display_value.is_empty()).then(|| view! {
                    <span class="whitespace-nowrap">
                        {display_label}
                        " "
                        <Amount
                            kind=display_kind
                            text=display_value
                            negative=display_negative
                            positive=display_positive
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
