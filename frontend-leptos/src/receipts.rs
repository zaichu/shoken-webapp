use leptos::prelude::*;
use serde::Deserialize;
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

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct ReceiptRow {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub settlement_date: String,
    #[serde(default)]
    pub trade_date: String,
    #[serde(default)]
    pub product: String,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub security_code: String,
    #[serde(default)]
    pub security_name: String,
    #[serde(default)]
    pub fund_name: String,
    #[serde(default)]
    pub unit_price: String,
    #[serde(default)]
    pub shares: String,
    #[serde(default)]
    pub dividends_before_tax: String,
    #[serde(default)]
    pub taxes: String,
    #[serde(default)]
    pub net_amount_received: String,
    #[serde(default)]
    pub realized_profit_and_loss: String,
}

impl ReceiptRow {
    pub fn date(&self, tab: ReceiptsTab) -> &str {
        match tab {
            ReceiptsTab::Dividend => &self.settlement_date,
            _ => &self.trade_date,
        }
    }

    pub fn name(&self, tab: ReceiptsTab) -> &str {
        match tab {
            ReceiptsTab::MutualFund => &self.fund_name,
            _ => &self.security_name,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct PaginatedList {
    #[serde(default)]
    data: Vec<ReceiptRow>,
}

#[derive(Clone, Debug, Deserialize)]
struct SessionUser {
    #[serde(default)]
    id: String,
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

async fn fetch_list(tab: ReceiptsTab) -> Result<Vec<ReceiptRow>, String> {
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
    let list = response
        .json::<PaginatedList>()
        .await
        .map_err(|_| "データ取得に失敗しました".to_string())?;
    Ok(list.data)
}

#[derive(Clone, Debug)]
pub enum TabState {
    Loading,
    Ready(Vec<ReceiptRow>),
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
    pub fn rows(&self, tab: ReceiptsTab) -> Vec<ReceiptRow> {
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
