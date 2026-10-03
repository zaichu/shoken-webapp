use crate::features::asset_balance::search::clear_search_query;
use crate::support::list_search::SearchOption;
use crate::ui::choice::Select;
use crate::ui::collapsible_search_card::CollapsibleSearchCard;
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
                    <Select
                        id="securities-search"
                        label="銘柄"
                        value=Signal::derive(move || query.get())
                        options=options.as_ref().clone()
                        on_change=move |value| query.set(value)
                        class="search-select"
                    />
                </div>
            </CollapsibleSearchCard>
        </section>
    }
}
