use crate::api::dto::SessionUser;
use crate::session::{use_session, SessionStore};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::card::{Card, CardVariant};
use crate::ui::confirm_modal::ConfirmDeleteModal;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

const NAV_LINKS: &[(&str, &str)] = &[
    ("/search", "銘柄検索"),
    ("/assetbalance", "資産管理"),
    ("/receipts", "取引明細"),
];

const FOOTER_LINKS: &[(&str, &str)] = &[
    (
        "プライバシーポリシー",
        "https://github.com/zaichu/shoken-webapp/blob/main/docs/privacy-policy.md",
    ),
    (
        "利用規約",
        "https://github.com/zaichu/shoken-webapp/blob/main/docs/terms.md",
    ),
    (
        "Cookie ポリシー",
        "https://github.com/zaichu/shoken-webapp/blob/main/docs/cookie-policy.md",
    ),
];

const NAV_LINK_BASE: &str = "rounded px-3.5 py-2 text-sm font-bold transition-[background-color,color,box-shadow] max-sm:inline-flex max-sm:min-h-11 max-sm:min-w-11 max-sm:shrink-0 max-sm:items-center max-sm:justify-center max-sm:whitespace-nowrap max-sm:px-2 max-sm:text-xs max-sm:leading-5";
const NAV_LINK_ACTIVE: &str = "bg-surface text-ink shadow-edge-accent";
const NAV_LINK_INACTIVE: &str =
    "text-text-inverse-muted hover:bg-surface/10 hover:text-text-inverse";

pub(crate) fn current_path() -> String {
    web_sys::window()
        .and_then(|window| window.location().pathname().ok())
        .unwrap_or_default()
}

pub(crate) fn current_location() -> String {
    web_sys::window()
        .and_then(|window| {
            let location = window.location();
            let pathname = location.pathname().ok()?;
            let search = location.search().ok()?;
            Some(format!("{pathname}{search}"))
        })
        .unwrap_or_default()
}

/// アプリ内遷移でナビのアクティブ表示を追随させるための現在パス(pathname + search)
#[derive(Clone, Copy)]
pub(crate) struct CurrentPath(pub RwSignal<String>);

fn is_nav_active(path: &str, to: &str) -> bool {
    let path = path.split(['?', '#']).next().unwrap_or(path);
    path == to || path.starts_with(&format!("{to}/"))
}

pub fn get_initials(name: Option<&str>, email: Option<&str>) -> String {
    if let Some(name) = name.filter(|name| !name.is_empty()) {
        let parts: Vec<&str> = name.split_whitespace().collect();
        if parts.len() >= 2 {
            let first = parts[0].chars().next().unwrap_or_default();
            let last = parts[parts.len() - 1].chars().next().unwrap_or_default();
            return format!("{first}{last}").to_uppercase();
        }
        return name.chars().take(2).collect::<String>().to_uppercase();
    }
    if let Some(email) = email.filter(|email| !email.is_empty()) {
        return email.chars().take(2).collect::<String>().to_uppercase();
    }
    "U".to_string()
}

