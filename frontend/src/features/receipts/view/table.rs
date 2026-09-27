use super::cards::{
    card_key, card_ordinal, card_row_data, is_negative_text, is_profit_label, preview_row_ordinals,
    summary_is_profit, CardRowData, MobileCardGroup,
};
use super::groups::{table_groups, TableGroup};
use super::TAB_IDS;
use crate::features::receipts::filter::{column_order, promoted_column};
use crate::features::receipts::kind::ColumnTier;
use crate::features::receipts::{ReceiptCell, ReceiptRow, ReceiptsTab};
use crate::ui::security_link::{CopyableInstrumentName, SecurityCodeLink};
use leptos::ev;
use leptos::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

pub(crate) fn table_headers(tab: ReceiptsTab) -> &'static [&'static str] {
    tab.headers()
}

// 幅は列に追随させるため基本順で持ち、表示時に column_order と同じ並びにする。空は残り幅を使う列
pub(crate) fn table_column_widths(tab: ReceiptsTab) -> &'static [&'static str] {
    tab.column_widths()
}

pub(crate) fn table_column_tiers(tab: ReceiptsTab) -> &'static [ColumnTier] {
    tab.column_tiers()
}

// 検索で前に出した列は、狭い画面でも隠さない
fn displayed_tiers(tab: ReceiptsTab, order: &[usize], promoted: Option<usize>) -> Vec<ColumnTier> {
    order
        .iter()
        .map(|&column| {
            if promoted == Some(column) {
                ColumnTier::Core
            } else {
                table_column_tiers(tab)[column]
            }
        })
        .collect()
}

fn table_column_aligns(tab: ReceiptsTab) -> &'static [&'static str] {
    tab.column_aligns()
}

// Closure は Send/Sync でないためシグナルや on_cleanup の捕捉に置けず、
// マウント中だけ生存させたいので thread_local で管理する
type HeaderHeightObserver = (
    web_sys::ResizeObserver,
    Closure<dyn FnMut(Vec<web_sys::ResizeObserverEntry>)>,
);
thread_local! {
    static HEADER_HEIGHT_OBSERVERS: RefCell<HashMap<usize, HeaderHeightObserver>> =
        RefCell::new(HashMap::new());
    static HEADER_HEIGHT_OBSERVER_NEXT_ID: Cell<usize> = const { Cell::new(0) };
}

fn site_header() -> Option<web_sys::Element> {
    web_sys::window()?
        .document()?
        .query_selector(".site-header")
        .ok()
        .flatten()
}

