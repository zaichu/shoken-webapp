//! 資産管理の検索欄。取引明細と同じタグ入力+「条件を足す」の操作に揃え、
//! 部品は ui::search_controls を共有する

use crate::features::asset_balance::fund::AssetSearch;
use crate::support::list_search::SearchOption;
use crate::ui::badge::account_display_name;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::labels::{ACCOUNT, DISTRIBUTION_COURSE};
use crate::ui::search_controls::{
    AddConditionButton, AddConditionRow, ConditionOptionList, ConditionPanel, SearchBox,
    SearchClearButton, SearchCount, SearchTag, SearchTextInput,
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
    let placeholder_mobile = placeholder.clone();

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
            // PC: タグ入力+件数+クリアを1つの箱に収める(取引明細と同じ構造)
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
                {move || {
                    has_anything.get().then(|| {
                        view! {
                            <SearchCount
                                filtered=move || search_count.get().0
                                total=move || search_count.get().1
                                testid="asset-search-count-pc"
                            />
                        }
                    })
                }}
                <SearchClearButton
                    disabled=move || !has_anything.get()
                    testid="asset-search-clear-pc"
                    on_clear=on_clear_all
                />
            </SearchBox>
            // スマホ: 入力欄だけを一覧の上に常時出す
            <SearchBox class="lg:hidden">
                <SearchTextInput
                    placeholder=placeholder_mobile
                    testid="asset-search-input"
                    value=input_value
                    on_input=on_input
                />
            </SearchBox>
            {(!conditions.is_empty())
                .then(|| {
                    view! {
                        <AddConditionRow>
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
                        </AddConditionRow>
                    }
                })}
        </div>
    }
}
