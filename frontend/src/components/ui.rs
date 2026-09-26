use crate::components::confirm_modal::ConfirmDeleteModal;
use crate::dto::SessionUser;
use crate::session::{use_session, SessionStore};
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

const NAV_LINK_BASE: &str = "rounded px-3.5 py-2 text-sm font-bold transition-[background-color,color,box-shadow] max-sm:inline-flex max-sm:min-h-[44px] max-sm:min-w-[44px] max-sm:shrink-0 max-sm:items-center max-sm:justify-center max-sm:whitespace-nowrap max-sm:px-2 max-sm:text-[12px]";
const NAV_LINK_ACTIVE: &str = "bg-white text-slate-950 shadow-[inset_0_-2px_0_#f59e0b]";
const NAV_LINK_INACTIVE: &str = "text-slate-300 hover:bg-white/10 hover:text-white";

const HEADER_BUTTON: &str = "inline-flex items-center justify-center rounded-md font-bold transition-[background-color,border-color,color,box-shadow,transform] focus:outline-none focus:ring-2 focus:ring-amber-500/50 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60 no-print border border-white/70 text-white hover:bg-white hover:text-slate-950 px-3 py-1.5 text-sm max-sm:min-h-[44px]";

pub(crate) fn current_path() -> String {
    web_sys::window()
        .and_then(|window| window.location().pathname().ok())
        .unwrap_or_default()
}

