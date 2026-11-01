//! 資産管理の検索欄。取引明細と同じタグ入力+「条件を足す」の操作に揃え、
//! 部品は ui::search_controls を共有する

use crate::features::asset_balance::fund::AssetSearch;
use crate::features::asset_balance::portfolio::SortOrder;
use crate::support::list_search::SearchOption;
use crate::ui::badge::account_display_name;
use crate::ui::bottom_sheet::BottomSheet;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::labels::{
    ACCOUNT, DISTRIBUTION_COURSE, SEARCH_AND_SORT, SHEET_APPLY, SHEET_CLEAR, SORT_BY,
    SORT_HIGH_TO_LOW, SORT_LOW_TO_HIGH,
};
use crate::ui::search_controls::{
    AddConditionButton, AddConditionRow, ConditionOptionList, ConditionPanel, ConditionRowEnd,
    SearchBox, SearchClearButton, SearchCount, SearchTag, SearchTextInput,
};
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConditionKind {
    Account,
    Course,
}

impl ConditionKind {
    fn label(self) -> &'static str {
        match self {
            ConditionKind::Account => ACCOUNT,
            ConditionKind::Course => DISTRIBUTION_COURSE,
        }
    }

    fn title(self) -> &'static str {
        match self {
            ConditionKind::Account => "口座を選ぶ",
            ConditionKind::Course => "分配金コースを選ぶ",
        }
    }

    fn testid(self) -> &'static str {
        match self {
            ConditionKind::Account => "asset-condition-account",
            ConditionKind::Course => "asset-condition-course",
        }
    }
}

/// 小窓の開閉状態と、閉じた後にフォーカスを戻すトリガーのボタン
#[derive(Clone, Copy)]
struct ConditionControls {
    open: RwSignal<Option<ConditionKind>>,
    account_ref: NodeRef<leptos::html::Button>,
    course_ref: NodeRef<leptos::html::Button>,
}

impl ConditionControls {
    fn new() -> Self {
        Self {
            open: RwSignal::new(None),
            account_ref: NodeRef::new(),
            course_ref: NodeRef::new(),
        }
    }

    fn toggle(&self, kind: ConditionKind) {
        if self.open.get_untracked() == Some(kind) {
            self.open.set(None);
        } else {
            self.open.set(Some(kind));
        }
    }

    /// Esc・背景クリック・閉じるのどれで閉じても、トリガーのボタンへフォーカスを戻す
    fn finish(&self, kind: ConditionKind) {
        self.open.set(None);
        let reference = match kind {
            ConditionKind::Account => self.account_ref,
            ConditionKind::Course => self.course_ref,
        };
        if let Some(button) = reference.get() {
            let _ = button.focus();
        }
    }
}

/// 条件を足す小窓の中身。口座・分配金コースは複数選択でき、
/// 選んだ状態はすぐ反映する(小窓は開いたまま)
#[component]
fn AssetConditionPopover(
    kind: ConditionKind,
    search: RwSignal<AssetSearch>,
    options: Memo<Vec<SearchOption>>,
    on_close: impl Fn() + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let selected = Memo::new(move |_| {
        search.with(|s| match kind {
            ConditionKind::Account => s.accounts.clone(),
            ConditionKind::Course => s.courses.clone(),
        })
    });
    let toggle = move |value: String| {
        search.update(|s| {
            let list = match kind {
                ConditionKind::Account => &mut s.accounts,
                ConditionKind::Course => &mut s.courses,
            };
            if let Some(index) = list.iter().position(|item| *item == value) {
                list.remove(index);
            } else {
                list.push(value);
            }
        });
    };
    view! {
        <ConditionPanel title=kind.title() on_close=on_close>
            <ConditionOptionList options=options selected=selected on_toggle=toggle />
            <div class="condition-panel-actions">
                <Button
                    variant=ButtonVariant::Secondary(ButtonSize::Md)
                    on_click=move |_| on_close()
                >
                    "閉じる"
                </Button>
            </div>
        </ConditionPanel>
    }
}

