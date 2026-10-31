use crate::api::{ApiClient, ApiError};
use crate::session::use_session;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::card::{Card, CardVariant};
use crate::ui::confirm_modal::ConfirmDeleteModal;
use crate::ui::empty_state::EmptyState;
use crate::ui::labels::{
    DEVICE_COL_LAST_USED, DEVICE_CURRENT, DEVICE_EMPTY, DEVICE_LOGOUT_ALL, DEVICE_REVOKE,
    LOGIN_HISTORY_COL_AGENT, LOGIN_HISTORY_COL_DATETIME, LOGIN_HISTORY_COL_PLACE,
    LOGIN_HISTORY_COL_RESULT, LOGIN_HISTORY_COOKIE_NOTE, LOGIN_HISTORY_EMPTY,
    LOGIN_HISTORY_SECTION, LOGIN_RESULT_FAILURE, LOGIN_RESULT_LOCATION_CHANGED,
    LOGIN_RESULT_NEW_DEVICE, LOGIN_RESULT_SUCCESS, REGISTERED_DEVICES_SECTION,
    device_revoke_confirm_button,
};
use crate::ui::state::{ListLoadError, Loading};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LoginHistoryEntry {
    occurred_at: String,
    result: String,
    country: Option<String>,
    region: Option<String>,
    browser: Option<String>,
    os: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LoginHistoryListResponse {
    #[serde(default)]
    data: Vec<LoginHistoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct DeviceEntry {
    id: String,
    last_used_at: String,
    country: Option<String>,
    region: Option<String>,
    browser: Option<String>,
    os: Option<String>,
    is_current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct DeviceListResponse {
    #[serde(default)]
    data: Vec<DeviceEntry>,
}

fn result_badge_variant(result: &str) -> BadgeVariant {
    match result {
        "new_device" => BadgeVariant::Info,
        "location_changed" => BadgeVariant::Accent,
        "failure" => BadgeVariant::Danger,
        _ => BadgeVariant::Positive,
    }
}

fn result_label(result: &str) -> &'static str {
    match result {
        "new_device" => LOGIN_RESULT_NEW_DEVICE,
        "location_changed" => LOGIN_RESULT_LOCATION_CHANGED,
        "failure" => LOGIN_RESULT_FAILURE,
        _ => LOGIN_RESULT_SUCCESS,
    }
}

fn format_place(country: &Option<String>, region: &Option<String>) -> String {
    match (country.as_deref(), region.as_deref()) {
        (Some(country), Some(region)) if !region.is_empty() => format!("{country} {region}"),
        (Some(country), _) => country.to_string(),
        (None, Some(region)) if !region.is_empty() => region.to_string(),
        _ => crate::ui::labels::MISSING.to_string(),
    }
}

fn format_agent(browser: &Option<String>, os: &Option<String>) -> String {
    match (browser.as_deref(), os.as_deref()) {
        (Some(browser), Some(os)) => format!("{browser} / {os}"),
        (Some(browser), None) => browser.to_string(),
        (None, Some(os)) => os.to_string(),
        _ => crate::ui::labels::MISSING.to_string(),
    }
}

fn format_datetime(occurred_at: &str) -> String {
    let date = crate::ui::format::jst_date(occurred_at);
    let time = crate::ui::format::format_jst_time(occurred_at);
    match date {
        Some((_, month, day)) => format!("{month}/{day} {time}"),
        None => occurred_at.to_string(),
    }
}

async fn fetch_history() -> Result<Vec<LoginHistoryEntry>, ApiError> {
    ApiClient::read_client()
        .get_json::<LoginHistoryListResponse>("/api/v1/login-history", &[("limit", "50")])
        .await
        .map(|response| response.data)
}

async fn fetch_devices() -> Result<Vec<DeviceEntry>, ApiError> {
    ApiClient::read_client()
        .get_json::<DeviceListResponse>("/api/v1/devices", &[])
        .await
        .map(|response| response.data)
}

#[component]
fn ResultBadge(result: String) -> impl IntoView {
    let label = result_label(&result);
    let variant = result_badge_variant(&result);
    view! { <Badge variant=variant>{label}</Badge> }
}

#[component]
pub fn SectionLoginHistory() -> impl IntoView {
    let history = RwSignal::new(Option::<Result<Vec<LoginHistoryEntry>, String>>::None);
    let load = move || {
        history.set(None);
        leptos::task::spawn_local(async move {
            history.set(Some(
                fetch_history().await.map_err(|error| error.user_message()),
            ));
        });
    };
    Effect::new(move |_| {
        load();
    });

    view! {
        <section id="login-history" aria-labelledby="login-history-heading">
            <Card variant=CardVariant::Panel class="p-5">
                <h2 id="login-history-heading" class="text-lg font-bold mb-4">
                    {LOGIN_HISTORY_SECTION}
                </h2>
                <p class="pb-3 text-xs text-text-muted">{LOGIN_HISTORY_COOKIE_NOTE}</p>
                {move || match history.get() {
                    None => view! { <Loading /> }.into_any(),
                    Some(Err(message)) => {
                        view! {
                            <ListLoadError message=message on_retry=move || load() />
                        }
                            .into_any()
                    }
                    Some(Ok(entries)) => {
                        if entries.is_empty() {
                            view! { <EmptyState title=LOGIN_HISTORY_EMPTY.to_string() /> }
                                .into_any()
                        } else {
                            let desktop_rows = entries.clone();
                            let mobile_rows = entries.clone();
                            view! {
                                <div class="table-frame overflow-x-auto max-sm:hidden">
                                    <table class="asset-table login-history-table">
                                        <thead>
                                            <tr>
                                                <th scope="col">{LOGIN_HISTORY_COL_DATETIME}</th>
                                                <th scope="col">{LOGIN_HISTORY_COL_RESULT}</th>
                                                <th scope="col">{LOGIN_HISTORY_COL_PLACE}</th>
                                                <th scope="col">{LOGIN_HISTORY_COL_AGENT}</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {desktop_rows
                                                .into_iter()
                                                .map(|entry| {
                                                    let place = format_place(
                                                        &entry.country,
                                                        &entry.region,
                                                    );
                                                    let agent = format_agent(&entry.browser, &entry.os);
                                                    view! {
                                                        <tr>
                                                            <td class="login-history-datetime tabular-nums">
                                                                {format_datetime(&entry.occurred_at)}
                                                            </td>
                                                            <td class="login-history-result">
                                                                <ResultBadge result=entry.result.clone() />
                                                            </td>
                                                            <td>{place}</td>
                                                            <td>{agent}</td>
                                                        </tr>
                                                    }
                                                })
                                                .collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                                <ul class="sm:hidden space-y-2" role="list">
                                    {mobile_rows
                                        .into_iter()
                                        .map(|entry| {
                                            let place = format_place(&entry.country, &entry.region);
                                            let agent = format_agent(&entry.browser, &entry.os);
                                            view! {
                                                <li>
                                                    <Card variant=CardVariant::Item class="p-3">
                                                        <div class="flex items-center justify-between gap-2">
                                                            <span class="tabular-nums text-sm font-medium">
                                                                {format_datetime(&entry.occurred_at)}
                                                            </span>
                                                            <ResultBadge result=entry.result.clone() />
                                                        </div>
                                                        <dl class="mt-2 space-y-1 text-sm">
                                                            <div class="flex gap-2">
                                                                <dt class="shrink-0 text-text-muted">
                                                                    {LOGIN_HISTORY_COL_PLACE}
                                                                </dt>
                                                                <dd class="min-w-0 break-words">{place}</dd>
                                                            </div>
                                                            <div class="flex gap-2">
                                                                <dt class="shrink-0 text-text-muted">
                                                                    {LOGIN_HISTORY_COL_AGENT}
                                                                </dt>
                                                                <dd class="min-w-0 break-words">{agent}</dd>
                                                            </div>
                                                        </dl>
                                                    </Card>
                                                </li>
                                            }
                                        })
                                        .collect_view()}
                                </ul>
                            }
                                .into_any()
                        }
                    }
                }}
            </Card>
        </section>
    }
}

#[component]
pub fn SectionDevices() -> impl IntoView {
    let session = use_session();
    let devices = RwSignal::new(Option::<Result<Vec<DeviceEntry>, String>>::None);
    let load = move || {
        devices.set(None);
        leptos::task::spawn_local(async move {
            devices.set(Some(
                fetch_devices().await.map_err(|error| error.user_message()),
            ));
        });
    };
    Effect::new(move |_| {
        load();
    });
    let pending_revoke = RwSignal::new(Option::<DeviceEntry>::None);
    let revoke_failed = RwSignal::new(Option::<String>::None);
    let revoke_running = RwSignal::new(false);
    let logout_confirm_open = RwSignal::new(false);
    let logout_running = RwSignal::new(false);
    let logout_failed = RwSignal::new(Option::<String>::None);

    let revoke_loading = Memo::new(move |_| revoke_running.get());
    let logout_loading = Memo::new(move |_| logout_running.get());

    let start_revoke = move |entry: DeviceEntry| {
        revoke_failed.set(None);
        pending_revoke.set(Some(entry));
    };
    let cancel_revoke = move || {
        if !revoke_running.get() {
            pending_revoke.set(None);
        }
    };
    let confirm_revoke = move || {
        let Some(entry) = pending_revoke.get() else {
            return;
        };
        let revoked_current = entry.is_current;
        revoke_running.set(true);
        revoke_failed.set(None);
        leptos::task::spawn_local(async move {
            let result = ApiClient::default_client()
                .delete_empty(&format!("/api/v1/devices/{}", entry.id))
                .await;
            match result {
                Ok(()) => {
                    pending_revoke.set(None);
                    if revoked_current {
                        session.mark_unauthenticated();
                    } else {
                        load();
                    }
                }
                Err(error) => revoke_failed.set(Some(error.user_message())),
            }
            revoke_running.set(false);
        });
    };
    let confirm_logout_all = move || {
        logout_running.set(true);
        logout_failed.set(None);
        leptos::task::spawn_local(async move {
            let result = ApiClient::default_client()
                .delete_empty("/api/v1/sessions")
                .await;
            match result {
                Ok(()) => {
                    logout_confirm_open.set(false);
                    session.mark_unauthenticated();
                }
                Err(error) => logout_failed.set(Some(error.user_message())),
            }
            logout_running.set(false);
        });
    };

    view! {
        <section id="devices" aria-labelledby="devices-heading">
            <Card variant=CardVariant::Panel class="p-5">
                <h2 id="devices-heading" class="text-lg font-bold mb-4">
                    {REGISTERED_DEVICES_SECTION}
                </h2>
                <p class="pb-3 text-xs text-text-muted">{LOGIN_HISTORY_COOKIE_NOTE}</p>
                {move || match devices.get() {
                    None => view! { <Loading /> }.into_any(),
                    Some(Err(message)) => {
                        view! { <ListLoadError message=message on_retry=move || load() /> }
                            .into_any()
                    }
                    Some(Ok(entries)) => {
                        if entries.is_empty() {
                            view! { <EmptyState title=DEVICE_EMPTY.to_string() /> }
                                .into_any()
                        } else {
                            view! {
                                <ul class="divide-y divide-ink/10 border-t border-ink/10" role="list">
                                    {entries
                                        .into_iter()
                                        .map(|entry| {
                                            let revoke_entry = entry.clone();
                                            let (country, region) = entry.device_place();
                                            let place = format_place(&country, &region);
                                            let agent = format_agent(&entry.browser, &entry.os);
                                            view! {
                                                <li class="flex items-center justify-between gap-3 py-3 max-sm:flex-col max-sm:items-stretch">
                                                    <div class="min-w-0">
                                                        <div class="flex flex-wrap items-center gap-2">
                                                            <span class="tabular-nums text-sm font-medium">
                                                                {format_datetime(&entry.last_used_at)}
                                                            </span>
                                                            {entry
                                                                .is_current
                                                                .then(|| {
                                                                    view! {
                                                                        <Badge variant=BadgeVariant::Info>
                                                                            {DEVICE_CURRENT}
                                                                        </Badge>
                                                                    }
                                                                })}
                                                        </div>
                                                        <div class="mt-1 text-sm break-words">
                                                            {format!("{place} · {agent}")}
                                                        </div>
                                                        <div class="text-xs text-text-muted">
                                                            {DEVICE_COL_LAST_USED}
                                                        </div>
                                                    </div>
                                                    <div class="shrink-0">
                                                        <Button
                                                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                                                            testid="device-revoke"
                                                            on_click=move |_| start_revoke(revoke_entry.clone())
                                                        >
                                                            {DEVICE_REVOKE}
                                                        </Button>
                                                    </div>
                                                </li>
                                            }
                                        })
                                        .collect_view()}
                                </ul>
                            }
                                .into_any()
                        }
                    }
                }}
                <div class="mt-4 border-t border-negative-border/50 pt-4">
                    <Button
                        variant=ButtonVariant::Secondary(ButtonSize::Sm)
                        class="shrink-0 whitespace-nowrap text-negative border-negative-border hover:bg-negative/10"
                        testid="device-logout-all"
                        on_click=move |_| {
                            logout_failed.set(None);
                            logout_confirm_open.set(true);
                        }
                    >
                        {DEVICE_LOGOUT_ALL}
                    </Button>
                </div>
            </Card>
            {move || {
                pending_revoke.get().map(|entry| {
                    let place = format_place(&entry.device_place().0, &entry.device_place().1);
                    view! {
                        <ConfirmDeleteModal
                            title=REGISTERED_DEVICES_SECTION.to_string()
                            description=place.clone()
                            confirm_label=device_revoke_confirm_button(&place)
                            loading=revoke_loading
                            error=revoke_failed
                            on_confirm=confirm_revoke
                            on_cancel=cancel_revoke
                        />
                    }
                })
            }}
            {move || {
                logout_confirm_open.get().then(|| {
                    view! {
                        <ConfirmDeleteModal
                            title=DEVICE_LOGOUT_ALL.to_string()
                            description=LOGIN_HISTORY_COOKIE_NOTE.to_string()
                            confirm_label=DEVICE_LOGOUT_ALL.to_string()
                            loading=logout_loading
                            error=logout_failed
                            on_confirm=confirm_logout_all
                            on_cancel=move || {
                                if !logout_running.get() {
                                    logout_confirm_open.set(false);
                                }
                            }
                        />
                    }
                })
            }}
        </section>
    }
}

impl DeviceEntry {
    fn device_place(&self) -> (Option<String>, Option<String>) {
        (self.country.clone(), self.region.clone())
    }
}