#[component]
pub fn SiteHeader() -> impl IntoView {
    let session = use_session();
    // App 配下では SPA 遷移の path を使い、単独で描くテスト等では現在の URL にフォールバック
    let path = use_context::<CurrentPath>()
        .map(|current| current.0)
        .unwrap_or_else(|| RwSignal::new(current_path()));
    let delete_confirm_open = RwSignal::new(false);
    let delete_error = RwSignal::new(Option::<String>::None);
    let deleting = RwSignal::new(false);
    let deleting_memo = Memo::new(move |_| deleting.get());
    // ナビにホーム項目は置かず、ロゴがホームへのリンクを担う
    let home_active = move || {
        path.get()
            .split(['?', '#'])
            .next()
            .is_some_and(|pathname| pathname == "/")
    };
    let on_login_page = move || {
        path.get()
            .split(['?', '#'])
            .next()
            .is_some_and(|pathname| pathname == "/login")
    };
    Effect::new(move |_| {
        if delete_confirm_open.get() {
            delete_error.set(None);
        }
    });
    // ワークスペース系ページでは MS Learn 型3カラムに合わせた全幅バーにする
    let is_workspace = move || {
        matches!(
            path.get().split(['?', '#']).next().unwrap_or_default(),
            "/receipts" | "/assetbalance"
        )
    };
    view! {
        <header class="site-header no-print">
            <div class=move || {
                if is_workspace() {
                    "w-full px-4 sm:px-6 ws-header-inner py-3".to_string()
                } else {
                    "mx-auto w-full max-w-wide px-4 sm:px-6 lg:px-8 py-3".to_string()
                }
            }>
                <div class=move || {
                    if is_workspace() {
                        "ws-header-flow flex flex-row items-center gap-2 max-sm:gap-1.5 lg:gap-3"
                            .to_string()
                    } else {
                        "flex flex-row items-center gap-2 max-sm:gap-1.5 lg:gap-3".to_string()
                    }
                }>
                    <div class=move || {
                        if is_workspace() {
                            "ws-header-brand flex items-center justify-between gap-4".to_string()
                        } else {
                            "flex items-center justify-between gap-4".to_string()
                        }
                    }>
                        <a
                            href="/"
                            class=move || {
                                format!(
                                    "group inline-flex items-center gap-3 rounded-md text-text-inverse transition-colors hover:text-accent-softer max-sm:min-h-11 max-sm:min-w-11 max-sm:justify-center{}",
                                    if home_active() { " shadow-edge-accent" } else { "" }
                                )
                            }
                            aria-current=move || home_active().then_some("page")
                        >
                            <span class="header-logo">
                                "証"
                            </span>
                            // スマホではサービス名を隠し、ロゴ・ナビ・ユーザーを1行に収める
                            <span class="header-brand max-sm:hidden">
                                "証券Web"
                            </span>
                        </a>
                    </div>
                    <nav
                        class=move || {
                            if is_workspace() {
                                "header-nav ws-header-nav".to_string()
                            } else {
                                "header-nav".to_string()
                            }
                        }
                        aria-label="主要ナビゲーション"
                    >
                        {NAV_LINKS
                            .iter()
                            .map(|(to, label)| {
                                let to = *to;
                                let active = move || is_nav_active(&path.get(), to);
                                view! {
                                    <a href=to
                                        class=move || {
                                            format!(
                                                "{NAV_LINK_BASE} {}",
                                                if active() { NAV_LINK_ACTIVE } else { NAV_LINK_INACTIVE }
                                            )
                                        }
                                        aria-current=move || active().then_some("page")
                                    >
                                        {*label}
                                    </a>
                                }
                            })
                            .collect_view()}
                    </nav>
                    <div class=move || {
                        if is_workspace() {
                            "ws-header-user ml-auto flex shrink-0 items-center gap-3 max-sm:gap-1.5"
                                .to_string()
                        } else {
                            "ml-auto flex shrink-0 items-center gap-3 max-sm:gap-1.5".to_string()
                        }
                    }>
                        {move || {
                            if !session.loaded.get() {
                                view! {
                                    <span class="text-sm font-semibold text-text-inverse/70">
                                        "読み込み中..."
                                    </span>
                                }
                                    .into_any()
                            } else if let Some(user) = session.user.get() {
                                view! {
                                    <UserMenu
                                        user=user
                                        session=session
                                        delete_confirm_open=delete_confirm_open
                                    />
                                }
                                    .into_any()
                            } else if on_login_page() {
                                ().into_any()
                            } else {
                                view! {
                                    <Button
                                        variant=ButtonVariant::Header(ButtonSize::Sm)
                                        on_click=move |_| session.login()
                                    >
                                        "ログイン"
                                    </Button>
                                }
                                    .into_any()
                            }
                        }}
                    </div>
                </div>
            </div>
        </header>
        {move || {
            if !delete_confirm_open.get() {
                return ().into_any();
            }
            view! {
                <ConfirmDeleteModal
                    title="アカウント削除の確認".to_string()
                    description="アカウントを削除すると、資産管理・配当金・取引履歴などすべてのデータが削除されます。"
                        .to_string()
                    confirm_label="削除する"
                    loading=deleting_memo
                    error=delete_error
                    on_confirm=move || {
                        deleting.set(true);
                        delete_error.set(None);
                        leptos::task::spawn_local(async move {
                            let result = session.delete_account().await;
                            deleting.set(false);
                            match result {
                                Ok(()) => delete_confirm_open.set(false),
                                Err(error) => delete_error.set(Some(error.user_message())),
                            }
                        });
                    }
                    on_cancel=move || delete_confirm_open.set(false)
                />
            }
                .into_any()
        }}
    }
}

