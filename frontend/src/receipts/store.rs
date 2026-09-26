use super::{ReceiptItem, ReceiptSummary, ReceiptTabData, ReceiptsTab, TabState};
use crate::api::{ApiClient, ApiError};
use crate::csv_flow::{csv_error_message, CsvTabState};
use crate::dto::{
    CsvPreviewResponse, CsvUploadResponse, DividendListResponse, DomesticStockListResponse,
    MutualfundListResponse,
};
use crate::pagination::PageCollector;
use crate::receipts::csv::{to_preview, CsvPreviewRow};
use crate::receipts::filter::ReceiptSearch;
use crate::session::SessionStore;
use leptos::prelude::*;
use serde::de::DeserializeOwned;
use std::collections::{HashMap, HashSet};

pub(crate) const RECEIPT_LIST_PER_PAGE: usize = 1000;
// API の total が実データより大きい等の不整合でも必ず終了するためのページ数上限
pub(crate) const RECEIPT_LIST_MAX_PAGES: usize = 100;

async fn fetch_receipt_page<R: DeserializeOwned>(
    client: &ApiClient,
    path: &str,
    page_no: usize,
) -> Result<R, ApiError> {
    let per_page = RECEIPT_LIST_PER_PAGE.to_string();
    let page = page_no.to_string();
    let query = [
        ("per_page", per_page.as_str()),
        ("page", page.as_str()),
        (
            "include_summary",
            if page_no == 1 { "true" } else { "false" },
        ),
    ];
    client.get_json::<R>(path, &query).await
}

pub(crate) async fn fetch_pages<R, T, S, Fut>(
    fetch_page: impl Fn(usize) -> Fut,
    into_parts: impl Fn(R) -> (Vec<T>, i64, Option<S>),
) -> Result<(Vec<T>, Option<S>, bool), ApiError>
where
    Fut: std::future::Future<Output = Result<R, ApiError>>,
{
    let mut pages = PageCollector::new(RECEIPT_LIST_PER_PAGE, RECEIPT_LIST_MAX_PAGES);
    let mut summary = None;
    loop {
        let page_no = pages.next_page();
        let (data, total, page_summary) = into_parts(fetch_page(page_no).await?);
        if page_no == 1 {
            summary = page_summary;
        }
        if !pages.push(data, total) {
            break;
        }
    }
    let truncated = pages.truncated();
    Ok((pages.into_rows(), summary, truncated))
}

async fn fetch_list(tab: ReceiptsTab) -> Result<ReceiptTabData, ApiError> {
    let client = ApiClient::read_client();
    match tab {
        ReceiptsTab::Dividend => fetch_pages(
            |page_no| fetch_receipt_page(&client, tab.list_path(), page_no),
            |r: DividendListResponse| (r.data, r.total, r.summary),
        )
        .await
        .map(|(rows, summary, truncated)| ReceiptTabData {
            rows: rows.into_iter().map(ReceiptItem::Dividend).collect(),
            summary: summary.map(ReceiptSummary::Dividend),
            truncated,
        }),
        ReceiptsTab::DomesticStock => fetch_pages(
            |page_no| fetch_receipt_page(&client, tab.list_path(), page_no),
            |r: DomesticStockListResponse| (r.data, r.total, r.summary),
        )
        .await
        .map(|(rows, summary, truncated)| ReceiptTabData {
            rows: rows.into_iter().map(ReceiptItem::DomesticStock).collect(),
            summary: summary.map(ReceiptSummary::DomesticStock),
            truncated,
        }),
        ReceiptsTab::MutualFund => fetch_pages(
            |page_no| fetch_receipt_page(&client, tab.list_path(), page_no),
            |r: MutualfundListResponse| (r.data, r.total, r.summary),
        )
        .await
        .map(|(rows, summary, truncated)| ReceiptTabData {
            rows: rows.into_iter().map(ReceiptItem::MutualFund).collect(),
            summary: summary.map(ReceiptSummary::MutualFund),
            truncated,
        }),
    }
}

