use super::use_asset_balance_state;
use crate::app::CurrentPath;
use crate::features::asset_balance::holdings::holding_stock_href;
use crate::features::asset_balance::store::load_asset_balances;
use crate::session::use_session;
use crate::ui::empty_state::EmptyState;
use crate::ui::labels;
use crate::ui::page_frame::PageFrame;
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use leptos::prelude::*;

/// 旧 `/assetbalance/holdings/:id` は銘柄画面(`/search?code=`)へ置き換える。
/// 保存済み行の ID から銘柄コードを引いて同じ銘柄の画面を開く
#[component]
pub fn AssetBalanceHoldingPage(id: String) -> impl IntoView {
    let session = use_session();
    let state = use_asset_balance_state(session);
    let balances = state.balances;
    let dividends = state.dividends;
    let lookup = state.lookup;
    let data_ops = state.data_ops;
    if session.user.get_untracked().is_some() {
        load_asset_balances(
            session,
            session.generation.get_untracked(),
            balances,
            dividends,
            lookup,
            data_ops,
        );
    }
    let resolved = move || {
        let generation = session.generation.get();
        balances.with(|slot| {
            crate::features::asset_balance::store::csv::resolve_asset_balance(generation, slot)
        })
    };
    let reload = move || {
        if session.user.get_untracked().is_none() {
            return;
        }
        balances.set(None);
        load_asset_balances(
            session,
            session.generation.get_untracked(),
            balances,
            dividends,
            lookup,
            data_ops,
        );
    };
    // 行が見つかれば銘柄画面へ置き換える(履歴は残さない)。描画の副作用で1度だけ行う
    let redirected = std::cell::Cell::new(false);
    let redirect_to_search = move |code: &str| {
        if redirected.get() {
            return;
        }
        redirected.set(true);
        let path = holding_stock_href(code);
        if let Some(window) = web_sys::window()
            && let Ok(history) = window.history()
        {
            let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&path));
        }
        if let Some(current) = use_context::<CurrentPath>() {
            current.0.set(path);
        }
    };
    view! {
        <PageFrame
            title=labels::ASSET_HOLDING_DETAIL_TITLE
            back_href="/assetbalance"
            back_label=labels::ASSET_BALANCE_BACK
        >
            <div class="flex flex-1 flex-col">
                {move || {
                    data_ops
                        .with(|ops| ops.refresh_error.clone())
                        .map(|message| {
                            view! {
                                <div class="no-print mb-3">
                                    <Alert variant=AlertVariant::Danger>
                                        <strong>{labels::ERROR_PREFIX}</strong>
                                        " "
                                        {message}
                                    </Alert>
                                </div>
                            }
                        })
                }}
                {move || {
                    let generation = session.generation.get();
                    let list_error = balances.with(|slot| match slot {
                        Some((cached, Err(message))) if *cached == generation => {
                            Some(message.clone())
                        }
                        _ => None,
                    });
                    let view = resolved();
                    if let Some(message) = list_error {
                        return view! { <ListLoadError message=message on_retry=reload /> }
                            .into_any();
                    }
                    match view {
                        None => view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }.into_any(),
                        Some(view) => {
                            let found = view.rows.iter().find(|row| {
                                row.id.to_string() == id
                            });
                            let Some(row) = found else {
                                return view! {
                                    <div class="px-4 py-4">
                                        <EmptyState title=labels::ASSET_HOLDING_NOT_FOUND />
                                    </div>
                                }
                                    .into_any();
                            };
                            let code = row.security_code.to_string();
                            redirect_to_search(&code);
                            view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }
                                .into_any()
                        }
                    }
                }}
            </div>
        </PageFrame>
    }
}
