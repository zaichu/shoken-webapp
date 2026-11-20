use crate::api::{ApiClient, ApiError};
#[cfg(test)]
mod tests;
use crate::session;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::confirm_modal::ConfirmDeleteModal;
use crate::ui::empty_state::EmptyState;
use crate::ui::labels;
use crate::ui::list_row::ListRowStatic;
use crate::ui::search_entry::ShowMoreButton;
use crate::ui::section_heading::SectionHeading;
use crate::ui::settings as settings_ui;
use crate::ui::state::{ListLoadError, ListSkeleton, ListSkeletonVariant};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct OperationLogEntry {
    occurred_at: String,
    operation: String,
    target: String,
    count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct OperationLogListResponse {
    #[serde(default)]
    data: Vec<OperationLogEntry>,
}

fn result_badge_variant(result: &str) -> BadgeVariant {
    match result {
        "failure" => BadgeVariant::Danger,
        _ => BadgeVariant::Positive,
    }
}

fn result_label(result: &str) -> &'static str {
    match result {
        "failure" => labels::LOGIN_RESULT_FAILURE,
        // 新しい端末・場所の変化もログイン自体は成功。注意は別の列に出す
        _ => labels::LOGIN_RESULT_SUCCESS,
    }
}

/// 注意の種類(バッジの文言・配色・短い補足)。成否以外の記録だけ返す
pub(crate) fn attention_of(result: &str) -> Option<(&'static str, BadgeVariant, &'static str)> {
    match result {
        "new_device" => Some((
            labels::LOGIN_RESULT_NEW_DEVICE,
            BadgeVariant::Info,
            labels::LOGIN_ATTENTION_NEW_DEVICE_NOTE,
        )),
        "location_changed" => Some((
            labels::LOGIN_RESULT_LOCATION_CHANGED,
            BadgeVariant::Accent,
            labels::LOGIN_ATTENTION_LOCATION_CHANGED_NOTE,
        )),
        _ => None,
    }
}

pub(crate) fn format_place(country: &Option<String>, region: &Option<String>) -> String {
    shared::login_display::format_place(country.as_deref(), region.as_deref())
}

pub(crate) fn format_agent(browser: &Option<String>, os: &Option<String>) -> String {
    shared::login_display::format_agent_with_unknown_label(
        browser.as_deref(),
        os.as_deref(),
        labels::LOGIN_UNKNOWN_BROWSER,
    )
}

pub(crate) fn format_datetime(occurred_at: &str) -> String {
    shared::login_display::format_iso_utc_as_jst(occurred_at)
        .unwrap_or_else(|| occurred_at.to_string())
}

const HISTORY_PAGE_SIZE: i64 = 10;
const HISTORY_MAX_TOTAL: usize = 500;

async fn fetch_history_page(limit: i64, offset: i64) -> Result<Vec<LoginHistoryEntry>, ApiError> {
    let limit_value = limit.to_string();
    let offset_value = offset.to_string();
    ApiClient::read_client()
        .get_json::<LoginHistoryListResponse>(
            "/api/v1/login-history",
            &[
                ("limit", limit_value.as_str()),
                ("offset", offset_value.as_str()),
            ],
        )
        .await
        .map(|response| response.data)
}

async fn fetch_devices() -> Result<Vec<DeviceEntry>, ApiError> {
    ApiClient::read_client()
        .get_json::<DeviceListResponse>("/api/v1/devices", &[])
        .await
        .map(|response| response.data)
}

async fn fetch_operation_log_page(
    limit: i64,
    offset: i64,
) -> Result<Vec<OperationLogEntry>, ApiError> {
    let limit_value = limit.to_string();
    let offset_value = offset.to_string();
    ApiClient::read_client()
        .get_json::<OperationLogListResponse>(
            "/api/v1/operation-logs",
            &[
                ("limit", limit_value.as_str()),
                ("offset", offset_value.as_str()),
            ],
        )
        .await
        .map(|response| response.data)
}

