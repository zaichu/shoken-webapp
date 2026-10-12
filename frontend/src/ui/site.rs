use crate::api::dto::SessionUser;
use crate::app::{CurrentPath, current_location, pathname_of};
use crate::session::{SessionStore, use_session};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::confirm_modal::ConfirmDeleteModal;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

pub const NAV_ITEMS: &[(&str, &str)] = &[
    ("/", "ホーム"),
    ("/assetbalance", "資産管理"),
    ("/receipts", "取引明細"),
    ("/search", "銘柄検索"),
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

fn is_nav_active(path: &str, to: &str) -> bool {
    let path = pathname_of(path);
    if to == "/" {
        return path == "/";
    }
    path == to || path.starts_with(&format!("{to}/"))
}

fn current_path() -> RwSignal<String> {
    // App 配下では SPA 遷移の path を使い、単独で描くテスト等では現在の URL にフォールバック
    use_context::<CurrentPath>()
        .map(|current| current.0)
        .unwrap_or_else(|| RwSignal::new(current_location()))
}

/// 文字付きの下部ナビ。主要4ページへ1タップで移動できる
#[component]
pub fn BottomNav() -> impl IntoView {
    let path = current_path();
    view! {
        <nav class="app-bottom-nav no-print" aria-label="主要ナビゲーション">
            {NAV_ITEMS
                .iter()
                .map(|(to, label)| {
                    let to = *to;
                    let active = move || is_nav_active(&path.get(), to);
                    view! {
                        <a
                            href=to
                            class="bottom-nav-link"
                            aria-current=move || active().then_some("page")
                        >
                            {*label}
                        </a>
                    }
                })
                .collect_view()}
        </nav>
    }
}

/// 削除確認の開閉をページをまたいで共有する。メニューはページ枠内で遷移ごとに
/// 作り直されるため、結果表示中にアンマウントされないようダイアログはシェルが持つ
#[derive(Clone, Copy)]
pub(crate) struct AccountDeleteConfirm(pub(crate) RwSignal<bool>);

/// ページ枠の右上に置くアカウント導線(ログイン/メニュー)。アカウント削除の確認だけを扱う
#[component]
pub fn AccountMenu() -> impl IntoView {
    let session = use_session();
    let path = current_path();
    let delete_confirm_open = use_context::<AccountDeleteConfirm>()
        .map(|confirm| confirm.0)
        .unwrap_or_else(|| RwSignal::new(false));
    let on_login_page = move || pathname_of(&path.get()) == "/login";
    view! {
        <div class="flex shrink-0 items-center gap-3">
            {move || {
                if !session.loaded.get() {
                    view! {
                        <span class="text-sm font-semibold text-text-muted">"読み込み中..."</span>
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
                            variant=ButtonVariant::Secondary(ButtonSize::Sm)
                            on_click=move |_| session.login()
                        >
                            "ログイン"
                        </Button>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

/// 削除確認ダイアログ本体。ログイン画面への遷移でメニューが消えても結果表示を残すため、
/// App がシェル直下に1つだけ描く
#[component]
pub fn AccountDeleteDialog() -> impl IntoView {
    let Some(delete_confirm_open) = use_context::<AccountDeleteConfirm>().map(|confirm| confirm.0)
    else {
        return ().into_any();
    };
    let session = use_session();
    let delete_error = RwSignal::new(Option::<String>::None);
    let deleting = RwSignal::new(false);
    let deleting_memo = Memo::new(move |_| deleting.get());
    Effect::new(move |_| {
        if delete_confirm_open.get() {
            delete_error.set(None);
        }
    });
    let content = view! {
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
    };
    content.into_any()
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
                            class="h-9 w-9 rounded-md border border-ink/15 bg-surface-raised object-cover max-sm:hidden"
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
            <span class="max-w-64 truncate text-sm font-semibold text-text-soft max-sm:hidden">
                {display_name}
            </span>
            <div class="relative" node_ref=menu_container>
                <Button
                    variant=ButtonVariant::Secondary(ButtonSize::Sm)
                    attr:aria-haspopup="menu"
                    aria_expanded=move || menu_open.get()
                    attr:aria-controls="user-menu"
                    on_click=move |_| menu_open.update(|open| *open = !*open)
                >
                    "メニュー"
                </Button>
                <Show when=move || menu_open.get()>
                    <ul
                        id="user-menu"
                        class="absolute right-0 z-40 mt-2 w-56 overflow-hidden rounded-lg border border-ink/10 bg-surface text-text-deep shadow-2xl"
                        role="menu"
                        aria-label="ユーザーメニュー"
                    >
                        <li role="none">
                            <Button
                                variant=ButtonVariant::MenuItem
                                attr:role="menuitem"
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
                                attr:role="menuitem"
                                attr:aria-describedby="delete-warning"
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
            <div class="page-gutter flex flex-wrap items-center justify-center gap-x-6 gap-y-1 text-sm text-text-subtle">
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

#[cfg(test)]
mod tests;
