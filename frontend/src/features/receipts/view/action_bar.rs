use crate::features::receipts::filter::{
    DateSegment, ReceiptSearch, SearchKey, filter_receipts, search_categories,
};
use crate::features::receipts::kind::group_label;
use crate::features::receipts::{ReceiptsStore, ReceiptsTab, TabState};
use crate::support::list_search::SearchOption;
use crate::ui::badge::account_display_name;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::labels::{self, ACCOUNT, PERIOD, PRODUCT};
use crate::ui::search_controls::{
    AddConditionButton, AddConditionRow, ConditionOptionList, ConditionPanel, ConditionRowEnd,
    SearchBox, SearchClearButton, SearchCount, SearchTag, SearchTextInput,
};
use leptos::prelude::*;

// 銘柄名・コードの文字列検索は入力欄で行うため、条件として数えるのは
// 期間・商品・口座の3種類だけにする
const CONDITION_KEYS: [SearchKey; 3] = [SearchKey::Date, SearchKey::Products, SearchKey::Accounts];

pub(crate) fn condition_count(search: &ReceiptSearch) -> usize {
    CONDITION_KEYS
        .iter()
        .filter(|key| !search.selected_queries.get(**key).is_empty())
        .count()
}

/// 「条件を足す」で小窓・シートから選べる条件の種別
#[derive(Clone, Copy, PartialEq)]
enum ConditionKind {
    Period,
    Account,
    Product,
}

impl ConditionKind {
    fn label(self) -> &'static str {
        match self {
            Self::Period => PERIOD,
            Self::Account => ACCOUNT,
            Self::Product => PRODUCT,
        }
    }

    fn title(self) -> String {
        labels::pick_title(self.label())
    }
}

/// 期間タグの表示値。クエリの生値(2024・2024-03・日付・範囲)を読める形に直す
fn date_tag_text(value: &str) -> String {
    if let Some((start, end)) = value.split_once("..") {
        let (start, end) = (date_part_text(start), date_part_text(end));
        return match (start.is_empty(), end.is_empty()) {
            (true, _) => format!("〜{end}"),
            (_, true) => format!("{start}〜"),
            (false, false) => format!("{start}〜{end}"),
        };
    }
    date_part_text(value)
}

fn date_part_text(value: &str) -> String {
    if value.len() == 4 {
        format!("{value}年")
    } else {
        group_label(value)
    }
}

#[component]
fn ConditionTags(
    search: RwSignal<ReceiptSearch>,
    has_years: impl Fn() -> bool + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let date_value =
        Memo::new(move |_| date_tag_text(&search.with(|s| s.selected_queries.date.clone())));
    let products_value = Memo::new(move |_| search.with(|s| s.selected_queries.products.clone()));
    // クエリの生値は空白を含むと引用符で囲まれるため、札と同じ表示名に直して出す
    let accounts_value = Memo::new(move |_| {
        search.with(|s| {
            account_display_name(s.selected_queries.accounts.trim_matches('"')).to_string()
        })
    });
    view! {
        <SearchTag
            label=PERIOD
            value=date_value
            on_clear=move || search.update(|s| s.clear_key(SearchKey::Date, has_years()))
        />
        <SearchTag
            label=PRODUCT
            value=products_value
            on_clear=move || search.update(|s| s.clear_key(SearchKey::Products, has_years()))
        />
        <SearchTag
            label=ACCOUNT
            value=accounts_value
            on_clear=move || search.update(|s| s.clear_key(SearchKey::Accounts, has_years()))
        />
    }
}

#[component]
fn SecuritiesInput(search: RwSignal<ReceiptSearch>) -> impl IntoView {
    view! {
        <SearchTextInput
            placeholder=labels::search_placeholder("銘柄名・コード")
            value=move || search.with(|s| s.selected_queries.get(SearchKey::Securities).to_string())
            on_input=move |value| search.update(|s| s.set_securities_query(value))
        />
    }
}

/// 絞り込み中の件数と全件数。取得中は出さない
#[component]
fn ReceiptSearchCount(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let count = Memo::new(move |_| match store.tab_state(tab) {
        TabState::Ready(data) => {
            let query = store.search.with(|s| s.query.clone());
            let total = data.total.unwrap_or(data.rows.len());
            Some((filter_receipts(tab, &data.rows, &query).len(), total))
        }
        _ => None,
    });
    move || {
        count.get().map(|(filtered, total)| {
            view! {
                <SearchCount filtered=move || filtered total=move || total />
            }
        })
    }
}

