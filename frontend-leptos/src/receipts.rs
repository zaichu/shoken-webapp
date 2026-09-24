use crate::api::{ApiClient, ApiError};
use crate::dto::{
    CsvPreviewResponse, CsvUploadResponse, DividendListResponse, DividendSummary,
    DomesticStockListResponse, DomesticStockSummary, MutualfundListResponse, MutualfundSummary,
};
use crate::receipts_csv::{csv_error_message, to_preview, CsvTabState};
use crate::receipts_domain::{format_currency, format_date, format_number};
use crate::receipts_filter::ReceiptSearch;
use crate::receipts_pagination::PageCollector;
use crate::session::SessionStore;
use leptos::prelude::*;
use serde::de::DeserializeOwned;
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

    pub(crate) fn list_path(&self) -> &'static str {
        match self {
            ReceiptsTab::Dividend => "/api/v1/dividends",
            ReceiptsTab::DomesticStock => "/api/v1/domestic-stock-transactions",
            ReceiptsTab::MutualFund => "/api/v1/mutual-fund-transactions",
        }
    }

    pub(crate) fn preview_path(&self) -> &'static str {
        match self {
            ReceiptsTab::Dividend => "/api/v1/dividend-import-validations",
            ReceiptsTab::DomesticStock => "/api/v1/domestic-stock-import-validations",
            ReceiptsTab::MutualFund => "/api/v1/mutual-fund-import-validations",
        }
    }

    pub(crate) fn import_path(&self) -> &'static str {
        match self {
            ReceiptsTab::Dividend => "/api/v1/dividend-imports",
            ReceiptsTab::DomesticStock => "/api/v1/domestic-stock-imports",
            ReceiptsTab::MutualFund => "/api/v1/mutual-fund-imports",
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
                format_date(&row.settlement_date),
                row.product.clone(),
                row.account.clone(),
                row.security_code.clone(),
                row.security_name.clone(),
                format_currency(row.unit_price),
                format_number(row.shares, 2),
                format_currency(row.dividends_before_tax),
                format_currency(row.taxes),
                format_currency(row.net_amount_received),
            ],
            ReceiptItem::DomesticStock(row) => vec![
                format_date(&row.trade_date),
                row.security_code.clone(),
                row.security_name.clone(),
                row.account.clone(),
                format_number(row.shares, 2),
                format_currency(row.asked_price),
                format_currency(row.proceeds),
                format_currency(row.purchase_price),
                format_currency(row.realized_profit_and_loss),
                format_currency(row.taxes),
                format_currency(row.realized_profit_and_loss_after_tax),
            ],
            ReceiptItem::MutualFund(row) => vec![
                format_date(&row.trade_date),
                row.fund_name.clone(),
                row.account.clone(),
                format_number(row.shares, 2),
                format_currency(row.cancellation_unit_price_yen),
                format_currency(row.cancellation_amount_yen),
                format_currency(row.average_acquisition_price_yen),
                format_currency(row.realized_profit_and_loss),
                format_currency(row.taxes),
                format_currency(row.realized_profit_and_loss_after_tax),
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReceiptSummary {
    Dividend(DividendSummary),
    DomesticStock(DomesticStockSummary),
    MutualFund(MutualfundSummary),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReceiptTabData {
    pub rows: Vec<ReceiptItem>,
    pub summary: Option<ReceiptSummary>,
}

pub fn select_header_summary<T: Clone>(
    api_summary: Option<&T>,
    has_preview: bool,
    search_query: &str,
    client_summary: T,
) -> T {
    if let Some(summary) = api_summary.filter(|_| !has_preview && search_query.is_empty()) {
        summary.clone()
    } else {
        client_summary
    }
}

const RECEIPT_LIST_PER_PAGE: usize = 1000;
// API の total が実データより大きい等の不整合でも必ず終了するためのページ数上限
const RECEIPT_LIST_MAX_PAGES: usize = 100;

async fn fetch_pages<R, T, S>(
    client: &ApiClient,
    path: &str,
    into_parts: impl Fn(R) -> (Vec<T>, i64, Option<S>),
) -> Result<(Vec<T>, Option<S>), ApiError>
where
    R: DeserializeOwned,
{
    let mut pages = PageCollector::new(RECEIPT_LIST_PER_PAGE, RECEIPT_LIST_MAX_PAGES);
    let per_page = RECEIPT_LIST_PER_PAGE.to_string();
    let mut summary = None;
    loop {
        let page_no = pages.next_page();
        let page = page_no.to_string();
        let query = [
            ("per_page", per_page.as_str()),
            ("page", page.as_str()),
            (
                "include_summary",
                if page_no == 1 { "true" } else { "false" },
            ),
        ];
        let (data, total, page_summary) = into_parts(client.get_json::<R>(path, &query).await?);
        if page_no == 1 {
            summary = page_summary;
        }
        if !pages.push(data, total) {
            break;
        }
    }
    Ok((pages.into_rows(), summary))
}

async fn fetch_list(tab: ReceiptsTab) -> Result<ReceiptTabData, ApiError> {
    let client = ApiClient::read_client();
    match tab {
        ReceiptsTab::Dividend => {
            fetch_pages(&client, tab.list_path(), |r: DividendListResponse| {
                (r.data, r.total, r.summary)
            })
            .await
            .map(|(rows, summary)| ReceiptTabData {
                rows: rows.into_iter().map(ReceiptItem::Dividend).collect(),
                summary: summary.map(ReceiptSummary::Dividend),
            })
        }
        ReceiptsTab::DomesticStock => {
            fetch_pages(&client, tab.list_path(), |r: DomesticStockListResponse| {
                (r.data, r.total, r.summary)
            })
            .await
            .map(|(rows, summary)| ReceiptTabData {
                rows: rows.into_iter().map(ReceiptItem::DomesticStock).collect(),
                summary: summary.map(ReceiptSummary::DomesticStock),
            })
        }
        ReceiptsTab::MutualFund => {
            fetch_pages(&client, tab.list_path(), |r: MutualfundListResponse| {
                (r.data, r.total, r.summary)
            })
            .await
            .map(|(rows, summary)| ReceiptTabData {
                rows: rows.into_iter().map(ReceiptItem::MutualFund).collect(),
                summary: summary.map(ReceiptSummary::MutualFund),
            })
        }
    }
}

fn fetch_error_message(error: &ApiError) -> String {
    if error.is_unauthorized() {
        error.user_message()
    } else {
        "データ取得に失敗しました".to_string()
    }
}

#[derive(Clone, Debug)]
pub enum TabState {
    Loading,
    Ready(ReceiptTabData),
    Failed(String),
}
#[derive(Clone)]
pub struct ReceiptsStore {
    session: SessionStore,
    pub active_tab: RwSignal<ReceiptsTab>,
    pub search: RwSignal<ReceiptSearch>,
    visited: RwSignal<HashSet<ReceiptsTab>>,
    cache: RwSignal<HashMap<(u64, ReceiptsTab), TabState>>,
    fetch: Action<(u64, ReceiptsTab), ()>,
    // 一覧キャッシュと同じく世代で区切り、ログアウト・ユーザー切替で自動的に無効化する
    csv: RwSignal<HashMap<(u64, ReceiptsTab), CsvTabState>>,
    csv_files: RwSignal<HashMap<(u64, ReceiptsTab), web_sys::File>>,
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
        }
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
            .with_untracked(|map| map.keys().any(|(cached, _)| *cached != generation))
            || self
                .csv
                .with_untracked(|map| map.keys().any(|(cached, _)| *cached != generation))
            || self
                .csv_files
                .with_untracked(|map| map.keys().any(|(cached, _)| *cached != generation));
        if needs_prune {
            self.cache.update(|map| {
                map.retain(|key, _| key.0 == generation);
            });
            self.csv.update(|map| {
                map.retain(|key, _| key.0 == generation);
            });
            self.csv_files.update(|map| {
                map.retain(|key, _| key.0 == generation);
            });
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
    pub fn csv_state(&self, tab: ReceiptsTab) -> CsvTabState {
        let generation = self.session.generation.get();
        self.csv
            .with(|map| map.get(&(generation, tab)).cloned().unwrap_or_default())
    }

    pub fn csv_busy(&self, tab: ReceiptsTab) -> bool {
        self.csv_state(tab).busy()
    }

    // React の previewData.length > 0 相当。プレビュー行があれば一覧の表示を置き換える
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
            let result = crate::receipts_csv::preview_csv(tab, &file).await;
            store.apply_preview_result(generation, tab, result);
        });
    }

    pub fn save_csv(&self, tab: ReceiptsTab) {
        let Some((generation, file)) = self.try_begin_save(tab) else {
            return;
        };
        let store = self.clone();
        leptos::task::spawn_local(async move {
            let result = crate::receipts_csv::upload_csv(tab, &file).await;
            if store.apply_upload_result(generation, tab, result) {
                store.fetch.dispatch((generation, tab));
            }
        });
    }

    fn try_begin_save(&self, tab: ReceiptsTab) -> Option<(u64, web_sys::File)> {
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
            let result = crate::receipts_csv::delete_all(tab).await;
            store.apply_delete_result(generation, tab, result);
        });
    }

    fn try_begin_delete(&self, tab: ReceiptsTab) -> Option<u64> {
        self.session.user.get_untracked()?;
        let generation = self.session.generation.get_untracked();
        let mut started = false;
        self.csv.update(|map| {
            started = map.entry((generation, tab)).or_default().begin_delete();
        });
        started.then_some(generation)
    }

    fn apply_preview_result(
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
    fn apply_upload_result(
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

    fn apply_delete_result(&self, generation: u64, tab: ReceiptsTab, result: Result<(), ApiError>) {
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
                // React はキャッシュが残っている場合だけ空データで上書きし、無ければ再生成しない
                self.cache.update(|map| {
                    if let Some(entry) = map.get_mut(&(generation, tab)) {
                        *entry = TabState::Ready(ReceiptTabData {
                            rows: Vec::new(),
                            summary: None,
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

    // 取込成功後の一覧無効化。再取得が必要なら true を返す。fetch の起動(dispatch)は呼び出し側が行う
    // React の invalidateQueries と同じく、再取得してもタブを訪問済みにはしない
    fn refresh_tab_list(&self, generation: u64, tab: ReceiptsTab) -> bool {
        let mut refresh = false;
        self.cache.update(|map| {
            refresh = mark_tab_for_refresh(map, generation, tab);
        });
        refresh
    }

    // 外部からの強制再取得。取込・削除の成功時は内部で refresh_tab_list されるため未使用
    #[allow(dead_code)]
    pub fn invalidate_tab(&self, tab: ReceiptsTab) {
        if self.session.user.get_untracked().is_none() {
            return;
        }
        let generation = self.session.generation.get_untracked();
        if self.refresh_tab_list(generation, tab) {
            self.fetch.dispatch((generation, tab));
        }
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

fn mark_tab_for_refresh(
    map: &mut HashMap<(u64, ReceiptsTab), TabState>,
    generation: u64,
    tab: ReceiptsTab,
) -> bool {
    // React の invalidateQueries 相当: キャッシュが残っているタブだけ再取得する
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
            cache_signal.update(|map| {
                map.insert(
                    (generation, tab),
                    match rows {
                        Ok(rows) => TabState::Ready(rows),
                        Err(error) => TabState::Failed(fetch_error_message(&error)),
                    },
                );
            });
        }
    });

    let store = ReceiptsStore {
        session: session.clone(),
        active_tab,
        search: RwSignal::new(ReceiptSearch::default()),
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
    fn unauthorized_receipts_error_uses_react_message() {
        assert_eq!(
            fetch_error_message(&ApiError::Http { status: 401 }),
            "認証が必要です"
        );
    }

    #[test]
    fn header_summary_uses_api_value_without_preview_or_search() {
        assert_eq!(
            select_header_summary(Some(&"api"), false, "", "client"),
            "api"
        );
    }

    #[test]
    fn header_summary_uses_client_value_during_search() {
        assert_eq!(
            select_header_summary(Some(&"api"), false, "7203", "client"),
            "client"
        );
    }

    #[test]
    fn header_summary_uses_client_value_during_preview_or_without_api_summary() {
        assert_eq!(
            select_header_summary(Some(&"api"), true, "", "client"),
            "client"
        );
        assert_eq!(select_header_summary(None, false, "", "client"), "client");
    }

    #[test]
    fn failed_tabs_are_not_fetched_again_in_the_same_generation() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let fetch = Action::new_unsync(|_: &(u64, ReceiptsTab)| async {});

            for tab in ReceiptsTab::ALL {
                let cache = RwSignal::new(HashMap::from([(
                    (generation, tab),
                    TabState::Failed("データ取得に失敗しました".to_string()),
                )]));
                let store = ReceiptsStore {
                    session: session.clone(),
                    active_tab: RwSignal::new(tab),
                    search: RwSignal::new(ReceiptSearch::default()),
                    visited: RwSignal::new(HashSet::from([tab])),
                    cache,
                    fetch,
                    csv: RwSignal::new(HashMap::new()),
                    csv_files: RwSignal::new(HashMap::new()),
                };

                let ensure_result =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| store.ensure(tab)));

                assert!(ensure_result.is_ok(), "失敗済みタブを再取得しようとした");
                assert!(matches!(
                    cache.with_untracked(|map| map.get(&(generation, tab)).cloned()),
                    Some(TabState::Failed(_))
                ));
            }
        });
    }

    #[test]
    fn dividend_cells_match_react_columns_and_formatting() {
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
                "2024/03/01",
                "特定口座",
                "SBI証券",
                "7203",
                "トヨタ自動車",
                "¥ 30",
                "100",
                "¥ 3,000",
                "¥ 609",
                "¥ 2,391",
            ]
        );
    }
}

