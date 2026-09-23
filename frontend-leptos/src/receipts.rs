use crate::dto::{
    Dividend, DividendListResponse, DomesticStock, DomesticStockListResponse, Mutualfund,
    MutualfundListResponse, SessionUser,
};
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ReceiptsTab {
    Dividend,
    DomesticStock,
    MutualFund,
}

impl ReceiptsTab {
    pub const ALL: [ReceiptsTab; 3] = [
        ReceiptsTab::Dividend,
        ReceiptsTab::DomesticStock,
        ReceiptsTab::MutualFund,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            ReceiptsTab::Dividend => "配当金",
            ReceiptsTab::DomesticStock => "国内株式",
            ReceiptsTab::MutualFund => "投資信託",
        }
    }

    fn list_path(&self) -> &'static str {
        match self {
            ReceiptsTab::Dividend => "/api/v1/dividends",
            ReceiptsTab::DomesticStock => "/api/v1/domestic-stock-transactions",
            ReceiptsTab::MutualFund => "/api/v1/mutual-fund-transactions",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReceiptItem {
    Dividend(Dividend),
    DomesticStock(DomesticStock),
    MutualFund(Mutualfund),
}

impl ReceiptItem {
    pub fn cells(&self) -> Vec<String> {
        match self {
            ReceiptItem::Dividend(row) => vec![
                row.settlement_date.clone(),
                row.product.clone(),
                row.account.clone(),
                row.security_code.clone(),
                row.security_name.clone(),
                row.unit_price.to_string(),
                row.dividends_before_tax.to_string(),
                row.taxes.to_string(),
                row.net_amount_received.to_string(),
            ],
            ReceiptItem::DomesticStock(row) => vec![
                row.trade_date.clone(),
                row.security_code.clone(),
                row.security_name.clone(),
                row.account.clone(),
                row.shares.to_string(),
                row.realized_profit_and_loss.to_string(),
                row.taxes.to_string(),
            ],
            ReceiptItem::MutualFund(row) => vec![
                row.trade_date.clone(),
                row.fund_name.clone(),
                row.account.clone(),
                row.shares.to_string(),
                row.realized_profit_and_loss.to_string(),
                row.taxes.to_string(),
            ],
        }
    }
}

async fn fetch_session_user_id() -> Result<String, String> {
    let response = gloo_net::http::Request::get("/api/v1/session")
        .send()
        .await
        .map_err(|_| "セッションの取得に失敗しました。".to_string())?;
    if !response.ok() {
        return Err(format!("未認証: {}", response.status()));
    }
    let user = response
        .json::<SessionUser>()
        .await
        .map_err(|_| "セッションの解析に失敗しました。".to_string())?;
    if user.id.is_empty() {
        return Err("ユーザーIDが空です。".to_string());
    }
    Ok(user.id)
}

async fn fetch_list(tab: ReceiptsTab) -> Result<Vec<ReceiptItem>, String> {
    let url = format!(
        "{}?per_page=1000&page=1&include_summary=true",
        tab.list_path()
    );
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|_| "データ取得に失敗しました".to_string())?;
    if !response.ok() {
        return Err("データ取得に失敗しました".to_string());
    }
    match tab {
        ReceiptsTab::Dividend => response
            .json::<DividendListResponse>()
            .await
            .map(|list| list.data.into_iter().map(ReceiptItem::Dividend).collect())
            .map_err(|_| "データ取得に失敗しました".to_string()),
        ReceiptsTab::DomesticStock => response
            .json::<DomesticStockListResponse>()
            .await
            .map(|list| {
                list.data
                    .into_iter()
                    .map(ReceiptItem::DomesticStock)
                    .collect()
            })
            .map_err(|_| "データ取得に失敗しました".to_string()),
        ReceiptsTab::MutualFund => response
            .json::<MutualfundListResponse>()
            .await
            .map(|list| list.data.into_iter().map(ReceiptItem::MutualFund).collect())
            .map_err(|_| "データ取得に失敗しました".to_string()),
    }
}

#[derive(Clone, Debug)]
pub enum TabState {
    Loading,
    Ready(Vec<ReceiptItem>),
    Failed(String),
}
#[derive(Clone)]
pub struct ReceiptsStore {
    pub user_id: RwSignal<String>,
    pub active_tab: RwSignal<ReceiptsTab>,
    visited: RwSignal<HashSet<ReceiptsTab>>,
    cache: RwSignal<HashMap<(String, ReceiptsTab), TabState>>,
    fetch: Action<(String, ReceiptsTab), ()>,
}

