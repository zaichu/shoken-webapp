use crate::features::asset_balance::fund::AssetKind;
use crate::ui::button::TabButton;
use crate::ui::labels;
use crate::ui::underline_tabs;
use crate::ui::underline_tabs::SwipeMotion;
use leptos::prelude::*;

const fn tab_id(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::All => "tab-asset-all",
        AssetKind::Stock => "tab-asset-stock",
        AssetKind::Fund => "tab-asset-fund",
    }
}

/// 保有種別の切り替えピル(すべて/国内株式/投資信託)。
/// 取引明細のタブと同じ構造。
/// スマホでは面の外にもう1組出すため、その分は suffix="-mobile" で
/// id と testid を分け、隠れている側と取り違えないようにする
#[component]
pub(crate) fn AssetKindTabs(
    kind: RwSignal<AssetKind>,
    motion: SwipeMotion,
    #[prop(optional)] suffix: Option<&'static str>,
) -> impl IntoView {
    let suffix = suffix.unwrap_or("");
    view! {
        <div
            class="page-tab-list no-print relative"
            role="tablist"
            aria-label=labels::ASSET_KIND_TABS_LABEL
            data-testid=format!("asset-kind-tabs{suffix}")
            data-swiping=move || motion.dragging.get().to_string()
        >
            {AssetKind::ALL
                .into_iter()
                .map(|item| {
                    let selected = move || kind.get() == item;
                    let keyed = kind;
                    let clicked = kind;
                    let guard = motion;
                    view! {
                        <TabButton
                            id=format!("{}{}", tab_id(item), suffix)
                            selected=selected
                            testid=format!("asset-kind-tab-{}{}", item.value(), suffix)
                            on_click=move |event| {
                                if event.detail() != 0
                                    && underline_tabs::swipe_should_ignore_input(
                                        guard.settling.get_untracked(),
                                        underline_tabs::prefers_reduced_motion(),
                                    )
                                {
                                    return;
                                }
                                clicked.set(item);
                            }
                            on_keydown=move |event| {
                                let current = AssetKind::ALL
                                    .iter()
                                    .position(|candidate| *candidate == keyed.get_untracked())
                                    .unwrap_or(0);
                                if let Some(next) = underline_tabs::next_tab_index(3, current, &event.key()) {
                                    event.prevent_default();
                                    keyed.set(AssetKind::ALL[next]);
                                    underline_tabs::focus_tab_by_id_after_paint(format!(
                                        "{}{}",
                                        tab_id(AssetKind::ALL[next]),
                                        suffix
                                    ));
                                }
                            }
                        >
                            {item.label()}
                        </TabButton>
                    }
                })
                .collect_view()}
            <span
                class="tab-swipe-ink"
                data-on=move || motion.dragging.get().to_string()
                style:width=format!("calc(100% / {})", AssetKind::ALL.len())
                style:transform=move || {
                    let position = AssetKind::ALL
                        .iter()
                        .position(|item| *item == kind.get())
                        .unwrap_or(0);
                    format!(
                        "translateX({}%)",
                        underline_tabs::tab_ink_offset(position, motion.progress.get()) * 100.0
                    )
                }
            ></span>
        </div>
    }
}