fn operation_label(operation: &str) -> &'static str {
    match operation {
        "account_delete" => labels::OPERATION_LOG_ACCOUNT_DELETE,
        "logout_all" => labels::OPERATION_LOG_LOGOUT_ALL,
        "device_revoke" => labels::OPERATION_LOG_DEVICE_REVOKE,
        _ => labels::OPERATION_LOG_CSV_DELETE,
    }
}

fn operation_target_label(target: &str) -> &'static str {
    match target {
        "dividends" => labels::OPERATION_LOG_TARGET_DIVIDENDS,
        "domestic_stocks" => labels::OPERATION_LOG_TARGET_DOMESTIC_STOCKS,
        "mutualfunds" => labels::OPERATION_LOG_TARGET_MUTUAL_FUNDS,
        "asset_balances" => labels::OPERATION_LOG_TARGET_ASSET_BALANCES,
        "fund_balances" => labels::OPERATION_LOG_TARGET_FUND_BALANCES,
        "sessions" => labels::OPERATION_LOG_TARGET_SESSIONS,
        "device" => labels::OPERATION_LOG_TARGET_DEVICE,
        _ => labels::OPERATION_LOG_TARGET_ACCOUNT,
    }
}

#[component]
pub fn SectionOperationLog() -> impl IntoView {
    let entries = RwSignal::new(Vec::<OperationLogEntry>::new());
    let initial = RwSignal::new(Option::<Result<(), String>>::None);
    let has_more = RwSignal::new(false);
    let loading_more = RwSignal::new(false);
    let more_error = RwSignal::new(Option::<String>::None);
    let load = move || {
        initial.set(None);
        has_more.set(false);
        more_error.set(None);
        leptos::task::spawn_local(async move {
            match fetch_operation_log_page(HISTORY_PAGE_SIZE, 0).await {
                Ok(rows) => {
                    has_more.set(rows.len() == HISTORY_PAGE_SIZE as usize);
                    entries.set(rows);
                    initial.set(Some(Ok(())));
                }
                Err(error) => initial.set(Some(Err(error.user_message()))),
            }
        });
    };
    let load_more = move || {
        if loading_more.get_untracked() {
            return;
        }
        loading_more.set(true);
        leptos::task::spawn_local(async move {
            let offset = i64::try_from(entries.with_untracked(Vec::len)).unwrap_or(i64::MAX);
            match fetch_operation_log_page(HISTORY_PAGE_SIZE, offset).await {
                Ok(rows) => {
                    has_more.set(rows.len() == HISTORY_PAGE_SIZE as usize);
                    entries.update(|all| all.extend(rows));
                }
                Err(error) => more_error.set(Some(error.user_message())),
            }
            loading_more.set(false);
        });
    };
    Effect::new(move |_| load());

    view! {
        <settings_ui::SettingsSection id="operation-log" labelledby="operation-log-heading">
            <settings_ui::SettingsPanel>
                <SectionHeading id="operation-log-heading">{labels::OPERATION_LOG_SECTION}</SectionHeading>
                {move || match initial.get() {
                    None => view! { <ListSkeleton variant=ListSkeletonVariant::Table /> }.into_any(),
                    Some(Err(message)) => view! { <ListLoadError message=message on_retry=move || load() /> }.into_any(),
                    Some(Ok(())) if entries.with(Vec::is_empty) => view! { <EmptyState title=labels::OPERATION_LOG_EMPTY.to_string() /> }.into_any(),
                    Some(Ok(())) => view! {
                        <ul class="asset-compact-list" role="list">
                            {entries.with(std::clone::Clone::clone).into_iter().map(|entry| {
                                let title = format!("{} · {}", operation_label(&entry.operation), operation_target_label(&entry.target));
                                let datetime = self::format_datetime(&entry.occurred_at);
                                let count = format!("{} {}", entry.count, labels::OPERATION_LOG_COUNT);
                                view! {
                                    <li class="list-row-item">
                                        <ListRowStatic name=move || datetime.clone()
                                            primary=move || view! { {title.clone()} }
                                            meta=move || count.clone() secondary=move || String::new() />
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                        {move || more_error.get().map(|message| view! { <settings_ui::SettingsRetry><ListLoadError message=message on_retry=move || load_more() /></settings_ui::SettingsRetry> })}
                        {move || has_more.get().then(|| view! { <ShowMoreButton testid="operation-log-more" disabled=Signal::derive(move || loading_more.get()) on_click=load_more /> })}
                    }.into_any(),
                }}
            </settings_ui::SettingsPanel>
        </settings_ui::SettingsSection>
    }
}

#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned by the view"
)]
pub(crate) fn ResultBadge(result: String) -> impl IntoView {
    let label = self::result_label(&result);
    let variant = self::result_badge_variant(&result);
    view! { <Badge variant=variant>{label}</Badge> }
}

/// 注意の列・行。該当しない記録は「—」にする
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned by the view"
)]
pub(crate) fn AttentionCell(result: String) -> impl IntoView {
    match self::attention_of(&result) {
        Some((label, variant, note)) => view! {
            <Badge variant=variant>{label}</Badge>
            {" "}
            {note}
        }
        .into_any(),
        None => view! { {labels::MISSING} }.into_any(),
    }
}

