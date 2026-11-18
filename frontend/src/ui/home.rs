use crate::session;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::card::{Card, CardVariant};
use crate::ui::empty_state::EmptyState;
use crate::ui::labels;
use crate::ui::login_prompt::LoginPrompt;
use crate::ui::more_link::MoreLink;
use crate::ui::state::{ListLoadError, ListSkeleton, ListSkeletonVariant};
use leptos::prelude::*;

#[component]
pub fn HomeLogin() -> impl IntoView {
    view! { <LoginPrompt class="mx-auto w-full max-w-md" /> }
}

#[component]
pub fn HomePartialError() -> impl IntoView {
    view! { <span class="mt-1 block">{labels::HOME_PARTIAL_ERROR}</span> }
}

#[component]
pub fn HomeSummarySection(children: Children) -> impl IntoView {
    view! { <div class="content-gutter lg:mt-3">{children()}</div> }
}

#[component]
pub fn HomeMain(children: Children) -> impl IntoView {
    view! { <div class="page-gutter flex flex-1 flex-col pb-4">{children()}</div> }
}

#[component]
pub fn HomeCardHead(
    title: &'static str,
    href: &'static str,
    link_label: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-3 px-4 pt-3.5">
            <h2 class="text-heading font-bold text-text">{title}</h2>
            <MoreLink href=href.to_string() label=link_label.to_string() />
        </div>
    }
}

#[component]
pub fn HomeCardSkeleton() -> impl IntoView {
    view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }
}

#[component]
pub fn HomeCardError(#[prop(into)] message: String) -> impl IntoView {
    let session = session::use_session();
    view! {
        <div class="px-4 py-3">
            <ListLoadError
                message=message
                on_retry=move || session.generation.update(|generation| *generation = generation.next())
            />
        </div>
    }
}

#[component]
pub fn HomeCardEmpty(
    #[prop(into)] title: String,
    #[prop(into, optional)] description: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let extra: AnyView = children.map_or_else(|| ().into_any(), |children| children());
    view! {
        <div class="px-4 py-4">
            {match description {
                Some(description) => view! {
                    <EmptyState title=title description=description>{extra}</EmptyState>
                }
                    .into_any(),
                None => view! { <EmptyState title=title>{extra}</EmptyState> }.into_any(),
            }}
        </div>
    }
}

#[component]
pub fn HomeDetailCard(testid: &'static str, children: Children) -> impl IntoView {
    view! {
        <Card variant=CardVariant::Panel class="pb-4" testid=testid>
            {children()}
        </Card>
    }
}

#[component]
pub fn HomeListCard(testid: &'static str, children: Children) -> impl IntoView {
    view! {
        <Card variant=CardVariant::Panel class="overflow-clip" testid=testid>
            {children()}
        </Card>
    }
}

#[component]
pub fn HomeCompositionTop(children: Children) -> impl IntoView {
    view! { <div class="px-4 pt-3.5">{children()}</div> }
}

#[component]
pub fn HomeCompositionAccounts(children: Children) -> impl IntoView {
    view! { <div class="px-4 pt-4">{children()}</div> }
}

#[component]
pub fn HomeStackedBar(#[prop(optional)] secondary: bool, children: Children) -> impl IntoView {
    let class = if secondary {
        "home-stacked-bar mt-1.5"
    } else {
        "home-stacked-bar mt-3"
    };
    view! { <div class=class>{children()}</div> }
}

#[component]
pub fn HomeCompositionRows(children: Children) -> impl IntoView {
    view! { <ul class="home-composition-rows mt-1.5">{children()}</ul> }
}

#[component]
pub fn HomeDividendHead(
    #[prop(into)] received_text: String,
    received_negative: bool,
    received_positive: bool,
    forecast_text: Option<String>,
    #[prop(into)] progress_text: String,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-baseline gap-x-2 gap-y-1 px-4 pt-3">
            <span class="text-subhero">
                {labels::HOME_RECEIVED_LABEL}
                <strong class="text-subhero">
                    <Amount
                        kind=AmountKind::Delta
                        text=received_text
                        negative=received_negative
                        positive=received_positive
                    />
                </strong>
            </span>
            <span class="text-label text-text-muted">
                {forecast_text.map_or_else(
                    || view! { {labels::HOME_FORECAST_LOADING} }.into_any(),
                    |forecast| {
                        view! {
                            {labels::home_forecast("")}
                            <Amount text=forecast />
                        }
                            .into_any()
                    },
                )}
            </span>
            <Amount class="ml-auto text-label font-bold" text=progress_text />
        </div>
    }
}

#[component]
pub fn HomeDividendNote(
    #[prop(into)] prefix: String,
    #[prop(into)] remaining: String,
    #[prop(into)] suffix: String,
    previous: Option<(String, String)>,
    #[prop(into)] yoy_text: String,
    yoy_negative: bool,
    yoy_positive: bool,
) -> impl IntoView {
    view! {
        <p class="break-keep px-4 pt-1.5 text-label text-text-muted">
            {prefix}
            <Amount text=remaining />
            {suffix}
            {previous.map(|(span, amount)| view! {
                {span}
                <Amount text=amount />
                {labels::HOME_DEPOSIT_SUFFIX}
            })}
            {labels::HOME_DIVIDEND_YOY_LABEL}
            <Amount
                kind=AmountKind::Delta
                text=yoy_text
                negative=yoy_negative
                positive=yoy_positive
            />
        </p>
    }
}

#[component]
pub fn HomeChartSection(#[prop(into)] label: String, children: Children) -> impl IntoView {
    view! {
        <div class="mt-3 px-4" role="group" aria-label=label>
            {children()}
        </div>
    }
}

#[component]
pub fn HomeChartLegend(year: u32, previous_year: u32) -> impl IntoView {
    view! {
        <div class="mt-2 flex justify-center gap-4 px-4 text-cell text-text-muted">
            <span class="flex items-center gap-1">
                <span class="home-swatch bg-chart-1"></span>
                {labels::receipt_year_label(&year.to_string())}
            </span>
            <span class="flex items-center gap-1">
                <span class="home-swatch home-chart-previous"></span>
                {labels::receipt_year_label(&previous_year.to_string())}
            </span>
        </div>
    }
}

#[component]
pub fn HomeDividendTooltip(
    year: u32,
    previous_year: u32,
    #[prop(into)] cur_text: String,
    #[prop(into)] prev_text: String,
    #[prop(into)] diff_text: String,
    diff_negative: bool,
    diff_positive: bool,
) -> impl IntoView {
    view! {
        <dl class="mt-1 space-y-0.5">
            <div class="flex items-center gap-1.5">
                <dt class="home-swatch bg-chart-1"></dt>
                <dd>{labels::home_year_amount(year, "")}<Amount text=cur_text /></dd>
            </div>
            <div class="flex items-center gap-1.5">
                <dt class="home-swatch home-chart-previous"></dt>
                <dd>{labels::home_year_amount(previous_year, "")}<Amount text=prev_text /></dd>
            </div>
            <div class="flex items-center gap-1.5">
                <dt class="text-text-muted">{labels::HOME_DIFF_LABEL}</dt>
                <dd>
                    <Amount
                        kind=AmountKind::Delta
                        text=diff_text
                        negative=diff_negative
                        positive=diff_positive
                    />
                </dd>
            </div>
        </dl>
    }
}
