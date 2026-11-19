use crate::features::asset_balance::portfolio::SortOrder;
use crate::ui::search_controls::SortColumnButton;
use leptos::prelude::*;

/// ソート可能な列見出し。同じキーを再押下で昇降を切り替え、別キーは降順で始める
/// 並べ替えは数値列にだけ使うので、見出しは数値列の種別クラス(.col-num)にする
pub(crate) fn sort_header<K: Copy + PartialEq + Send + Sync + 'static>(
    label: &'static str,
    key: K,
    sort_key: RwSignal<K>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    sort_header_titled(label, None, None, key, sort_key, sort_order)
}

/// 算出条件の説明付きのソート列見出し。見た目は sort_header と同じで、
/// ホバー・読み上げの説明だけ足す。2行目は長い列見出しが1280で表をはみ出さないよう
/// 改行して添える(読み上げ名は1行の場合とそろえる)
pub(crate) fn sort_header_titled<K: Copy + PartialEq + Send + Sync + 'static>(
    label: &'static str,
    sub: Option<&'static str>,
    title: Option<&'static str>,
    key: K,
    sort_key: RwSignal<K>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    let active = move || sort_key.get() == key;
    let on_click = move || {
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
            class="col-num"
            title=title
            aria-sort=move || {
                active().then(|| {
                    match sort_order.get() {
                        SortOrder::Desc => "descending",
                        SortOrder::Asc => "ascending",
                    }
                })
            }
        >
            <SortColumnButton
                label=label
                sub=sub.unwrap_or_default()
                active=Signal::derive(active)
                descending=Signal::derive(move || sort_order.get() == SortOrder::Desc)
                on_click=on_click
            />
        </th>
    }
}
