use crate::features::asset_balance::fund::AssetKind;
use leptos::prelude::*;

/// 保有種別の切り替えピル(すべて/国内株式/投資信託)。
/// 取引明細のタブと同じ構造で、件数はラベルの横に出す
#[component]
pub(crate) fn AssetKindTabs(
    kind: RwSignal<AssetKind>,
    all_count: usize,
    stock_count: usize,
    fund_count: usize,
) -> impl IntoView {
    let count = move |k: AssetKind| match k {
        AssetKind::All => all_count,
        AssetKind::Stock => stock_count,
        AssetKind::Fund => fund_count,
    };
    view! {
        <div
            class="receipts-tab-list no-print"
            role="tablist"
            aria-label="保有の種類"
            data-testid="asset-kind-tabs"
        >
            {AssetKind::ALL
                .into_iter()
                .map(|item| {
                    let selected = move || kind.get() == item;
                    view! {
                        <button
                            type="button"
                            role="tab"
                            aria-selected=move || selected().to_string()
                            class="receipts-tab"
                            data-testid=format!("asset-kind-tab-{}", item.value())
                            on:click=move |_| kind.set(item)
                        >
                            {item.label()}
                            <span class="receipts-tab-count">{count(item)}</span>
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}
