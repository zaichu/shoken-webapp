mod export;
mod login_history;
mod toc;
pub(crate) use login_history::{
    AttentionBadge, AttentionCell, AttentionNote, ResultBadge, format_agent, format_datetime,
    format_place,
};

use crate::session;
use crate::settings;
use crate::settings::{AmountUnit, AssetSort, Theme};
use crate::ui::button::{Button, ButtonSize, ButtonVariant, LinkButton, Switch};
use crate::ui::labels;
use crate::ui::more_link::MoreLink;
use crate::ui::page_frame::PageFrame;
use crate::ui::section_heading::{SectionHeading, SectionHeadingTone};
use crate::ui::segmented::Segmented;
use crate::ui::select::{Select, SelectVariant};
use crate::ui::settings as settings_ui;
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use export::ExportKind;
use leptos::prelude::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let session = session::use_session();
    let user = move || session.user.get();

    let active = RwSignal::new(toc::initial_section());
    let locked = RwSignal::new(false);
    Effect::new(move |_| {
        toc::track_toc(active, locked);
    });

    view! {
        <PageFrame title=labels::SETTINGS_TITLE>
            <settings_ui::SettingsLayout navigation=view! { <toc::TocNav active=active locked=locked /> }.into_any()>
                <SectionNotifyLogout />
                {move || {
                    user().map_or_else(|| view! { <settings_ui::SettingsLoginRequired /> }.into_any(), |user| view! {
                            <SectionAccount user=user />
                            <SectionDisplay />
                            <SectionData />
                            <SectionSecurity />
                            <login_history::SectionDevices />
                            <login_history::SectionLoginHistory />
                            <SectionDanger />
                        }.into_any())
                }}
            </settings_ui::SettingsLayout>
        </PageFrame>
    }
}

#[component]
fn SectionAccount(user: crate::api::dto::SessionUser) -> impl IntoView {
    let display_name = user
        .name
        .clone()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| user.email.clone());

    view! {
        <settings_ui::SettingsSection id="account" labelledby="account-heading">
            <settings_ui::SettingsPanel>
                <SectionHeading id="account-heading">{labels::ACCOUNT_BUTTON}</SectionHeading>
                <settings_ui::AccountIdentity
                    name=user.name.clone()
                    email=user.email.clone()
                    picture_url=user.picture_url.clone()
                    display_name=display_name
                />
            </settings_ui::SettingsPanel>
        </settings_ui::SettingsSection>
    }
}

