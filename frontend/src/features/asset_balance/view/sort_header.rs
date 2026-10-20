use crate::features::asset_balance::portfolio::SortOrder;
use crate::ui::button::{Button, ButtonVariant};
use leptos::prelude::*;

/// ソート可能な列見出し。同じキーを再押下で昇降を切り替え、別キーは降順で始める
pub(crate) fn sort_header<K: Copy + PartialEq + Send + Sync + 'static>(
    label: &'static str,
    key: K,
    sort_key: RwSignal<K>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    let active = move || sort_key.get() == key;
    let arrow = move || {
        if !active() {
            return String::new();
        }
        match sort_order.get() {
            SortOrder::Desc => " ▼".to_string(),
            SortOrder::Asc => " ▲".to_string(),
        }
    };
    let on_click = move |_| {
        if sort_key.get_untracked() == key {
            sort_order.update(|order| {
                *order = match *order {
                    SortOrder::Desc => SortOrder::Asc,
                    SortOrder::Asc => SortOrder::Desc,
                }
            });
        } else {
            sort_key.set(key);
            sort_order.set(SortOrder::Desc);
        }
    };
    view! {
        <th
            scope="col"
            class="text-right"
            aria-sort=move || {
                active().then(|| {
                    match sort_order.get() {
                        SortOrder::Desc => "descending",
                        SortOrder::Asc => "ascending",
                    }
                })
            }
        >
            <Button
                variant=ButtonVariant::AssetSortColumn
                aria_pressed=Signal::derive(move || active().to_string())
                on_click=on_click
            >
                {label}
                <span class="asset-sort-arrow">{arrow}</span>
            </Button>
        </th>
    }
}