#[derive(Clone)]
pub struct ReceiptsStore {
    pub(crate) session: SessionStore,
    pub active_tab: RwSignal<ReceiptsTab>,
    pub search: RwSignal<ReceiptSearch>,
    // 開閉状態は検索変更や一覧再描画でビューが作り直されても消えないようストア側に持つ
    pub expanded: RwSignal<HashSet<String>>,
    pub mobile_summary_expanded: RwSignal<bool>,
    // 再マウントなしのアカウント切替で前のユーザーの開閉状態を残さないためのセッション世代
    pub(crate) expanded_epoch: RwSignal<Option<u64>>,
    pub(crate) visited: RwSignal<HashSet<ReceiptsTab>>,
    pub(crate) cache: RwSignal<HashMap<(u64, ReceiptsTab), TabState>>,
    pub(crate) fetch: Action<(u64, ReceiptsTab), ()>,
    // 一覧キャッシュと同じく世代で区切り、ログアウト・ユーザー切替で自動的に無効化する
    pub(crate) csv: RwSignal<HashMap<(u64, ReceiptsTab), CsvTabState<CsvPreviewRow>>>,
    pub(crate) csv_files: RwSignal<HashMap<(u64, ReceiptsTab), web_sys::File>>,
}

impl ReceiptsStore {
    pub fn rows(&self, tab: ReceiptsTab) -> Vec<ReceiptItem> {
        let generation = self.session.generation.get();
        self.cache.with(|map| match map.get(&(generation, tab)) {
            Some(TabState::Ready(data)) => data.rows.clone(),
            _ => Vec::new(),
        })
    }

    pub fn count(&self, tab: ReceiptsTab) -> usize {
        self.rows(tab).len()
    }

    // CSV エラーはタブ内の操作レール側が表示するため、ここでは一覧取得失敗だけを返す
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

