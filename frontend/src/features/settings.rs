mod export;
mod login_history;
pub(crate) use login_history::{ResultBadge, format_agent, format_datetime, format_place};

use crate::session::use_session;
use crate::settings::{AmountUnit, AssetSort, IDLE_LOGOUT_CHOICES, Theme, use_settings};
use crate::ui::avatar::UserAvatar;
use crate::ui::button::{Button, ButtonSize, ButtonVariant, LinkButton, Switch};
use crate::ui::card::Card;
use crate::ui::card::CardVariant;
use crate::ui::labels;
use crate::ui::more_link::MoreLink;
use crate::ui::page_frame::PageFrame;
use crate::ui::segmented::Segmented;
use crate::ui::select::Select;
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use export::ExportKind;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

const SECTION_IDS: [&str; 7] = [
    "account",
    "display",
    "data",
    "security",
    "login-history",
    "devices",
    "danger",
];

fn initial_section() -> String {
    let fragment = web_sys::window()
        .and_then(|window| window.location().hash().ok())
        .unwrap_or_default();
    let fragment = fragment.trim_start_matches('#');
    if SECTION_IDS.contains(&fragment) {
        return fragment.to_string();
    }
    SECTION_IDS[0].to_string()
}

/// 目次の並びを DOM から読む(目次の並びと本文の並びは同じ前提)。
/// 並び替えがあっても追随がずれないようにするため、順序の正本は持たない
fn toc_ids_in_dom_order(document: &web_sys::Document) -> Vec<String> {
    let mut ids = Vec::new();
    let selector = format!(
        "nav[aria-label=\"{}\"] a[href^=\"/settings#\"]",
        labels::SETTINGS_TOC_LABEL
    );
    let Ok(links) = document.query_selector_all(&selector) else {
        return ids;
    };
    for index in 0..links.length() {
        let Some(anchor) = links.item(index) else {
            continue;
        };
        let anchor: &web_sys::Element = anchor.unchecked_ref();
        let Some(href) = anchor.get_attribute("href") else {
            continue;
        };
        if let Some(id) = href.strip_prefix("/settings#")
            && SECTION_IDS.contains(&id)
            && !ids.iter().any(|known| known == id)
        {
            ids.push(id.to_string());
        }
    }
    ids
}
/// 目次の強調をスクロールに追随させる。押下直後は押した項目に固定し、
/// 自分の指での操作があってから追随を再開する(末尾の項目は上端まで
/// 届かないため、押した直後のスクロールで上書きするとずれる)。
fn track_toc(active: RwSignal<String>, locked: RwSignal<bool>) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    // scroll はバブルしないため、内側のスクロール領域も拾えるよう
    // document のキャプチャ段で追従させる
    let sync = move || {
        if locked.get_untracked() {
            return;
        }
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        let Some(main) = document.get_element_by_id("main-content") else {
            return;
        };
        let header_bottom = document
            .query_selector(".pc-header")
            .ok()
            .flatten()
            .map(|header| header.get_bounding_client_rect().height())
            .unwrap_or_default();
        let line = main.get_bounding_client_rect().top() + header_bottom + 8.0;
        let ids = toc_ids_in_dom_order(&document);
        let mut current = ids.first().map(String::as_str).unwrap_or(SECTION_IDS[0]);
        for id in &ids {
            let top = document
                .get_element_by_id(id)
                .map(|section| section.get_bounding_client_rect().top())
                .unwrap_or(f64::MAX);
            if top <= line {
                current = id;
            }
        }
        active.set(current.to_string());
    };
    let scroll_listener: wasm_bindgen::closure::Closure<dyn FnMut()> =
        wasm_bindgen::closure::Closure::wrap(Box::new(sync) as Box<dyn FnMut()>);
    let _ = document.add_event_listener_with_callback_and_bool(
        "scroll",
        scroll_listener.as_ref().unchecked_ref(),
        true,
    );
    let unlock: wasm_bindgen::closure::Closure<dyn FnMut()> = wasm_bindgen::closure::Closure::wrap(
        Box::new(move || locked.set(false)) as Box<dyn FnMut()>,
    );
    for name in ["pointerdown", "wheel", "keydown"] {
        let _ = document.add_event_listener_with_callback_and_bool(
            name,
            unlock.as_ref().unchecked_ref(),
            true,
        );
    }
    let pair = send_wrapper::SendWrapper::new((document, scroll_listener, unlock));
    on_cleanup(move || {
        let (document, scroll_listener, unlock) = &*pair;
        let _ = document.remove_event_listener_with_callback_and_bool(
            "scroll",
            scroll_listener.as_ref().unchecked_ref(),
            true,
        );
        for name in ["pointerdown", "wheel", "keydown"] {
            let _ = document.remove_event_listener_with_callback_and_bool(
                name,
                unlock.as_ref().unchecked_ref(),
                true,
            );
        }
    });
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    let session = use_session();
    let user = move || session.user.get();

    let sections = [
        ("account", labels::ACCOUNT_BUTTON),
        ("display", labels::SETTINGS_DISPLAY_SECTION),
        ("data", labels::SETTINGS_DATA_SECTION),
        ("security", labels::SETTINGS_SECURITY_SECTION),
        ("devices", labels::REGISTERED_DEVICES_SECTION),
        ("login-history", labels::LOGIN_HISTORY_SECTION),
        ("danger", labels::SETTINGS_DANGER_SECTION),
    ];

    let active = RwSignal::new(initial_section());
    let locked = RwSignal::new(false);
    Effect::new(move |_| {
        track_toc(active, locked);
    });

    view! {
        <PageFrame title=labels::SETTINGS_TITLE>
            <div class="flex flex-col lg:flex-row gap-7">
                <nav class="hidden lg:block lg:w-48 shrink-0 lg:sticky lg:top-16 lg:self-start" aria-label=labels::SETTINGS_TOC_LABEL>
                    <ul class="flex flex-col gap-1" role="list">
                        {sections.iter().map(|(id, label)| {
                            let id = *id;
                            let label = *label;
                            let link_id = id.to_string();
                            let click_id = id.to_string();
                            view! {
                                <li>
                                    <a
                                        href={format!("/settings#{}", id)}
                                        data-testid={format!("settings-toc-{}", id)}
                                        aria-current=move || (active.get() == link_id).then_some("true")
                                        on:click=move |_| {
                                            active.set(click_id.clone());
                                            locked.set(true);
                                        }
                                        class="settings-toc-link flex min-h-11 items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-text-deep transition-colors no-underline hover:bg-surface-raised"
                                    >
                                        {label}
                                    </a>
                                </li>
                            }
                        }).collect_view()}
                    </ul>
                </nav>

                <div class="flex-1 min-w-0 space-y-4">
                    <SectionNotifyLogout />
                    {move || {
                        if let Some(user) = user() {
                            view! {
                                <SectionAccount user=user />
                                <SectionDisplay />
                                <SectionData />
                                <SectionSecurity />
                                <login_history::SectionDevices />
                                <login_history::SectionLoginHistory />
                                <SectionDanger />
                            }.into_any()
                        } else {
                            view! { <div class="text-center py-12">{labels::LOGIN_REQUIRED_TITLE}</div> }.into_any()
                        }
                    }}
                </div>
            </div>
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
    let login_method_text = labels::SETTINGS_GOOGLE_LOGIN;

    view! {
        <section id="account" aria-labelledby="account-heading" class="scroll-mt-18">
            <Card variant=CardVariant::Panel class="p-5">
                <h2 id="account-heading" class="text-lg font-bold mb-4">{labels::ACCOUNT_BUTTON}</h2>
                <div class="flex items-center gap-4">
                    <UserAvatar
                        picture_url=user.picture_url.clone()
                        name=user.name.clone()
                        email=user.email.clone()
                        large=true
                        alt=display_name.clone()
                    />
                    <div class="min-w-0">
                        <div class="font-bold text-lg break-words">{display_name}</div>
                        <div class="text-sm text-text-muted break-all">{user.email.clone()}</div>
                        <div class="text-sm text-text-muted">{login_method_text}</div>
                    </div>
                </div>
            </Card>
        </section>
    }
}