/// 検索欄(タグ入力+条件を足す)。PC・スマホ共通。
/// 条件ボタンは `conditions` に挙げた条件だけ出す
#[component]
pub(crate) fn AssetSearchBar(
    search: RwSignal<AssetSearch>,
    search_count: RwSignal<(usize, usize)>,
    #[prop(into)] placeholder: String,
    conditions: &'static [ConditionKind],
    account_options: Memo<Vec<String>>,
    course_options: Memo<Vec<String>>,
) -> impl IntoView {
    let controls = ConditionControls::new();
    let accounts_tag = Memo::new(move |_| {
        search.with(|s| {
            s.accounts
                .iter()
                .map(|value| account_display_name(value))
                .collect::<Vec<_>>()
                .join("・")
        })
    });
    let courses_tag = Memo::new(move |_| search.with(|s| s.courses.join("・")));
    let has_anything = Signal::derive(move || search.with(|s| !s.is_empty()));
    let on_clear_all = move || search.update(|s| *s = AssetSearch::default());
    let input_value = move || search.with(|s| s.text.clone());
    let on_input = move |value: String| search.update(|s| s.text = value);
    let to_options = |options: Memo<Vec<String>>, to_label: fn(&str) -> &str| {
        Memo::new(move |_| {
            options
                .get()
                .into_iter()
                .map(|value| SearchOption {
                    label: to_label(&value).to_string(),
                    value,
                })
                .collect()
        })
    };
    let account_options = to_options(account_options, account_display_name);
    let course_options = to_options(course_options, |value| value);
    let placeholder_pc = placeholder.clone();

    let condition_button = move |kind: ConditionKind, node_ref: NodeRef<leptos::html::Button>| {
        view! {
            <AddConditionButton
                label=kind.label()
                expanded=move || controls.open.get() == Some(kind)
                node_ref=node_ref
                testid=kind.testid()
                on_click=move || controls.toggle(kind)
            />
        }
    };

    view! {
        <div class="no-print" role="search" aria-label="資産の検索">
            // PC: タグ入力+条件を足す+件数を1つの段にまとめる(取引明細と同じ構造)
            <SearchBox class="hidden lg:flex">
                <SearchTag
                    label=ACCOUNT
                    value=accounts_tag
                    on_clear=move || search.update(|s| s.accounts.clear())
                />
                <SearchTag
                    label=DISTRIBUTION_COURSE
                    value=courses_tag
                    on_clear=move || search.update(|s| s.courses.clear())
                />
                <SearchTextInput
                    placeholder=placeholder_pc
                    testid="asset-search-input-pc"
                    value=input_value
                    on_input=on_input
                />
            </SearchBox>
            <div class="hidden lg:block">
                <AddConditionRow show_label=!conditions.is_empty()>
                    {conditions.iter().map(|kind| {
                        let node_ref = match kind {
                            ConditionKind::Account => controls.account_ref,
                            ConditionKind::Course => controls.course_ref,
                        };
                        condition_button(*kind, node_ref)
                    }).collect_view()}
                    {move || {
                        controls.open.get().map(|kind| {
                            let options = match kind {
                                ConditionKind::Account => account_options,
                                ConditionKind::Course => course_options,
                            };
                            view! {
                                <AssetConditionPopover
                                    kind=kind
                                    search=search
                                    options=options
                                    on_close=move || controls.finish(kind)
                                />
                            }
                        })
                    }}
                    <ConditionRowEnd>
                        <SearchCount
                            filtered=move || search_count.get().0
                            total=move || search_count.get().1
                            testid="asset-search-count"
                        />
                        <SearchClearButton
                            disabled=move || !has_anything.get()
                            testid="asset-search-clear"
                            on_clear=on_clear_all
                        />
                    </ConditionRowEnd>
                </AddConditionRow>
            </div>
        </div>
    }
}

/// シートの下書きで条件を選ぶ。選んでいれば外し、無ければ足す
pub(crate) fn toggle_sheet_condition(list: &mut Vec<String>, value: String) {
    if let Some(index) = list.iter().position(|item| *item == value) {
        list.remove(index);
    } else {
        list.push(value);
    }
}

/// 下書きを一覧へ反映する。シートの適用ボタンと単体テストの共通入口
pub(crate) fn apply_asset_sheet_draft(search: RwSignal<AssetSearch>, draft: &AssetSearch) {
    search.set(draft.clone());
}

