use crate::features::receipts::filter::{
    DateSegment, ReceiptSearch, ReceiptSortOrder, SearchKey, filter_receipts, search_categories,
};
use crate::features::receipts::kind::group_label;
use crate::features::receipts::{ReceiptsStore, ReceiptsTab, TabState};
use crate::support::list_search::SearchOption;
use crate::ui::badge::account_display_name;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::labels::{
    self, ACCOUNT, ALL, CONDITION, DATE, END_DATE, MONTH, PAYMENT_DATE, PERIOD, PERIOD_DAY,
    PERIOD_KIND_LABEL, PERIOD_RANGE, PERIOD_YEAR, PERIOD_YEAR_MONTH, PRODUCT, RANGE_SEPARATOR,
    RECEIPT_APPLY_PERIOD, RECEIPT_MONTH_SELECT, RECEIPT_NO_YEARS, RECEIPT_SEARCH_TARGET,
    RECEIPT_YEAR_SELECT, SEARCH, SEARCH_AND_SORT, SORT_BY, SORT_NEWEST_FIRST, SORT_OLDEST_FIRST,
    SORT_ORDER_LABEL, SORT_TARGET, START_DATE, TRADE_DATE, receipt_month_label,
    receipt_range_between, receipt_range_from, receipt_range_until, receipt_year_label,
};
use crate::ui::search_controls::{
    AddConditionButton, AddConditionRow, ConditionOptionList, ConditionPanel, ConditionRowEnd,
    SearchBox, SearchClearButton, SearchCount, SearchTag, SearchTextInput,
};
use crate::ui::search_dialog::SearchDialog;
use crate::ui::search_entry::MobileSearchEntryShell;
use crate::ui::select::Select;
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
    const fn label(self) -> &'static str {
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
            (true, _) => receipt_range_until(&end),
            (_, true) => receipt_range_from(&start),
            (false, false) => receipt_range_between(&start, &end),
        };
    }
    date_part_text(value)
}

fn date_part_text(value: &str) -> String {
    if value.len() == 4 {
        receipt_year_label(value)
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
            placeholder=labels::search_placeholder(RECEIPT_SEARCH_TARGET)
            value=move || search.with(|s| s.selected_queries.get(SearchKey::Securities).to_string())
            on_input=move |value| search.update(|s| s.set_securities_query(value))
        />
    }
}

