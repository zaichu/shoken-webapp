use crate::features::asset_balance::fund::AssetKind;
use crate::ui::underline_tabs::{focus_tab_by_id, next_tab_index};
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
) -> impl IntoView {
    let count = move |k: AssetKind| match k {
        AssetKind::All => all_count.get(),
        AssetKind::Stock => stock_count.get(),
        AssetKind::Fund => fund_count.get(),
    };
    view! {
        <div
            class="page-tab-list no-print"
            role="tablist"
            aria-label="保有の種類"
            data-testid="asset-kind-tabs"
        >
            {AssetKind::ALL
                .into_iter()
                .map(|item| {
                    let selected = move || kind.get() == item;
                    let keyed = kind;
                    let clicked = kind;
                    view! {
                        <button
                            type="button"
                            role="tab"
                            id=tab_id(item)
                            aria-selected=move || selected().to_string()
                            tabindex=move || if selected() { "0" } else { "-1" }
                            class="page-tab"
                            data-testid=format!("asset-kind-tab-{}", item.value())
                            on:click=move |_| clicked.set(item)
                            on:keydown=move |event| {
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
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}
