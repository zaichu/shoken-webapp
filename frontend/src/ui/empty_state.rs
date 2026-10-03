use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptyStateIcon {
    Search,
    Tray,
    Warning,
}

impl EmptyStateIcon {
    fn svg_class(self) -> &'static str {
        match self {
            Self::Warning => "h-16 w-16",
            Self::Search | Self::Tray => "h-10 w-10",
        }
    }

    fn path(self) -> &'static str {
        match self {
            Self::Warning => "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z",
            Self::Search => "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z",
            Self::Tray => "M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293H9.414a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 005.586 13H4",
        }
    }
}

/// データが空の画面の見せ方を1種類にまとめる。
/// icon は見出しの上、children は説明の下(次の行動)に出る。
#[component]
pub fn EmptyState(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    #[prop(optional)] icon: Option<EmptyStateIcon>,
    #[prop(optional)] as_h1: bool,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=move || {
            let extra = class.get();
            if extra.is_empty() {
                "empty-state".to_string()
            } else {
                format!("empty-state {extra}")
            }
        }>
            {icon.map(|icon| {
                view! {
                    <div class="mb-3 text-text-faint" aria-hidden="true">
                        <svg
                            class=icon.svg_class()
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                            aria-hidden="true"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="1.5"
                                d=icon.path()
                            />
                        </svg>
                    </div>
                }
            })}
            {if as_h1 {
                view! { <h1 class="text-balance text-2xl font-black text-ink">{title.clone()}</h1> }.into_any()
            } else {
                view! { <h3 class="text-balance text-base font-black text-ink">{title.clone()}</h3> }.into_any()
            }}
            <p class="mt-1.5 max-w-md text-sm font-medium text-text-muted">{description}</p>
            {children.map(|children| children())}
        </div>
    }
}
