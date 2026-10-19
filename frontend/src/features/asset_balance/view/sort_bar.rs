use crate::features::asset_balance::portfolio::SortOrder;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::select::Select;
use leptos::prelude::*;

/// スマホ用の下部操作帯。並べ替えは項目の選択と昇順/降順の切り替えを
/// 同じ段に置く。種別ごとに選べる項目だけ変わる
#[component]
pub(crate) fn AssetMobileSortBar<K: Copy + PartialEq + Send + Sync + 'static>(
    options: &'static [K],
    key_label: fn(K) -> &'static str,
    key_value: fn(K) -> &'static str,
    sort_key: RwSignal<K>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    view! {
        <div class="asset-mobile-bar">
            <div class="asset-mobile-bar-row" role="group" aria-label="並べ替え">
                <label class="asset-mobile-sort">
                    <span class="shrink-0">"並べ替え:"</span>
                    <Select
                        class="asset-mobile-sort-select".to_string()
                        aria_label="並べ替える項目".to_string()
                        testid="asset-sort-select"
                        value=Signal::derive(move || key_value(sort_key.get()).to_string())
                        on_change=move |value| {
                            if let Some(key) =
                                options.iter().copied().find(|key| key_value(*key) == value)
                            {
                                sort_key.set(key);
                            }
                        }
                    >
                        {options
                            .iter()
                            .map(|key| {
                                view! {
                                    <option value=key_value(*key)>{key_label(*key)}</option>
                                }
                            })
                            .collect_view()}
                    </Select>
                    <span aria-hidden="true">"▾"</span>
                </label>
                <Button
                    variant=ButtonVariant::AssetMobileOrder
                    testid="asset-sort-order-toggle"
                    on_click=move |_| {
                        sort_order.update(|order| {
                            *order = match *order {
                                SortOrder::Desc => SortOrder::Asc,
                                SortOrder::Asc => SortOrder::Desc,
                            }
                        });
                    }
                >
                    {move || sort_order.get().label()}
                </Button>
            </div>
        </div>
    }
}
