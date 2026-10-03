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

#[component]
pub fn WorkspaceShell(
    #[prop(into)] workspace_testid: &'static str,
    #[prop(into)] rail_testid: &'static str,
    #[prop(into)] main_testid: &'static str,
    #[prop(into)] panel_id: Signal<String>,
    #[prop(into)] toggle_testid: &'static str,
    #[prop(into)] panel_open: Signal<bool>,
    on_toggle: Callback<()>,
    on_close: Callback<()>,
    panel: AnyView,
    children: Children,
) -> impl IntoView {
    // ドロワー表示中だけ Esc で閉じる。デスクトップの常設パネルには干渉しない
    let on_key_down = window_event_listener(ev::keydown, move |ev| {
        if ev.key() == "Escape" && panel_open.get_untracked() && is_compact_viewport() {
            on_close.run(());
        }
    });
    on_cleanup(move || on_key_down.remove());
    let panel_id_toggle = panel_id;
    view! {
        <div
            class=move || {
                if panel_open.get() { "ws-sidegrid".to_string() } else { "ws-sidegrid ws-closed".to_string() }
            }
            data-testid=workspace_testid
        >
            <div
                class="ws-backdrop"
                hidden=move || !panel_open.get()
                on:click=move |_| on_close.run(())
                aria-hidden="true"
            />
            // DOM 順はパネル先(キーボード・読み上げ順)。見た目も左パネルが先なので order は使わない
            <aside
                class="ws-panel"
                id=panel_id
                data-testid=rail_testid
                aria-label="取り込み・検索"
            >
                <div class="phead">
                    <button
                        type="button"
                        class="ws-toggle"
                        data-testid=toggle_testid
                        aria-expanded=move || panel_open.get().to_string()
                        aria-controls=panel_id_toggle
                        aria-label=move || {
                            if panel_open.get() {
                                "取り込み・検索パネルを閉じる".to_string()
                            } else {
                                "取り込み・検索パネルを開く".to_string()
                            }
                        }
                        on:click=move |_| on_toggle.run(())
                    >
                        <span aria-hidden="true">
                            {move || if panel_open.get() { "«" } else { "»" }}
                        </span>
                        <span class="ws-toggle-label">"取り込み・検索"</span>
                    </button>
                </div>
                <div class="ws-panel-body" hidden=move || !panel_open.get()>
                    {panel}
                </div>
            </aside>
            <div class="ws-main" data-testid=main_testid>
                {children()}
            </div>
        </div>
    }
}
