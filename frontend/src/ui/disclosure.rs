use leptos::prelude::*;

// 開閉(ディスクロージャ)の基本部品。トリガーはすべてこの部品を通し、
// 山形は ChevronIcon の1種類に統一する。

/// 開閉で回転する山形アイコン
#[component]
pub fn ChevronIcon(
    #[prop(into)] expanded: Signal<bool>,
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    view! {
        <svg
            aria-hidden="true"
            class=move || {
                if expanded.get() {
                    format!("{class} rotate-180")
                } else {
                    class.to_string()
                }
            }
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M19 9l-7 7-7-7"
            />
        </svg>
    }
}

/// 「開く/閉じる」の文字と山形のセット(帯・レールのトリガー右端)
#[component]
pub fn DisclosureHint(#[prop(into)] expanded: Signal<bool>) -> impl IntoView {
    view! {
        <span class="flex shrink-0 items-center gap-1 text-text-soft">
            <span class="text-xs font-semibold">
                {move || if expanded.get() { "閉じる" } else { "開く" }}
            </span>
            <ChevronIcon expanded=expanded class="h-4 w-4 text-text-subtle transition-transform duration-200" />
        </span>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisclosureStyle {
    /// 帯の中の開閉
    Collapsible,
    /// レールの開閉
    Rail,
    /// グループ見出し
    GroupCard,
    /// 見出しの下線についた開閉(集計情報デスクトップ)
    HeaderFlat,
    /// 保有カード(モバイル)
    AssetCard,
    /// 検索オプションの見出し
    SearchCard,
}

impl DisclosureStyle {
    fn class(self) -> &'static str {
        match self {
            Self::Collapsible => "collapsible-trigger",
            Self::Rail => "rail-toggle",
            Self::GroupCard => "group-card-trigger",
            Self::HeaderFlat => {
                "flex w-full items-start justify-between gap-3 border-b border-ink/10 pb-2.5 text-left"
            }
            Self::AssetCard => "block min-h-11 w-full px-3.5 py-4 text-left",
            Self::SearchCard => {
                "flex w-full min-w-0 items-center gap-2.5 text-left select-none cursor-pointer max-sm:min-h-11"
            }
        }
    }
}

/// aria-expanded / aria-controls を持つ開閉トリガー
#[component]
pub fn DisclosureToggle(
    style: DisclosureStyle,
    #[prop(into)] expanded: Signal<bool>,
    #[prop(into)] controls: String,
    #[prop(into, optional)] id: Option<String>,
    #[prop(into, optional)] aria_label: Option<Signal<String>>,
    #[prop(optional)] testid: Option<&'static str>,
    /// 末尾に「開く/閉じる」+山形を付ける(付けない側は children に ChevronIcon を置く)
    #[prop(optional)]
    hint: bool,
    #[prop(into, optional)] class: Signal<String>,
    on_toggle: impl Fn() + 'static,
    children: Children,
) -> impl IntoView {
    let style_class = style.class();
    let classes = move || {
        let extra = class.get();
        if extra.is_empty() {
            style_class.to_string()
        } else {
            format!("{style_class} {extra}")
        }
    };
    view! {
        <button
            type="button"
            id=id
            class=classes
            aria-expanded=move || expanded.get().to_string()
            aria-controls=controls
            aria-label=move || aria_label.map(|label| label.get())
            data-testid=testid
            on:click=move |_| on_toggle()
        >
            {children()}
            {hint.then(|| view! { <DisclosureHint expanded=expanded /> })}
        </button>
    }
}

#[cfg(test)]
mod tests;