/// スマホの検索・並べ替えの入口とシート。PC の検索欄・条件の行は変えず、
/// スマホ幅ではこの1行と条件タグだけを出し、中身はシートで編集する
#[component]
pub(crate) fn AssetMobileSearchEntry<K: Copy + PartialEq + Send + Sync + 'static>(
    search: RwSignal<AssetSearch>,
    search_count: RwSignal<(usize, usize)>,
    #[prop(into)] placeholder: String,
    conditions: &'static [ConditionKind],
    account_options: Memo<Vec<String>>,
    course_options: Memo<Vec<String>>,
    sort_options: &'static [K],
    key_label: fn(K) -> &'static str,
    key_value: fn(K) -> &'static str,
    sort_key: RwSignal<K>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let entry_ref = NodeRef::<leptos::html::Button>::new();
    let accounts_tag = Memo::new(move |_| {
        search.with(|s| {
            s.accounts
                .iter()
                .map(|value| account_display_name(value))
                .collect::<Vec<_>>()
                .join("・")
        })
    });
    let courses_tag = Memo::new(move |_| search.with(|s| s.courses.join("・")));
    let sort_summary = move || {
        let order = match sort_order.get() {
            SortOrder::Desc => SORT_HIGH_TO_LOW,
            SortOrder::Asc => SORT_LOW_TO_HIGH,
        };
        format!("{} {order}", key_label(sort_key.get()))
    };
    let sheet_placeholder = placeholder.clone();
    view! {
        <div class="no-print lg:hidden" role="search" aria-label="資産の検索">
            <Button
                variant=ButtonVariant::Secondary(ButtonSize::Md)
                class="search-sort-entry".to_string()
                aria_expanded=Signal::derive(move || open.get())
                testid="asset-search-sheet-entry"
                node_ref=entry_ref
                on_click=move |_| open.set(true)
            >
                <span class="search-sort-entry-title">
                    <svg aria-hidden="true" class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="11" cy="11" r="7"></circle>
                        <path d="M20 20l-3.5-3.5"></path>
                    </svg>
                    {SEARCH_AND_SORT}
                </span>
                <span class="search-sort-entry-sub">{sort_summary}</span>
            </Button>
            <div class="search-sort-entry-tags">
                <SearchTag
                    label=ACCOUNT
                    value=accounts_tag
                    on_clear=move || search.update(|s| s.accounts.clear())
                />
                <SearchTag
                    label=DISTRIBUTION_COURSE
                    value=courses_tag
                    on_clear=move || search.update(|s| s.courses.clear())
                />
                <SearchCount
                    filtered=move || search_count.get().0
                    total=move || search_count.get().1
                    testid="asset-search-count-mobile"
                />
            </div>
            {move || {
                let sheet_text = sheet_placeholder.clone();
                open.get()
                    .then(|| {
                        view! {
                            <BottomSheet
                                title=SEARCH_AND_SORT.to_string()
                                on_close=move || open.set(false)
                                testid="asset-search-sheet"
                            >
                                <AssetSheetBody
                                    search=search
                                    sort_key=sort_key
                                    sort_order=sort_order
                                    sort_options=sort_options
                                    key_label=key_label
                                    key_value=key_value
                                    conditions=conditions
                                    account_options=account_options
                                    course_options=course_options
                                    placeholder=sheet_text.clone()
                                    on_close=move || open.set(false)
                                />
                            </BottomSheet>
                        }
                    })
            }}
        </div>
    }
}

