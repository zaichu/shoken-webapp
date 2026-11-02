//! 資産管理の検索欄。取引明細と同じタグ入力+「条件を足す」の操作に揃え、
//! 部品は ui::search_controls を共有する

use crate::features::asset_balance::fund::AssetSearch;
use crate::features::asset_balance::portfolio::SortOrder;
use crate::support::list_search::SearchOption;
use crate::ui::badge::account_display_name;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::labels;
use crate::ui::more_link::MoreLink;
use crate::ui::search_controls::{
    AddConditionButton, AddConditionRow, ConditionOptionList, ConditionPanel, SearchBox,
    SearchCount, SearchTag, SearchTextInput,
};
use crate::ui::search_dialog::SearchDialog;
use crate::ui::search_entry::{MobileSearchEntryShell, SearchCountAndClear};
use crate::ui::select::Select;
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConditionKind {
    Account,
    Course,
}

impl ConditionKind {
    fn label(self) -> &'static str {
        match self {
            ConditionKind::Account => labels::ACCOUNT,
            ConditionKind::Course => labels::DISTRIBUTION_COURSE,
        }
    }

    fn title(self) -> String {
        match self {
            ConditionKind::Account => labels::pick_title(labels::ACCOUNT),
            ConditionKind::Course => labels::pick_title(labels::DISTRIBUTION_COURSE),
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
                    {labels::DIALOG_CLOSE}
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
                .join(labels::TAG_SEPARATOR)
        })
    });
    let courses_tag = Memo::new(move |_| search.with(|s| s.courses.join(labels::TAG_SEPARATOR)));
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
        <div class="no-print" role="search" aria-label=labels::ASSET_SEARCH_REGION>
            // PC: タグ入力+条件を足す+件数を1つの段にまとめる(取引明細と同じ構造)
            <SearchBox class="hidden lg:flex">
                <SearchTag
                    label=labels::ACCOUNT
                    value=accounts_tag
                    on_clear=move || search.update(|s| s.accounts.clear())
                />
                <SearchTag
                    label=labels::DISTRIBUTION_COURSE
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
                    <SearchCountAndClear
                        filtered=move || search_count.get().0
                        total=move || search_count.get().1
                        clear_disabled=move || !has_anything.get()
                        on_clear=on_clear_all
                        testid="asset-search-count"
                        clear_testid="asset-search-clear"
                    />
                </AddConditionRow>
            </div>
        </div>
    }
}

/// ダイアログの下書きで条件を1件に絞る。空はすべてに戻す
pub(crate) fn dialog_single_value(list: &[String]) -> String {
    list.first().cloned().unwrap_or_default()
}

/// 下書きの条件を単一選択で置き換える
pub(crate) fn set_dialog_single(list: &mut Vec<String>, value: String) {
    list.clear();
    if !value.is_empty() {
        list.push(value);
    }
}

/// 下書きを一覧へ反映する。検索ボタンと単体テストの共通入口
pub(crate) fn apply_asset_sheet_draft(search: RwSignal<AssetSearch>, draft: &AssetSearch) {
    search.set(draft.clone());
}

/// ダイアログのクリアは下書きの検索条件だけを消し、並べ替えは保つ
pub(crate) fn clear_asset_dialog_draft(draft: RwSignal<AssetSearch>) {
    draft.update(|s| {
        s.text.clear();
        s.accounts.clear();
        s.courses.clear();
    });
}

