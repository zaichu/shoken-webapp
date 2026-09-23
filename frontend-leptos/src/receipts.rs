use crate::api::{ApiClient, ApiError};
use crate::dto::{DividendListResponse, DomesticStockListResponse, MutualfundListResponse};
use crate::session::SessionStore;
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
    Dividend(crate::dto::Dividend),
    DomesticStock(crate::dto::DomesticStock),
    MutualFund(crate::dto::Mutualfund),
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

async fn fetch_list(tab: ReceiptsTab) -> Result<Vec<ReceiptItem>, ApiError> {
    let client = ApiClient::read_client();
    let query = &[
        ("per_page", "1000"),
        ("page", "1"),
        ("include_summary", "true"),
    ];
    match tab {
        ReceiptsTab::Dividend => client
            .get_json::<DividendListResponse>(tab.list_path(), query)
            .await
            .map(|list| list.data.into_iter().map(ReceiptItem::Dividend).collect()),
        ReceiptsTab::DomesticStock => client
            .get_json::<DomesticStockListResponse>(tab.list_path(), query)
            .await
            .map(|list| {
                list.data
                    .into_iter()
                    .map(ReceiptItem::DomesticStock)
                    .collect()
            }),
        ReceiptsTab::MutualFund => client
            .get_json::<MutualfundListResponse>(tab.list_path(), query)
            .await
            .map(|list| list.data.into_iter().map(ReceiptItem::MutualFund).collect()),
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
    session: SessionStore,
    pub active_tab: RwSignal<ReceiptsTab>,
    visited: RwSignal<HashSet<ReceiptsTab>>,
    cache: RwSignal<HashMap<(u64, ReceiptsTab), TabState>>,
    fetch: Action<(u64, ReceiptsTab), ()>,
}

impl ReceiptsStore {
    pub fn rows(&self, tab: ReceiptsTab) -> Vec<ReceiptItem> {
        let generation = self.session.generation.get();
        self.cache.with(|map| match map.get(&(generation, tab)) {
            Some(TabState::Ready(rows)) => rows.clone(),
            _ => Vec::new(),
        })
    }

    pub fn count(&self, tab: ReceiptsTab) -> usize {
        self.rows(tab).len()
    }

    pub fn error(&self) -> Option<String> {
        let generation = self.session.generation.get();
        self.cache.with(|map| {
            ReceiptsTab::ALL
                .iter()
                .find_map(|tab| match map.get(&(generation, *tab)) {
                    Some(TabState::Failed(message)) => Some(message.clone()),
                    _ => None,
                })
        })
    }

    pub fn tab_state(&self, tab: ReceiptsTab) -> TabState {
        let generation = self.session.generation.get();
        self.cache.with(|map| {
            map.get(&(generation, tab))
                .cloned()
                .unwrap_or(TabState::Loading)
        })
    }

    pub fn is_authenticated(&self) -> bool {
        self.session.user.get().is_some()
    }

    pub fn select_tab(&self, tab: ReceiptsTab) {
        self.visited.update(|visited| {
            visited.insert(tab);
        });
        self.active_tab.set(tab);
        self.ensure(tab);
    }

    fn ensure(&self, tab: ReceiptsTab) {
        if self.session.user.get_untracked().is_none() {
            return;
        }
        let generation = self.session.generation.get_untracked();
        let needs_prune = self
            .cache
            .with_untracked(|map| map.keys().any(|(cached, _)| *cached != generation));
        if needs_prune {
            self.cache.update(|map| {
                map.retain(|key, _| key.0 == generation);
            });
        }
        let in_flight_or_ready = self.cache.with_untracked(|map| {
            matches!(
                map.get(&(generation, tab)),
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
            map.insert((generation, tab), TabState::Loading);
        });
        self.fetch.dispatch((generation, tab));
    }
}

fn tab_settled(store: &ReceiptsStore, generation: u64, tab: ReceiptsTab) -> bool {
    store.cache.with(|map| {
        matches!(
            map.get(&(generation, tab)),
            Some(TabState::Ready(_)) | Some(TabState::Failed(_))
        )
    })
}

fn should_apply_fetch_result(session: &SessionStore, generation: u64) -> bool {
    session.is_current(generation)
}

pub fn use_receipts_data(session: SessionStore, initial_tab: ReceiptsTab) -> ReceiptsStore {
    let active_tab = RwSignal::new(initial_tab);
    let visited = RwSignal::new(HashSet::from([initial_tab]));
    let cache = RwSignal::new(HashMap::new());

    let fetch_session = session.clone();
    let cache_signal = cache;
    let fetch = Action::new_unsync(move |(generation, tab): &(u64, ReceiptsTab)| {
        let (generation, tab) = (*generation, *tab);
        let session = fetch_session.clone();
        async move {
            let rows = fetch_list(tab).await;
            if !should_apply_fetch_result(&session, generation) {
                return;
            }
            if let Err(error) = &rows {
                if error.is_unauthorized() {
                    session.mark_unauthenticated();
                }
            }
            cache_signal.update(|map| {
                map.insert(
                    (generation, tab),
                    match rows {
                        Ok(rows) => TabState::Ready(rows),
                        Err(_) => TabState::Failed("データ取得に失敗しました".to_string()),
                    },
                );
            });
        }
    });

    let store = ReceiptsStore {
        session: session.clone(),
        active_tab,
        visited,
        cache,
        fetch,
    };

    Effect::new({
        let store = store.clone();
        move |_| {
            let user = session.user.get();
            let active = active_tab.get();
            if user.is_none() {
                return;
            }
            let generation = session.generation.get();
            store.ensure(active);
            if tab_settled(&store, generation, active) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn user(id: &str) -> crate::dto::SessionUser {
        crate::dto::SessionUser {
            id: id.to_string(),
            email: format!("{id}@example.com"),
            name: None,
            picture_url: None,
        }
    }

    #[test]
    fn stale_receipts_result_is_rejected_after_same_or_different_user_login() {
        let owner = Owner::new();
        owner.with(|| {
            for next_user in [user("alice"), user("bob")] {
                let session = SessionStore::new();
                session.user.set(Some(user("alice")));
                let fetch_generation = session.generation.get_untracked();

                session.mark_unauthenticated();
                session.user.set(Some(next_user));

                assert!(!should_apply_fetch_result(&session, fetch_generation));
            }
        });
    }

    #[test]
    fn cells_keep_decimal_text() {
        let row: crate::dto::Dividend = serde_json::from_value(serde_json::json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "settlement_date": "2024-03-01",
            "product": "特定口座",
            "account": "SBI証券",
            "security_code": "7203",
            "security_name": "トヨタ自動車",
            "unit_price": 30.0,
            "shares": 100,
            "dividends_before_tax": 3000,
            "taxes": 609,
            "net_amount_received": 2391,
            "created_at": "2024-03-01T00:00:00Z",
            "updated_at": "2024-03-01T00:00:00Z"
        }))
        .expect("deserialize");
        let cells = ReceiptItem::Dividend(row).cells();
        assert_eq!(
            cells,
            vec![
                "2024-03-01",
                "特定口座",
                "SBI証券",
                "7203",
                "トヨタ自動車",
                "30.0",
                "3000",
                "609",
                "2391",
            ]
        );
    }
}
