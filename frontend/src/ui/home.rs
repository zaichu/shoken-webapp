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

/// 構成比カードの補足文。補足は 400 のままにする
#[component]
pub fn HomeCompositionNote(children: Children) -> impl IntoView {
    view! { <p class="text-label font-normal text-text-muted">{children()}</p> }
}

/// 一部取得失敗の注意書き。補足は 400 のままにする
#[component]
pub fn HomeStatusNote(children: Children) -> impl IntoView {
    view! {
        <p class="text-label font-normal text-text-muted" role="status" aria-live="polite">
            {children()}
        </p>
    }
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
                {labels::DIVIDEND_AMOUNT}
                {" "}
                <strong class="text-subhero">
                    <Amount
                        kind=AmountKind::Delta
                        text=received_text
                        negative=received_negative
                        positive=received_positive
                    />
                </strong>
            </span>
            // PC は配当金の横に「/ 予想」を置く(1行に収まるため今の並びのまま)
            <span class="text-label text-text-muted max-sm:hidden">
                {forecast_text.clone().map_or_else(
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
            <span class="ml-auto text-label font-bold max-sm:hidden">
                {labels::HOME_PROGRESS_LABEL}
                {" "}
                <Amount text=progress_text.clone() />
            </span>
            // スマホは配当金の下の1行に「年間予想(税引後) ¥… ・ 進捗率 …」とまとめる。
            // 行頭の「/」は付けない。超えの状態はバーの下の補足に出す
            <span class="hidden text-label text-text-muted max-sm:inline">
                {forecast_text.map_or_else(
                    || view! {
                        {labels::HOME_FORECAST_NAME}
                        {labels::HOME_FORECAST_LOADING_SUFFIX}
                    }
                        .into_any(),
                    |forecast| {
                        view! {
                            {labels::HOME_FORECAST_NAME}
                            {" "}
                            <Amount text=forecast />
                            {labels::HOME_COMPOSITION_DETAIL_SEPARATOR}
                            {labels::HOME_PROGRESS_LABEL}
                            {" "}
                            <Amount text=progress_text />
                        }
                            .into_any()
                    },
                )}
            </span>
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
    /// 配当金が年間予想(税引後)を超えたか。超えたときだけ文末に状態を添える
    #[prop(optional)]
    over: bool,
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
            {over.then(|| view! {
                {" "}
                {labels::HOME_PROGRESS_OVER}
            })}
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