#[component]
fn SectionDisplay() -> impl IntoView {
    let settings = settings::use_settings();
    let asset_sort = settings.asset_sort;
    let initial_tab = settings.receipts_initial_tab;
    let amount_unit = settings.amount_unit;
    let theme = settings.theme;

    view! {
        <settings_ui::SettingsSection id="display" labelledby="display-heading">
            <settings_ui::SettingsPanel>
                <SectionHeading id="display-heading">{labels::SETTINGS_DISPLAY_SECTION}</SectionHeading>
                <div class="divide-y divide-ink/10 border-t border-ink/10">
                    <settings_ui::SettingRow title=labels::SETTINGS_THEME>
                        <Segmented
                            options=&self::THEME_SEGMENTS
                            value=Signal::derive(move || theme.get().value())
                            aria_label=labels::SETTINGS_THEME
                            testid="setting-theme"
                            on_change=move |value| {
                                if let Some(parsed) = Theme::from_value(&value) {
                                    theme.set(parsed);
                                }
                            }
                        />
                    </settings_ui::SettingRow>
                    <settings_ui::SettingRow title=labels::SETTINGS_ASSET_SORT>
                        <Select
                            value=Signal::derive(move || asset_sort.get().value())
                            aria_label=labels::SETTINGS_ASSET_SORT_INITIAL.to_string()
                            testid="setting-asset-sort"
                            variant=SelectVariant::Setting
                            on_change=move |value| {
                                if let Some(sort) = AssetSort::parse(&value) {
                                    asset_sort.set(sort);
                                }
                            }
                        >
                            {AssetSort::ALL.iter().map(|sort| {
                                view! {
                                    <option value={sort.value()}>{sort.label()}</option>
                                }
                            }).collect_view()}
                        </Select>
                    </settings_ui::SettingRow>
                    <settings_ui::SettingRow title=labels::SETTINGS_RECEIPTS_TAB>
                        <Select
                            value=Signal::derive(move || initial_tab.get().kind_param().to_string())
                            aria_label=labels::SETTINGS_RECEIPTS_TAB.to_string()
                            testid="setting-receipts-tab"
                            variant=SelectVariant::Setting
                            on_change=move |value| {
                                if let Some(tab) = crate::features::receipts::ReceiptsTab::ALL
                                    .iter()
                                    .copied()
                                    .find(|tab| tab.kind_param() == value)
                                {
                                    initial_tab.set(tab);
                                }
                            }
                        >
                            {crate::features::receipts::ReceiptsTab::ALL.iter().map(|tab| {
                                view! {
                                    <option value={tab.kind_param()}>{tab.label()}</option>
                                }
                            }).collect_view()}
                        </Select>
                    </settings_ui::SettingRow>
                    <settings_ui::SettingRow title=labels::SETTINGS_AMOUNT_UNIT>
                        <Segmented
                            options=&[
                                ("yen", labels::YEN),
                                ("sen", labels::THOUSAND_YEN),
                                ("man", labels::TEN_THOUSAND_YEN),
                            ]
                            value=Signal::derive(move || amount_unit.get().value())
                            aria_label=labels::SETTINGS_AMOUNT_UNIT
                            testid="setting-amount-unit"
                            on_change=move |value| {
                                if let Some(unit) = AmountUnit::from_value(&value) {
                                    amount_unit.set(unit);
                                }
                            }
                        />
                    </settings_ui::SettingRow>
                </div>
                <settings_ui::SettingsNote after=true>{labels::SETTINGS_AMOUNT_UNIT_NOTE}</settings_ui::SettingsNote>
            </settings_ui::SettingsPanel>
        </settings_ui::SettingsSection>
    }
}

#[component]
fn SectionData() -> impl IntoView {
    let exporting = RwSignal::new(Option::<ExportKind>::None);
    let export_message = RwSignal::new(Option::<String>::None);
    let export_failed = RwSignal::new(false);

    let on_export = move |kind: ExportKind| {
        exporting.set(Some(kind));
        export_message.set(None);
        export_failed.set(false);
        leptos::task::spawn_local(async move {
            match export::fetch_csv(kind).await {
                Ok((filename, content)) => {
                    export::download_csv(&filename, &content);
                    export_message.set(Some(labels::settings_export_saved(&filename)));
                }
                Err(error) => {
                    export_failed.set(true);
                    export_message.set(Some(labels::settings_export_failed(
                        kind.label(),
                        &error.user_message(),
                    )));
                }
            }
            exporting.set(None);
        });
    };

    view! {
        <settings_ui::SettingsSection id="data" labelledby="data-heading">
            <settings_ui::SettingsPanel>
                <SectionHeading id="data-heading">{labels::SETTINGS_DATA_SECTION}</SectionHeading>
                <div class="divide-y divide-ink/10 border-t border-ink/10">
                    <settings_ui::SettingRow title=labels::SETTINGS_EXPORT_CSV>
                        {ExportKind::ALL.iter().map(|kind| {
                            let kind = *kind;
                            view! {
                                <Button
                                    variant=ButtonVariant::SecondaryNoWrap(ButtonSize::Sm)
                                    disabled=Signal::derive(move || exporting.get().is_some())
                                    testid=match kind {
                                        ExportKind::Dividend => "export-dividends",
                                        ExportKind::AssetBalance => "export-asset-balances",
                                        ExportKind::Transaction => "export-transactions",
                                    }
                                    on_click=move |_| on_export(kind)
                                >
                                    {move || {
                                        if exporting.get() == Some(kind) {
                                            labels::CSV_MANAGE_FETCH_RUNNING.to_string()
                                        } else {
                                            kind.label().to_string()
                                        }
                                    }}
                                </Button>
                            }
                        }).collect_view()}
                    </settings_ui::SettingRow>
                    {move || {
                        export_message.get().map(|message| {
                            view! { <settings_ui::SettingsFeedback message=message failed=export_failed.get() /> }
                        })
                    }}
                    <settings_ui::SettingRow title=labels::SETTINGS_CSV_IMPORT_DELETE>
                        <MoreLink href="/csv".to_string() label=labels::SETTINGS_CSV_MANAGE_LINK.to_string() />
                    </settings_ui::SettingRow>
                </div>
            </settings_ui::SettingsPanel>
        </settings_ui::SettingsSection>
    }
}