/// シートの中身。開くたびに作り直され、下書きの上で編集して適用で反映する
#[component]
fn AssetSheetBody<K: Copy + PartialEq + Send + Sync + 'static>(
    search: RwSignal<AssetSearch>,
    sort_key: RwSignal<K>,
    sort_order: RwSignal<SortOrder>,
    sort_options: &'static [K],
    key_label: fn(K) -> &'static str,
    key_value: fn(K) -> &'static str,
    conditions: &'static [ConditionKind],
    account_options: Memo<Vec<String>>,
    course_options: Memo<Vec<String>>,
    #[prop(into)] placeholder: String,
    on_close: impl Fn() + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let draft = RwSignal::new(search.get_untracked());
    let draft_key = RwSignal::new(sort_key.get_untracked());
    let draft_order = RwSignal::new(sort_order.get_untracked());
    let to_options = |options: Memo<Vec<String>>, to_label: fn(&str) -> &str| {
        Memo::new(move |_| {
            options
                .get()
                .into_iter()
                .map(|value| SearchOption {
                    label: to_label(&value).to_string(),
                    value,
                })
                .collect()
        })
    };
    let account_list = to_options(account_options, account_display_name);
    let course_list = to_options(course_options, |value| value);
    let draft_text = move || draft.with(|s| s.text.clone());
    let set_draft_text = move |value: String| draft.update(|s| s.text = value);
    let toggle_draft = move |kind: ConditionKind, value: String| {
        draft.update(|s| {
            toggle_sheet_condition(
                match kind {
                    ConditionKind::Account => &mut s.accounts,
                    ConditionKind::Course => &mut s.courses,
                },
                value,
            );
        });
    };
    let apply = move || {
        apply_asset_sheet_draft(search, &draft.get_untracked());
        sort_key.set(draft_key.get_untracked());
        sort_order.set(draft_order.get_untracked());
        on_close();
    };
    let clear = move || {
        search.set(AssetSearch::default());
        draft.set(AssetSearch::default());
    };
    view! {
        <div class="flex flex-col gap-4">
            <section aria-label="検索">
                <p class="bottom-sheet-section-title">"検索"</p>
                <div class="mt-2">
                    <SearchBox>
                        <SearchTextInput
                            placeholder=placeholder.clone()
                            testid="asset-search-input"
                            value=draft_text
                            on_input=set_draft_text
                        />
                    </SearchBox>
                </div>
            </section>
            {(!conditions.is_empty())
                .then(|| {
                    view! {
                        <section aria-label="条件を足す">
                            <p class="bottom-sheet-section-title">"条件を足す"</p>
                            {conditions
                                .iter()
                                .map(|kind| {
                                    let options = match kind {
                                        ConditionKind::Account => account_list,
                                        ConditionKind::Course => course_list,
                                    };
                                    let selected = Memo::new(move |_| {
                                        draft.with(|s| match kind {
                                            ConditionKind::Account => s.accounts.clone(),
                                            ConditionKind::Course => s.courses.clone(),
                                        })
                                    });
                                    let kind = *kind;
                                    view! {
                                        <div class="mt-2">
                                            <p class="text-sm font-medium text-text-deep">{kind.label()}</p>
                                            <div class="mt-1">
                                                <ConditionOptionList
                                                    options=options
                                                    selected=selected
                                                    on_toggle=move |value| toggle_draft(kind, value)
                                                />
                                            </div>
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </section>
                    }
                })}
            <section aria-label=SORT_BY.to_string()>
                <p class="bottom-sheet-section-title">{SORT_BY}</p>
                <div class="sort-option-list mt-2" role="group" aria-label=SORT_BY.to_string()>
                    {sort_options
                        .iter()
                        .map(|key| {
                            let key = *key;
                            view! {
                                <Button
                                    variant=ButtonVariant::Ghost
                                    class="sort-option".to_string()
                                    aria_pressed=move || (draft_key.get() == key).to_string()
                                    testid=key_value(key)
                                    on_click=move |_| draft_key.set(key)
                                >
                                    <span class="sort-option-check" aria-hidden="true">
                                        {move || if draft_key.get() == key { "✓" } else { "" }}
                                    </span>
                                    {key_label(key)}
                                </Button>
                            }
                        })
                        .collect_view()}
                </div>
                <div class="sort-order-row mt-2" role="group" aria-label="順序">
                    {[(SortOrder::Desc, SORT_HIGH_TO_LOW), (SortOrder::Asc, SORT_LOW_TO_HIGH)]
                        .map(|(order, label)| {
                            view! {
                                <Button
                                    variant=ButtonVariant::Secondary(ButtonSize::Md)
                                    class="sort-order-option".to_string()
                                    aria_pressed=move || (draft_order.get() == order).to_string()
                                    attr:data-selected=move || (draft_order.get() == order).to_string()
                                    on_click=move |_| draft_order.set(order)
                                >
                                    {label}
                                    <svg aria-hidden="true" class="h-4 w-4" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                                        {if order == SortOrder::Desc {
                                            view! { <path d="M8 3v10M4 9l4 4 4-4" /> }.into_any()
                                        } else {
                                            view! { <path d="M8 13V3M4 7l4-4 4 4" /> }.into_any()
                                        }}
                                    </svg>
                                </Button>
                            }
                        })}
                </div>
            </section>
            <div class="bottom-sheet-actions">
                <Button
                    variant=ButtonVariant::Secondary(ButtonSize::Md)
                    testid="asset-sheet-clear"
                    on_click=move |_| clear()
                >
                    {SHEET_CLEAR}
                </Button>
                <Button
                    variant=ButtonVariant::Primary(ButtonSize::Md)
                    testid="asset-sheet-apply"
                    on_click=move |_| apply()
                >
                    {SHEET_APPLY}
                </Button>
            </div>
        </div>
    }
}