/// スマホ行の primary に付ける注意のバッジ。該当しない記録は何も出さない
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned by the view"
)]
pub(crate) fn AttentionBadge(result: String) -> impl IntoView {
    match self::attention_of(&result) {
        Some((label, variant, _)) => view! { <Badge variant=variant>{label}</Badge> }.into_any(),
        None => ().into_any(),
    }
}

/// スマホ行の meta に付ける注意の短い補足。該当しない記録は何も出さない
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned by the view"
)]
pub(crate) fn AttentionNote(result: String) -> impl IntoView {
    match self::attention_of(&result) {
        Some((_, _, note)) => view! { {" · "} {note} }.into_any(),
        None => ().into_any(),
    }
}

#[component]
pub fn SectionLoginHistory() -> impl IntoView {
    let entries = RwSignal::new(Vec::<LoginHistoryEntry>::new());
    let initial = RwSignal::new(Option::<Result<(), String>>::None);
    let has_more = RwSignal::new(false);
    let loading_more = RwSignal::new(false);
    let more_error = RwSignal::new(Option::<String>::None);
    let load = move || {
        initial.set(None);
        more_error.set(None);
        has_more.set(false);
        leptos::task::spawn_local(async move {
            match fetch_history_page(HISTORY_PAGE_SIZE, 0).await {
                Ok(rows) => {
                    has_more
                        .set(i64::try_from(rows.len()).unwrap_or(i64::MAX) == HISTORY_PAGE_SIZE);
                    entries.set(rows);
                    initial.set(Some(Ok(())));
                }
                Err(error) => initial.set(Some(Err(error.user_message()))),
            }
        });
    };
    let load_more = move || {
        if loading_more.get_untracked() {
            return;
        }
        loading_more.set(true);
        more_error.set(None);
        leptos::task::spawn_local(async move {
            let offset = i64::try_from(entries.with_untracked(Vec::len)).unwrap_or(i64::MAX);
            match fetch_history_page(HISTORY_PAGE_SIZE, offset).await {
                Ok(rows) => {
                    let exhausted =
                        i64::try_from(rows.len()).unwrap_or(i64::MAX) != HISTORY_PAGE_SIZE;
                    entries.update(|all| {
                        all.extend(rows);
                        all.truncate(HISTORY_MAX_TOTAL);
                    });
                    if exhausted || entries.with_untracked(Vec::len) >= HISTORY_MAX_TOTAL {
                        has_more.set(false);
                    }
                }
                Err(error) => more_error.set(Some(error.user_message())),
            }
            loading_more.set(false);
        });
    };
    Effect::new(move |_| {
        load();
    });

    view! {
        <settings_ui::SettingsSection id="login-history" labelledby="login-history-heading">
            <settings_ui::SettingsPanel>
                <SectionHeading id="login-history-heading">
                    {labels::LOGIN_HISTORY_SECTION}
                </SectionHeading>
                <settings_ui::SettingsNote>{labels::LOGIN_HISTORY_COOKIE_NOTE}</settings_ui::SettingsNote>
                <settings_ui::SettingsNote>{labels::LOGIN_HISTORY_LEGEND}</settings_ui::SettingsNote>
                {move || match initial.get() {
                    None => view! { <ListSkeleton variant=ListSkeletonVariant::Table /> }.into_any(),
                    Some(Err(message)) => {
                        view! {
                            <ListLoadError message=message on_retry=move || load() />
                        }
                            .into_any()
                    }
                    Some(Ok(())) => {
                        if entries.with(Vec::is_empty) {
                            view! { <EmptyState title=labels::LOGIN_HISTORY_EMPTY.to_string() /> }
                                .into_any()
                        } else {
                            let desktop_rows = entries.with(std::clone::Clone::clone);
                            let mobile_rows = entries.with(std::clone::Clone::clone);
                            view! {
                                <div class="table-card max-sm:hidden">
                                    <table class="asset-table login-history-table">
                                        <thead>
                                            <tr>
                                                <th scope="col">{labels::LOGIN_HISTORY_COL_DATETIME}</th>
                                                <th scope="col">{labels::LOGIN_HISTORY_COL_RESULT}</th>
                                                <th scope="col">{labels::LOGIN_HISTORY_COL_ATTENTION}</th>
                                                <th scope="col">{labels::LOGIN_HISTORY_COL_PLACE}</th>
                                                <th scope="col">{labels::LOGIN_HISTORY_COL_AGENT}</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {desktop_rows
                                                .into_iter()
                                                .map(|entry| {
                                                    let place = self::format_place(
                                                        &entry.country,
                                                        &entry.region,
                                                    );
                                                    let agent = self::format_agent(&entry.browser, &entry.os);
                                                    view! {
                                                        <tr>
                                                            <td class="login-history-datetime tabular-nums">
                                                                {self::format_datetime(&entry.occurred_at)}
                                                            </td>
                                                            <td class="login-history-result">
                                                                <ResultBadge result=entry.result.clone() />
                                                            </td>
                                                            <td class="login-history-result">
                                                                <AttentionCell result=entry.result.clone() />
                                                            </td>
                                                            <td class="login-history-place">{place}</td>
                                                            <td class="login-history-agent">{agent}</td>
                                                        </tr>
                                                    }
                                                })
                                                .collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                                <ul class="asset-compact-list login-history-compact sm:hidden" role="list">
                                    {mobile_rows
                                        .into_iter()
                                        .map(|entry| {
                                            let place = self::format_place(&entry.country, &entry.region);
                                            let agent = self::format_agent(&entry.browser, &entry.os);
                                            let datetime = self::format_datetime(&entry.occurred_at);
                                            let attention_result = entry.result.clone();
                                            view! {
                                                <li class="list-row-item">
                                                    <ListRowStatic
                                                        name=move || datetime.clone()
                                                        primary=move || view! {
                                                            <ResultBadge result=entry.result.clone() />
                                                            {" "}
                                                            <AttentionBadge result=entry.result.clone() />
                                                        }
                                                        meta=move || view! {
                                                            {place.clone()}
                                                            <AttentionNote result=attention_result.clone() />
                                                        }
                                                        secondary=move || agent.clone()
                                                    />
                                                </li>
                                            }
                                        })
                                        .collect_view()}
                                </ul>
                                {move || {
                                    more_error
                                        .get()
                                        .map(|message| {
                                            view! {
                                                <settings_ui::SettingsRetry>
                                                    <ListLoadError
                                                        message=message
                                                        on_retry=move || load_more()
                                                    />
                                                </settings_ui::SettingsRetry>
                                            }
                                        })
                                }}
                                {move || {
                                    has_more
                                        .get()
                                        .then(|| {
                                            view! {
                                                <ShowMoreButton
                                                    testid="login-history-more"
                                                    disabled=Signal::derive(move || loading_more.get())
                                                    on_click=load_more
                                                />
                                            }
                                        })
                                }}
                            }
                                .into_any()
                        }
                    }
                }}
            </settings_ui::SettingsPanel>
        </settings_ui::SettingsSection>
    }
}

#[component]
pub fn SectionDevices() -> impl IntoView {
    let session = session::use_session();
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
        <settings_ui::SettingsSection id="devices" labelledby="devices-heading">
            <settings_ui::SettingsPanel>
                <SectionHeading id="devices-heading">
                    {labels::REGISTERED_DEVICES_SECTION}
                </SectionHeading>
                <settings_ui::SettingsNote>{labels::LOGIN_HISTORY_COOKIE_NOTE}</settings_ui::SettingsNote>
                {move || match devices.get() {
                    None => view! { <ListSkeleton variant=ListSkeletonVariant::Table /> }.into_any(),
                    Some(Err(message)) => {
                        view! { <ListLoadError message=message on_retry=move || load() /> }
                            .into_any()
                    }
                    Some(Ok(entries)) => {
                        if entries.is_empty() {
                            view! { <EmptyState title=labels::DEVICE_EMPTY.to_string() /> }
                                .into_any()
                        } else {
                            view! {
                                <settings_ui::DeviceList>
                                    {entries
                                        .into_iter()
                                        .map(|entry| {
                                            let revoke_entry = entry.clone();
                                            let (country, region) = entry.device_place();
                                            let place = self::format_place(&country, &region);
                                            let agent = self::format_agent(&entry.browser, &entry.os);
                                            view! {
                                                <settings_ui::DeviceRow
                                                    datetime=self::format_datetime(&entry.last_used_at)
                                                    description=format!("{place} · {agent}")
                                                    current=entry.is_current
                                                >
                                                    <Button
                                                        variant=ButtonVariant::Secondary(ButtonSize::Sm)
                                                        testid="device-revoke"
                                                        on_click=move |_| start_revoke(revoke_entry.clone())
                                                    >
                                                        {labels::DEVICE_REVOKE}
                                                    </Button>
                                                </settings_ui::DeviceRow>
                                            }
                                        })
                                        .collect_view()}
                                </settings_ui::DeviceList>
                            }
                                .into_any()
                        }
                    }
                }}
                <settings_ui::DeviceDangerActions>
                    <Button
                        variant=ButtonVariant::SecondaryDanger(ButtonSize::Sm)
                        testid="device-logout-all"
                        on_click=move |_| {
                            logout_failed.set(None);
                            logout_confirm_open.set(true);
                        }
                    >
                        {labels::DEVICE_LOGOUT_ALL}
                    </Button>
                </settings_ui::DeviceDangerActions>
            </settings_ui::SettingsPanel>
            {move || {
                pending_revoke.get().map(|entry| {
                    let place = self::format_place(&entry.device_place().0, &entry.device_place().1);
                    view! {
                        <ConfirmDeleteModal
                            title=labels::REGISTERED_DEVICES_SECTION.to_string()
                            description=place.clone()
                            confirm_label=labels::device_revoke_confirm_button(&place)
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
                            title=labels::DEVICE_LOGOUT_ALL.to_string()
                            description=labels::LOGIN_HISTORY_COOKIE_NOTE.to_string()
                            confirm_label=labels::DEVICE_LOGOUT_ALL.to_string()
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
        </settings_ui::SettingsSection>
    }
}

impl DeviceEntry {
    fn device_place(&self) -> (Option<String>, Option<String>) {
        (self.country.clone(), self.region.clone())
    }
}