/// 絞り込み中の件数と全件数。取得中は出さない
#[component]
fn ReceiptSearchCount(
    store: ReceiptsStore,
    #[prop(into)] tab: Signal<ReceiptsTab>,
) -> impl IntoView {
    let count = Memo::new(move |_| match store.tab_state(tab.get()) {
        TabState::Ready(data) => {
            let query = store.search.with(|s| s.query.clone());
            let total = data.total.unwrap_or(data.rows.len());
            Some((filter_receipts(tab.get(), &data.rows, &query).len(), total))
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
                            {RECEIPT_APPLY_PERIOD}
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
    let segment_options = [
        ALL,
        PERIOD_YEAR,
        PERIOD_YEAR_MONTH,
        PERIOD_DAY,
        PERIOD_RANGE,
    ];

    view! {
        <div class="condition-segment" role="group" aria-label=PERIOD_KIND_LABEL>
            {segment_options
                .iter()
                .map(|opt| {
                    let opt = *opt;
                    let is_selected = move || {
                        draft.with(|s| {
                            if s.date_all {
                                opt == ALL
                            } else {
                                s.date_segment.value() == opt
                            }
                        })
                    };
                    view! {
                        <Button
                            variant=ButtonVariant::ConditionSegment
                            aria_pressed=Signal::derive(move || is_selected().to_string())
                            on_click=move |_| {
                                draft.update(|s| {
                                    if opt == ALL {
                                        s.clear_key(SearchKey::Date, has_years())
                                    } else if opt == PERIOD_YEAR {
                                        s.change_date_segment(DateSegment::Year)
                                    } else if opt == PERIOD_YEAR_MONTH {
                                        s.change_date_segment(DateSegment::Month)
                                    } else if opt == PERIOD_DAY {
                                        s.change_date_segment(DateSegment::Date)
                                    } else {
                                        s.change_date_segment(DateSegment::Range)
                                    }
                                })
                            }
                        >
                            {opt}
                        </Button>
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
                    return view! { <span class="text-sm text-text-muted">{RECEIPT_NO_YEARS}</span> }.into_any();
                }
                view! {
                    {years.iter().map(|y| {
                        let y_value = y.value.clone();
                        let y_label = y.label.clone();
                        let y_value_for_selected = y_value.clone();
                        let selected = Memo::new(move |_| search.with(|s| s.date_inputs.year_value.clone() == y_value_for_selected));
                        view! {
                            <Button
                                variant=ButtonVariant::YearOption
                                aria_pressed=Signal::derive(move || selected.get().to_string())
                                on_click=move |_| search.update(|s| s.select_year(y_value.clone()))
                            >
                                {y_label}
                            </Button>
                        }
                    }).collect_view()}
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn MonthPicker(search: RwSignal<ReceiptSearch>, years: Memo<Vec<SearchOption>>) -> impl IntoView {
    let months = (1..=12).map(|m| format!("{m:02}")).collect::<Vec<_>>();
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
                                let m_label = receipt_month_label(m_value.parse::<u32>().unwrap_or(0));
                                let m_value_for_selected = m_value.clone();
                                let is_selected = move || selected_month.get() == m_value_for_selected;
                                view! {
                                    <Button
                                        variant=ButtonVariant::YearOption
                                        aria_pressed=Signal::derive(move || is_selected().to_string())
                                        on_click=move |_| {
                                            search.update(|s| {
                                                s.date_inputs.month_value = m_value.clone();
                                                let year = s.date_inputs.year_value.clone();
                                                if !year.is_empty() {
                                                    s.set_date_query(format!("{year}-{m_value}"));
                                                }
                                            });
                                        }
                                    >
                                        {m_label}
                                    </Button>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any();
                }
                view! {
                    <div class="flex items-center gap-1">
                        <select
                            class="h-9 px-3 border border-border-strong bg-surface rounded-lg text-sm max-sm:min-h-11"
                            aria-label=PERIOD_YEAR
                            prop:value=move || selected_year.get()
                            on:change=move |ev| search.update(|s| s.select_year(event_target_value(&ev)))
                        >
                            <option value="">{RECEIPT_YEAR_SELECT}</option>
                            {years.iter().map(|y| {
                                view! { <option value={y.value.clone()}>{y.label.clone()}</option> }
                            }).collect_view()}
                        </select>
                        <select
                            class="h-9 px-3 border border-border-strong bg-surface rounded-lg text-sm max-sm:min-h-11"
                            aria-label=MONTH
                            prop:value=move || selected_month.get()
                            on:change=move |ev| {
                                let month = event_target_value(&ev);
                                search.update(|s| {
                                    s.date_inputs.month_value = month.clone();
                                    let year = s.date_inputs.year_value.clone();
                                    if year.is_empty() || month.is_empty() {
                                        s.set_date_query(year);
                                    } else {
                                        s.set_date_query(format!("{year}-{month}"));
                                    }
                                });
                            }
                        >
                            <option value="">{RECEIPT_MONTH_SELECT}</option>
                            {(1..=12).map(|m| {
                                let m_str = format!("{m:02}");
                                let m_label = receipt_month_label(m);
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
            <label class="block text-xs text-text-muted mb-1">{DATE}</label>
            <input
                type="date"
                class="h-10 px-3 border border-border-strong bg-surface rounded-lg text-sm w-full max-sm:min-h-11"
                aria-label=DATE
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
                <label class="block text-xs text-text-muted mb-1">{START_DATE}</label>
                <input
                    type="date"
                    class="h-10 px-3 border border-border-strong bg-surface rounded-lg text-sm w-full max-sm:min-h-11"
                    aria-label=START_DATE
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
            <span class="text-text-muted">{RANGE_SEPARATOR}</span>
            <div class="flex-1 min-w-40">
                <label class="block text-xs text-text-muted mb-1">{END_DATE}</label>
                <input
                    type="date"
                    class="h-10 px-3 border border-border-strong bg-surface rounded-lg text-sm w-full max-sm:min-h-11"
                    aria-label=END_DATE
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
    #[prop(into)] tab: Signal<ReceiptsTab>,
    #[prop(optional, default = "h-4 w-4")] icon_size: &'static str,
) -> impl IntoView {
    let search = store.search;

    let display_rows = Memo::new(move |_| match store.tab_state(tab.get()) {
        TabState::Ready(data) => data.rows.clone(),
        _ => Vec::new(),
    });

    let categories = Memo::new(move |_| search_categories(tab.get(), &display_rows.get()));
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

/// 下書きを一覧へ反映する。ダイアログの検索ボタンと単体テストの共通入口
pub(crate) fn apply_receipt_sheet_draft(search: RwSignal<ReceiptSearch>, draft: &ReceiptSearch) {
    search.set(draft.clone());
}

/// ダイアログのクリアは下書きの検索条件だけを消す
pub(crate) fn clear_receipt_dialog_draft(draft: RwSignal<ReceiptSearch>, has_years: bool) {
    draft.update(|s| s.clear(has_years));
}

/// タブの日付の列見出し。入口とシートの並べ替え表示で使う
pub(crate) const fn receipt_date_label(tab: ReceiptsTab) -> &'static str {
    match tab {
        ReceiptsTab::Dividend => PAYMENT_DATE,
        ReceiptsTab::DomesticStock | ReceiptsTab::MutualFund => TRADE_DATE,
    }
}

/// スマホの検索・並べ替えの入口とダイアログ。資産管理と同じく帯のすぐ下に置く。
/// PC の検索欄・条件の行は変えず、スマホ幅では入口の1行と条件タグだけを出し中身は中央のダイアログで編集する
#[component]
pub fn ReceiptsMobileSearchEntry(store: ReceiptsStore) -> impl IntoView {
    let tab = store.active_tab;
    let search = store.search;
    let sort_order = store.sort_order;
    let open = RwSignal::new(false);

    let display_rows = Memo::new(move |_| match store.tab_state(tab.get()) {
        TabState::Ready(data) => data.rows.clone(),
        _ => Vec::new(),
    });
    let categories = Memo::new(move |_| search_categories(tab.get(), &display_rows.get()));
    let years = Memo::new(move |_| categories.with(|c| c.years.clone()));
    let products = Memo::new(move |_| categories.with(|c| c.products.clone()));
    let accounts = Memo::new(move |_| categories.with(|c| c.accounts.clone()));
    let has_years = move || !years.get().is_empty();
    let has_years_for_tags = has_years;
    let sort_summary = move || {
        let order = match sort_order.get() {
            ReceiptSortOrder::Newest => SORT_NEWEST_FIRST,
            ReceiptSortOrder::Oldest => SORT_OLDEST_FIRST,
        };
        format!("{} {order}", receipt_date_label(tab.get()))
    };

    let sort_summary = Signal::derive(sort_summary);
    view! {
        <MobileSearchEntryShell
            entry_testid="receipts-search-sheet-entry"
            search_label=SEARCH.to_string()
            sort_summary=sort_summary
            open=open
        >
            <ConditionTags search=search has_years=has_years_for_tags />
            <ReceiptSearchCount store=store tab=tab />
        </MobileSearchEntryShell>
        {move || {
            open.get()
                .then(|| {
                    let draft = RwSignal::new(search.get_untracked());
                    let draft_order = RwSignal::new(sort_order.get_untracked());
                    let do_search = move || {
                        apply_receipt_sheet_draft(search, &draft.get_untracked());
                        sort_order.set(draft_order.get_untracked());
                        open.set(false);
                    };
                    let years_flag = has_years();
                    let do_clear = move || clear_receipt_dialog_draft(draft, years_flag);
                    view! {
                        <SearchDialog
                            title=SEARCH_AND_SORT.to_string()
                            on_close=move || open.set(false)
                            on_search=do_search
                            on_clear=do_clear
                            testid="receipts-search-sheet"
                        >
                            <ReceiptDialogFields
                                draft=draft
                                draft_order=draft_order
                                tab=tab
                                years=years
                                products=products
                                accounts=accounts
                                has_years=has_years
                                on_search=do_search
                            />
                        </SearchDialog>
                    }
                })
        }}
    }
}

/// ダイアログの中身。開くたびに作り直され、下書きの上で編集して検索で反映する
#[component]
fn ReceiptDialogFields(
    draft: RwSignal<ReceiptSearch>,
    draft_order: RwSignal<ReceiptSortOrder>,
    #[prop(into)] tab: Signal<ReceiptsTab>,
    years: Memo<Vec<SearchOption>>,
    products: Memo<Vec<SearchOption>>,
    accounts: Memo<Vec<SearchOption>>,
    has_years: impl Fn() -> bool + Copy + 'static + Send + Sync,
    on_search: impl Fn() + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let draft_text = move || draft.with(|s| s.selected_queries.securities.clone());
    let set_draft_text = move |value: String| draft.update(|s| s.set_securities_query(value));
    let search_callback = Callback::new(move |_| on_search());
    let date_label = move || receipt_date_label(tab.get());
    let period_value = Memo::new(move |_| {
        let date = draft.with(|s| {
            if s.date_all {
                String::new()
            } else {
                s.selected_queries.date.clone()
            }
        });
        if date.is_empty() {
            return String::new();
        }
        let is_year = years.get().into_iter().any(|item| item.value == date);
        if is_year { date } else { String::new() }
    });
    let account_value = Memo::new(move |_| draft.with(|s| s.selected_queries.accounts.clone()));
    let product_value = Memo::new(move |_| draft.with(|s| s.selected_queries.products.clone()));
    view! {
        <div class="flex flex-col gap-4">
            <section aria-label=SEARCH>
                <p class="search-dialog-section-title">{SEARCH}</p>
                <div class="search-dialog-field">
                    <SearchBox>
                        <SearchTextInput
                            placeholder=labels::search_placeholder(RECEIPT_SEARCH_TARGET)
                            testid="receipts-search-input"
                            value=draft_text
                            on_input=set_draft_text
                            on_enter=search_callback
                        />
                    </SearchBox>
                    <p class="search-dialog-hint">{labels::search_placeholder(RECEIPT_SEARCH_TARGET)}</p>
                </div>
            </section>
            <section aria-label=CONDITION>
                <p class="search-dialog-section-title">{CONDITION}</p>
                <div class="search-dialog-field">
                    <label class="mb-1 block text-sm font-medium text-text-deep">
                        {PERIOD}
                        <Select
                            value=Signal::derive(move || period_value.get())
                            aria_label=PERIOD.to_string()
                            testid="receipts-dialog-period"
                            class="search-dialog-select".to_string()
                            on_change=move |next: String| {
                                draft.update(|s| {
                                    if next.is_empty() {
                                        s.clear_key(SearchKey::Date, has_years());
                                    } else {
                                        s.date_segment = DateSegment::Year;
                                        s.date_all = false;
                                        s.select_year(next);
                                    }
                                });
                            }
                        >
                            <option value="">{ALL}</option>
                            {move || {
                                years
                                    .get()
                                    .into_iter()
                                    .map(|item| {
                                        let value = item.value.clone();
                                        let label = item.label.clone();
                                        view! { <option value=value>{label}</option> }
                                    })
                                    .collect_view()
                            }}
                        </Select>
                    </label>
                </div>
                {move || {
                    (!accounts.get().is_empty())
                        .then(|| {
                            view! {
                                <div class="search-dialog-field">
                                    <label class="mb-1 block text-sm font-medium text-text-deep">
                                        {ACCOUNT}
                                        <Select
                                            value=Signal::derive(move || account_value.get())
                                            aria_label=ACCOUNT.to_string()
                                            testid="receipts-dialog-account"
                                            class="search-dialog-select".to_string()
                                            on_change=move |next: String| {
                                                draft.update(|s| {
                                                    if next.is_empty() {
                                                        s.clear_key(SearchKey::Accounts, has_years());
                                                    } else {
                                                        s.selected_queries.accounts = next;
                                                        s.rebuild_query();
                                                    }
                                                });
                                            }
                                        >
                                            <option value="">{ALL}</option>
                                            {move || {
                                                accounts
                                                    .get()
                                                    .into_iter()
                                                    .map(|item| {
                                                        let value = item.value.clone();
                                                        let label = item.label.clone();
                                                        view! { <option value=value>{label}</option> }
                                                    })
                                                    .collect_view()
                                            }}
                                        </Select>
                                    </label>
                                </div>
                            }
                        })
                }}
                {move || {
                    (!products.get().is_empty())
                        .then(|| {
                            view! {
                                <div class="search-dialog-field">
                                    <label class="mb-1 block text-sm font-medium text-text-deep">
                                        {PRODUCT}
                                        <Select
                                            value=Signal::derive(move || product_value.get())
                                            aria_label=PRODUCT.to_string()
                                            testid="receipts-dialog-product"
                                            class="search-dialog-select".to_string()
                                            on_change=move |next: String| {
                                                draft.update(|s| {
                                                    if next.is_empty() {
                                                        s.clear_key(SearchKey::Products, has_years());
                                                    } else {
                                                        s.selected_queries.products = next;
                                                        s.rebuild_query();
                                                    }
                                                });
                                            }
                                        >
                                            <option value="">{ALL}</option>
                                            {move || {
                                                products
                                                    .get()
                                                    .into_iter()
                                                    .map(|item| {
                                                        let value = item.value.clone();
                                                        let label = item.label.clone();
                                                        view! { <option value=value>{label}</option> }
                                                    })
                                                    .collect_view()
                                            }}
                                        </Select>
                                    </label>
                                </div>
                            }
                        })
                }}
            </section>
            <section aria-label=SORT_BY.to_string()>
                <p class="search-dialog-section-title">{SORT_BY}</p>
                <div class="search-dialog-sort-row">
                    <div>
                        <label class="mb-1 block text-sm font-medium text-text-deep">
                            {SORT_TARGET}
                            <Select
                                value=Signal::derive(move || "date".to_string())
                                aria_label=SORT_TARGET.to_string()
                                testid="receipts-dialog-sort-target"
                                class="search-dialog-select".to_string()
                                on_change=move |_| {}
                            >
                                <option value="date">{date_label}</option>
                            </Select>
                        </label>
                    </div>
                    <div>
                        <label class="mb-1 block text-sm font-medium text-text-deep">
                            {SORT_ORDER_LABEL}
                            <Select
                                value=Signal::derive(move || draft_order.get().value().to_string())
                                aria_label=SORT_ORDER_LABEL.to_string()
                                testid="receipts-dialog-sort-order"
                                class="search-dialog-select".to_string()
                                on_change=move |next: String| {
                                    if let Some(order) = ReceiptSortOrder::from_value(&next) {
                                        draft_order.set(order);
                                    }
                                }
                            >
                                <option value="newest">{SORT_NEWEST_FIRST}</option>
                                <option value="oldest">{SORT_OLDEST_FIRST}</option>
                            </Select>
                        </label>
                    </div>
                </div>
            </section>
        </div>
    }
}
