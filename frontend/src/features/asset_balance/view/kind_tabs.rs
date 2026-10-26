use crate::features::asset_balance::fund::AssetKind;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

fn tab_id(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::All => "tab-asset-all",
        AssetKind::Stock => "tab-asset-stock",
        AssetKind::Fund => "tab-asset-fund",
    }
}

fn next_index(current: usize, key: &str) -> Option<usize> {
    match key {
        "ArrowRight" => Some((current + 1) % 3),
        "ArrowLeft" => Some((current + 3 - 1) % 3),
        "Home" => Some(0),
        "End" => Some(2),
        _ => None,
    }
}

fn focus_tab(kind: AssetKind) {
    let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(tab_id(kind)))
    else {
        return;
    };
    if let Some(element) = element.dyn_ref::<web_sys::HtmlElement>() {
        let _ = element.focus();
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
                                if let Some(next) = next_index(current, &event.key()) {
                                    event.prevent_default();
                                    keyed.set(AssetKind::ALL[next]);
                                    focus_tab(AssetKind::ALL[next]);
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