impl ReceiptsStore {
    pub fn rows(&self, tab: ReceiptsTab) -> Vec<ReceiptItem> {
        let user = self.user_id.get();
        self.cache.with(|map| match map.get(&(user, tab)) {
            Some(TabState::Ready(rows)) => rows.clone(),
            _ => Vec::new(),
        })
    }

    pub fn count(&self, tab: ReceiptsTab) -> usize {
        self.rows(tab).len()
    }

    pub fn error(&self) -> Option<String> {
        let user = self.user_id.get();
        self.cache.with(|map| {
            ReceiptsTab::ALL
                .iter()
                .find_map(|tab| match map.get(&(user.clone(), *tab)) {
                    Some(TabState::Failed(message)) => Some(message.clone()),
                    _ => None,
                })
        })
    }

    pub fn tab_state(&self, tab: ReceiptsTab) -> TabState {
        let user = self.user_id.get();
        self.cache
            .with(|map| map.get(&(user, tab)).cloned().unwrap_or(TabState::Loading))
    }

    pub fn is_authenticated(&self) -> bool {
        !self.user_id.get().is_empty()
    }

    pub fn select_tab(&self, tab: ReceiptsTab) {
        self.visited.update(|visited| {
            visited.insert(tab);
        });
        self.active_tab.set(tab);
        self.ensure(tab);
    }

    pub fn logout(&self) {
        let this = self.clone();
        batch(move || {
            this.user_id.set(String::new());
            this.cache.update(|map| map.clear());
        });
    }

    pub fn login_as(&self, user_id: String) {
        self.user_id.set(user_id);
    }

    fn ensure(&self, tab: ReceiptsTab) {
        let user = self.user_id.get_untracked();
        if user.is_empty() {
            return;
        }
        let in_flight_or_ready = self.cache.with_untracked(|map| {
            matches!(
                map.get(&(user.clone(), tab)),
                Some(TabState::Loading) | Some(TabState::Ready(_))
            )
        });
        if in_flight_or_ready {
            return;
        }
        self.visited.update(|visited| {
            visited.insert(tab);
        });
        self.cache.update(|map| {
            map.insert((user.clone(), tab), TabState::Loading);
        });
        self.fetch.dispatch((user, tab));
    }
}

fn tab_settled(store: &ReceiptsStore, user: &str, tab: ReceiptsTab) -> bool {
    store.cache.with(|map| {
        matches!(
            map.get(&(user.to_string(), tab)),
            Some(TabState::Ready(_)) | Some(TabState::Failed(_))
        )
    })
}

pub fn use_receipts_data(initial_tab: ReceiptsTab) -> ReceiptsStore {
    let user_id = RwSignal::new(String::new());
    let active_tab = RwSignal::new(initial_tab);
    let visited = RwSignal::new(HashSet::from([initial_tab]));
    let cache = RwSignal::new(HashMap::new());

    let fetch = Action::new_unsync(move |(user, tab): &(String, ReceiptsTab)| {
        let (user, tab) = (user.clone(), *tab);
        async move {
            let rows = fetch_list(tab).await;
            if user_id.get_untracked() != user {
                return;
            }
            cache.update(|map| {
                map.insert(
                    (user, tab),
                    match rows {
                        Ok(rows) => TabState::Ready(rows),
                        Err(message) => TabState::Failed(message),
                    },
                );
            });
        }
    });

    let store = ReceiptsStore {
        user_id,
        active_tab,
        visited,
        cache,
        fetch,
    };

    leptos::task::spawn_local(async move {
        if let Ok(id) = fetch_session_user_id().await {
            user_id.set(id);
        }
    });

    Effect::new({
        let store = store.clone();
        move |_| {
            let user = user_id.get();
            let active = active_tab.get();
            if user.is_empty() {
                return;
            }
            store.ensure(active);
            if tab_settled(&store, &user, active) {
                let mut background: Vec<ReceiptsTab> = ReceiptsTab::ALL
                    .iter()
                    .copied()
                    .filter(|tab| *tab != active)
                    .collect();
                background.sort_by_key(|tab| {
                    !store
                        .visited
                        .with_untracked(|visited| visited.contains(tab))
                });
                for tab in background {
                    store.ensure(tab);
                }
            }
        }
    });

    store
}