#[component]
fn SectionSecurity() -> impl IntoView {
    let settings = settings::use_settings();
    let idle_minutes = settings.idle_logout_minutes;
    let notify = RwSignal::new(Option::<Result<bool, String>>::None);
    let notify_pending = RwSignal::new(false);
    let load_notify = move || {
        notify.set(None);
        leptos::task::spawn_local(async move {
            notify.set(Some(
                crate::api::ApiClient::read_client()
                    .get_json::<NotifySettingsResponse>("/api/v1/notification-settings", &[])
                    .await
                    .map(|response| response.login_notify_enabled)
                    .map_err(|error| error.user_message()),
            ));
        });
    };
    Effect::new(move |_| {
        load_notify();
    });
    let on_toggle = move |enabled: bool| {
        notify_pending.set(true);
        leptos::task::spawn_local(async move {
            let result = crate::api::ApiClient::default_client()
                .put_json::<NotifySettingsUpdate, NotifySettingsResponse>(
                    "/api/v1/notification-settings",
                    &NotifySettingsUpdate {
                        login_notify_enabled: enabled,
                    },
                )
                .await
                .map(|response| response.login_notify_enabled)
                .map_err(|error| error.user_message());
            notify_pending.set(false);
            notify.set(Some(result));
        });
    };

    view! {
        <settings_ui::SettingsSection id="security" labelledby="security-heading">
            <settings_ui::SettingsPanel>
                <SectionHeading id="security-heading">{labels::SETTINGS_SECURITY_SECTION}</SectionHeading>
                <div class="divide-y divide-ink/10 border-t border-ink/10">
                    <settings_ui::SettingRow title=labels::SETTINGS_IDLE_LOGOUT>
                        <Segmented
                            options=&self::IDLE_SEGMENTS
                            value=Signal::derive(move || idle_minutes.get().to_string())
                            aria_label=labels::SETTINGS_IDLE_LOGOUT_TIME
                            testid="setting-idle-minutes"
                            on_change=move |value| {
                                if let Ok(minutes) = value.parse::<u32>()
                                    && settings::IDLE_LOGOUT_CHOICES.contains(&minutes)
                                {
                                    idle_minutes.set(minutes);
                                }
                            }
                        />
                    </settings_ui::SettingRow>
                    <settings_ui::SettingRow title=labels::LOGIN_NOTIFY_TITLE>
                        {move || match notify.get() {
                            None => {
                                view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }
                                    .into_any()
                            }
                            Some(Err(message)) => {
                                view! {
                                    <ListLoadError message=message on_retry=load_notify />
                                }
                                    .into_any()
                            }
                            Some(Ok(enabled)) => {
                                view! {
                                    <Switch
                                        checked=Signal::derive(move || enabled)
                                        label=labels::LOGIN_NOTIFY_TITLE.to_string()
                                        testid="setting-login-notify"
                                        disabled=Signal::derive(move || notify_pending.get())
                                        on_toggle=on_toggle
                                    />
                                }
                                    .into_any()
                            }
                        }}
                    </settings_ui::SettingRow>
                </div>
                <settings_ui::SettingsNote after=true>{labels::LOGIN_NOTIFY_NOTE}</settings_ui::SettingsNote>
            </settings_ui::SettingsPanel>
        </settings_ui::SettingsSection>
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct NotifySettingsResponse {
    #[serde(default)]
    login_notify_enabled: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct NotifySettingsUpdate {
    login_notify_enabled: bool,
}

fn notify_token_from_search() -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    for pair in search.trim_start_matches('?').split('&') {
        let (key, value) = pair.split_once('=')?;
        if key == "notify_token" && !value.trim().is_empty() {
            return Some(value.trim().to_string());
        }
    }
    None
}

#[component]
fn SectionNotifyLogout() -> impl IntoView {
    let session = session::use_session();
    let token = RwSignal::new(self::notify_token_from_search());
    let state = RwSignal::new(Option::<Result<bool, String>>::None);
    let running = RwSignal::new(false);
    let done = RwSignal::new(false);
    let failed = RwSignal::new(Option::<String>::None);
    let check = move || {
        let Some(token) = token.get() else {
            return;
        };
        state.set(None);
        leptos::task::spawn_local(async move {
            let result = crate::api::ApiClient::read_client()
                .get_json::<NotifyLogoutInfo>(
                    "/api/v1/notification-logout",
                    &[("token", token.as_str())],
                )
                .await
                .map(|_| true)
                .map_err(|error| {
                    if error.is_unauthorized()
                        || matches!(error, crate::api::ApiError::Http { status: 404, .. })
                    {
                        labels::NOTIFY_LOGOUT_INVALID.to_string()
                    } else {
                        error.user_message()
                    }
                });
            state.set(Some(result));
        });
    };
    Effect::new(move |_| {
        if token.get().is_some() {
            check();
        }
    });
    let confirm = move || {
        let Some(value) = token.get() else {
            return;
        };
        running.set(true);
        failed.set(None);
        leptos::task::spawn_local(async move {
            let result = crate::api::ApiClient::default_client()
                .post_json::<NotifyLogoutRequest, crate::api::dto::MessageResponse>(
                    "/api/v1/notification-logout",
                    &NotifyLogoutRequest { token: value },
                )
                .await;
            running.set(false);
            match result {
                Ok(_) => {
                    done.set(true);
                    token.set(None);
                    session.mark_unauthenticated();
                }
                Err(error) => failed.set(Some(error.user_message())),
            }
        });
    };
    view! {
        {move || {
            token.get().map(|_| {
                view! {
                    <section id="notify-logout" aria-labelledby="notify-logout-heading">
                        <settings_ui::SettingsPanel tone=settings_ui::SettingsSectionTone::Notification>
                            <settings_ui::SettingsNotificationHeading />
                            {move || {
                                if done.get() {
                                    view! { <p role="status">{labels::NOTIFY_LOGOUT_DONE}</p> }.into_any()
                                } else {
                                    match state.get() {
                                        None => {
                                            view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }
                                                .into_any()
                                        }
                                        Some(Err(message)) => {
                                            view! {
                                                <Alert variant=AlertVariant::Danger>{message}</Alert>
                                            }
                                                .into_any()
                                        }
                                        Some(Ok(_)) => {
                                            view! {
                                                <settings_ui::SettingsNotificationActions>
                                                    <Button
                                                        variant=ButtonVariant::DangerSolid(ButtonSize::Sm)
                                                        disabled=Signal::derive(move || running.get())
                                                        testid="notify-logout-confirm"
                                                        on_click=move |_| confirm()
                                                    >
                                                        {labels::NOTIFY_LOGOUT_CONFIRM}
                                                    </Button>
                                                    {move || {
                                                        failed
                                                            .get()
                                                            .map(|message| {
                                                                view! {
                                                                    <Alert variant=AlertVariant::Danger>{message}</Alert>
                                                                }
                                                            })
                                                    }}
                                                </settings_ui::SettingsNotificationActions>
                                            }
                                                .into_any()
                                        }
                                    }
                                }
                            }}
                        </settings_ui::SettingsPanel>
                    </section>
                }
            })
        }}
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct NotifyLogoutRequest {
    token: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct NotifyLogoutInfo {
    #[serde(default)]
    expires_at: String,
}

const IDLE_SEGMENTS: [(&str, &str); 3] = [
    ("15", labels::SETTINGS_IDLE_15),
    ("30", labels::SETTINGS_IDLE_30),
    ("60", labels::SETTINGS_IDLE_60),
];

const THEME_SEGMENTS: [(&str, &str); 3] = [
    ("system", labels::SETTINGS_THEME_SYSTEM),
    ("light", labels::SETTINGS_THEME_LIGHT),
    ("dark", labels::SETTINGS_THEME_DARK),
];

#[component]
fn SectionDanger() -> impl IntoView {
    let delete_confirm_open = use_context::<crate::ui::site::AccountDeleteConfirm>()
        .map_or_else(|| RwSignal::new(false), |confirm| confirm.0);

    view! {
        <settings_ui::SettingsSection id="danger" labelledby="danger-heading">
            <settings_ui::SettingsPanel tone=settings_ui::SettingsSectionTone::Danger>
                <SectionHeading id="danger-heading" tone=SectionHeadingTone::Danger>{labels::SETTINGS_DANGER_SECTION}</SectionHeading>
                <settings_ui::SettingsDangerRows>
                    <settings_ui::SettingsDangerRow title=labels::SETTINGS_DELETE_DIVIDEND note=labels::SETTINGS_DELETE_DIVIDEND_NOTE>
                        <LinkButton
                            variant=ButtonVariant::SecondaryDanger(ButtonSize::Sm)
                            href="/csv"
                            class="no-print"
                            aria_label=labels::SETTINGS_DELETE_DIVIDEND_ACTION
                        >
                            {labels::SETTINGS_DELETE}
                        </LinkButton>
                    </settings_ui::SettingsDangerRow>
                    <settings_ui::SettingsDangerRow title=labels::SETTINGS_DELETE_STOCK note=labels::SETTINGS_DELETE_STOCK_NOTE>
                        <LinkButton
                            variant=ButtonVariant::SecondaryDanger(ButtonSize::Sm)
                            href="/csv"
                            class="no-print"
                            aria_label=labels::SETTINGS_DELETE_STOCK_ACTION
                        >
                            {labels::SETTINGS_DELETE}
                        </LinkButton>
                    </settings_ui::SettingsDangerRow>
                    <settings_ui::SettingsDangerRow title=labels::SETTINGS_DELETE_FUND note=labels::SETTINGS_DELETE_FUND_NOTE>
                        <LinkButton
                            variant=ButtonVariant::SecondaryDanger(ButtonSize::Sm)
                            href="/csv"
                            class="no-print"
                            aria_label=labels::SETTINGS_DELETE_FUND_ACTION
                        >
                            {labels::SETTINGS_DELETE}
                        </LinkButton>
                    </settings_ui::SettingsDangerRow>
                    <settings_ui::SettingsDangerRow title=labels::SETTINGS_DELETE_ASSETS note=labels::SETTINGS_DELETE_ASSETS_NOTE>
                        <LinkButton
                            variant=ButtonVariant::SecondaryDanger(ButtonSize::Sm)
                            href="/csv"
                            class="no-print"
                            aria_label=labels::SETTINGS_DELETE_ASSETS
                        >
                            {labels::SETTINGS_DELETE}
                        </LinkButton>
                    </settings_ui::SettingsDangerRow>
                    <settings_ui::SettingsDangerRow title=labels::SETTINGS_DELETE_ACCOUNT note=labels::SETTINGS_DELETE_ACCOUNT_NOTE>
                        <Button
                            variant=ButtonVariant::SecondaryDanger(ButtonSize::Sm)
                            on_click=move |_| delete_confirm_open.set(true)
                            aria_label=labels::SETTINGS_DELETE_ACCOUNT
                        >
                            {labels::SETTINGS_DELETE}
                        </Button>
                    </settings_ui::SettingsDangerRow>
                </settings_ui::SettingsDangerRows>
            </settings_ui::SettingsPanel>
        </settings_ui::SettingsSection>
    }
}