#[cfg(test)]
mod csv_tests {
    use super::*;

    fn user(id: &str) -> crate::dto::SessionUser {
        crate::dto::SessionUser {
            id: id.to_string(),
            email: format!("{id}@example.com"),
            name: None,
            picture_url: None,
        }
    }

    fn test_store(
        session: &SessionStore,
        cache: HashMap<(u64, ReceiptsTab), TabState>,
        csv: HashMap<(u64, ReceiptsTab), CsvTabState>,
    ) -> ReceiptsStore {
        ReceiptsStore {
            session: session.clone(),
            active_tab: RwSignal::new(ReceiptsTab::Dividend),
            search: RwSignal::new(ReceiptSearch::default()),
            visited: RwSignal::new(HashSet::new()),
            cache: RwSignal::new(cache),
            fetch: Action::new_unsync(|_: &(u64, ReceiptsTab)| async {}),
            csv: RwSignal::new(csv),
            csv_files: RwSignal::new(HashMap::new()),
        }
    }

    fn upload_response(inserted: usize) -> CsvUploadResponse {
        CsvUploadResponse {
            inserted,
            skipped: 0,
            errors: vec![],
        }
    }

    #[test]
    fn csv_state_is_scoped_to_generation_and_cleared_by_logout() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let store = test_store(
                &session,
                HashMap::new(),
                HashMap::from([(
                    (generation, ReceiptsTab::Dividend),
                    CsvTabState {
                        file_name: Some("dividend.csv".to_string()),
                        import_result: Some(upload_response(3)),
                        show_delete_confirm: true,
                        ..Default::default()
                    },
                )]),
            );

