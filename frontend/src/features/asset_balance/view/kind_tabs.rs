use crate::features::asset_balance::fund::AssetKind;
use crate::ui::button::TabButton;
use crate::ui::labels;
use crate::ui::underline_tabs::{
    SwipeMotion, focus_tab_by_id, next_tab_index, prefers_reduced_motion,
    swipe_should_ignore_input, tab_ink_offset,
};
use leptos::prelude::*;

fn tab_id(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::All => "tab-asset-all",
        AssetKind::Stock => "tab-asset-stock",
        AssetKind::Fund => "tab-asset-fund",
    }
}

/// 保有種別の切り替えピル(すべて/国内株式/投資信託)。
/// 取引明細のタブと同じ構造で、件数はラベルの横に出す
/// 件数は Signal<usize> で受け取り、データ読み込み完了時に更新されるようにする
#[component]
pub(crate) fn AssetKindTabs(
    kind: RwSignal<AssetKind>,
    all_count: Signal<usize>,
    stock_count: Signal<usize>,
    fund_count: Signal<usize>,
    motion: SwipeMotion,
) -> impl IntoView {
    let count = move |k: AssetKind| match k {
        AssetKind::All => all_count.get(),
        AssetKind::Stock => stock_count.get(),
        AssetKind::Fund => fund_count.get(),
    };
    view! {
        <div
            class="page-tab-list no-print relative"
            role="tablist"
            aria-label=labels::ASSET_KIND_TABS_LABEL
            data-testid="asset-kind-tabs"
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
                            id=tab_id(item).to_string()
                            selected=selected
                            testid=format!("asset-kind-tab-{}", item.value())
                            on_click=move |event| {
                                if event.detail() != 0
                                    && swipe_should_ignore_input(
                                        guard.settling.get_untracked(),
                                        prefers_reduced_motion(),
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
                                if let Some(next) = next_tab_index(3, current, &event.key()) {
                                    event.prevent_default();
                                    keyed.set(AssetKind::ALL[next]);
                                    focus_tab_by_id(tab_id(AssetKind::ALL[next]));
                                }
                            }
                        >
                            {item.label()}
                            <span class="page-tab-count">{move || count(item)}</span>
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
                        tab_ink_offset(position, motion.progress.get()) * 100.0
                    )
                }
            ></span>
        </div>
    }
}
