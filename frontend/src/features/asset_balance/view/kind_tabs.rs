use crate::features::asset_balance::fund::AssetKind;
use crate::ui::underline_tabs::{UnderlineTabButton, focus_tab_by_id, next_tab_index};
use leptos::prelude::*;

fn tab_id(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::All => "tab-asset-all",
        AssetKind::Stock => "tab-asset-stock",
        AssetKind::Fund => "tab-asset-fund",
    }
}

fn next_kind_index(current: usize, key: &str) -> Option<usize> {
    next_tab_index(AssetKind::ALL.len(), current, key)
}

fn focus_tab(kind: AssetKind) {
    focus_tab_by_id(tab_id(kind));
}

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
                    let selected = Signal::derive(move || kind.get() == item);
                    let count_value = Signal::derive(move || count(item).to_string());
                    let keyed = kind;
                    let clicked = kind;
                    view! {
                        <UnderlineTabButton
                            id=tab_id(item).to_string()
                            label=item.label().to_string()
                            count=count_value
                            selected=selected
                            testid=format!("asset-kind-tab-{}", item.value())
                            on_select=move || clicked.set(item)
                            on_keydown=move |event: web_sys::KeyboardEvent| {
                                let current = AssetKind::ALL
                                    .iter()
                                    .position(|candidate| *candidate == keyed.get_untracked())
                                    .unwrap_or(0);
                                if let Some(next) = next_kind_index(current, &event.key()) {
                                    event.prevent_default();
                                    keyed.set(AssetKind::ALL[next]);
                                    focus_tab(AssetKind::ALL[next]);
                                }
                            }
                        />
                    }
                })
                .collect_view()}
        </div>
    }
}
