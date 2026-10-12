use crate::features::asset_balance::portfolio::{SortKey, SortOrder};
use crate::ui::button::{Button, ButtonVariant};
use leptos::prelude::*;

const SORT_KEYS: &[(SortKey, &str)] = &[
    (SortKey::MarketValue, "評価額"),
    (SortKey::ProfitLoss, "損益"),
    (SortKey::Dividend, "配当"),
    (SortKey::Yield, "利回り"),
];

#[component]
pub fn SortControls(sort_key: RwSignal<SortKey>, sort_order: RwSignal<SortOrder>) -> impl IntoView {
    view! {
        <div
            class="asset-sort-bar"
            role="group"
            aria-label="並べ替え"
            data-testid="asset-sort-controls"
        >
            {SORT_KEYS
                .iter()
                .map(|(key, label)| {
                    let key = *key;
                    let label = *label;
                    let active = move || sort_key.get() == key;
                    view! {
                        <Button
                            variant=ButtonVariant::AssetSortChip
                            aria_pressed=Signal::derive(move || active().to_string())
                            on_click=move |_| sort_key.set(key)
                        >
                            {label}
                        </Button>
                    }
                })
                .collect_view()}
            <Button
                variant=ButtonVariant::AssetSortDirection
                on_click=move |_| {
                    sort_order.update(|order| {
                        *order = match *order {
                            SortOrder::Desc => SortOrder::Asc,
                            SortOrder::Asc => SortOrder::Desc,
                        }
                    })
                }
            >
                {move || {
                    match sort_order.get() {
                        SortOrder::Desc => "高い順 ↓",
                        SortOrder::Asc => "低い順 ↑",
                    }
                }}
            </Button>
        </div>
    }
}