#[component]
pub(crate) fn ReceiptTable(
    tab: ReceiptsTab,
    rows: Vec<ReceiptRow>,
    all_rows: Vec<ReceiptRow>,
    query: String,
    expanded_ids: RwSignal<HashSet<String>>,
) -> impl IntoView {
    let headers: &[&'static str] = table_headers(tab);
    let groups: Vec<TableGroup> = table_groups(tab, &rows, &all_rows, &query);
    let order = column_order(tab, &rows, &query);
    let promoted = promoted_column(tab, &rows, &query);
    let fields = tab.card_fields();
    let labels = tab.summary_labels();
    let slug = TAB_IDS[tab as usize];
    let mut card_ordinals: HashMap<String, VecDeque<usize>> = preview_row_ordinals(&all_rows);
    let card_groups: Vec<_> = groups
        .iter()
        .enumerate()
        .map(|(group_index, group)| {
            let summary: Vec<(&'static str, String)> = labels
                .iter()
                .copied()
                .zip(group.summary.iter().cloned())
                .collect();
            let cards: Vec<CardRowData> = group
                .rows
                .iter()
                .map(|(id, raw_key, cells)| {
                    let ordinal = card_ordinal(&mut card_ordinals, id.as_deref(), raw_key);
                    card_row_data(
                        card_key(slug, id.as_deref(), raw_key, ordinal),
                        cells,
                        headers,
                        &order,
                        fields,
                    )
                })
                .collect();
            (
                group_index,
                group.key.clone(),
                group.label.clone(),
                group.rows.len(),
                summary,
                cards,
            )
        })
        .collect();
    let headers: Vec<_> = order.iter().map(|i| headers[*i]).collect();
    let widths: Vec<_> = order.iter().map(|i| table_column_widths(tab)[*i]).collect();
    let cell_classes: Vec<String> = order
        .iter()
        .zip(displayed_tiers(tab, &order, promoted))
        .map(|(i, tier)| {
            let align = match table_column_aligns(tab)[*i] {
                "center" => "text-center",
                "right" => "text-right tabular-nums",
                _ => "text-left",
            };
            format!("{align}{}", tier.class())
        })
        .collect();
    let tiers = displayed_tiers(tab, &order, promoted);
    let group_label_spans: Vec<(&'static str, usize)> = [
        (" xl:hidden print:hidden", ColumnTier::Core),
        (
            " hidden xl:table-cell 2xl:hidden print:hidden",
            ColumnTier::Wide,
        ),
        (" hidden 2xl:table-cell print:table-cell", ColumnTier::Wider),
    ]
    .into_iter()
    .map(|(class, level)| {
        let visible = tiers.iter().filter(|tier| **tier <= level).count();
        (class, visible.saturating_sub(labels.len()))
    })
    .collect();
    // サイトのヘッダーも sticky なので、表の見出し行はその直下で止める
    let header_offset = RwSignal::new(Option::<f64>::None);
    let measure_header = move || {
        let height = site_header().map_or(0.0, |header| header.get_bounding_client_rect().height());
        header_offset.set(Some(height));
    };
    let on_window_resize = window_event_listener(ev::resize, move |_| measure_header());
    let observer_key = site_header().and_then(|header| {
        let callback = Closure::<dyn FnMut(Vec<web_sys::ResizeObserverEntry>)>::new(move |_| {
            measure_header();
        });
        web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref())
            .ok()
            .map(|observer| {
                observer.observe(&header);
                let key = HEADER_HEIGHT_OBSERVER_NEXT_ID.with(|next| {
                    let key = next.get();
                    next.set(key + 1);
                    key
                });
                HEADER_HEIGHT_OBSERVERS.with(|observers| {
                    observers.borrow_mut().insert(key, (observer, callback));
                });
                key
            })
    });
    measure_header();
    on_cleanup(move || {
        on_window_resize.remove();
        if let Some(key) = observer_key {
            HEADER_HEIGHT_OBSERVERS.with(|observers| {
                if let Some((observer, _callback)) = observers.borrow_mut().remove(&key) {
                    observer.disconnect();
                }
            });
        }
    });
    view! {
        <div
            class="table-card"
            data-testid="receipt-card"
        >
            <div class="p-0" data-testid="receipt-card-body">
                <div class="hidden sm:block">
                    <div class="table-frame">
                        <table class="receipt-table">
                            <thead
                                class="sticky z-10 bg-surface-raised text-text print:static"
                                style:top=move || {
                                    header_offset
                                        .get()
                                        .map(|height| format!("{height}px"))
                                        .unwrap_or_default()
                                }
                            >
                                <tr class="bg-surface-sunken">
                                    {headers
                                        .iter()
                                        .zip(widths.iter())
                                        .zip(tiers.iter())
                                        .map(|((header, width), tier)| {
                                            let width = (!width.is_empty()).then_some(*width);
                                            view! {
                                                <th
                                                    class=format!(
                                                        "text-center font-black text-text{}",
                                                        tier.class(),
                                                    )
                                                    scope="col"
                                                    style:width=width
                                                    style:max-width=width
                                                >
                                                    {*header}
                                                </th>
                                            }
                                        })
                                        .collect_view()}
                                </tr>
                            </thead>
                            <tbody>
                                {groups
                                    .iter()
                                    .enumerate()
                                    .map(|(group_index, group)| {
                                        let count = group.rows.len();
                                        let top = if group_index > 0 {
                                            " border-t-2 border-border-strong"
                                        } else {
                                            ""
                                        };
                                        view! {
                                            <tr>
                                                {group_label_spans
                                                    .iter()
                                                    .map(|(class, span)| {
                                                        view! {
                                                            <td
                                                                colspan=*span
                                                                class=format!(
                                                                    "whitespace-normal bg-surface-raised text-text-muted font-semibold border-l-2 border-border-xstrong{top}{class}"
                                                                )
                                                            >
                                                                <span class="text-sm font-medium">{group.label.clone()}</span>
                                                                {(count >= 2)
                                                                    .then(|| {
                                                                        view! {
                                                                            <span class="ml-2 text-xs font-medium text-text-muted">
                                                                                {format!("{count}件")}
                                                                            </span>
                                                                        }
                                                                    })}
                                                            </td>
                                                        }
                                                    })
                                                    .collect_view()}
                                                {group
                                                    .summary
                                                    .iter()
                                                    .zip(labels.iter())
                                                    .map(|(value, label)| {
                                                        let negative = summary_is_profit(tab, label)
                                                            && is_negative_text(value);
                                                        view! {
                                                            <td
                                                                class=format!(
                                                                    "bg-surface-raised text-text text-right font-semibold{top}"
                                                                )
                                                                data-negative=negative.then_some("true")
                                                            >
                                                                {value.clone()}
                                                            </td>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </tr>
                                            {group
                                                .rows
                                                .iter()
                                                .map(|(_, _, cells)| {
                                                    let cells: Vec<_> = order.iter().map(|i| cells[*i].clone()).collect();
                                                    view! {
                                                        <tr>
                                                            {cells
                                                                .into_iter()
                                                                .enumerate()
                                                                .map(|(col_index, cell)| {
                                                                    let align = cell_classes[col_index].clone();
                                                                    match cell {
                                                                        ReceiptCell::SecurityCode(code) if code.trim().is_empty() => view! {
                                                                            <td class=align>
                                                                                <SecurityCodeLink value=code />
                                                                            </td>
                                                                        }
                                                                        .into_any(),
                                                                        ReceiptCell::SecurityCode(code) => view! {
                                                                            <td class=align>
                                                                                <span class="code-badge py-0">
                                                                                    <SecurityCodeLink value=code class="font-semibold".to_string() />
                                                                                </span>
                                                                            </td>
                                                                        }
                                                                        .into_any(),
                                                                        ReceiptCell::InstrumentName { name, code } => view! {
                                                                            <td class=align>
                                                                                <CopyableInstrumentName name=name code=code.unwrap_or_default() />
                                                                            </td>
                                                                        }
                                                                        .into_any(),
                                                                        ReceiptCell::Text(value) => {
                                                                            let negative =
                                                                                is_profit_label(
                                                                                    table_headers(
                                                                                        tab,
                                                                                    )
                                                                                    [order[col_index]],
                                                                                ) && is_negative_text(
                                                                                &value,
                                                                            );
                                                                            let title = value.clone();
                                                                            view! {
                                                                                <td
                                                                                    class=align
                                                                                    title=title
                                                                                    data-negative=negative.then_some("true")
                                                                                >
                                                                                    {value}
                                                                                </td>
                                                                            }
                                                                            .into_any()
                                                                        }
                                                                    }
                                                                })
                                                                .collect_view()}
                                                    </tr>
                                                }
                                            })
                                            .collect_view()}
                                    }
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                    </div>
                </div>
                <div class="sm:hidden" data-testid="receipt-card-list">
                    <div class="space-y-4">
                        {card_groups
                            .into_iter()
                            .map(|(group_index, key, label, count, summary, cards)| {
                                view! {
                                    <MobileCardGroup
                                        tab=tab
                                        label=label
                                        count=count
                                        summary=summary
                                        cards=cards
                                        id_prefix=format!("receipt-{slug}-group-{group_index}")
                                        expanded_key=format!("{slug}:g:{key}")
                                        expanded_ids=expanded_ids
                                    />
                                }
                            })
                            .collect_view()}
                    </div>
                </div>
            </div>
        </div>
    }
}