/// セクション内の1行。右側に操作(子要素)を置く
#[component]
fn SettingRow(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between gap-3 py-3 max-sm:flex-col max-sm:items-start">
            <div class="min-w-0">
                <div class="font-medium">{title}</div>
            </div>
            <div class="shrink-0 flex items-center gap-2">{children()}</div>
        </div>
    }
}

#[component]
fn SectionDisplay() -> impl IntoView {
    let settings = use_settings();
    let asset_sort = settings.asset_sort;
    let initial_tab = settings.receipts_initial_tab;
    let amount_unit = settings.amount_unit;
    let theme = settings.theme;

    view! {
        <section id="display" aria-labelledby="display-heading" class="scroll-mt-18">
            <Card variant=CardVariant::Panel class="p-5">
                <h2 id="display-heading" class="text-lg font-bold mb-4">{labels::SETTINGS_DISPLAY_SECTION}</h2>
                <div class="divide-y divide-ink/10 border-t border-ink/10">
                    <SettingRow title=labels::SETTINGS_THEME>
                        <Segmented
                            options=&THEME_SEGMENTS
                            value=Signal::derive(move || theme.get().value())
                            aria_label=labels::SETTINGS_THEME
                            testid="setting-theme"
                            on_change=move |value| {
                                if let Some(parsed) = Theme::from_value(&value) {
                                    theme.set(parsed);
                                }
                            }
                        />
                    </SettingRow>
                    <SettingRow title=labels::SETTINGS_ASSET_SORT>
                        <Select
                            value=Signal::derive(move || asset_sort.get().value())
                            aria_label=labels::SETTINGS_ASSET_SORT_INITIAL.to_string()
                            testid="setting-asset-sort"
                            class="min-w-44 rounded-md border border-border-strong bg-surface px-3 py-2 text-sm max-sm:min-h-11".to_string()
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
                    </SettingRow>
                    <SettingRow title=labels::SETTINGS_RECEIPTS_TAB>
                        <Select
                            value=Signal::derive(move || initial_tab.get().kind_param().to_string())
                            aria_label=labels::SETTINGS_RECEIPTS_TAB.to_string()
                            testid="setting-receipts-tab"
                            class="min-w-44 rounded-md border border-border-strong bg-surface px-3 py-2 text-sm max-sm:min-h-11".to_string()
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
                    </SettingRow>
                    <SettingRow title=labels::SETTINGS_AMOUNT_UNIT>
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
                    </SettingRow>
                </div>
            </Card>
        </section>
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
        <section id="data" aria-labelledby="data-heading" class="scroll-mt-18">
            <Card variant=CardVariant::Panel class="p-5">
                <h2 id="data-heading" class="text-lg font-bold mb-4">{labels::SETTINGS_DATA_SECTION}</h2>
                <div class="divide-y divide-ink/10 border-t border-ink/10">
                    <SettingRow title=labels::SETTINGS_EXPORT_CSV>
                        {ExportKind::ALL.iter().map(|kind| {
                            let kind = *kind;
                            view! {
                                <Button
                                    variant=ButtonVariant::Secondary(ButtonSize::Sm)
                                    class="whitespace-nowrap"
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
                    </SettingRow>
                    {move || {
                        export_message.get().map(|message| {
                            let tone = if export_failed.get() {
                                "text-negative"
                            } else {
                                "text-text-muted"
                            };
                            view! { <p class=format!("pb-2 text-xs {tone}")>{message}</p> }
                        })
                    }}
                    <SettingRow title=labels::SETTINGS_CSV_IMPORT_DELETE>
                        <MoreLink href="/csv".to_string() label=labels::SETTINGS_CSV_MANAGE_LINK.to_string() />
                    </SettingRow>
                </div>
            </Card>
        </section>
    }
}

#[component]
fn SectionSecurity() -> impl IntoView {
    let settings = use_settings();
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
        <section id="security" aria-labelledby="security-heading" class="scroll-mt-18">
            <Card variant=CardVariant::Panel class="p-5">
                <h2 id="security-heading" class="text-lg font-bold mb-4">{labels::SETTINGS_SECURITY_SECTION}</h2>
                <div class="divide-y divide-ink/10 border-t border-ink/10">
                    <SettingRow title=labels::SETTINGS_IDLE_LOGOUT>
                        <Segmented
                            options=&IDLE_SEGMENTS
                            value=Signal::derive(move || idle_minutes.get().to_string())
                            aria_label=labels::SETTINGS_IDLE_LOGOUT_TIME
                            testid="setting-idle-minutes"
                            on_change=move |value| {
                                if let Ok(minutes) = value.parse::<u32>()
                                    && IDLE_LOGOUT_CHOICES.contains(&minutes)
                                {
                                    idle_minutes.set(minutes);
                                }
                            }
                        />
                    </SettingRow>
                    <SettingRow title=labels::LOGIN_NOTIFY_TITLE>
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
                    </SettingRow>
                </div>
                <p class="pt-2 text-xs text-text-muted">{labels::LOGIN_NOTIFY_NOTE}</p>
            </Card>
        </section>
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
    let session = use_session();
    let token = RwSignal::new(notify_token_from_search());
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
                        <Card variant=CardVariant::Panel class="p-5 border-negative-border">
                            <h2 id="notify-logout-heading" class="text-lg font-bold mb-2">
                                {labels::NOTIFY_LOGOUT_TITLE}
                            </h2>
                            <p class="pb-3 text-sm">{labels::NOTIFY_LOGOUT_DESCRIPTION}</p>
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
                                                <div class="space-y-2">
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
                                                </div>
                                            }
                                                .into_any()
                                        }
                                    }
                                }
                            }}
                        </Card>
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
        .map(|confirm| confirm.0)
        .unwrap_or_else(|| RwSignal::new(false));

    view! {
        <section id="danger" aria-labelledby="danger-heading" class="scroll-mt-18">
            <Card variant=CardVariant::Panel class="p-5 border-negative-border bg-negative/5">
                <h2 id="danger-heading" class="text-lg font-bold text-negative mb-4">{labels::SETTINGS_DANGER_SECTION}</h2>
                <div class="space-y-3 border-t border-negative-border/50">
                    <div class="flex items-center justify-between py-3">
                        <div>
                            <div class="font-medium text-negative">{labels::SETTINGS_DELETE_DIVIDEND}</div>
                            <div class="text-xs text-text-muted">{labels::SETTINGS_DELETE_DIVIDEND_NOTE}</div>
                        </div>
                        <LinkButton
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            href="/csv"
                            class="no-print shrink-0 whitespace-nowrap text-negative border-negative-border hover:bg-negative/10"
                            aria_label=labels::SETTINGS_DELETE_DIVIDEND_ACTION
                        >
                            {labels::SETTINGS_DELETE}
                        </LinkButton>
                    </div>
                    <div class="flex items-center justify-between py-3">
                        <div>
                            <div class="font-medium text-negative">{labels::SETTINGS_DELETE_STOCK}</div>
                            <div class="text-xs text-text-muted">{labels::SETTINGS_DELETE_STOCK_NOTE}</div>
                        </div>
                        <LinkButton
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            href="/csv"
                            class="no-print shrink-0 whitespace-nowrap text-negative border-negative-border hover:bg-negative/10"
                            aria_label=labels::SETTINGS_DELETE_STOCK_ACTION
                        >
                            {labels::SETTINGS_DELETE}
                        </LinkButton>
                    </div>
                    <div class="flex items-center justify-between py-3">
                        <div>
                            <div class="font-medium text-negative">{labels::SETTINGS_DELETE_FUND}</div>
                            <div class="text-xs text-text-muted">{labels::SETTINGS_DELETE_FUND_NOTE}</div>
                        </div>
                        <LinkButton
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            href="/csv"
                            class="no-print shrink-0 whitespace-nowrap text-negative border-negative-border hover:bg-negative/10"
                            aria_label=labels::SETTINGS_DELETE_FUND_ACTION
                        >
                            {labels::SETTINGS_DELETE}
                        </LinkButton>
                    </div>
                    <div class="flex items-center justify-between py-3">
                        <div>
                            <div class="font-medium text-negative">{labels::SETTINGS_DELETE_ASSETS}</div>
                            <div class="text-xs text-text-muted">{labels::SETTINGS_DELETE_ASSETS_NOTE}</div>
                        </div>
                        <LinkButton
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            href="/csv"
                            class="no-print shrink-0 whitespace-nowrap text-negative border-negative-border hover:bg-negative/10"
                            aria_label=labels::SETTINGS_DELETE_ASSETS
                        >
                            {labels::SETTINGS_DELETE}
                        </LinkButton>
                    </div>
                    <div class="flex items-center justify-between py-3">
                        <div>
                            <div class="font-medium text-negative">{labels::SETTINGS_DELETE_ACCOUNT}</div>
                            <div class="text-xs text-text-muted">{labels::SETTINGS_DELETE_ACCOUNT_NOTE}</div>
                        </div>
                        <Button
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            class="shrink-0 whitespace-nowrap text-negative border-negative-border hover:bg-negative/10"
                            on_click=move |_| delete_confirm_open.set(true)
                            aria_label=labels::SETTINGS_DELETE_ACCOUNT
                        >
                            {labels::SETTINGS_DELETE}
                        </Button>
                    </div>
                </div>
            </Card>
        </section>
    }
}