#[component]
fn ClearAllButton(
    search: RwSignal<ReceiptSearch>,
    has_years: impl Fn() -> bool + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let nothing_to_clear = move || {
        search.with(|s| condition_count(s) == 0 && s.selected_queries.securities.is_empty())
    };
    view! {
        <SearchClearButton
            disabled=nothing_to_clear
            on_clear=move || search.update(|s| s.clear(has_years()))
        />
    }
}

#[component]
fn ReceiptAddConditionButton(
    kind: ConditionKind,
    open: RwSignal<Option<ConditionKind>>,
    node_ref: NodeRef<leptos::html::Button>,
) -> impl IntoView {
    let is_open = move || open.get() == Some(kind);
    view! {
        <AddConditionButton
            label=kind.label()
            expanded=move || open.get() == Some(kind)
            node_ref=node_ref
            on_click=move || {
                if is_open() {
                    open.set(None);
                } else {
                    open.set(Some(kind));
                }
            }
        />
    }
}

/// 条件を足す小窓(PC)・下から出るシート(スマホ)。開くたびに作り直される。
/// 期間は下書きシグナルの上で編集し確定で反映する。口座・商品は今のチップと
/// 同じ選択の挙動(select_quick の切り替え)のまま、選んですぐ反映する
#[component]
fn ConditionPopover(
    kind: ConditionKind,
    search: RwSignal<ReceiptSearch>,
    years: Memo<Vec<SearchOption>>,
    products: Memo<Vec<SearchOption>>,
    accounts: Memo<Vec<SearchOption>>,
    has_years: impl Fn() -> bool + Copy + 'static + Send + Sync,
    /// 背景のクリック・Esc・確定のどれで閉じてもトリガーのボタンへフォーカスを戻す
    on_close: impl Fn(ConditionKind) + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let draft = RwSignal::new(search.get_untracked());

    let apply_period = move |_| {
        let draft = draft.get_untracked();
        search.update(|s| {
            if draft.date_all {
                s.clear_key(SearchKey::Date, has_years());
            } else {
                s.date_segment = draft.date_segment;
                s.date_inputs = draft.date_inputs.clone();
                s.date_all = false;
                s.selected_queries.date = draft.selected_queries.date.clone();
                s.rebuild_query();
            }
        });
        on_close(kind);
    };

    let select_option = move |key: SearchKey, value: String| {
        search.update(|s| s.select_quick(key, value));
        on_close(kind);
    };
    // 口座・商品は単一選択。選んだらすぐ反映して小窓を閉じる
    let selected_of = move |get: fn(&ReceiptSearch) -> String| {
        Memo::new(move |_| {
            let value = search.with(get);
            if value.is_empty() {
                Vec::new()
            } else {
                vec![value]
            }
        })
    };
    let selected_accounts = selected_of(|s| s.selected_queries.accounts.clone());
    let selected_products = selected_of(|s| s.selected_queries.products.clone());

    view! {
        <ConditionPanel title=kind.title() on_close=move || on_close(kind)>
            {match kind {
                ConditionKind::Period => view! {
                    <PeriodDraftContent draft=draft years=years has_years=has_years />
                    <div class="condition-panel-actions">
                        <Button variant=ButtonVariant::Primary(ButtonSize::Md) on_click=apply_period>
                            "この期間で絞る"
                        </Button>
                    </div>
                }
                    .into_any(),
                ConditionKind::Account => view! {
                    <ConditionOptionList
                        options=accounts
                        selected=selected_accounts
                        on_toggle=move |value| select_option(SearchKey::Accounts, value)
                    />
                }
                    .into_any(),
                ConditionKind::Product => view! {
                    <ConditionOptionList
                        options=products
                        selected=selected_products
                        on_toggle=move |value| select_option(SearchKey::Products, value)
                    />
                }
                    .into_any(),
            }}
        </ConditionPanel>
    }
}

