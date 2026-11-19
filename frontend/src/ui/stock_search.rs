use crate::ui::amount::{self, AmountKind};
use crate::ui::badge;
use crate::ui::card::{self, CardVariant};
use crate::ui::empty_state::{self, EmptyStateIcon};
use crate::ui::labels;
use crate::ui::search_controls;
use crate::ui::state::{self, AlertVariant};
use leptos::prelude::*;

#[component]
pub fn SearchBar(
    stock_code: RwSignal<String>,
    loading: Signal<bool>,
    #[prop(optional)] mobile: bool,
    on_submit: impl Fn(web_sys::SubmitEvent) + Send + Sync + 'static,
) -> impl IntoView {
    let class = if mobile {
        "py-2"
    } else {
        "hidden lg:block lg:pb-4"
    };
    let icon_size = if mobile { "h-5 w-5" } else { "h-4 w-4" };
    view! {
        <div class=class role="search" aria-label=labels::STOCK_SEARCH_TITLE>
            <form on:submit=on_submit class="w-full">
                <search_controls::SearchBox icon_size=icon_size>
                    <search_controls::SearchTextInput
                        placeholder=labels::search_placeholder(labels::STOCK_SEARCH_TARGET)
                        disabled=Signal::derive(move || loading.get())
                        value=move || stock_code.get()
                        on_input=move |value| stock_code.set(value)
                    />
                    <search_controls::SearchSubmitButton
                        disabled=move || loading.get() || stock_code.get().is_empty()
                        loading=move || loading.get()
                        label=labels::STOCK_SEARCH_ACTION
                        loading_label=labels::SEARCH_LOADING
                    />
                </search_controls::SearchBox>
            </form>
        </div>
    }
}

#[component]
pub fn RefreshError(message: String) -> impl IntoView {
    view! {
        <div class="no-print mb-3">
            <state::Alert variant=AlertVariant::Danger>
                <strong>{labels::ERROR_PREFIX}</strong>
                " "
                {message}
            </state::Alert>
        </div>
    }
}

#[component]
pub fn BalanceError(
    message: String,
    on_retry: impl Fn() + Send + Sync + Copy + 'static,
) -> impl IntoView {
    view! {
        <div class="mb-3">
            <state::ListLoadError message=message on_retry=on_retry />
        </div>
    }
}

#[component]
pub fn EmptySearch(title: &'static str, description: &'static str) -> impl IntoView {
    view! {
        <empty_state::EmptyState
            class="py-10"
            title=title
            description=description
            icon=EmptyStateIcon::Search
        >
            <ul class="mt-4 space-y-1 text-body font-normal text-text-muted">
                <li>{labels::STOCK_SEARCH_TIP_CODE}</li>
                <li>{labels::STOCK_SEARCH_TIP_NAME}</li>
                <li>{labels::STOCK_SEARCH_TIP_LINKS}</li>
            </ul>
        </empty_state::EmptyState>
    }
}

#[component]
pub fn StockInfoCard(
    name: String,
    code: String,
    meta: [(&'static str, String); 4],
    links: Vec<(&'static str, String)>,
) -> impl IntoView {
    view! {
        <div>
            <card::Card variant=CardVariant::Panel class="mb-6 overflow-hidden">
                <div class="border-b border-ink/10 bg-ink px-4 py-4 text-page sm:px-5">
                    <div class="flex flex-wrap items-center justify-between gap-3">
                        <h2 class="min-w-0 flex-1 break-words text-subhero font-bold leading-tight">{name}</h2>
                        <badge::CodeBadge>{code}</badge::CodeBadge>
                    </div>
                </div>
                <div>
                    <dl class="grid grid-cols-2 gap-px bg-fill text-body lg:grid-cols-4">
                        {meta
                            .into_iter()
                            .map(|(label, value)| {
                                let display = if value.is_empty() {
                                    "—".to_string()
                                } else {
                                    value
                                };
                                view! {
                                    <div class="flex min-h-16 flex-col justify-center bg-surface px-4 py-3">
                                        <dt class="text-cell font-bold text-text-muted">
                                            {label}
                                        </dt>
                                        <dd class="mt-1 break-words font-bold tabular-nums text-text-deep">{display}</dd>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </dl>
                    <div class="px-4 py-4">
                        <self::StockInfoLinks links=links />
                    </div>
                </div>
            </card::Card>
        </div>
    }
}

#[component]
fn StockInfoLinks(links: Vec<(&'static str, String)>) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 gap-2 lg:grid-cols-4">
            {links
                .into_iter()
                .map(|(name, href)| {
                    view! {
                        <a
                            class="stock-link-button"
                            href=href
                            target="_blank"
                            rel="noopener noreferrer"
                            aria-label=labels::stock_search_new_tab(name)
                        >
                            {name}
                            <svg
                                class="h-3.5 w-3.5 shrink-0 text-text-faint"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                                aria-hidden="true"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                                />
                            </svg>
                        </a>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
pub fn HoldingSection(children: Children) -> impl IntoView {
    view! {
        <section aria-label=labels::STOCK_HOLDING_SECTION class="mb-6">
            <h2 class="mb-3 text-heading font-bold text-text-deep">
                {labels::STOCK_HOLDING_SECTION}
            </h2>
            {children()}
        </section>
    }
}

#[component]
pub fn HoldingTotalHeading() -> impl IntoView {
    view! {
        <h3 class="mb-2 text-label font-bold text-text-muted">
            {labels::STOCK_HOLDING_TOTAL}
        </h3>
    }
}

#[component]
pub fn HoldingAccounts(children: Children) -> impl IntoView {
    view! { <div class="mt-4 space-y-4">{children()}</div> }
}

#[component]
pub fn HoldingAccount(account: Option<String>, children: Children) -> impl IntoView {
    view! {
        <div>
            <p class="mb-2">
                {account.map_or_else(|| labels::unavailable_text(labels::UnavailableKind::Unassigned).into_any(), |label| {
                    view! { <badge::AccountBadge label=label /> }.into_any()
                })}
            </p>
            {children()}
        </div>
    }
}

#[component]
pub fn HoldingProfitLoss(
    text: String,
    negative: bool,
    positive: bool,
    rate: String,
    rate_negative: bool,
    rate_positive: bool,
) -> impl IntoView {
    view! {
        <amount::Amount
            kind=AmountKind::Delta
            class="tabular-nums"
            text=text
            negative=negative
            positive=positive
        />
        <amount::Amount
            kind=AmountKind::Delta
            class="ml-1 text-cell"
            text=rate
            negative=rate_negative
            positive=rate_positive
        />
    }
}
