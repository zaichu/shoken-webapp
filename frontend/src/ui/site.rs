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
    use_context::<CurrentPath>()
        .map(|current| current.0)
        .unwrap_or_else(|| RwSignal::new(current_location()))
}

/// ナビのリンク。現在ページを指すものだけ aria-current を付ける
#[component]
pub fn NavLink(
    #[prop(into)] to: String,
    #[prop(into)] label: String,
    class: &'static str,
) -> impl IntoView {
    let path = current_path();
    let href = to.clone();
    view! {
        <a
            href=href
            class=class
            aria-current=move || is_nav_active(&path.get(), &to).then_some("page")
        >
            {label}
        </a>
    }
}

/// 文字付きの下部ナビ。主要4ページへ1タップで移動できる
#[component]
pub fn BottomNav(#[prop(optional, into)] class: Option<String>) -> impl IntoView {
    let path = current_path();
    let base_class = "app-bottom-nav no-print";
    let class = move || format!("{} {}", base_class, class.clone().unwrap_or_default());
    view! {
        <nav class=class aria-label="主要ナビゲーション">
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
pub fn AccountMenu(
    /// 表示名を出さない(2箇所に描画するときの DOM 重複を避ける)
    #[prop(optional)]
    hide_name: bool,
) -> impl IntoView {
    let session = use_session();
    let path = current_path();
    let on_login_page = move || pathname_of(&path.get()) == "/login";
    view! {
        <div class="flex shrink-0 items-center gap-3">
            {move || {
                if !session.loaded.get() {
                    view! {
                        <span class="text-sm font-semibold text-brand-muted">"読み込み中..."</span>
                    }
                        .into_any()
                } else if let Some(user) = session.user.get() {
                    view! {
                        <UserMenu
                            user=user
                            session=session
                            hide_name=hide_name
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

#[allow(dead_code)]
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

fn login_method(_email: &str) -> &'static str {
    // 現在は Google OAuth のみ対応
    "Google でログイン中"
}

#[component]
fn UserMenu(
    user: SessionUser,
    session: SessionStore,
    #[prop(optional)] hide_name: bool,
) -> impl IntoView {
    let menu_open = RwSignal::new(false);
    let menu_container = NodeRef::<html::Div>::new();
    let menu_items_ref = NodeRef::<html::Ul>::new();
    let focused_index = RwSignal::new(-1);
    let trigger_ref = NodeRef::<html::Button>::new();
    // メニュー開閉の世代番号。開くたびにインクリメントし、古い Effect のフォーカス試行を無効化する
    let menu_generation = RwSignal::new(0u32);

    // メニュー項目（role="menuitem" を持つ要素）を収集するヘルパー関数
    let collect_menuitems = move |ul: &web_sys::HtmlUListElement| -> Vec<web_sys::HtmlElement> {
        (0..ul.children().length())
            .filter_map(|i| ul.children().item(i))
            .filter_map(|li| li.query_selector("[role=\"menuitem\"]").ok().flatten())
            .filter_map(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
            .collect()
    };

    let on_pointer_down = window_event_listener(ev::pointerdown, move |ev| {
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
            focused_index.set(-1);
            if let Some(trigger) = trigger_ref.get() {
                let _ = trigger.focus();
            }
        }
    });

    let on_key_down = window_event_listener(ev::keydown, move |ev| {
        if !menu_open.get_untracked() {
            return;
        }

        match ev.key().as_str() {
            "Escape" => {
                menu_open.set(false);
                focused_index.set(-1);
                if let Some(trigger) = trigger_ref.get() {
                    let _ = trigger.focus();
                }
            }
            "ArrowDown" => {
                ev.prevent_default();
                if let Some(ul) = menu_items_ref.get() {
                    let items = collect_menuitems(&ul);
                    if !items.is_empty() {
                        let next = (focused_index.get() + 1).min(items.len() as i32 - 1);
                        focused_index.set(next);
                        let _ = items[next as usize].focus();
                    }
                }
            }
            "ArrowUp" => {
                ev.prevent_default();
                if let Some(ul) = menu_items_ref.get() {
                    let items = collect_menuitems(&ul);
                    if !items.is_empty() {
                        let next = (focused_index.get() - 1).max(0);
                        focused_index.set(next);
                        let _ = items[next as usize].focus();
                    }
                }
            }
            "Tab" => {
                if let Some(ul) = menu_items_ref.get() {
                    let items = collect_menuitems(&ul);
                    if !items.is_empty() {
                        let active = web_sys::window()
                            .and_then(|w| w.document())
                            .and_then(|d| d.active_element())
                            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
                        if let Some(active) = active
                            && let Some(pos) = items.iter().position(|b| b.is_same_node(Some(&active)))
                        {
                            if ev.shift_key() && pos == 0 {
                                ev.prevent_default();
                                let _ = items.last().unwrap().focus();
                                focused_index.set((items.len() - 1) as i32);
                            } else if !ev.shift_key() && pos == items.len() - 1 {
                                ev.prevent_default();
                                let _ = items[0].focus();
                                focused_index.set(0);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    });

    on_cleanup(move || {
        on_pointer_down.remove();
        on_key_down.remove();
    });

    // メニューが開かれたときに先頭項目（設定）へフォーカスを移す
    // Show コンポーネントの遅延描画を待つため Effect で実行
    // 世代番号を使って古い Effect のフォーカス試行を無効化
    Effect::new(move |_| {
        if menu_open.get() {
            let current_gen = menu_generation.get();
            focused_index.set(0);
            // 次のマイクロタスクでフォーカスを試行（Show の描画完了を待つ）
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(0).await;
                // この Effect の世代が最新でかつメニューがまだ開いている場合のみフォーカス
                if menu_open.get_untracked()
                    && menu_generation.get_untracked() == current_gen
                    && let Some(ul) = menu_items_ref.get_untracked()
                {
                    let items = collect_menuitems(&ul);
                    if !items.is_empty() {
                        let _ = items[0].focus();
                    }
                }
            });
        }
    });

    let user_name = user.name.clone();
    let user_email = user.email.clone();
    let display_name = Memo::new({
        let user_name = user_name.clone();
        let user_email = user_email.clone();
        move |_| {
            user_name
                .clone()
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| user_email.clone())
        }
    });
    let aria_name = user_name
        .clone()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "ユーザー".to_string());
    let initials = get_initials(user_name.as_deref(), Some(&user_email));
    let login_method_text = login_method(&user_email);
    let menu_label = format!("{aria_name}のアカウントメニュー");

    view! {
        <div class="flex items-center gap-3">
            <Show when=move || !hide_name>
                <span class="hidden max-w-64 truncate text-sm font-semibold text-brand-muted lg:inline">
                    {display_name.get()}
                </span>
            </Show>
            <div class="relative" node_ref=menu_container>
                <Button
                    variant=ButtonVariant::Header(ButtonSize::Sm)
                    attr:aria-haspopup="menu"
                    aria_label=menu_label
                    aria_expanded=move || menu_open.get()
                    attr:aria-controls="user-menu"
                    on_click=move |_| {
                        menu_open.update(|open| *open = !*open);
                        if !menu_open.get() {
                            focused_index.set(-1);
                        } else {
                            // メニューを開くとき世代を進め、古い Effect を無効化
                            menu_generation.update(|g| *g = g.wrapping_add(1));
                        }
                    }
                    node_ref=trigger_ref
                >
                    "アカウント"
                </Button>
                <Show when=move || menu_open.get()>
                    <ul
                        id="user-menu"
                        node_ref=menu_items_ref
                        class="absolute right-0 z-40 mt-2 w-72 overflow-hidden rounded-lg border border-ink/10 bg-surface text-text-deep shadow-2xl"
                        role="menu"
                        aria-label="ユーザーメニュー"
                    >
                        <li role="none" class="p-3 border-b border-ink/10">
                            <div class="flex items-center gap-3">
                                <span class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-gold text-brand font-bold text-lg">
                                    {initials.clone()}
                                </span>
                                <div class="min-w-0">
                                    <div class="font-semibold text-sm truncate">{display_name.get()}</div>
                                    <div class="text-xs text-text-muted truncate">{user_email.clone()}</div>
                                    <div class="text-[11px] text-text-muted">{login_method_text}</div>
                                </div>
                            </div>
                        </li>
                        <li role="none">
                            <a
                                href="/settings"
                                class="flex min-h-11 items-center gap-3 px-3 py-2 text-sm font-semibold text-text-deep hover:bg-surface-raised no-underline"
                                role="menuitem"
                                tabindex="0"
                                on:click=move |_| menu_open.set(false)
                            >
                                <svg class="shrink-0 w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                                    <circle cx="12" cy="12" r="3"/>
                                    <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
                                </svg>
                                <span>設定</span>
                            </a>
                        </li>
                        <li role="none">
                            <Button
                                variant=ButtonVariant::MenuItem
                                attr:role="menuitem"
                                testid="logout"
                                class="w-full min-h-11"
                                on_click=move |_| {
                                    menu_open.set(false);
                                    leptos::task::spawn_local(async move {
                                        session.logout().await;
                                    });
                                }
                            >
                                <svg class="shrink-0 w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                                    <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4"/>
                                    <polyline points="16 17 21 12 16 7"/>
                                    <line x1="21" y1="12" x2="9" y2="12"/>
                                </svg>
                                <span>ログアウト</span>
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
