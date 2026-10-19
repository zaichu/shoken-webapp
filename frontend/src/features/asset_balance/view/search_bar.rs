//! 資産管理の絞り込み欄。取引明細と同じタグ入力+「条件を足す」の操作に揃える

use crate::features::asset_balance::fund::AssetSearch;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::focus::trap_focus;
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConditionKind {
    Account,
    Course,
}

impl ConditionKind {
    fn label(self) -> &'static str {
        match self {
            ConditionKind::Account => "口座",
            ConditionKind::Course => "分配金コース",
        }
    }
}

#[component]
fn SearchTag(
    label: &'static str,
    value: Memo<String>,
    on_clear: impl Fn() + Copy + 'static + Send,
) -> impl IntoView {
    move || {
        let value = value.get();
        (!value.is_empty()).then(|| {
            let aria_label = format!("{label} {value} を解除");
            view! {
                <Button
                    variant=ButtonVariant::ConditionChip
                    aria_label=aria_label
                    on_click=move |_| on_clear()
                >
                    {format!("{label}: {value}")}
                    <span aria-hidden="true">"✕"</span>
                </Button>
            }
        })
    }
}

/// 「条件を足す」の小窓(PC)・下から出るシート(スマホ)。
/// 口座・分配金コースは選んですぐ反映し、複数選択できる
#[component]
fn ConditionPopover(
    kind: ConditionKind,
    options: Memo<Vec<String>>,
    selected: Memo<Vec<String>>,
    on_toggle: impl Fn(String) + Copy + 'static + Send,
    on_close: impl Fn() + Copy + 'static + Send,
) -> impl IntoView {
    let panel_ref = NodeRef::<leptos::html::Div>::new();
    Effect::new(move |_| {
        if let Some(panel) = panel_ref.get() {
            let _ = panel.focus();
        }
    });
    view! {
        <div class="condition-backdrop" on:click=move |_| on_close() />
        <div
            class="condition-panel"
            role="dialog"
            aria-label=kind.label()
            tabindex="-1"
            node_ref=panel_ref
            on:keydown=move |event| {
                if event.key() == "Escape" {
                    event.prevent_default();
                    on_close();
                    return;
                }
                if event.key() == "Tab" {
                    trap_focus(&event);
                }
            }
        >
            <p class="condition-panel-title">{kind.label()}</p>
            <div class="condition-option-list">
                {move || {
                    options
                        .get()
                        .into_iter()
                        .map(|option| {
                            let pressed_value = option.clone();
                            let click_value = option.clone();
                            view! {
                                <Button
                                    variant=ButtonVariant::ConditionOption
                                    aria_pressed=move || {
                                        selected.get().contains(&pressed_value).to_string()
                                    }
                                    on_click=move |_| on_toggle(click_value.clone())
                                >
                                    {option.clone()}
                                </Button>
                            }
                        })
                        .collect_view()
                }}
            </div>
            <div class="condition-panel-actions">
                <Button
                    variant=ButtonVariant::Secondary(ButtonSize::Md)
                    on_click=move |_| on_close()
                >
                    "閉じる"
                </Button>
            </div>
        </div>
    }
}

