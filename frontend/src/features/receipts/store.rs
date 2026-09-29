use super::{ReceiptRow, ReceiptTabData, ReceiptsTab, TabState};
use crate::api::dto::{CsvPreviewResponse, CsvUploadResponse};
use crate::api::ApiError;
use crate::features::receipts::csv::{to_preview, CsvPreviewRow};
use crate::features::receipts::filter::ReceiptSearch;
use crate::session::{Generation, SessionStore};
use crate::support::csv_flow::{csv_error_message, CsvTabState};
use leptos::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Clone, Copy)]
pub struct ReceiptsStore {
    pub(crate) session: SessionStore,
    pub active_tab: RwSignal<ReceiptsTab>,
    pub search: RwSignal<ReceiptSearch>,
    // 開閉状態は検索変更や一覧再描画でビューが作り直されても消えないようストア側に持つ
    pub expanded: RwSignal<HashSet<String>>,
    pub mobile_summary_expanded: RwSignal<bool>,
    // Cookie ポリシーへの記載を避けるため localStorage には保存せず、
    // セッション内だけの画面状態として持つ。初期値はデータ有無で一度だけ決める
    pub utility_rail_open: RwSignal<bool>,
    pub(crate) utility_rail_decided: RwSignal<bool>,
    pub(crate) utility_rail_initialized: RwSignal<HashSet<ReceiptsTab>>,
    // 再マウントなしのアカウント切替で前のユーザーの開閉状態を残さないためのセッション世代
    pub(crate) expanded_epoch: RwSignal<Option<Generation>>,
    pub(crate) visited: RwSignal<HashSet<ReceiptsTab>>,
    pub(crate) cache: RwSignal<HashMap<(Generation, ReceiptsTab), TabState>>,
    pub(crate) fetch: Action<(Generation, ReceiptsTab), ()>,
    // 一覧キャッシュと同じく世代で区切り、ログアウト・ユーザー切替で自動的に無効化する
    pub(crate) csv: RwSignal<HashMap<(Generation, ReceiptsTab), CsvTabState<CsvPreviewRow>>>,
    pub(crate) csv_files: RwSignal<HashMap<(Generation, ReceiptsTab), web_sys::File>>,
}

impl ReceiptsStore {
    pub fn rows(&self, tab: ReceiptsTab) -> Vec<ReceiptRow> {
        let generation = self.session.generation.get();
        self.cache.with(|map| match map.get(&(generation, tab)) {
            Some(TabState::Ready(data)) => data.rows.clone(),
            _ => Vec::new(),
        })
    }

