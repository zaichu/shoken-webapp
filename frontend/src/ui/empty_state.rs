use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptyStateIcon {
    Search,
    Tray,
    Warning,
    Lock,
}

impl EmptyStateIcon {
    const fn path(self) -> &'static str {
        match self {
            Self::Warning => {
                "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
            }
            Self::Search => "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z",
            Self::Tray => {
                "M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293H9.414a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 005.586 13H4"
            }
            Self::Lock => {
                "M16.5 10.5V6.75a4.5 4.5 0 10-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 002.25-2.25v-6.75a2.25 2.25 0 00-2.25-2.25H6.75a2.25 2.25 0 00-2.25 2.25v6.75a2.25 2.25 0 002.25 2.25z"
            }
        }
    }
}

/// データが空の画面の見せ方を1種類にまとめる。
/// 見本の空行の型: 1行目に見出し、2行目に補足(灰)。次の行動(children)は行の下に出る。
/// icon は渡されたときだけ行頭に小さく出す。見出しの要素(h1/h3)は変えない。
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned by the view"
)]
pub fn EmptyState(
    #[prop(into)] title: String,
    #[prop(into, optional)] description: Option<String>,
    #[prop(optional)] icon: Option<EmptyStateIcon>,
    #[prop(optional)] as_h1: bool,
    #[prop(into, optional)] class: Signal<String>,
    /// テストから空状態卡片を拾うための識別子
    #[prop(optional, default = "empty-state")]
    testid: &'static str,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            class=move || {
                let extra = class.get();
                if extra.is_empty() {
                    "flex flex-col gap-3 px-4 py-3".to_string()
                } else {
                    format!("flex flex-col gap-3 px-4 py-3 {extra}")
                }
            }
            data-testid=testid
        >
            <div class="flex min-w-0 items-start gap-2">
                {icon.map(|icon| {
                    view! {
                        <span class="mt-0.5 shrink-0 text-text-faint" aria-hidden="true">
                            <svg
                                class="h-5 w-5"
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
                        </span>
                    }
                })}
                <div class="min-w-0 flex-1">
                    {if as_h1 {
                        view! {
                            <h1 class="text-body font-bold text-text-deep">{title.clone()}</h1>
                        }
                            .into_any()
                    } else {
                        view! {
                            <h3 class="text-body font-bold text-text-deep">{title.clone()}</h3>
                        }
                            .into_any()
                    }}
                    {description.clone().map(|description| {
                        view! {
                            <p class="mt-0.5 text-cell font-normal text-text-muted">{description}</p>
                        }
                    })}
                </div>
            </div>
            {children.map(|children| children())}
        </div>
    }
}
