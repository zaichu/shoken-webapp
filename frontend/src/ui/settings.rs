use crate::ui::avatar::UserAvatar;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::card::{Card, CardVariant};
use crate::ui::labels;
use leptos::prelude::*;

#[component]
pub fn SettingsLayout(navigation: AnyView, children: Children) -> impl IntoView {
    view! {
        <div class="flex flex-col lg:flex-row gap-7">
            {navigation}
            <div class="flex-1 min-w-0 space-y-4">{children()}</div>
        </div>
    }
}

#[component]
pub fn SettingsNavigation(children: Children) -> impl IntoView {
    view! {
        <nav class="hidden lg:block lg:w-48 shrink-0 lg:sticky lg:top-16 lg:self-start" aria-label=labels::SETTINGS_TOC_LABEL>
            <ul class="flex flex-col gap-1" role="list">{children()}</ul>
        </nav>
    }
}

#[derive(Clone, Copy, Default)]
pub enum SettingsSectionTone {
    #[default]
    Default,
    Danger,
    Notification,
}

#[component]
pub fn SettingsSection(
    id: &'static str,
    labelledby: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <section id=id aria-labelledby=labelledby class="scroll-mt-18">{children()}</section>
    }
}

#[component]
pub fn SettingsPanel(
    #[prop(optional)] tone: SettingsSectionTone,
    children: Children,
) -> impl IntoView {
    let class = match tone {
        SettingsSectionTone::Default => "p-5",
        SettingsSectionTone::Danger => "p-5 border-negative-border bg-negative/5",
        SettingsSectionTone::Notification => "p-5 border-negative-border",
    };
    view! {
        <Card variant=CardVariant::Panel class=class>{children()}</Card>
    }
}

#[component]
pub fn SettingsLoginRequired() -> impl IntoView {
    view! { <div class="text-center py-12">{labels::LOGIN_REQUIRED_TITLE}</div> }
}

#[component]
pub fn AccountIdentity(
    name: Option<String>,
    email: String,
    picture_url: Option<String>,
    display_name: String,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-4">
            <UserAvatar picture_url=picture_url name=name email=email.clone() large=true alt=display_name.clone() />
            <div class="min-w-0">
                <div class="font-bold text-heading break-words">{display_name}</div>
                <div class="text-label text-text-muted break-all">{email}</div>
                <div class="text-label text-text-muted">{labels::SETTINGS_GOOGLE_LOGIN}</div>
            </div>
        </div>
    }
}

#[component]
pub fn SettingRow(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between gap-3 py-3 max-sm:flex-col max-sm:items-start">
            <div class="min-w-0"><div class="font-bold">{title}</div></div>
            <div class="shrink-0 flex items-center gap-2">{children()}</div>
        </div>
    }
}

#[component]
pub fn SettingsDangerRows(children: Children) -> impl IntoView {
    view! { <div class="space-y-3 border-t border-negative-border/50">{children()}</div> }
}

#[component]
pub fn SettingsDangerRow(
    title: &'static str,
    note: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between py-3">
            <div>
                <div class="font-bold text-negative">{title}</div>
                <div class="text-cell text-text-muted">{note}</div>
            </div>
            {children()}
        </div>
    }
}

#[component]
pub fn SettingsNote(#[prop(optional)] after: bool, children: Children) -> impl IntoView {
    view! { <p class=if after { "pt-2 text-cell text-text-muted" } else { "pb-3 text-cell text-text-muted" }>{children()}</p> }
}

#[component]
pub fn SettingsFeedback(message: String, failed: bool) -> impl IntoView {
    let tone = if failed {
        "text-negative"
    } else {
        "text-text-muted"
    };
    view! { <p class=format!("pb-2 text-cell {tone}")>{message}</p> }
}

#[component]
pub fn SettingsNotificationHeading() -> impl IntoView {
    view! {
        <h2 id="notify-logout-heading" class="text-heading font-bold mb-2">{labels::NOTIFY_LOGOUT_TITLE}</h2>
        <p class="pb-3 text-body">{labels::NOTIFY_LOGOUT_DESCRIPTION}</p>
    }
}

#[component]
pub fn SettingsNotificationActions(children: Children) -> impl IntoView {
    view! { <div class="space-y-2">{children()}</div> }
}

#[component]
pub fn SettingsRetry(children: Children) -> impl IntoView {
    view! { <div class="mt-4">{children()}</div> }
}

#[component]
pub fn DeviceList(children: Children) -> impl IntoView {
    view! { <ul class="divide-y divide-ink/10 border-t border-ink/10" role="list">{children()}</ul> }
}

#[component]
pub fn DeviceRow(
    datetime: String,
    description: String,
    current: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <li class="flex items-center justify-between gap-3 py-3 max-sm:flex-col max-sm:items-stretch">
            <div class="min-w-0">
                <div class="flex flex-wrap items-center gap-2">
                    <span class="tabular-nums text-cell font-normal">{datetime}</span>
                    {current.then(|| view! { <Badge variant=BadgeVariant::Info>{labels::DEVICE_CURRENT}</Badge> })}
                </div>
                <div class="mt-1 text-body break-words">{description}</div>
                <div class="text-cell text-text-muted">{labels::DEVICE_COL_LAST_USED}</div>
            </div>
            <div class="shrink-0">{children()}</div>
        </li>
    }
}

#[component]
pub fn DeviceDangerActions(children: Children) -> impl IntoView {
    view! { <div class="mt-4 border-t border-negative-border/50 pt-4">{children()}</div> }
}