    pub fn count(&self, tab: ReceiptsTab) -> usize {
        self.rows(tab).len()
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

    // ユーザー操作を優先するため、トグル済みなら以後のデータ到着で初期値を上書きしない
    pub fn toggle_utility_rail(&self) {
        self.utility_rail_decided.set(true);
        self.utility_rail_open.update(|open| *open = !*open);
    }

    /// 初期状態(データ→畳む、0件→開く)はタブごとの最初の Ready で一度だけ決める。
    /// 再取得で workspace が作り直されても状態を巻き戻さないため、
    /// 初期化済みのタブとユーザーがトグル済みの場合は何もしない
    pub(crate) fn init_utility_rail(&self, tab: ReceiptsTab, has_rows: bool) {
        if self.utility_rail_decided.get_untracked() {
            return;
        }
        if self
            .utility_rail_initialized
            .with(|done| done.contains(&tab))
        {
            return;
        }
        self.utility_rail_initialized.update(|done| {
            done.insert(tab);
        });
        self.utility_rail_open.set(!has_rows);
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
            self.utility_rail_initialized.update(|set| set.clear());
            self.utility_rail_decided.set(false);
            self.utility_rail_open.set(true);
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
            .is_some_and(|preview| !preview.rows.is_empty())
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
        let store = *self;
        leptos::task::spawn_local(async move {
            let result = crate::support::csv_flow::preview_csv(tab.preview_path(), &file).await;
            store.apply_preview_result(generation, tab, result);
        });
    }

    pub fn save_csv(&self, tab: ReceiptsTab) {
        let Some((generation, file)) = self.try_begin_save(tab) else {
            return;
        };
        let store = *self;
        leptos::task::spawn_local(async move {
            let result = crate::support::csv_flow::upload_csv(tab.import_path(), &file).await;
            if store.apply_upload_result(generation, tab, result) {
                store.fetch.dispatch((generation, tab));
            }
        });
    }

    pub(crate) fn try_begin_save(&self, tab: ReceiptsTab) -> Option<(Generation, web_sys::File)> {
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
        let store = *self;
        leptos::task::spawn_local(async move {
            let result = crate::support::csv_flow::delete_all(tab.list_path()).await;
            store.apply_delete_result(generation, tab, result);
        });
    }

    pub(crate) fn try_begin_delete(&self, tab: ReceiptsTab) -> Option<Generation> {
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
        generation: Generation,
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
        generation: Generation,
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
        generation: Generation,
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

    pub fn reload(&self, tab: ReceiptsTab) {
        if self.session.user.get_untracked().is_none() {
            return;
        }
        let generation = self.session.generation.get_untracked();
        if self.refresh_tab_list(generation, tab) {
            self.fetch.dispatch((generation, tab));
        }
    }

    // 再取得が必要なら true を返す。fetch の起動(dispatch)は呼び出し側が行う
    pub(crate) fn refresh_tab_list(&self, generation: Generation, tab: ReceiptsTab) -> bool {
        let mut refresh = false;
        self.cache.update(|map| {
            refresh = mark_tab_for_refresh(map, generation, tab);
        });
        refresh
    }
}

pub(crate) fn has_stale_generation<V>(
    map: &HashMap<(Generation, ReceiptsTab), V>,
    generation: Generation,
) -> bool {
    map.keys().any(|(cached, _)| *cached != generation)
}

pub(crate) fn prune_stale_generation<V>(
    map: &mut HashMap<(Generation, ReceiptsTab), V>,
    generation: Generation,
) {
    map.retain(|key, _| key.0 == generation);
}

pub(crate) fn tab_settled(store: &ReceiptsStore, generation: Generation, tab: ReceiptsTab) -> bool {
    store.cache.with(|map| {
        matches!(
            map.get(&(generation, tab)),
            Some(TabState::Ready(_)) | Some(TabState::Failed(_))
        )
    })
}

pub(crate) fn should_apply_fetch_result(session: &SessionStore, generation: Generation) -> bool {
    session.is_current(generation)
}

pub(crate) fn mark_tab_for_refresh(
    map: &mut HashMap<(Generation, ReceiptsTab), TabState>,
    generation: Generation,
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
    // 同じタブの取得が重なったとき、先に始めた取得の結果で新しい結果を上書きしないため
    let latest_fetch: Rc<RefCell<HashMap<(Generation, ReceiptsTab), u64>>> = Rc::default();
    let fetch = Action::new_unsync(move |(generation, tab): &(Generation, ReceiptsTab)| {
        let (generation, tab) = (*generation, *tab);
        let session = fetch_session;
        let rev = {
            let mut latest = latest_fetch.borrow_mut();
            let rev = latest.entry((generation, tab)).or_default();
            *rev += 1;
            *rev
        };
        let latest_fetch = Rc::clone(&latest_fetch);
        async move {
            let rows = tab.fetch_list().await;
            if !should_apply_fetch_result(&session, generation)
                || latest_fetch.borrow().get(&(generation, tab)) != Some(&rev)
            {
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
        utility_rail_open: RwSignal::new(true),
        utility_rail_decided: RwSignal::new(false),
        utility_rail_initialized: RwSignal::new(HashSet::new()),
        expanded_epoch: RwSignal::new(None),
        visited,
        cache,
        fetch,
        csv,
        csv_files,
    };

    Effect::new(move |_| {
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
    });

    store
}