    // 選択中タブの失敗を他タブの取得エラーより先に返し、バックグラウンドの失敗で CSV の結果が隠れないようにする
    pub fn rail_error(&self, tab: ReceiptsTab) -> Option<String> {
        if let TabState::Failed(message) = self.tab_state(tab) {
            return Some(message);
        }
        self.csv_state(tab).error.or_else(|| self.error())
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

    pub fn auth_loading(&self) -> bool {
        !self.session.loaded.get()
    }

    pub fn select_tab(&self, tab: ReceiptsTab) {
        if self.active_tab.get_untracked() != tab {
            self.search.set(ReceiptSearch::default());
            self.expanded.update(|set| set.clear());
            self.mobile_summary_expanded.set(false);
        }
        self.visited.update(|visited| {
            visited.insert(tab);
        });
        self.active_tab.set(tab);
        self.ensure(tab);
    }

    pub(crate) fn ensure(&self, tab: ReceiptsTab) {
        if self.session.user.get_untracked().is_none() {
            return;
        }
        let generation = self.session.generation.get_untracked();
        if self.expanded_epoch.get_untracked() != Some(generation) {
            self.expanded_epoch.set(Some(generation));
            self.expanded.update(|set| set.clear());
            self.mobile_summary_expanded.set(false);
        }
        let needs_prune = self
            .cache
            .with_untracked(|map| has_stale_generation(map, generation))
            || self
                .csv
                .with_untracked(|map| has_stale_generation(map, generation))
            || self
                .csv_files
                .with_untracked(|map| has_stale_generation(map, generation));
        if needs_prune {
            self.cache
                .update(|map| prune_stale_generation(map, generation));
            self.csv
                .update(|map| prune_stale_generation(map, generation));
            self.csv_files
                .update(|map| prune_stale_generation(map, generation));
        }
        let already_requested = self.cache.with_untracked(|map| {
            matches!(
                map.get(&(generation, tab)),
                Some(TabState::Loading) | Some(TabState::Ready(_)) | Some(TabState::Failed(_))
            )
        });
        if already_requested {
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

impl ReceiptsStore {
    pub fn csv_state(&self, tab: ReceiptsTab) -> CsvTabState<CsvPreviewRow> {
        let generation = self.session.generation.get();
        self.csv
            .with(|map| map.get(&(generation, tab)).cloned().unwrap_or_default())
    }

    pub fn csv_busy(&self, tab: ReceiptsTab) -> bool {
        self.csv_state(tab).busy()
    }

    pub fn has_csv_preview(&self, tab: ReceiptsTab) -> bool {
        self.csv_state(tab)
            .preview
            .map(|preview| !preview.rows.is_empty())
            .unwrap_or(false)
    }

    pub fn any_tab_fetching(&self) -> bool {
        let generation = self.session.generation.get();
        self.cache.with(|map| {
            ReceiptsTab::ALL
                .iter()
                .any(|tab| matches!(map.get(&(generation, *tab)), Some(TabState::Loading)))
        })
    }

    pub fn select_file(&self, tab: ReceiptsTab, file: web_sys::File) {
        if self.session.user.get_untracked().is_none() {
            return;
        }
        let generation = self.session.generation.get_untracked();
        let mut started = false;
        self.csv.update(|map| {
            started = map
                .entry((generation, tab))
                .or_default()
                .begin_preview(file.name());
        });
        if !started {
            return;
        }
        self.csv_files.update(|map| {
            map.insert((generation, tab), file.clone());
        });
        let store = self.clone();
        leptos::task::spawn_local(async move {
            let result = crate::csv_flow::preview_csv(tab.preview_path(), &file).await;
            store.apply_preview_result(generation, tab, result);
        });
    }

    pub fn save_csv(&self, tab: ReceiptsTab) {
        let Some((generation, file)) = self.try_begin_save(tab) else {
            return;
        };
        let store = self.clone();
        leptos::task::spawn_local(async move {
            let result = crate::csv_flow::upload_csv(tab.import_path(), &file).await;
            if store.apply_upload_result(generation, tab, result) {
                store.fetch.dispatch((generation, tab));
            }
        });
    }

    pub(crate) fn try_begin_save(&self, tab: ReceiptsTab) -> Option<(u64, web_sys::File)> {
        self.session.user.get_untracked()?;
        let generation = self.session.generation.get_untracked();
        let file = self
            .csv_files
            .with_untracked(|map| map.get(&(generation, tab)).cloned())?;
        let mut started = false;
        self.csv.update(|map| {
            started = map.entry((generation, tab)).or_default().begin_save();
        });
        started.then_some((generation, file))
    }

    pub fn open_delete_confirm(&self, tab: ReceiptsTab) {
        if self.session.user.get_untracked().is_none() {
            return;
        }
        let generation = self.session.generation.get_untracked();
        self.csv.update(|map| {
            map.entry((generation, tab))
                .or_default()
                .open_delete_confirm();
        });
    }

    pub fn close_delete_confirm(&self, tab: ReceiptsTab) {
        if self.session.user.get_untracked().is_none() {
            return;
        }
        let generation = self.session.generation.get_untracked();
        self.csv.update(|map| {
            map.entry((generation, tab))
                .or_default()
                .close_delete_confirm();
        });
    }

    pub fn confirm_delete_all(&self, tab: ReceiptsTab) {
        let Some(generation) = self.try_begin_delete(tab) else {
            return;
        };
        let store = self.clone();
        leptos::task::spawn_local(async move {
            let result = crate::csv_flow::delete_all(tab.list_path()).await;
            store.apply_delete_result(generation, tab, result);
        });
    }

    pub(crate) fn try_begin_delete(&self, tab: ReceiptsTab) -> Option<u64> {
        self.session.user.get_untracked()?;
        let generation = self.session.generation.get_untracked();
        let mut started = false;
        self.csv.update(|map| {
            started = map.entry((generation, tab)).or_default().begin_delete();
        });
        started.then_some(generation)
    }

    pub(crate) fn apply_preview_result(
        &self,
        generation: u64,
        tab: ReceiptsTab,
        result: Result<CsvPreviewResponse, ApiError>,
    ) {
        if !self.session.is_current(generation) {
            return;
        }
        self.csv.update(|map| {
            map.entry((generation, tab))
                .or_default()
                .finish_preview(result.ok().map(|response| to_preview(tab, response)));
        });
    }

    // 一覧の再取得が必要になったら true を返す。fetch の起動(dispatch)は呼び出し側が行う
    pub(crate) fn apply_upload_result(
        &self,
        generation: u64,
        tab: ReceiptsTab,
        result: Result<CsvUploadResponse, ApiError>,
    ) -> bool {
        if !self.session.is_current(generation) {
            return false;
        }
        match result {
            Ok(response) => {
                self.csv.update(|map| {
                    map.entry((generation, tab))
                        .or_default()
                        .finish_save(Ok(response));
                });
                self.csv_files.update(|map| {
                    map.remove(&(generation, tab));
                });
                self.refresh_tab_list(generation, tab)
            }
            Err(error) => {
                let message = csv_error_message(&error);
                self.csv.update(|map| {
                    map.entry((generation, tab))
                        .or_default()
                        .finish_save(Err(message));
                });
                false
            }
        }
    }

    pub(crate) fn apply_delete_result(
        &self,
        generation: u64,
        tab: ReceiptsTab,
        result: Result<(), ApiError>,
    ) {
        if !self.session.is_current(generation) {
            return;
        }
        match result {
            Ok(()) => {
                self.csv.update(|map| {
                    map.entry((generation, tab))
                        .or_default()
                        .finish_delete(Ok(()));
                });
                // 未取得タブに空の Ready を作ると以後の再取得が抑止されるため、キャッシュ済みの時だけ上書き
                self.cache.update(|map| {
                    if let Some(entry) = map.get_mut(&(generation, tab)) {
                        *entry = TabState::Ready(ReceiptTabData {
                            rows: Vec::new(),
                            summary: None,
                            truncated: false,
                        });
                    }
                });
            }
            Err(error) => {
                let message = csv_error_message(&error);
                self.csv.update(|map| {
                    map.entry((generation, tab))
                        .or_default()
                        .finish_delete(Err(message));
                });
            }
        }
    }

    // 再取得が必要なら true を返す。fetch の起動(dispatch)は呼び出し側が行う
    pub(crate) fn refresh_tab_list(&self, generation: u64, tab: ReceiptsTab) -> bool {
        let mut refresh = false;
        self.cache.update(|map| {
            refresh = mark_tab_for_refresh(map, generation, tab);
        });
        refresh
    }
}

pub(crate) fn has_stale_generation<V>(
    map: &HashMap<(u64, ReceiptsTab), V>,
    generation: u64,
) -> bool {
    map.keys().any(|(cached, _)| *cached != generation)
}

pub(crate) fn prune_stale_generation<V>(map: &mut HashMap<(u64, ReceiptsTab), V>, generation: u64) {
    map.retain(|key, _| key.0 == generation);
}

pub(crate) fn tab_settled(store: &ReceiptsStore, generation: u64, tab: ReceiptsTab) -> bool {
    store.cache.with(|map| {
        matches!(
            map.get(&(generation, tab)),
            Some(TabState::Ready(_)) | Some(TabState::Failed(_))
        )
    })
}

pub(crate) fn should_apply_fetch_result(session: &SessionStore, generation: u64) -> bool {
    session.is_current(generation)
}

pub(crate) fn mark_tab_for_refresh(
    map: &mut HashMap<(u64, ReceiptsTab), TabState>,
    generation: u64,
    tab: ReceiptsTab,
) -> bool {
    if !map.contains_key(&(generation, tab)) {
        return false;
    }
    map.insert((generation, tab), TabState::Loading);
    true
}

pub fn use_receipts_data(session: SessionStore, initial_tab: ReceiptsTab) -> ReceiptsStore {
    let active_tab = RwSignal::new(initial_tab);
    let visited = RwSignal::new(HashSet::from([initial_tab]));
    let cache = RwSignal::new(HashMap::new());
    let csv = RwSignal::new(HashMap::new());
    let csv_files = RwSignal::new(HashMap::new());

    let fetch_session = session;
    let cache_signal = cache;
    let fetch = Action::new_unsync(move |(generation, tab): &(u64, ReceiptsTab)| {
        let (generation, tab) = (*generation, *tab);
        let session = fetch_session;
        async move {
            let rows = fetch_list(tab).await;
            if !should_apply_fetch_result(&session, generation) {
                return;
            }
            cache_signal.update(|map| {
                map.insert(
                    (generation, tab),
                    match rows {
                        Ok(rows) => TabState::Ready(rows),
                        Err(error) => TabState::Failed(error.message()),
                    },
                );
            });
        }
    });

    let store = ReceiptsStore {
        session,
        active_tab,
        search: RwSignal::new(ReceiptSearch::default()),
        expanded: RwSignal::new(HashSet::new()),
        mobile_summary_expanded: RwSignal::new(false),
        expanded_epoch: RwSignal::new(None),
        visited,
        cache,
        fetch,
        csv,
        csv_files,
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