#[component]
fn PeriodDraftContent(
    draft: RwSignal<ReceiptSearch>,
    years: Memo<Vec<SearchOption>>,
    has_years: impl Fn() -> bool + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let segment = Memo::new(move |_| draft.with(|s| s.date_segment));
    let segment_options = ["すべて", "年", "年月", "日", "範囲"];

    view! {
        <div class="condition-segment" role="group" aria-label="期間の種類">
            {segment_options
                .iter()
                .map(|opt| {
                    let opt = *opt;
                    let is_selected = move || {
                        draft.with(|s| {
                            if s.date_all {
                                opt == "すべて"
                            } else {
                                s.date_segment.value() == opt
                            }
                        })
                    };
                    view! {
                        <button
                            type="button"
                            class="condition-segment-option"
                            aria-pressed=move || is_selected().to_string()
                            on:click=move |_| {
                                draft.update(|s| match opt {
                                    "すべて" => s.clear_key(SearchKey::Date, has_years()),
                                    "年" => s.change_date_segment(DateSegment::Year),
                                    "年月" => s.change_date_segment(DateSegment::Month),
                                    "日" => s.change_date_segment(DateSegment::Date),
                                    _ => s.change_date_segment(DateSegment::Range),
                                })
                            }
                        >
                            {opt}
                        </button>
                    }
                })
                .collect_view()}
        </div>
        {move || {
            if draft.with(|s| s.date_all) {
                return ().into_any();
            }
            match segment.get() {
                DateSegment::Year => view! { <YearPicker search=draft years=years /> }.into_any(),
                DateSegment::Month => view! { <MonthPicker search=draft years=years /> }.into_any(),
                DateSegment::Date => view! { <DatePicker search=draft /> }.into_any(),
                DateSegment::Range => view! { <RangePicker search=draft /> }.into_any(),
            }
        }}
    }
}