#[component]
fn UserMenu(
    user: SessionUser,
    session: SessionStore,
    delete_confirm_open: RwSignal<bool>,
) -> impl IntoView {
    let menu_open = RwSignal::new(false);
    let image_error = RwSignal::new(false);
    let menu_container = NodeRef::<html::Div>::new();

    let on_mouse_down = window_event_listener(ev::mousedown, move |ev| {
        if !menu_open.get_untracked() {
            return;
        }
        let inside = menu_container
            .get()
            .zip(
                ev.target()
                    .and_then(|target| target.dyn_into::<web_sys::Node>().ok()),
            )
            .is_some_and(|(container, target)| container.contains(Some(&target)));
        if !inside {
            menu_open.set(false);
        }
    });
    let on_key_down = window_event_listener(ev::keydown, move |ev| {
        if menu_open.get_untracked() && ev.key() == "Escape" {
            menu_open.set(false);
        }
    });
    on_cleanup(move || {
        on_mouse_down.remove();
        on_key_down.remove();
    });

    let display_name = user
        .name
        .clone()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| user.email.clone());
    let aria_name = user
        .name
        .clone()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "ユーザー".to_string());
    let initials = get_initials(user.name.as_deref(), Some(user.email.as_str()));
    let picture_url = user.picture_url.filter(|url| !url.is_empty());

    view! {
        <div class="flex flex-wrap items-center gap-3">
            {move || match picture_url.clone() {
                Some(url) if !image_error.get() => {
                    let alt = aria_name.clone();
                    view! {
                        <img
                            src=url
                            alt=alt
                            class="h-9 w-9 rounded-md border border-text-inverse/20 bg-night-soft object-cover max-sm:hidden"
                            on:error=move |_| image_error.set(true)
                        />
                    }
                        .into_any()
                }
                _ => {
                    let label = aria_name.clone();
                    let initials = initials.clone();
                    view! {
                        <div
                            class="header-avatar"
                            aria-label=label
                        >
                            {initials}
                        </div>
                    }
                        .into_any()
                }
            }}
            <span class="max-w-64 truncate text-sm font-semibold text-text-inverse/85 max-sm:hidden">
                {display_name}
            </span>
            <div class="relative" node_ref=menu_container>
                <Button
                    variant=ButtonVariant::Header(ButtonSize::Sm)
                    aria_haspopup="menu"
                    aria_expanded=move || menu_open.get()
                    aria_controls="user-menu"
                    on_click=move |_| menu_open.update(|open| *open = !*open)
                >
                    "メニュー"
                </Button>
                <Show when=move || menu_open.get()>
                    <ul
                        id="user-menu"
                        class="absolute right-0 mt-2 w-56 overflow-hidden rounded-lg border border-ink/10 bg-surface text-text-deep shadow-2xl"
                        role="menu"
                        aria-label="ユーザーメニュー"
                    >
                        <li role="none">
                            <Button
                                variant=ButtonVariant::MenuItem
                                role="menuitem"
                                testid="logout"
                                on_click=move |_| {
                                    menu_open.set(false);
                                    leptos::task::spawn_local(async move {
                                        session.logout().await;
                                    });
                                }
                            >
                                "ログアウト"
                            </Button>
                        </li>
                        <li role="none">
                            <hr class="border-border" />
                        </li>
                        <li
                            role="none"
                            class="px-3 py-2 text-xs font-bold text-text-quiet"
                        >
                            "危険な操作"
                        </li>
                        <li role="none">
                            <Button
                                variant=ButtonVariant::MenuItemDanger
                                role="menuitem"
                                aria_describedby="delete-warning"
                                on_click=move |_| delete_confirm_open.set(true)
                            >
                                <span id="delete-warning" class="sr-only">
                                    "警告: この操作は取り消せません"
                                </span>
                                "アカウント削除"
                            </Button>
                        </li>
                    </ul>
                </Show>
            </div>
        </div>
    }
}

#[component]
pub fn SiteFooter() -> impl IntoView {
    view! {
        <footer class="mt-auto border-t border-border-subtle bg-surface py-4">
            <div class="mx-auto w-full max-w-wide px-4 sm:px-6 lg:px-8 flex flex-wrap items-center justify-center gap-x-6 gap-y-1 text-sm text-text-subtle">
                {FOOTER_LINKS
                    .iter()
                    .map(|(label, href)| {
                        view! {
                            <a
                                href={*href}
                                target="_blank"
                                rel="noopener noreferrer"
                                class="inline-flex min-h-6 items-center hover:text-text-soft hover:underline max-sm:min-h-11"
                                aria-label={format!("{label}（新しいタブで開く）")}
                            >
                                {*label}
                            </a>
                        }
                    })
                    .collect_view()}
                <span>"© 2026 証券Web"</span>
            </div>
        </footer>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertVariant {
    Warning,
    Danger,
}

impl AlertVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Warning => "border-accent-border bg-accent-soft text-accent-text",
            Self::Danger => "border-negative-border bg-negative-soft text-negative-vivid",
        }
    }
}