/// 検索欄(タグ入力+条件を足す)。PC・スマホ共通。
/// 条件ボタンは条件のある種別(投資信託)だけに出す
#[component]
pub(crate) fn AssetSearchBar(
    search: RwSignal<AssetSearch>,
    search_count: RwSignal<(usize, usize)>,
    placeholder: &'static str,
    show_conditions: bool,
    account_options: Memo<Vec<String>>,
    course_options: Memo<Vec<String>>,
) -> impl IntoView {
    let open = RwSignal::new(None::<ConditionKind>);
    let accounts_tag = Memo::new(move |_| search.with(|s| s.accounts.join("・")));
    let courses_tag = Memo::new(move |_| search.with(|s| s.courses.join("・")));
    let has_anything = Signal::derive(move || search.with(|s| !s.is_empty()));
    let toggle = move |kind: ConditionKind, value: String| {
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
    let selected_accounts = Memo::new(move |_| search.with(|s| s.accounts.clone()));
    let selected_courses = Memo::new(move |_| search.with(|s| s.courses.clone()));

    view! {
        <div class="no-print" role="search" aria-label="資産の絞り込み">
            // PC: タグ入力+件数+クリアを1つの箱に収める(取引明細と同じ構造)
            <div class="receipts-search-box hidden lg:flex">
                <SearchTag
                    label="口座"
                    value=accounts_tag
                    on_clear=move || search.update(|s| s.accounts.clear())
                />
                <SearchTag
                    label="分配金コース"
                    value=courses_tag
                    on_clear=move || search.update(|s| s.courses.clear())
                />
                <input
                    type="search"
                    class="receipts-search-input"
                    placeholder=placeholder
                    aria-label=placeholder
                    autocomplete="off"
                    data-testid="asset-search-input-pc"
                    enterkeyhint="search"
                    prop:value=move || search.with(|s| s.text.clone())
                    on:input=move |ev| {
                        search.update(|s| s.text = event_target_value(&ev));
                    }
                />
                {move || {
                    let (filtered, total) = search_count.get();
                    has_anything.get().then(|| {
                        view! {
                            <span
                                class="receipts-search-count"
                                data-testid="asset-search-count-pc"
                            >
                                {format!("{filtered} / {total}")}
                            </span>
                        }
                    })
                }}
                <Button
                    variant=ButtonVariant::SearchClear
                    testid="asset-search-clear-pc"
                    disabled=move || !has_anything.get()
                    on_click=move |_| search.update(|s| *s = AssetSearch::default())
                >
                    "すべてクリア"
                </Button>
            </div>
            // スマホ: 入力欄だけを一覧の上に常時出す
            <div class="receipts-search-box lg:hidden">
                <input
                    type="search"
                    class="receipts-search-input"
                    placeholder=placeholder
                    aria-label=placeholder
                    autocomplete="off"
                    data-testid="asset-search-input"
                    enterkeyhint="search"
                    prop:value=move || search.with(|s| s.text.clone())
                    on:input=move |ev| {
                        search.update(|s| s.text = event_target_value(&ev));
                    }
                />
            </div>
            {show_conditions
                .then(|| {
                    view! {
                        <div class="add-condition-row">
                            <span class="add-condition-label">"条件を足す:"</span>
                            {[ConditionKind::Account, ConditionKind::Course]
                                .into_iter()
                                .map(|kind_item| {
                                    let testid: &'static str = match kind_item {
                                        ConditionKind::Account => "asset-condition-account",
                                        ConditionKind::Course => "asset-condition-course",
                                    };
                                    view! {
                                        <Button
                                            variant=ButtonVariant::AddCondition
                                            aria_expanded=move || {
                                                open.get() == Some(kind_item)
                                            }
                                            testid=testid
                                            on_click=move |_| {
                                                if open.get() == Some(kind_item) {
                                                    open.set(None);
                                                } else {
                                                    open.set(Some(kind_item));
                                                }
                                            }
                                        >
                                            "＋ " {kind_item.label()}
                                        </Button>
                                    }
                                })
                                .collect_view()}
                            {move || {
                                open.get().map(|kind_item| {
                                    let (options, selected) = match kind_item {
                                        ConditionKind::Account => {
                                            (account_options, selected_accounts)
                                        }
                                        ConditionKind::Course => (course_options, selected_courses),
                                    };
                                    view! {
                                        <ConditionPopover
                                            kind=kind_item
                                            options=options
                                            selected=selected
                                            on_toggle=move |value| toggle(kind_item, value)
                                            on_close=move || open.set(None)
                                        />
                                    }
                                })
                            }}
                        </div>
                    }
                })}
        </div>
    }
}
