use super::cards::{
    card_fields, card_key, card_ordinal, card_row_data, idless_row_ordinals, is_negative_text,
    summary_labels, CardRowData, MobileCardGroup,
};
use super::groups::{table_groups, TableGroup};
use super::TAB_IDS;
use crate::components::security_link::{CopyableInstrumentName, SecurityCodeLink};
use crate::receipts::filter::column_order;
use crate::receipts::{ReceiptCell, ReceiptItem, ReceiptsTab};
use leptos::ev;
use leptos::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

pub(crate) fn table_headers(tab: ReceiptsTab) -> &'static [&'static str] {
    match tab {
        ReceiptsTab::Dividend => &[
            "入金日",
            "商品",
            "口座",
            "銘柄コード",
            "銘柄名",
            "単価",
            "数量",
            "配当金",
            "税額",
            "受取額",
        ],
        ReceiptsTab::DomesticStock => &[
            "約定日",
            "銘柄コード",
            "銘柄名",
            "口座",
            "数量",
            "売却単価",
            "売却額",
            "取得価額",
            "損益",
            "税額",
            "税引後",
        ],
        ReceiptsTab::MutualFund => &[
            "約定日",
            "ファンド名",
            "口座",
            "数量",
            "解約単価",
            "解約額",
            "取得価額",
            "実現損益",
            "税額",
            "税引損益",
        ],
    }
}

// 幅は列に追随させるため基本順で持ち、表示時に column_order と同じ並びにする
pub(crate) fn table_column_widths(tab: ReceiptsTab) -> &'static [&'static str] {
    match tab {
        ReceiptsTab::Dividend => &[
            "84px", "64px", "64px", "72px", "160px", "72px", "56px", "84px", "64px", "84px",
        ],
        ReceiptsTab::DomesticStock => &[
            "84px", "72px", "156px", "60px", "56px", "76px", "82px", "82px", "82px", "64px", "84px",
        ],
        ReceiptsTab::MutualFund => &[
            "112px", "300px", "60px", "112px", "98px", "128px", "116px", "112px", "106px", "118px",
        ],
    }
}

fn table_column_aligns(tab: ReceiptsTab) -> &'static [&'static str] {
    match tab {
        ReceiptsTab::Dividend => &[
            "left", "left", "left", "center", "left", "right", "right", "right", "right", "right",
        ],
        ReceiptsTab::DomesticStock => &[
            "left", "center", "left", "left", "right", "right", "right", "right", "right", "right",
            "right",
        ],
        ReceiptsTab::MutualFund => &[
            "left", "left", "left", "right", "right", "right", "right", "right", "right", "right",
        ],
    }
}

// Closure は Send/Sync でないためシグナルや on_cleanup の捕捉に置けず、
// マウント中だけ生存させたいので thread_local で管理する
type TableHeightObserver = (
    web_sys::ResizeObserver,
    Closure<dyn FnMut(Vec<web_sys::ResizeObserverEntry>)>,
);
thread_local! {
    static TABLE_HEIGHT_OBSERVERS: RefCell<HashMap<usize, TableHeightObserver>> =
        RefCell::new(HashMap::new());
    static TABLE_HEIGHT_OBSERVER_NEXT_ID: Cell<usize> = const { Cell::new(0) };
}