#[component]
pub fn Alert(variant: AlertVariant, children: Children) -> impl IntoView {
    let variant_class = variant.class();
    view! {
        <div
            class={format!(
                "rounded-lg border px-4 py-3 text-sm font-medium shadow-sm {variant_class}"
            )}
            role="alert"
        >
            {children()}
        </div>
    }
}

#[component]
pub fn PageHeader(title: &'static str, description: &'static str) -> impl IntoView {
    view! {
        <div class="mb-5 max-sm:mb-2">
            <div class="flex flex-col gap-3 border-l-4 border-accent-bright pl-4 sm:flex-row sm:items-end sm:justify-between">
                <div>
                    <h1 class="text-2xl font-black leading-tight tracking-normal text-ink max-sm:text-lg">
                        {title}
                    </h1>
                    <p class="mt-1 text-sm font-medium text-text-muted max-sm:hidden">{description}</p>
                </div>
            </div>
        </div>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpinnerSize {
    Sm,
    Lg,
}

impl SpinnerSize {
    fn class(self) -> &'static str {
        match self {
            Self::Sm => "h-4 w-4",
            Self::Lg => "h-8 w-8",
        }
    }
}

#[component]
pub fn Spinner(size: SpinnerSize, class: &'static str) -> impl IntoView {
    let size_class = size.class();
    view! {
        <span class={format!("inline-flex items-center {class}")}>
            <svg
                class={format!("animate-spin {size_class}")}
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                role="status"
                aria-label="読み込み中..."
            >
                <circle
                    class="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    stroke-width="4"
                />
                <path
                    class="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                />
            </svg>
        </span>
    }
}

#[component]
pub fn Loading() -> impl IntoView {
    view! { <p role="status">"読み込み中..."</p> }
}

#[component]
pub fn LoadingStrip(#[prop(into)] text: Signal<String>) -> impl IntoView {
    view! {
        <section class="px-5 py-4" role="status" aria-live="polite" aria-atomic="true">
            <div class="flex items-center gap-2 text-text-muted">
                <Spinner size=SpinnerSize::Sm class=""/>
                <p class="text-sm">{move || text.get()}</p>
            </div>
        </section>
    }
}

#[component]
pub fn Skeleton(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    view! {
        <div
            class=move || format!("animate-pulse rounded bg-fill {}", class.get())
            aria-hidden="true"
        ></div>
    }
}

#[component]
pub fn ListLoadError(
    message: String,
    on_retry: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <Card variant=CardVariant::Panel class="p-4" testid="list-load-error">
            <div
                class="rounded-lg border border-negative-border bg-negative-soft px-4 py-3 text-sm font-medium text-negative-vivid"
                role="alert"
            >
                <strong>"エラー:"</strong>
                " "
                {message}
            </div>
            <div class="mt-4">
                <Button variant=ButtonVariant::Retry on_click=move |_| on_retry()>
                    "再読み込み"
                </Button>
            </div>
        </Card>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListSkeletonVariant {
    Cards,
    Table,
}

#[component]
pub fn ListSkeleton(variant: ListSkeletonVariant) -> impl IntoView {
    let body = match variant {
        ListSkeletonVariant::Cards => view! {
            {(0..3)
                .map(|_| {
                    view! {
                        <div class="grid gap-2 rounded-lg border border-border-subtle p-4">
                            <div class="h-4 w-1/3 rounded bg-fill"></div>
                            <div class="h-6 w-1/2 rounded bg-fill"></div>
                            <div class="h-4 w-2/3 rounded bg-fill"></div>
                        </div>
                    }
                })
                .collect_view()}
        }
        .into_any(),
        ListSkeletonVariant::Table => view! {
            {(0..2)
                .map(|_| {
                    view! {
                        <div class="grid gap-2">
                            <div class="h-4 w-32 rounded bg-fill"></div>
                            <div class="grid gap-2 rounded-lg border border-border-subtle p-4">
                                {(0..4)
                                    .map(|_| {
                                        view! {
                                            <div class="skeleton-table-row">
                                                <div class="h-4 rounded bg-fill"></div>
                                                <div class="h-4 rounded bg-fill max-sm:hidden"></div>
                                                <div class="h-4 rounded bg-fill"></div>
                                            </div>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </div>
                    }
                })
                .collect_view()}
        }
        .into_any(),
    };
    view! {
        <Card variant=CardVariant::Panel class="p-4" role="status" testid="list-skeleton">
            <span class="sr-only">"データを読み込んでいます..."</span>
            <div class="grid animate-pulse gap-4" aria-hidden="true">
                {body}
            </div>
        </Card>
    }
}

#[cfg(test)]
mod tests;