#[component]
fn YearPicker(search: RwSignal<ReceiptSearch>, years: Memo<Vec<SearchOption>>) -> impl IntoView {
    view! {
        <div class="mt-2 flex flex-wrap gap-1.5">
            {move || {
                let years = years.get();
                if years.is_empty() {
                    return view! { <span class="text-sm text-text-muted">"該当する年がありません"</span> }.into_any();
                }
                view! {
                    {years.iter().map(|y| {
                        let y_value = y.value.clone();
                        let y_label = y.label.clone();
                        let y_value_for_selected = y_value.clone();
                        let selected = Memo::new(move |_| search.with(|s| s.date_inputs.year_value.clone() == y_value_for_selected));
                        view! {
                            <button
                                type="button"
                                class=move || format!(
                                    "px-3 py-1.5 rounded-full text-sm font-medium transition-colors max-sm:min-h-11 {}",
                                    if selected.get() {
                                        "bg-surface border border-brand-text text-text-deep"
                                    } else {
                                        "bg-surface border border-border-strong text-text-soft hover:bg-surface-sunken"
                                    }
                                )
                                on:click=move |_| search.update(|s| s.select_year(y_value.clone()))
                            >
                                {y_label}
                            </button>
                        }
                    }).collect_view()}
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn MonthPicker(search: RwSignal<ReceiptSearch>, years: Memo<Vec<SearchOption>>) -> impl IntoView {
    let months = (1..=12).map(|m| format!("{:02}", m)).collect::<Vec<_>>();
    let selected_month = Memo::new(move |_| search.with(|s| s.date_inputs.month_value.clone()));
    let selected_year = Memo::new(move |_| search.with(|s| s.date_inputs.year_value.clone()));

    view! {
        <div class="mt-2 flex flex-wrap gap-1.5">
            {move || {
                let years = years.get();
                if years.is_empty() {
                    return view! {
                        <div class="flex flex-wrap gap-1.5">
                            {months.iter().map(|m| {
                                let m_value = m.clone();
                                let m_label = format!("{}月", m_value.parse::<u32>().unwrap_or(0));
                                let m_value_for_selected = m_value.clone();
                                let is_selected = move || selected_month.get() == m_value_for_selected;
                                view! {
                                    <button
                                        type="button"
                                        class=move || format!(
                                            "px-3 py-1.5 rounded-full text-sm font-medium transition-colors max-sm:min-h-11 {}",
                                            if is_selected() {
                                                "bg-surface border border-brand-text text-text-deep"
                                            } else {
                                                "bg-surface border border-border-strong text-text-soft hover:bg-surface-sunken"
                                            }
                                        )
                                        on:click=move |_| {
                                            search.update(|s| {
                                                s.date_inputs.month_value = m_value.clone();
                                                let year = s.date_inputs.year_value.clone();
                                                if !year.is_empty() {
                                                    s.set_date_query(format!("{}-{}", year, m_value));
                                                }
                                            });
                                        }
                                    >
                                        {m_label}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any();
                }
                view! {
                    <div class="flex items-center gap-1">
                        <select
                            class="h-9 px-3 border border-border-strong bg-surface rounded-lg text-sm max-sm:min-h-11"
                            aria-label="年"
                            prop:value=move || selected_year.get()
                            on:change=move |ev| search.update(|s| s.select_year(event_target_value(&ev)))
                        >
                            <option value="">"年を選択"</option>
                            {years.iter().map(|y| {
                                view! { <option value={y.value.clone()}>{y.label.clone()}</option> }
                            }).collect_view()}
                        </select>
                        <select
                            class="h-9 px-3 border border-border-strong bg-surface rounded-lg text-sm max-sm:min-h-11"
                            aria-label="月"
                            prop:value=move || selected_month.get()
                            on:change=move |ev| {
                                let month = event_target_value(&ev);
                                search.update(|s| {
                                    s.date_inputs.month_value = month.clone();
                                    let year = s.date_inputs.year_value.clone();
                                    if year.is_empty() || month.is_empty() {
                                        s.set_date_query(year);
                                    } else {
                                        s.set_date_query(format!("{}-{}", year, month));
                                    }
                                });
                            }
                        >
                            <option value="">"月を選択"</option>
                            {(1..=12).map(|m| {
                                let m_str = format!("{:02}", m);
                                let m_label = format!("{}月", m);
                                view! { <option value={m_str.clone()}>{m_label}</option> }
                            }).collect_view()}
                        </select>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn DatePicker(search: RwSignal<ReceiptSearch>) -> impl IntoView {
    let selected_date = Memo::new(move |_| search.with(|s| s.date_inputs.date_value.clone()));

    view! {
        <div class="mt-2 max-w-xs">
            <label class="block text-xs text-text-muted mb-1">"日付"</label>
            <input
                type="date"
                class="h-10 px-3 border border-border-strong bg-surface rounded-lg text-sm w-full max-sm:min-h-11"
                aria-label="日付"
                data-empty=move || selected_date.get().is_empty().then_some("true")
                prop:value=move || selected_date.get()
                on:input=move |ev| {
                    search.update(|s| {
                        s.date_inputs.date_value = event_target_value(&ev);
                        s.set_date_query(event_target_value(&ev));
                    });
                }
            />
        </div>
    }
}

#[component]
fn RangePicker(search: RwSignal<ReceiptSearch>) -> impl IntoView {
    let range_start = Memo::new(move |_| search.with(|s| s.date_inputs.range_start.clone()));
    let range_end = Memo::new(move |_| search.with(|s| s.date_inputs.range_end.clone()));

    view! {
        <div class="mt-2 flex flex-wrap gap-2 items-end">
            <div class="flex-1 min-w-40">
                <label class="block text-xs text-text-muted mb-1">"開始日"</label>
                <input
                    type="date"
                    class="h-10 px-3 border border-border-strong bg-surface rounded-lg text-sm w-full max-sm:min-h-11"
                    aria-label="開始日"
                    data-empty=move || range_start.get().is_empty().then_some("true")
                    prop:value=move || range_start.get()
                    on:input=move |ev| {
                        search.update(|s| {
                            s.date_inputs.range_start = event_target_value(&ev);
                            s.update_range_query();
                        });
                    }
                />
            </div>
            <span class="text-text-muted">"〜"</span>
            <div class="flex-1 min-w-40">
                <label class="block text-xs text-text-muted mb-1">"終了日"</label>
                <input
                    type="date"
                    class="h-10 px-3 border border-border-strong bg-surface rounded-lg text-sm w-full max-sm:min-h-11"
                    aria-label="終了日"
                    data-empty=move || range_end.get().is_empty().then_some("true")
                    prop:value=move || range_end.get()
                    on:input=move |ev| {
                        search.update(|s| {
                            s.date_inputs.range_end = event_target_value(&ev);
                            s.update_range_query();
                        });
                    }
                />
            </div>
        </div>
    }
}

/// 小窓・シートの開閉状態と、閉じた後にフォーカスを戻すトリガーのボタン。
/// PC・スマホで同じ構造を使う
#[derive(Clone, Copy)]
struct ConditionControls {
    open: RwSignal<Option<ConditionKind>>,
    period_ref: NodeRef<leptos::html::Button>,
    account_ref: NodeRef<leptos::html::Button>,
    product_ref: NodeRef<leptos::html::Button>,
}

impl ConditionControls {
    fn new() -> Self {
        Self {
            open: RwSignal::new(None),
            period_ref: NodeRef::new(),
            account_ref: NodeRef::new(),
            product_ref: NodeRef::new(),
        }
    }

    /// Esc・確定で閉じ、トリガーのボタンへフォーカスを戻す
    fn finish(&self, kind: ConditionKind) {
        self.open.set(None);
        let reference = match kind {
            ConditionKind::Period => self.period_ref,
            ConditionKind::Account => self.account_ref,
            ConditionKind::Product => self.product_ref,
        };
        if let Some(button) = reference.get() {
            let _ = button.focus();
        }
    }
}

/// 取引明細の検索欄。検索欄と条件の行を1つの段にまとめ、件数は条件の行の右端に置く。
/// PC・スマホとも同じ作りを使う
#[component]
pub fn ReceiptsSearchBar(
    store: ReceiptsStore,
    tab: ReceiptsTab,
    #[prop(optional, default = "h-4 w-4")] icon_size: &'static str,
) -> impl IntoView {
    let search = store.search;

    let display_rows = Memo::new(move |_| match store.tab_state(tab) {
        TabState::Ready(data) => data.rows.clone(),
        _ => Vec::new(),
    });

    let categories = Memo::new(move |_| search_categories(tab, &display_rows.get()));
    let years = Memo::new(move |_| categories.with(|c| c.years.clone()));
    let products = Memo::new(move |_| categories.with(|c| c.products.clone()));
    let accounts = Memo::new(move |_| categories.with(|c| c.accounts.clone()));
    let has_years = move || !years.get().is_empty();

    let controls = ConditionControls::new();
    let on_close = move |kind: ConditionKind| controls.finish(kind);

    view! {
        <div class="pc-search-area no-print">
            <SearchBox icon_size=icon_size>
                <ConditionTags search=search has_years=has_years />
                <SecuritiesInput search=search />
            </SearchBox>
            <AddConditionRow show_label=true>
                <ReceiptAddConditionButton kind=ConditionKind::Period open=controls.open node_ref=controls.period_ref />
                {move || {
                    (!accounts.get().is_empty()).then(|| {
                        view! {
                            <ReceiptAddConditionButton kind=ConditionKind::Account open=controls.open node_ref=controls.account_ref />
                        }
                    })
                }}
                {move || {
                    (!products.get().is_empty()).then(|| {
                        view! {
                            <ReceiptAddConditionButton kind=ConditionKind::Product open=controls.open node_ref=controls.product_ref />
                        }
                    })
                }}
                {move || {
                    controls.open.get().map(|kind| {
                        view! {
                            <ConditionPopover
                                kind=kind
                                search=search
                                years=years
                                products=products
                                accounts=accounts
                                has_years=has_years
                                on_close=on_close
                            />
                        }
                    })
                }}
                <ConditionRowEnd>
                    <ReceiptSearchCount store=store tab=tab />
                    <ClearAllButton search=search has_years=has_years />
                </ConditionRowEnd>
            </AddConditionRow>
        </div>
    }
}

/// スマホの下部操作帯。検索欄・条件を足す・件数とすべてクリアを常設する
#[component]
pub fn ReceiptsMobileBottomSection(store: ReceiptsStore) -> impl IntoView {
    let tab = store.active_tab;

    view! {
        <div class="receipts-mobile-bottom-section no-print lg:hidden" role="search" aria-label="検索">
            <div class="page-gutter">
                <ReceiptsSearchBar store=store tab=tab.get() icon_size="h-5 w-5" />
            </div>
        </div>
    }
}
