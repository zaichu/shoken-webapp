use leptos::prelude::*;

/// 「項目名 … 値」を1行に並べる定義行。取引明細カードと資産管理カードの
/// 明細行で共有し、並べ方を画面ごとに作り分けない。`<dl>` の中で使う。
#[component]
pub fn StatRow(
    #[prop(into)] label: String,
    /// 数値は桁をそろえる(tabular-nums)
    #[prop(optional)]
    numeric: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="flex items-start justify-between gap-3">
            <dt class="shrink-0 pt-0.5 text-xs font-medium text-text-subtle">{label}</dt>
            <dd
                class=if numeric {
                    "min-w-0 break-words text-right text-sm font-semibold tabular-nums text-text"
                } else {
                    "min-w-0 break-words text-right text-sm font-semibold text-text"
                }
            >
                {children()}
            </dd>
        </div>
    }
}