            assert_eq!(
                store.csv_state(ReceiptsTab::Dividend).file_name.as_deref(),
                Some("dividend.csv")
            );

            session.mark_unauthenticated();
            assert_eq!(
                store.csv_state(ReceiptsTab::Dividend),
                CsvTabState::default(),
                "世代が進むとCSV状態は見えなくなる"
            );

            session.user.set(Some(user("alice")));
            assert_eq!(
                store.csv_state(ReceiptsTab::Dividend),
                CsvTabState::default()
            );
        });
    }

    #[test]
    fn upload_success_sets_result_and_refreshes_cached_list() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let tab = ReceiptsTab::Dividend;
            let store = test_store(
                &session,
                HashMap::from([(
                    (generation, tab),
                    TabState::Ready(ReceiptTabData {
                        rows: Vec::new(),
                        summary: None,
                    }),
                )]),
                HashMap::from([(
                    (generation, tab),
                    CsvTabState {
                        file_name: Some("a.csv".to_string()),
                        saving: true,
                        ..Default::default()
                    },
                )]),
            );

            assert!(
                store.apply_upload_result(generation, tab, Ok(upload_response(2))),
                "キャッシュ済みタブは再取得対象になる"
            );

            let state = store.csv_state(tab);
            assert!(!state.saving);
            assert!(state.file_name.is_none());
            assert_eq!(
                state.import_result.as_ref().map(|result| result.inserted),
                Some(2)
            );
            assert!(matches!(store.tab_state(tab), TabState::Loading,));
        });
    }

    #[test]
    fn upload_success_does_not_fetch_uncached_tab() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let tab = ReceiptsTab::MutualFund;
            let store = test_store(&session, HashMap::new(), HashMap::new());

            // React の invalidateQueries 相当: キャッシュが無いタブは再取得しない
            assert!(!store.apply_upload_result(generation, tab, Ok(upload_response(1))));

            assert!(store
                .cache
                .with_untracked(|map| !map.contains_key(&(generation, tab))));
        });
    }

    #[test]
    fn upload_error_sets_react_message_and_keeps_file() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let tab = ReceiptsTab::DomesticStock;
            let store = test_store(
                &session,
                HashMap::new(),
                HashMap::from([(
                    (generation, tab),
                    CsvTabState {
                        file_name: Some("stocks.csv".to_string()),
                        saving: true,
                        ..Default::default()
                    },
                )]),
            );

            store.apply_upload_result(generation, tab, Err(ApiError::Http { status: 401 }));

            let state = store.csv_state(tab);
            assert!(!state.saving);
            assert_eq!(state.file_name.as_deref(), Some("stocks.csv"));
            assert_eq!(state.error.as_deref(), Some("認証が必要です"));
            // ページ上部の error() は一覧取得失敗専用。CSV エラーは操作レール側が出す
            assert!(store.error().is_none());
        });
    }

    #[test]
    fn preview_result_applies_only_in_current_generation() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let tab = ReceiptsTab::Dividend;
            let store = test_store(
                &session,
                HashMap::new(),
                HashMap::from([(
                    (generation, tab),
                    CsvTabState {
                        file_name: Some("a.csv".to_string()),
                        previewing: true,
                        ..Default::default()
                    },
                )]),
            );

            let response = CsvPreviewResponse {
                total_rows: 2,
                valid_rows: 2,
                errors: vec![],
                rows: vec![
                    serde_json::json!({"security_name": "トヨタ自動車", "shares": 100}),
                    serde_json::json!({"security_name": "三菱UFJ", "shares": 200}),
                ],
            };
            store.apply_preview_result(generation, tab, Ok(response));

            let state = store.csv_state(tab);
            assert!(!state.previewing);
            let preview = state.preview.expect("preview set");
            assert_eq!(preview.rows.len(), 2);

            // プレビュー失敗は React と同様に通知せず解析中だけ解除する
            store.csv.update(|map| {
                map.entry((generation, tab)).or_default().previewing = true;
            });
            store.apply_preview_result(generation, tab, Err(ApiError::Http { status: 400 }));
            let state = store.csv_state(tab);
            assert!(!state.previewing);
            assert!(state.error.is_none());
        });
    }

    #[test]
    fn delete_success_empties_cached_list_and_clears_result() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let tab = ReceiptsTab::Dividend;
            let store = test_store(
                &session,
                HashMap::from([(
                    (generation, tab),
                    TabState::Ready(ReceiptTabData {
                        rows: vec![ReceiptItem::Dividend(
                            serde_json::from_value(serde_json::json!({
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
                            .expect("dividend"),
                        )],
                        summary: None,
                    }),
                )]),
                HashMap::from([(
                    (generation, tab),
                    CsvTabState {
                        import_result: Some(upload_response(1)),
                        deleting: true,
                        ..Default::default()
                    },
                )]),
            );

            store.apply_delete_result(generation, tab, Ok(()));

            let state = store.csv_state(tab);
            assert!(!state.deleting);
            assert!(state.import_result.is_none());
            match store.tab_state(tab) {
                TabState::Ready(data) => assert!(data.rows.is_empty()),
                _ => panic!("キャッシュは空の Ready に置き換わる"),
            }
        });
    }

    #[test]
    fn delete_success_without_cached_list_creates_nothing() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let tab = ReceiptsTab::MutualFund;
            let store = test_store(&session, HashMap::new(), HashMap::new());

            store.apply_delete_result(generation, tab, Ok(()));

            assert!(store
                .cache
                .with_untracked(|map| !map.contains_key(&(generation, tab))));
        });
    }

    #[test]
    fn stale_generation_results_are_dropped() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let tab = ReceiptsTab::Dividend;
            let store = test_store(
                &session,
                HashMap::from([((generation, tab), TabState::Loading)]),
                HashMap::from([(
                    (generation, tab),
                    CsvTabState {
                        saving: true,
                        deleting: true,
                        previewing: true,
                        ..Default::default()
                    },
                )]),
            );

            session.mark_unauthenticated();

            store.apply_preview_result(
                generation,
                tab,
                Ok(CsvPreviewResponse {
                    total_rows: 1,
                    valid_rows: 1,
                    errors: vec![],
                    rows: vec![serde_json::json!({})],
                }),
            );
            assert!(!store.apply_upload_result(generation, tab, Ok(upload_response(1))));
            store.apply_delete_result(generation, tab, Ok(()));

            // 旧世代のエントリは一切変化しない(新世代からも見えない)
            let stale = store
                .csv
                .with_untracked(|map| map.get(&(generation, tab)).cloned())
                .unwrap_or_default();
            assert!(stale.previewing && stale.saving && stale.deleting);
            assert!(stale.preview.is_none() && stale.import_result.is_none());
            assert!(matches!(
                store
                    .cache
                    .with_untracked(|map| map.get(&(generation, tab)).cloned()),
                Some(TabState::Loading)
            ));
        });
    }

    #[test]
    fn delete_confirm_opens_closes_and_guards_double_confirm() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let tab = ReceiptsTab::Dividend;
            let store = test_store(&session, HashMap::new(), HashMap::new());

            store.open_delete_confirm(tab);
            assert!(store.csv_state(tab).show_delete_confirm);
            store.close_delete_confirm(tab);
            assert!(!store.csv_state(tab).show_delete_confirm);

            store.open_delete_confirm(tab);
            assert!(store.try_begin_delete(tab).is_some());
            let state = store.csv_state(tab);
            assert!(!state.show_delete_confirm);
            assert!(state.deleting);
            assert!(
                store.try_begin_delete(tab).is_none(),
                "deleting 中の確定は開始しない"
            );
        });
    }

    #[test]
    fn csv_operations_require_authentication() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            let tab = ReceiptsTab::Dividend;
            let store = test_store(&session, HashMap::new(), HashMap::new());

            assert!(store.try_begin_save(tab).is_none());
            assert!(store.try_begin_delete(tab).is_none());
            store.open_delete_confirm(tab);
            assert!(!store.csv_state(tab).show_delete_confirm);
            store.invalidate_tab(tab);
        });
    }

    #[test]
    fn save_csv_requires_selected_file_and_idle_state() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let tab = ReceiptsTab::Dividend;
            let store = test_store(&session, HashMap::new(), HashMap::new());

            assert!(store.try_begin_save(tab).is_none(), "ファイル未選択");
            assert!(!store.csv_state(tab).saving);

            // プレビュー中は保存を開始しない(ファイルの有無に関わらず)
            store.csv.update(|map| {
                map.entry((generation, tab)).or_default().previewing = true;
            });
            assert!(store.try_begin_save(tab).is_none());
        });
    }

    #[test]
    fn error_returns_list_fetch_errors_only() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let tab = ReceiptsTab::Dividend;
            let csv_error = HashMap::from([(
                (generation, tab),
                CsvTabState {
                    error: Some("リクエストが不正です".to_string()),
                    ..Default::default()
                },
            )]);
            let store = test_store(
                &session,
                HashMap::from([(
                    (generation, ReceiptsTab::DomesticStock),
                    TabState::Failed("データ取得に失敗しました".to_string()),
                )]),
                csv_error,
            );

            assert_eq!(store.error().as_deref(), Some("データ取得に失敗しました"));

            let store = test_store(
                &session,
                HashMap::new(),
                HashMap::from([(
                    (generation, tab),
                    CsvTabState {
                        error: Some("リクエストが不正です".to_string()),
                        ..Default::default()
                    },
                )]),
            );
            assert!(store.error().is_none());
        });
    }

    #[test]
    fn refresh_tab_list_marks_cached_tab_loading_only() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let store = test_store(
                &session,
                HashMap::from([(
                    (generation, ReceiptsTab::Dividend),
                    TabState::Ready(ReceiptTabData {
                        rows: Vec::new(),
                        summary: None,
                    }),
                )]),
                HashMap::new(),
            );

            assert!(store.refresh_tab_list(generation, ReceiptsTab::Dividend));
            assert!(matches!(
                store.tab_state(ReceiptsTab::Dividend),
                TabState::Loading
            ));
            assert!(!store.refresh_tab_list(generation, ReceiptsTab::MutualFund));
            assert!(store
                .cache
                .with_untracked(|map| !map.contains_key(&(generation, ReceiptsTab::MutualFund))));
        });
    }

    #[test]
    fn mark_tab_for_refresh_marks_only_existing_entry() {
        let mut map = HashMap::new();
        assert!(!mark_tab_for_refresh(&mut map, 0, ReceiptsTab::Dividend));

        map.insert(
            (0, ReceiptsTab::Dividend),
            TabState::Ready(ReceiptTabData {
                rows: Vec::new(),
                summary: None,
            }),
        );
        assert!(mark_tab_for_refresh(&mut map, 0, ReceiptsTab::Dividend));
        assert!(matches!(
            map.get(&(0, ReceiptsTab::Dividend)),
            Some(TabState::Loading)
        ));
        // 他世代・未取得タブは対象外
        assert!(!mark_tab_for_refresh(&mut map, 1, ReceiptsTab::Dividend));
        assert!(!mark_tab_for_refresh(&mut map, 0, ReceiptsTab::MutualFund));
    }

    #[test]
    fn confirm_delete_all_requires_open_confirmation() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let tab = ReceiptsTab::Dividend;
            let store = test_store(&session, HashMap::new(), HashMap::new());

            // 確認を開いていなければ開始しない(dispatch も発生しない)
            store.confirm_delete_all(tab);
            let state = store.csv_state(tab);
            assert!(!state.deleting);
            assert!(state.error.is_none());
            assert!(store.try_begin_delete(tab).is_none());

            store.open_delete_confirm(tab);
            assert!(store.try_begin_delete(tab).is_some());
        });
    }
}