#[component]
pub(crate) fn ReceiptTable(
    tab: ReceiptsTab,
    rows: Vec<ReceiptItem>,
    all_rows: Vec<ReceiptItem>,
    query: String,
    expanded_ids: RwSignal<HashSet<String>>,
) -> impl IntoView {
    let headers: &[&'static str] = table_headers(tab);
    let groups: Vec<TableGroup> = table_groups(tab, &rows, &all_rows, &query);
    let order = column_order(tab, &rows, &query);
    let fields = card_fields(tab);
    let labels = summary_labels(tab);
    let slug = TAB_IDS[tab as usize];
    let mut card_ordinals: HashMap<String, VecDeque<usize>> = idless_row_ordinals(&all_rows);
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
                    let ordinal = card_ordinal(&mut card_ordinals, id, raw_key);
                    card_row_data(
                        card_key(slug, id, raw_key, ordinal),
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
    let cell_classes: Vec<&'static str> = order
        .iter()
        .map(|i| match table_column_aligns(tab)[*i] {
            "center" => "text-center",
            "right" => "text-right tabular-nums",
            _ => "text-left",
        })
        .collect();
    let table_scroll = NodeRef::<leptos::html::Div>::new();
    let table_max_height = RwSignal::new(Option::<f64>::None);
    let measure_table = move || {
        let Some(element) = table_scroll.get() else {
            return;
        };
        let Some(viewport) = web_sys::window()
            .and_then(|window| window.inner_height().ok())
            .and_then(|height| height.as_f64())
        else {
            return;
        };
        let available = viewport - element.get_bounding_client_rect().top() - 20.0;
        table_max_height.set(Some(available.max(200.0)));
    };
    let on_table_resize = window_event_listener(ev::resize, move |_| measure_table());
    let observer_key = web_sys::window().and_then(|_| {
        let callback = Closure::<dyn FnMut(Vec<web_sys::ResizeObserverEntry>)>::new(move |_| {
            measure_table();
        });
        web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref())
            .ok()
            .map(|observer| {
                let key = TABLE_HEIGHT_OBSERVER_NEXT_ID.with(|next| {
                    let key = next.get();
                    next.set(key + 1);
                    key
                });
                TABLE_HEIGHT_OBSERVERS.with(|observers| {
                    observers.borrow_mut().insert(key, (observer, callback));
                });
                key
            })
    });
    Effect::new(move |_| {
        measure_table();
        let (Some(element), Some(key)) = (table_scroll.get(), observer_key) else {
            return;
        };
        TABLE_HEIGHT_OBSERVERS.with(|observers| {
            let observers = observers.borrow();
            let Some((observer, _)) = observers.get(&key) else {
                return;
            };
            observer.observe(&element);
            if let Some(parent) = element.parent_element() {
                observer.observe(&parent);
            }
            if let Some(body) = web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.body())
            {
                observer.observe(&body);
            }
        });
    });
    on_cleanup(move || {
        on_table_resize.remove();
        if let Some(key) = observer_key {
            TABLE_HEIGHT_OBSERVERS.with(|observers| {
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
                    <div
                        node_ref=table_scroll
                        class="table-scroll"
                        style:max-height=move || {
                            table_max_height
                                .get()
                                .map(|height| format!("{height}px"))
                                .unwrap_or_default()
                        }
                    >
                        <table class="receipt-table">
                            <thead class="sticky top-0 z-10 bg-slate-100 text-slate-800">
                                <tr class="bg-slate-50">
                                    {headers
                                        .iter()
                                        .zip(widths.iter())
                                        .map(|(header, width)| {
                                            view! {
                                                <th
                                                    class="text-center font-black text-slate-800"
                                                    scope="col"
                                                    style:width=*width
                                                    style:max-width=*width
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
                                            " border-t-2 border-slate-300"
                                        } else {
                                            ""
                                        };
                                        view! {
                                            <tr>
                                                <td
                                                    colspan={headers.len() - 3}
                                                    class=format!(
                                                        "whitespace-normal bg-slate-100 text-slate-800 font-semibold border-l-2 border-slate-500{top}"
                                                    )
                                                >
                                                    <span class="text-sm font-medium">{group.label.clone()}</span>
                                                    <span class="ml-2 inline-flex items-center rounded bg-slate-600 px-2 py-0.5 text-xs font-medium text-white">
                                                        {format!("{count}件")}
                                                    </span>
                                                </td>
                                                {group
                                                    .summary
                                                    .iter()
                                                    .map(|value| {
                                                        let negative = is_negative_text(value);
                                                        view! {
                                                            <td
                                                                class=format!(
                                                                    "bg-slate-100 text-slate-800 text-right font-semibold{top}"
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
                                                                    let align = cell_classes[col_index];
                                                                    match cell {
                                                                        ReceiptCell::SecurityCode(code) => view! {
                                                                            <td class=align>
                                                                                <SecurityCodeLink value=code />
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
                                                                            let negative = is_negative_text(&value);
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