/// スマホの検索・並べ替えの入口とダイアログ。PC の検索欄・条件の行は変えず、
/// スマホ幅ではこの1行と条件タグだけを出し、中身は中央のダイアログで編集する
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
    key_from_value: fn(&str) -> Option<K>,
    sort_key: RwSignal<K>,
    sort_order: RwSignal<SortOrder>,
    /// true のとき件数の行の右端に構成比への入口を出す
    #[prop(optional)]
    composition: bool,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let accounts_tag = Memo::new(move |_| {
        search.with(|s| {
            s.accounts
                .iter()
                .map(|value| account_display_name(value))
                .collect::<Vec<_>>()
                .join(labels::TAG_SEPARATOR)
        })
    });
    let courses_tag = Memo::new(move |_| search.with(|s| s.courses.join(labels::TAG_SEPARATOR)));
    let sort_summary = move || {
        let order = match sort_order.get() {
            SortOrder::Desc => labels::SORT_HIGH_TO_LOW,
            SortOrder::Asc => labels::SORT_LOW_TO_HIGH,
        };
        format!("{} {order}", key_label(sort_key.get()))
    };
    let sheet_placeholder = placeholder.clone();
    let sort_summary = Signal::derive(sort_summary);
    view! {
        <MobileSearchEntryShell
            entry_testid="asset-search-sheet-entry"
            search_label=labels::ASSET_SEARCH_REGION.to_string()
            sort_summary=sort_summary
            open=open
        >
            <SearchTag
                label=labels::ACCOUNT
                value=accounts_tag
                on_clear=move || search.update(|s| s.accounts.clear())
            />
            <SearchTag
                label=labels::DISTRIBUTION_COURSE
                value=courses_tag
                on_clear=move || search.update(|s| s.courses.clear())
            />
            <SearchCount
                filtered=move || search_count.get().0
                total=move || search_count.get().1
                testid="asset-search-count-mobile"
            />
            {composition.then(|| {
                view! {
                    <span class="ml-auto">
                        <MoreLink
                            href="/assetbalance/composition".to_string()
                            label=labels::COMPOSITION_VIEW_LINK.to_string()
                            testid="asset-composition-link".to_string()
                        />
                    </span>
                }
            })}
        </MobileSearchEntryShell>
        {move || {
            let sheet_text = sheet_placeholder.clone();
            open.get()
                .then(|| {
                    let draft = RwSignal::new(search.get_untracked());
                    let draft_key = RwSignal::new(sort_key.get_untracked());
                    let draft_order = RwSignal::new(sort_order.get_untracked());
                    let do_search = move || {
                        apply_asset_sheet_draft(search, &draft.get_untracked());
                        sort_key.set(draft_key.get_untracked());
                        sort_order.set(draft_order.get_untracked());
                        open.set(false);
                    };
                    let do_clear = move || clear_asset_dialog_draft(draft);
                    view! {
                        <SearchDialog
                            title=labels::SEARCH_AND_SORT.to_string()
                            on_close=move || open.set(false)
                            on_search=do_search
                            on_clear=do_clear
                            testid="asset-search-sheet"
                        >
                            <AssetDialogFields
                                draft=draft
                                draft_key=draft_key
                                draft_order=draft_order
                                sort_options=sort_options
                                key_label=key_label
                                key_value=key_value
                                key_from_value=key_from_value
                                conditions=conditions
                                account_options=account_options
                                course_options=course_options
                                placeholder=sheet_text.clone()
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
fn AssetDialogFields<K: Copy + PartialEq + Send + Sync + 'static>(
    draft: RwSignal<AssetSearch>,
    draft_key: RwSignal<K>,
    draft_order: RwSignal<SortOrder>,
    sort_options: &'static [K],
    key_label: fn(K) -> &'static str,
    key_value: fn(K) -> &'static str,
    key_from_value: fn(&str) -> Option<K>,
    conditions: &'static [ConditionKind],
    account_options: Memo<Vec<String>>,
    course_options: Memo<Vec<String>>,
    #[prop(into)] placeholder: String,
    on_search: impl Fn() + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let draft_text = move || draft.with(|s| s.text.clone());
    let set_draft_text = move |value: String| draft.update(|s| s.text = value);
    let search_callback = Callback::new(move |_| on_search());
    let hint = placeholder.clone();
    view! {
        <div class="flex flex-col gap-4">
            <section aria-label=labels::SEARCH>
                <p class="search-dialog-section-title">{labels::SEARCH}</p>
                <div class="search-dialog-field">
                    <SearchBox>
                        <SearchTextInput
                            placeholder=placeholder.clone()
                            testid="asset-search-input"
                            value=draft_text
                            on_input=set_draft_text
                            on_enter=search_callback
                        />
                    </SearchBox>
                    <p class="search-dialog-hint">{hint}</p>
                </div>
            </section>
            {(!conditions.is_empty())
                .then(|| {
                    view! {
                        <section aria-label=labels::CONDITION>
                            <p class="search-dialog-section-title">{labels::CONDITION}</p>
                            {conditions
                                .iter()
                                .map(|kind| {
                                    let kind = *kind;
                                    let options = match kind {
                                        ConditionKind::Account => account_options,
                                        ConditionKind::Course => course_options,
                                    };
                                    let value = Memo::new(move |_| {
                                        draft.with(|s| match kind {
                                            ConditionKind::Account => dialog_single_value(&s.accounts),
                                            ConditionKind::Course => dialog_single_value(&s.courses),
                                        })
                                    });
                                    let label = kind.label();
                                    let testid = match kind {
                                        ConditionKind::Account => "asset-dialog-account",
                                        ConditionKind::Course => "asset-dialog-course",
                                    };
                                    let to_label: fn(&str) -> &str = match kind {
                                        ConditionKind::Account => account_display_name,
                                        ConditionKind::Course => |v| v,
                                    };
                                    view! {
                                        <div class="search-dialog-field">
                                            <label class="mb-1 block text-sm font-medium text-text-deep">
                                                {label}
                                                <Select
                                                    value=Signal::derive(move || value.get())
                                                    aria_label=label.to_string()
                                                    testid=testid
                                                    class="search-dialog-select".to_string()
                                                    on_change=move |next: String| {
                                                        draft.update(|s| match kind {
                                                            ConditionKind::Account => {
                                                                set_dialog_single(&mut s.accounts, next)
                                                            }
                                                            ConditionKind::Course => {
                                                                set_dialog_single(&mut s.courses, next)
                                                            }
                                                        });
                                                    }
                                                >
                                                    <option value="">{labels::ALL}</option>
                                                    {move || {
                                                        options
                                                            .get()
                                                            .into_iter()
                                                            .map(|item| {
                                                                let item_label = to_label(&item).to_string();
                                                                let item_value = item.clone();
                                                                view! {
                                                                    <option value=item_value.clone()>{item_label}</option>
                                                                }
                                                            })
                                                            .collect_view()
                                                    }}
                                                </Select>
                                            </label>
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </section>
                    }
                })}
            <section aria-label=labels::SORT_BY.to_string()>
                <p class="search-dialog-section-title">{labels::SORT_BY}</p>
                <div class="search-dialog-sort-row">
                    <div>
                        <label class="mb-1 block text-sm font-medium text-text-deep">
                            {labels::SORT_TARGET}
                            <Select
                                value=Signal::derive(move || key_value(draft_key.get()).to_string())
                                aria_label=labels::SORT_TARGET.to_string()
                                testid="asset-dialog-sort-target"
                                class="search-dialog-select".to_string()
                                on_change=move |next: String| {
                                    if let Some(key) = key_from_value(&next) {
                                        draft_key.set(key);
                                    }
                                }
                            >
                                {sort_options
                                    .iter()
                                    .map(|key| {
                                        let value = key_value(*key).to_string();
                                        let label = key_label(*key);
                                        view! { <option value=value>{label}</option> }
                                    })
                                    .collect_view()}
                            </Select>
                        </label>
                    </div>
                    <div>
                        <label class="mb-1 block text-sm font-medium text-text-deep">
                            {labels::SORT_ORDER_LABEL}
                            <Select
                                value=Signal::derive(move || draft_order.get().value().to_string())
                                aria_label=labels::SORT_ORDER_LABEL.to_string()
                                testid="asset-dialog-sort-order"
                                class="search-dialog-select".to_string()
                                on_change=move |next: String| {
                                    if let Some(order) = SortOrder::from_value(&next) {
                                        draft_order.set(order);
                                    }
                                }
                            >
                                <option value="desc">{labels::SORT_HIGH_TO_LOW}</option>
                                <option value="asc">{labels::SORT_LOW_TO_HIGH}</option>
                            </Select>
                        </label>
                    </div>
                </div>
            </section>
        </div>
    }
}