#[cfg(test)]
mod search_tests {
    use super::*;
    use crate::receipts_filter::filter_receipts;
    use crate::receipts_filter::tests::dividends;

    #[test]
    fn tab_switch_resets_search_but_same_tab_and_cache_keep_it() {
        let owner = Owner::new();
        owner.with(|| {
            let store = ReceiptsStore {
                session: SessionStore::new(),
                active_tab: RwSignal::new(ReceiptsTab::Dividend),
                search: RwSignal::new(ReceiptSearch {
                    query: "9432".into(),
                    ..Default::default()
                }),
                visited: RwSignal::new(HashSet::new()),
                cache: RwSignal::new(HashMap::from([(
                    (0, ReceiptsTab::Dividend),
                    TabState::Ready(ReceiptTabData {
                        rows: dividends(),
                        summary: None,
                    }),
                )])),
                fetch: Action::new_unsync(|_: &(u64, ReceiptsTab)| async {}),
                csv: RwSignal::new(HashMap::new()),
                csv_files: RwSignal::new(HashMap::new()),
            };
            assert_eq!(
                leptos::prelude::untrack(|| {
                    filter_receipts(
                        ReceiptsTab::Dividend,
                        &store.rows(ReceiptsTab::Dividend),
                        &store.search.get().query,
                    )
                })
                .len(),
                2
            );
            assert_eq!(
                leptos::prelude::untrack(|| store.count(ReceiptsTab::Dividend)),
                3
            );
            store.select_tab(ReceiptsTab::Dividend);
            assert_eq!(store.search.get_untracked().query, "9432");
            store.select_tab(ReceiptsTab::DomesticStock);
            assert_eq!(store.search.get_untracked(), ReceiptSearch::default());
            store.select_tab(ReceiptsTab::Dividend);
            assert_eq!(
                leptos::prelude::untrack(|| {
                    filter_receipts(
                        ReceiptsTab::Dividend,
                        &store.rows(ReceiptsTab::Dividend),
                        &store.search.get().query,
                    )
                })
                .len(),
                3
            );
        });
    }
}