fn is_nav_active(path: &str, to: &str) -> bool {
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
    let path = current_path();
    let delete_confirm_open = RwSignal::new(false);
    let delete_error = RwSignal::new(Option::<String>::None);
    let deleting = RwSignal::new(false);
    let deleting_memo = Memo::new(move |_| deleting.get());
    Effect::new(move |_| {
        if delete_confirm_open.get() {
            delete_error.set(None);
        }
    });
    view! {
        <header class="site-header no-print">
            <div class="mx-auto w-full max-w-[1680px] px-4 sm:px-6 lg:px-8 py-3">
                <div class="flex flex-row items-center gap-2 max-sm:gap-1.5 lg:gap-3">
                    <div class="flex items-center justify-between gap-4">
                        <a
                            href="/"
                            class="group inline-flex items-center gap-3 text-white transition-colors hover:text-amber-100"
                        >
                            <span class="header-logo">
                                "証"
                            </span>
                            // スマホでは副題とサービス名を隠し、ロゴ・ナビ・ユーザーを1行に収める
                            <span class="max-sm:hidden">
                                <span class="block text-[11px] font-bold uppercase tracking-[0.28em] text-amber-300/90">
                                    "Portfolio Desk"
                                </span>
                                <span class="block text-xl font-black leading-tight tracking-normal">
                                    "証券Web"
                                </span>
                            </span>
                        </a>
                    </div>
                    <nav
                        class="header-nav"
                        aria-label="主要ナビゲーション"
                    >
                        {NAV_LINKS
                            .iter()
                            .map(|(to, label)| {
                                let active = is_nav_active(&path, to);
                                let class = format!(
                                    "{NAV_LINK_BASE} {}",
                                    if active { NAV_LINK_ACTIVE } else { NAV_LINK_INACTIVE }
                                );
                                view! {
                                    <a href={*to} class=class aria-current=active.then_some("page")>
                                        {*label}
                                    </a>
                                }
                            })
                            .collect_view()}
                    </nav>
                    <div class="ml-auto flex shrink-0 items-center gap-3 max-sm:gap-1.5">
                        {move || {
                            if !session.loaded.get() {
                                view! {
                                    <span class="text-sm font-semibold text-white/70">
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
                            } else {
                                view! {
                                    <button
                                        type="button"
                                        class=HEADER_BUTTON
                                        on:click=move |_| session.login()
                                    >
                                        "ログイン"
                                    </button>
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
                    item_count=1
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
                            class="h-9 w-9 rounded-md border border-white/20 bg-slate-700 object-cover max-sm:hidden"
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
            <span class="max-w-[16rem] truncate text-sm font-semibold text-white/85 max-sm:hidden">
                {display_name}
            </span>
            <div class="relative" node_ref=menu_container>
                <button
                    type="button"
                    class=HEADER_BUTTON
                    aria-haspopup="menu"
                    aria-expanded=move || if menu_open.get() { "true" } else { "false" }
                    aria-controls="user-menu"
                    on:click=move |_| menu_open.update(|open| *open = !*open)
                >
                    "メニュー"
                </button>
                <Show when=move || menu_open.get()>
                    <ul
                        id="user-menu"
                        class="absolute right-0 mt-2 w-56 overflow-hidden rounded-lg border border-slate-950/10 bg-white text-dark shadow-2xl"
                        role="menu"
                        aria-label="ユーザーメニュー"
                    >
                        <li role="none">
                            <button
                                type="button"
                                class="w-full px-3 py-2 text-left text-sm font-semibold hover:bg-slate-100"
                                role="menuitem"
                                data-testid="logout"
                                on:click=move |_| {
                                    menu_open.set(false);
                                    leptos::task::spawn_local(async move {
                                        session.logout().await;
                                    });
                                }
                            >
                                "ログアウト"
                            </button>
                        </li>
                        <li role="none">
                            <hr class="border-border" />
                        </li>
                        <li
                            role="none"
                            class="px-3 py-2 text-xs font-bold uppercase tracking-[0.16em] text-secondary"
                        >
                            "危険な操作"
                        </li>
                        <li role="none">
                            <button
                                type="button"
                                class="w-full px-3 py-2 text-left text-sm font-bold text-danger hover:bg-danger/10"
                                role="menuitem"
                                aria-describedby="delete-warning"
                                on:click=move |_| delete_confirm_open.set(true)
                            >
                                <span id="delete-warning" class="sr-only">
                                    "警告: この操作は取り消せません"
                                </span>
                                "アカウント削除"
                            </button>
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
        <footer class="mt-auto border-t border-slate-200 bg-white py-4">
            <div class="mx-auto w-full max-w-[1680px] px-4 sm:px-6 lg:px-8 flex flex-wrap justify-center gap-x-6 gap-y-1 text-sm text-slate-500">
                {FOOTER_LINKS
                    .iter()
                    .map(|(label, href)| {
                        view! {
                            <a
                                href={*href}
                                target="_blank"
                                rel="noopener noreferrer"
                                class="hover:text-slate-700 hover:underline"
                                aria-label={format!("{label}（新しいタブで開く）")}
                            >
                                {*label}
                            </a>
                        }
                    })
                    .collect_view()}
                <span>"© 2026 shoken-webapp"</span>
            </div>
        </footer>
    }
}

fn alert_variant_class(variant: &str) -> &'static str {
    match variant {
        "warning" => "border-amber-200 bg-amber-50 text-amber-900",
        "danger" => "border-red-200 bg-red-50 text-red-700",
        "success" => "border-teal-200 bg-teal-50 text-teal-800",
        _ => "border-blue-200 bg-blue-50 text-blue-800",
    }
}

#[component]
pub fn Alert(variant: &'static str, children: Children) -> impl IntoView {
    let variant_class = alert_variant_class(variant);
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
pub fn PageHeader(
    title: &'static str,
    eyebrow: &'static str,
    description: &'static str,
) -> impl IntoView {
    view! {
        <div class="mb-5 max-sm:mb-2">
            <div class="flex flex-col gap-3 border-l-4 border-amber-500 pl-4 sm:flex-row sm:items-end sm:justify-between">
                <div>
                    <p class="mb-1 text-[11px] font-bold uppercase tracking-[0.22em] text-slate-500 max-sm:hidden">
                        {eyebrow}
                    </p>
                    <h1 class="text-2xl font-black leading-tight tracking-normal text-slate-950 max-sm:text-lg">
                        {title}
                    </h1>
                    <p class="mt-1 text-sm font-medium text-slate-600 max-sm:hidden">{description}</p>
                </div>
            </div>
        </div>
    }
}

fn spinner_size_class(size: &str) -> &'static str {
    match size {
        "sm" => "h-4 w-4",
        "lg" => "h-8 w-8",
        _ => "h-6 w-6",
    }
}

#[component]
pub fn Spinner(size: &'static str, class: &'static str) -> impl IntoView {
    let size_class = spinner_size_class(size);
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

const RETRY_BUTTON: &str = "inline-flex min-h-11 items-center justify-center rounded-md border border-slate-950 bg-slate-950 px-4 text-sm font-bold text-white transition-colors hover:bg-slate-800 focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-500/50 focus-visible:ring-offset-2";

#[component]
pub fn ListLoadError(message: String, on_retry: impl Fn() + 'static) -> impl IntoView {
    view! {
        <div class="panel-card p-4" data-testid="list-load-error">
            <div
                class="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm font-medium text-red-700"
                role="alert"
            >
                <strong>"エラー:"</strong>
                " "
                {message}
            </div>
            <div class="mt-4">
                <button type="button" class=RETRY_BUTTON on:click=move |_| on_retry()>
                    "再読み込み"
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn ListSkeleton() -> impl IntoView {
    view! {
        <div class="panel-card p-4" role="status" data-testid="list-skeleton">
            <span class="sr-only">"データを読み込んでいます..."</span>
            <div class="grid animate-pulse gap-4" aria-hidden="true">
                {(0..3)
                    .map(|_| {
                        view! {
                            <div class="grid gap-2 rounded-lg border border-slate-200 p-4">
                                <div class="h-4 w-1/3 rounded bg-slate-200"></div>
                                <div class="h-6 w-1/2 rounded bg-slate-200"></div>
                                <div class="h-4 w-2/3 rounded bg-slate-200"></div>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initials_from_two_word_name() {
        assert_eq!(get_initials(Some("John Doe"), None), "JD");
        assert_eq!(get_initials(Some("田中 太郎"), None), "田太");
    }

    #[test]
    fn initials_from_single_word_name() {
        assert_eq!(get_initials(Some("Taro"), None), "TA");
        assert_eq!(get_initials(Some("taro"), None), "TA");
    }

    #[test]
    fn initials_from_email() {
        assert_eq!(get_initials(None, Some("test@example.com")), "TE");
        assert_eq!(get_initials(Some(""), Some("test@example.com")), "TE");
    }

    #[test]
    fn initials_default_when_unset() {
        assert_eq!(get_initials(None, None), "U");
        assert_eq!(get_initials(None, Some("")), "U");
    }

    #[test]
    fn nav_active_matches_path_or_prefix() {
        assert!(is_nav_active("/receipts", "/receipts"));
        assert!(is_nav_active("/receipts/2024", "/receipts"));
        assert!(!is_nav_active("/receipt", "/receipts"));
        assert!(!is_nav_active("/", "/receipts"));
    }

    #[test]
    fn alert_variant_classes() {
        assert!(alert_variant_class("warning").contains("amber"));
        assert!(alert_variant_class("danger").contains("red"));
        assert!(alert_variant_class("success").contains("teal"));
        assert!(alert_variant_class("info").contains("blue"));
        assert!(alert_variant_class("").contains("blue"));
    }

    #[test]
    fn spinner_size_classes() {
        assert_eq!(spinner_size_class("sm"), "h-4 w-4");
        assert_eq!(spinner_size_class("lg"), "h-8 w-8");
        assert_eq!(spinner_size_class("md"), "h-6 w-6");
        assert_eq!(spinner_size_class(""), "h-6 w-6");
    }
}
