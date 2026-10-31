mod export;
mod login_history;
pub(crate) use login_history::{ResultBadge, format_agent, format_datetime, format_place};

use crate::session::use_session;
use crate::settings::{AmountUnit, AssetSort, IDLE_LOGOUT_CHOICES, Theme, use_settings};
use crate::ui::button::{Button, ButtonSize, ButtonVariant, LinkButton, Switch};
use crate::ui::card::Card;
use crate::ui::card::CardVariant;
use crate::ui::labels::{
    LOGIN_NOTIFY_NOTE, LOGIN_NOTIFY_TITLE, NOTIFY_LOGOUT_CONFIRM, NOTIFY_LOGOUT_DESCRIPTION,
    NOTIFY_LOGOUT_DONE, NOTIFY_LOGOUT_INVALID, NOTIFY_LOGOUT_TITLE,
};
use crate::ui::page_frame::PageFrame;
use crate::ui::segmented::Segmented;
use crate::ui::select::Select;
use crate::ui::site::get_initials;
use crate::ui::state::{Alert, AlertVariant, ListLoadError, Loading};
use export::ExportKind;
use leptos::prelude::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let session = use_session();
    let user = move || session.user.get();

    let sections = [
        ("account", "アカウント"),
        ("display", "表示"),
        ("data", "データ"),
        ("security", "セキュリティ"),
        ("devices", crate::ui::labels::REGISTERED_DEVICES_SECTION),
        ("login-history", crate::ui::labels::LOGIN_HISTORY_SECTION),
        ("danger", "危険な操作"),
    ];

    view! {
        <PageFrame title="設定">
            <div class="flex flex-col lg:flex-row gap-7">
                <nav class="hidden lg:block lg:w-48 shrink-0 lg:sticky lg:top-16 lg:self-start" aria-label="設定セクション">
                    <ul class="flex flex-col gap-1" role="list">
                        {sections.iter().map(|(id, label)| {
                            let id = *id;
                            let label = *label;
                            view! {
                                <li>
                                    <a
                                        href={format!("/settings#{}", id)}
                                        class="flex min-h-11 items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-text-deep transition-colors no-underline hover:bg-surface-raised"
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
                            view! { <div class="text-center py-12">"ログインが必要です"</div> }.into_any()
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
    let initials = get_initials(user.name.as_deref(), Some(&user.email));
    let login_method_text = "Google でログイン中";

    view! {
        <section id="account" aria-labelledby="account-heading">
            <Card variant=CardVariant::Panel class="p-5">
                <h2 id="account-heading" class="text-lg font-bold mb-4">"アカウント"</h2>
                <div class="flex items-center gap-4">
                    <span class="flex h-14 w-14 shrink-0 items-center justify-center rounded-full bg-gold text-brand font-bold text-2xl">
                        {initials}
                    </span>
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
        <section id="display" aria-labelledby="display-heading">
            <Card variant=CardVariant::Panel class="p-5">
                <h2 id="display-heading" class="text-lg font-bold mb-4">"表示"</h2>
                <div class="divide-y divide-ink/10 border-t border-ink/10">
                    <SettingRow title="テーマ">
                        <Segmented
                            options=&THEME_SEGMENTS
                            value=Signal::derive(move || theme.get().value())
                            aria_label="テーマ"
                            testid="setting-theme"
                            on_change=move |value| {
                                if let Some(parsed) = Theme::from_value(&value) {
                                    theme.set(parsed);
                                }
                            }
                        />
                    </SettingRow>
                    <SettingRow title="資産管理の並び順">
                        <Select
                            value=Signal::derive(move || asset_sort.get().value())
                            aria_label="資産管理の初期の並び順".to_string()
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
                    <SettingRow title="取引明細の初期タブ">
                        <Select
                            value=Signal::derive(move || initial_tab.get().kind_param().to_string())
                            aria_label="取引明細の初期タブ".to_string()
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
                    <SettingRow title="金額の表示単位">
                        <Segmented
                            options=&[
                                ("yen", "円"),
                                ("sen", "千円"),
                                ("man", "万円"),
                            ]
                            value=Signal::derive(move || amount_unit.get().value())
                            aria_label="金額の表示単位"
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
                    export_message.set(Some(format!("{filename} を保存しました")));
                }
                Err(error) => {
                    export_failed.set(true);
                    export_message.set(Some(format!(
                        "{}のエクスポートに失敗しました: {}",
                        kind.label(),
                        error.user_message()
                    )));
                }
            }
            exporting.set(None);
        });
    };

    view! {
        <section id="data" aria-labelledby="data-heading">
            <Card variant=CardVariant::Panel class="p-5">
                <h2 id="data-heading" class="text-lg font-bold mb-4">"データ"</h2>
                <div class="divide-y divide-ink/10 border-t border-ink/10">
                    <SettingRow title="CSVでエクスポート">
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
                                            "取得中…".to_string()
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
                    <SettingRow title="CSVの取り込み・保存済みデータの削除">
                        <LinkButton
                            variant=ButtonVariant::Ghost
                            href="/csv"
                            class="no-print shrink-0 whitespace-nowrap"
                        >
                            "CSV管理へ →"
                        </LinkButton>
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
        <section id="security" aria-labelledby="security-heading">
            <Card variant=CardVariant::Panel class="p-5">
                <h2 id="security-heading" class="text-lg font-bold mb-4">"セキュリティ"</h2>
                <div class="divide-y divide-ink/10 border-t border-ink/10">
                    <SettingRow title="操作がないときの自動ログアウト">
                        <Segmented
                            options=&IDLE_SEGMENTS
                            value=Signal::derive(move || idle_minutes.get().to_string())
                            aria_label="自動ログアウトまでの時間"
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
                    <SettingRow title=LOGIN_NOTIFY_TITLE>
                        {move || match notify.get() {
                            None => view! { <Loading /> }.into_any(),
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
                                        label=LOGIN_NOTIFY_TITLE.to_string()
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
                <p class="pt-2 text-xs text-text-muted">{LOGIN_NOTIFY_NOTE}</p>
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
                        NOTIFY_LOGOUT_INVALID.to_string()
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
                                {NOTIFY_LOGOUT_TITLE}
                            </h2>
                            <p class="pb-3 text-sm">{NOTIFY_LOGOUT_DESCRIPTION}</p>
                            {move || {
                                if done.get() {
                                    view! { <p role="status">{NOTIFY_LOGOUT_DONE}</p> }.into_any()
                                } else {
                                    match state.get() {
                                        None => view! { <Loading /> }.into_any(),
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
                                                        {NOTIFY_LOGOUT_CONFIRM}
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

const IDLE_SEGMENTS: [(&str, &str); 3] = [("15", "15分"), ("30", "30分"), ("60", "60分")];

const THEME_SEGMENTS: [(&str, &str); 3] = [
    ("system", "端末に合わせる"),
    ("light", "ライト"),
    ("dark", "ダーク"),
];

#[component]
fn SectionDanger() -> impl IntoView {
    let delete_confirm_open = use_context::<crate::ui::site::AccountDeleteConfirm>()
        .map(|confirm| confirm.0)
        .unwrap_or_else(|| RwSignal::new(false));

    view! {
        <section id="danger" aria-labelledby="danger-heading">
            <Card variant=CardVariant::Panel class="p-5 border-negative-border bg-negative/5">
                <h2 id="danger-heading" class="text-lg font-bold text-negative mb-4">"危険な操作"</h2>
                <div class="space-y-3 border-t border-negative-border/50">
                    <div class="flex items-center justify-between py-3">
                        <div>
                            <div class="font-medium text-negative">"取引明細：配当金を削除"</div>
                            <div class="text-xs text-text-muted">"取り込んだ配当金データ。アカウントは残ります。取り消せません"</div>
                        </div>
                        <LinkButton
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            href="/csv"
                            class="no-print shrink-0 whitespace-nowrap text-negative border-negative-border hover:bg-negative/10"
                            aria_label="配当金データを削除"
                        >
                            "削除…"
                        </LinkButton>
                    </div>
                    <div class="flex items-center justify-between py-3">
                        <div>
                            <div class="font-medium text-negative">"取引明細：国内株式を削除"</div>
                            <div class="text-xs text-text-muted">"取り込んだ国内株式売買データ。アカウントは残ります。取り消せません"</div>
                        </div>
                        <LinkButton
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            href="/csv"
                            class="no-print shrink-0 whitespace-nowrap text-negative border-negative-border hover:bg-negative/10"
                            aria_label="国内株式データを削除"
                        >
                            "削除…"
                        </LinkButton>
                    </div>
                    <div class="flex items-center justify-between py-3">
                        <div>
                            <div class="font-medium text-negative">"取引明細：投資信託を削除"</div>
                            <div class="text-xs text-text-muted">"取り込んだ投資信託売買データ。アカウントは残ります。取り消せません"</div>
                        </div>
                        <LinkButton
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            href="/csv"
                            class="no-print shrink-0 whitespace-nowrap text-negative border-negative-border hover:bg-negative/10"
                            aria_label="投資信託データを削除"
                        >
                            "削除…"
                        </LinkButton>
                    </div>
                    <div class="flex items-center justify-between py-3">
                        <div>
                            <div class="font-medium text-negative">"資産管理データを削除"</div>
                            <div class="text-xs text-text-muted">"保存済みの保有銘柄データ。アカウントは残ります。取り消せません"</div>
                        </div>
                        <LinkButton
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            href="/csv"
                            class="no-print shrink-0 whitespace-nowrap text-negative border-negative-border hover:bg-negative/10"
                            aria_label="資産管理データを削除"
                        >
                            "削除…"
                        </LinkButton>
                    </div>
                    <div class="flex items-center justify-between py-3">
                        <div>
                            <div class="font-medium text-negative">"アカウントを削除"</div>
                            <div class="text-xs text-text-muted">"データもすべて消えます。取り消せません"</div>
                        </div>
                        <Button
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            class="shrink-0 whitespace-nowrap text-negative border-negative-border hover:bg-negative/10"
                            on_click=move |_| delete_confirm_open.set(true)
                            aria_label="アカウントを削除"
                        >
                            "削除…"
                        </Button>
                    </div>
                </div>
            </Card>
        </section>
    }
}
