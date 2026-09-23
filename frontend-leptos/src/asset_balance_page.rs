use crate::api::{ApiClient, ApiError};
use crate::dto::{AssetBalance, AssetBalanceListResponse};
use crate::session::use_session;
use leptos::prelude::*;

async fn fetch_asset_balances() -> Result<Vec<AssetBalance>, ApiError> {
    ApiClient::read_client()
        .get_json::<AssetBalanceListResponse>(
            "/api/v1/asset-balances",
            &[
                ("per_page", "200"),
                ("page", "1"),
                ("include_summary", "true"),
            ],
        )
        .await
        .map(|list| list.data)
}

#[component]
pub fn AssetBalancePage() -> impl IntoView {
    let session = use_session();
    let balances = RwSignal::new(None::<(u64, Result<Vec<AssetBalance>, String>)>);
    Effect::new(move |_| {
        let generation = session.generation.get();
        if session.user.get().is_none() {
            return;
        }
        if balances
            .get_untracked()
            .is_some_and(|(cached, _)| cached == generation)
        {
            return;
        }
        let session = session.clone();
        leptos::task::spawn_local(async move {
            let result = match fetch_asset_balances().await {
                Ok(rows) => Ok(rows),
                Err(error) => {
                    if error.is_unauthorized() {
                        session.mark_unauthenticated();
                    }
                    Err("データ取得に失敗しました".to_string())
                }
            };
            if session.is_current(generation) {
                balances.set(Some((generation, result)));
            }
        });
    });
    view! {
        <div class="page-surface">
            <div class="mb-5 max-sm:mb-2">
                <div class="flex flex-col gap-3 border-l-4 border-amber-500 pl-4 sm:flex-row sm:items-end sm:justify-between">
                    <div>
                        <p class="mb-1 text-[11px] font-bold uppercase tracking-[0.22em] text-slate-500 max-sm:hidden">
                            "Portfolio"
                        </p>
                        <h1 class="text-2xl font-black leading-tight tracking-normal text-slate-950 max-sm:text-lg">
                            "資産管理"
                        </h1>
                        <p class="mt-1 text-sm font-medium text-slate-600 max-sm:hidden">
                            "保有している銘柄の一覧と評価額を確認できます。"
                        </p>
                    </div>
                </div>
            </div>
            <div data-testid="assetbalance-workspace">
                {move || match balances.get().map(|(_, result)| result) {
                    None => view! { <p role="status">"読み込み中..."</p> }.into_any(),
                    Some(Err(message)) => view! { <div role="alert">{message}</div> }.into_any(),
                    Some(Ok(rows)) => {
                        if rows.is_empty() {
                            view! {
                                <div>
                                    <h3>"資産管理データがありません"</h3>
                                    <p>"CSVで資産管理データを追加してください"</p>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! { <AssetBalanceTable rows=rows /> }.into_any()
                        }
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn AssetBalanceTable(rows: Vec<AssetBalance>) -> impl IntoView {
    view! {
        <table>
            <thead>
                <tr>
                    <th>"銘柄コード"</th>
                    <th>"銘柄名"</th>
                    <th>"保有株数"</th>
                    <th>"評価額"</th>
                    <th>"損益率"</th>
                </tr>
            </thead>
            <tbody>
                {rows
                    .into_iter()
                    .map(|row| {
                        view! {
                            <tr>
                                <td>{row.security_code}</td>
                                <td>{row.security_name}</td>
                                <td>{row.shares.to_string()}</td>
                                <td>{row.market_value.to_string()}</td>
                                <td>{format!("{}%", row.profit_loss_rate)}</td>
                            </tr>
                        }
                    })
                    .collect_view()}
            </tbody>
        </table>
    }
}
