use crate::asset_balance::search::clear_search_query;
use crate::components::collapsible_search_card::CollapsibleSearchCard;
use crate::receipts::search::SearchOption;
use leptos::prelude::*;

#[component]
pub(crate) fn AssetBalanceSearchCard(
    query: RwSignal<String>,
    options: Vec<SearchOption>,
) -> impl IntoView {
    let options = std::sync::Arc::new(options);
    view! {
        <section role="search" aria-label="資産管理の検索" data-testid="search-card">
            <CollapsibleSearchCard
                has_active_search=Signal::derive(move || !query.get().is_empty())
                is_default_state=Signal::derive(move || query.get().is_empty())
                on_clear=move || query.set(clear_search_query())
            >
                <div class="grid grid-cols-1 gap-3">
                    <div>
                        <label
                            class="mb-1 block text-sm font-bold text-slate-800"
                            for="securities-search"
                        >
                            "銘柄"
                        </label>
                        <select
                            id="securities-search"
                            class="w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm"
                            prop:value=move || query.get()
                            on:change=move |event| query.set(event_target_value(&event))
                        >
                            <option value="">"全て表示"</option>
                            {options
                                .iter()
                                .map(|option| {
                                    let option = option.clone();
                                    view! { <option value=option.value>{option.label}</option> }
                                })
                                .collect_view()}
                        </select>
                    </div>
                </div>
            </CollapsibleSearchCard>
        </section>
    }
}
