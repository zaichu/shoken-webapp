use leptos::ev;
use leptos::prelude::*;

// 非 wasm(単体テスト)では window が無いため開いて始める
pub fn workspace_panel_default_open() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|window| window.inner_width().ok())
            .and_then(|value| value.as_f64())
            .is_none_or(|width| width >= 1024.0)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        true
    }
}

fn is_compact_viewport() -> bool {
    !workspace_panel_default_open()
}

fn should_close_panel(compact: bool, open: bool) -> bool {
    compact && open
}

#[component]
pub fn WorkspaceShell(
    #[prop(into)] workspace_testid: &'static str,
    #[prop(into)] rail_testid: &'static str,
    #[prop(into)] main_testid: &'static str,
    #[prop(into)] panel_id: Signal<String>,
    #[prop(into)] toggle_testid: &'static str,
    #[prop(into)] panel_label: &'static str,
    /// false のときトグル・ドロワー機構を出さずパネルを常時表示する
    collapsible: bool,
    #[prop(into)] panel_open: Signal<bool>,
    on_toggle: Callback<()>,
    on_close: Callback<()>,
    panel: AnyView,
    children: Children,
) -> impl IntoView {
    // SPA 遷移をまたいで open 状態が残ったまま狭い帯で再マウントされると、
    // ドロワーが全面を塞ぐ。モバイルで畳んで始めるのと同じく、マウント時に畳み直す
    if collapsible && should_close_panel(is_compact_viewport(), panel_open.get_untracked()) {
        on_close.run(());
    }
    // デスクトップ→狭い帯の越境でも同じくドロワーが全面を塞ぐ。
    // change は境界越えでしか発火しないため、狭帯内のリサイズやトグルには干渉しない
    #[cfg(target_arch = "wasm32")]
    #[allow(clippy::collapsible_if)]
    if collapsible {
        if let Some(media) = web_sys::window()
            .and_then(|window| window.match_media("(max-width: 63.999rem)").ok().flatten())
        {
            use wasm_bindgen::JsCast;
            let media_check = media.clone();
            let on_change =
                wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
                    if should_close_panel(media_check.matches(), panel_open.get_untracked()) {
                        on_close.run(());
                    }
                }) as Box<dyn FnMut(_)>);
            media.set_onchange(Some(on_change.as_ref().unchecked_ref()));
            // Closure/MediaQueryList は wasm の単一スレッド前提で Send にできないため SendWrapper で保持
            let keep = send_wrapper::SendWrapper::new((media, on_change));
            on_cleanup(move || {
                let (media, _on_change) = &*keep;
                media.set_onchange(None);
            });
        }
    }
    // ドロワー表示中だけ Esc で閉じる。デスクトップの常設パネルには干渉しない
    if collapsible {
        let on_key_down = window_event_listener(ev::keydown, move |ev| {
            if ev.key() == "Escape" && panel_open.get_untracked() && is_compact_viewport() {
                on_close.run(());
            }
        });
        on_cleanup(move || on_key_down.remove());
    }
    let panel_id_toggle = panel_id;
    view! {
        <div
            class=move || {
                if !collapsible {
                    "ws-sidegrid ws-static".to_string()
                } else if panel_open.get() {
                    "ws-sidegrid".to_string()
                } else {
                    "ws-sidegrid ws-closed".to_string()
                }
            }
            data-testid=workspace_testid
        >
            {collapsible.then(|| {
                view! {
                    <div
                        class="ws-backdrop"
                        hidden=move || !panel_open.get()
                        on:click=move |_| on_close.run(())
                        aria-hidden="true"
                    />
                }
            })}
            // DOM 順はパネル先(キーボード・読み上げ順)。見た目も左パネルが先なので order は使わない
            <aside
                class="ws-panel"
                id=panel_id
                data-testid=rail_testid
                aria-label=panel_label
            >
                {collapsible.then(|| {
                    view! {
                        <div class="phead">
                            <button
                                type="button"
                                class="ws-toggle"
                                data-testid=toggle_testid
                                aria-expanded=move || panel_open.get().to_string()
                                aria-controls=panel_id_toggle
                                aria-label=move || {
                                    if panel_open.get() {
                                        format!("{panel_label}パネルを閉じる")
                                    } else {
                                        format!("{panel_label}パネルを開く")
                                    }
                                }
                                on:click=move |_| on_toggle.run(())
                            >
                                <span aria-hidden="true">
                                    {move || if panel_open.get() { "«" } else { "»" }}
                                </span>
                                <span class="ws-toggle-label">{panel_label}</span>
                            </button>
                        </div>
                    }
                })}
                <div class="ws-panel-body" hidden=move || collapsible && !panel_open.get()>
                    {panel}
                </div>
            </aside>
            <div class="ws-main" data-testid=main_testid>
                {children()}
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests;
