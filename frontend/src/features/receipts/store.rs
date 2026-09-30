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

#[derive(Clone, Copy)]
pub struct ReceiptsStore {
    pub(crate) session: SessionStore,
    pub active_tab: RwSignal<ReceiptsTab>,
    pub search: RwSignal<ReceiptSearch>,
    // 開閉状態は検索変更や一覧再描画でビューが作り直されても消えないようストア側に持つ
    pub expanded: RwSignal<HashSet<String>>,
    pub mobile_summary_expanded: RwSignal<bool>,
    // Cookie ポリシーへの記載を避けるため localStorage には保存せず、
    // セッション内だけの画面状態として持つ。初期値はデータ有無でタブごとに決める
    pub utility_rail_open: RwSignal<bool>,
    pub(crate) utility_rail_decided: RwSignal<bool>,
    // タブごとの初期開閉。ユーザーがトグルするまでは切替・再取得のたびにこの値へ戻す
    pub(crate) utility_rail_initials: RwSignal<HashMap<ReceiptsTab, bool>>,
    // 再マウントなしのアカウント切替で前のユーザーの開閉状態を残さないためのセッション世代
    pub(crate) expanded_epoch: RwSignal<Option<Generation>>,
    pub(crate) visited: RwSignal<HashSet<ReceiptsTab>>,
    pub(crate) cache: RwSignal<HashMap<(Generation, ReceiptsTab), TabState>>,
    pub(crate) fetch: Action<(Generation, ReceiptsTab), ()>,
    // 一覧キャッシュと同じく世代で区切り、ログアウト・ユーザー切替で自動的に無効化する
    pub(crate) csv: RwSignal<HashMap<(Generation, ReceiptsTab), CsvTabState<CsvPreviewRow>>>,
    pub(crate) csv_files: RwSignal<HashMap<(Generation, ReceiptsTab), web_sys::File>>,
    // 裏再取得の失敗は表示済みの一覧とは別に持ち、行を消さない
    pub(crate) refresh_error: RwSignal<HashMap<(Generation, ReceiptsTab), String>>,
    // 裏再取得の遅れ応答が全件削除や CSV 保存の結果を上書きしないための世代内リビジョン
    pub(crate) fetch_rev: RwSignal<HashMap<(Generation, ReceiptsTab), u64>>,
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

    /// 初期状態(データ→畳む、0件→開く)はタブごとの最初の Ready で決めて記憶し、
    /// ユーザーがトグルするまではタブ表示のたびにその値へ戻す。
    /// 戻ったタブが直前のタブの開閉状態を引き継がないようにするため
    pub(crate) fn init_utility_rail(&self, tab: ReceiptsTab, has_rows: bool) {
        if self.utility_rail_decided.get_untracked() {
            return;
        }
        let open = self
            .utility_rail_initials
            .with_untracked(|initials| initials.get(&tab).copied())
            .unwrap_or(!has_rows);
        self.utility_rail_initials.update(|initials| {
            initials.insert(tab, open);
        });
        self.utility_rail_open.set(open);
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

    // 再訪時に、表示済みの一覧を消さず裏で取り直す(未取得・取得中は既存の経路が担う)
    pub(crate) fn revisit(&self) {
        if self.session.user.get_untracked().is_none() {
            return;
        }
        // 前の裏再取得が残っている往復では要求を重ねない
        if self.fetch.pending().get_untracked() {
            return;
        }
        let generation = self.session.generation.get_untracked();
        for tab in ReceiptsTab::ALL {
            if !self
                .visited
                .with_untracked(|visited| visited.contains(&tab))
            {
                continue;
            }
            if tab_settled(self, generation, tab) {
                self.fetch.dispatch((generation, tab));
            }
        }
    }

    pub(crate) fn ensure(&self, tab: ReceiptsTab) {
        if self.session.user.get_untracked().is_none() {
            return;
        }
        let generation = self.session.generation.get_untracked();
        let mut tab = tab;
        if self.expanded_epoch.get_untracked() != Some(generation) {
            self.expanded_epoch.set(Some(generation));
            self.expanded.update(|set| set.clear());
            self.mobile_summary_expanded.set(false);
            self.utility_rail_initials.update(|map| map.clear());
            self.utility_rail_decided.set(false);
            self.utility_rail_open.set(true);
            // ユーザーが変わっても前の検索語・選択タブ・訪問済みを持ち越さない
            self.search.set(ReceiptSearch::default());
            self.active_tab.set(ReceiptsTab::Dividend);
            self.visited.set(HashSet::new());
            tab = ReceiptsTab::Dividend;
        }
        let needs_prune = self
            .cache
            .with_untracked(|map| has_stale_generation(map, generation))
            || self
                .csv
                .with_untracked(|map| has_stale_generation(map, generation))
            || self
                .csv_files
                .with_untracked(|map| has_stale_generation(map, generation))
            || self
                .refresh_error
                .with_untracked(|map| has_stale_generation(map, generation));
        if needs_prune {
            self.cache
                .update(|map| prune_stale_generation(map, generation));
            self.csv
                .update(|map| prune_stale_generation(map, generation));
            self.csv_files
                .update(|map| prune_stale_generation(map, generation));
            self.refresh_error
                .update(|map| prune_stale_generation(map, generation));
            self.fetch_rev
                .update(|map| map.retain(|key, _| key.0 == generation));
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
    // 裏再取得が失敗したときだけ Some(表示済みの一覧は残る)
    pub fn refresh_error(&self, tab: ReceiptsTab) -> Option<String> {
        let generation = self.session.generation.get();
        self.refresh_error
            .with(|map| map.get(&(generation, tab)).cloned())
    }

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
                // 削除前に出た裏再取得の遅れ応答が消した行を復活させないよう取得を失効させる
                self.fetch_rev
                    .update(|map| bump_fetch_rev(map, generation, tab));
                // 空になった一覧の上に直前の再取得エラーが残るのを防ぐ
                self.refresh_error.update(|map| {
                    map.remove(&(generation, tab));
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

pub(crate) fn bump_fetch_rev(
    map: &mut HashMap<(Generation, ReceiptsTab), u64>,
    generation: Generation,
    tab: ReceiptsTab,
) {
    *map.entry((generation, tab)).or_default() += 1;
}

pub(crate) fn is_current_fetch(
    map: &HashMap<(Generation, ReceiptsTab), u64>,
    generation: Generation,
    tab: ReceiptsTab,
    rev: u64,
) -> bool {
    map.get(&(generation, tab)) == Some(&rev)
}

// 裏再取得の失敗は表示済みの Ready を残してエラーを返す。初回取得の失敗だけ Failed にする
pub(crate) fn settle_tab_result(
    current_ready: bool,
    result: Result<ReceiptTabData, ApiError>,
) -> Result<TabState, String> {
    match result {
        Ok(data) => Ok(TabState::Ready(data)),
        Err(error) if current_ready => Err(error.message()),
        Err(error) => Ok(TabState::Failed(error.message())),
    }
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

thread_local! {
    // ページ遷移でビューを作り直しても取得済みデータを失わないよう、アプリ寿命のオーナーに作る。
    // Owner::new() は現オーナーの子として登録されページと一緒に破棄されるため、AppOwner の子を使う。
    // テストでは呼び出しごとに新しいストアを返して session ごとの独立性を保つ
    static SHARED_STORE: RefCell<Option<(Owner, ReceiptsStore)>> = const { RefCell::new(None) };
}

pub fn use_receipts_data(session: SessionStore, initial_tab: ReceiptsTab) -> ReceiptsStore {
    #[cfg(test)]
    {
        build_receipts_store(session, initial_tab)
    }
    #[cfg(not(test))]
    {
        let app_owner = use_context::<crate::app::AppOwner>()
            .map(|app| app.0)
            .unwrap_or_default();
        SHARED_STORE.with(|cell| {
            if let Some((_, store)) = cell.borrow().as_ref() {
                return *store;
            }
            let owner = app_owner.child();
            let store = owner.with(|| build_receipts_store(session, initial_tab));
            *cell.borrow_mut() = Some((owner, store));
            store
        })
    }
}

fn build_receipts_store(session: SessionStore, initial_tab: ReceiptsTab) -> ReceiptsStore {
    let active_tab = RwSignal::new(initial_tab);
    let visited = RwSignal::new(HashSet::from([initial_tab]));
    let cache = RwSignal::new(HashMap::new());
    let csv = RwSignal::new(HashMap::new());
    let csv_files = RwSignal::new(HashMap::new());
    let refresh_error = RwSignal::new(HashMap::new());
    let fetch_rev = RwSignal::new(HashMap::new());

    let fetch_session = session;
    let cache_signal = cache;
    let error_signal = refresh_error;
    let fetch = Action::new_unsync(move |(generation, tab): &(Generation, ReceiptsTab)| {
        let (generation, tab) = (*generation, *tab);
        let session = fetch_session;
        // 同じタブの取得が重なったとき、先に始めた取得の結果で新しい結果を上書きしないため
        let rev = fetch_rev
            .try_update(|map| {
                bump_fetch_rev(map, generation, tab);
                map[&(generation, tab)]
            })
            .unwrap_or_default();
        async move {
            let result = tab.fetch_list().await;
            if !should_apply_fetch_result(&session, generation)
                || !fetch_rev.with_untracked(|map| is_current_fetch(map, generation, tab, rev))
            {
                return;
            }
            let current_ready = cache_signal.with_untracked(|map| {
                matches!(map.get(&(generation, tab)), Some(TabState::Ready(_)))
            });
            match settle_tab_result(current_ready, result) {
                Ok(state) => {
                    cache_signal.update(|map| {
                        map.insert((generation, tab), state);
                    });
                    error_signal.update(|map| {
                        map.remove(&(generation, tab));
                    });
                }
                Err(message) => {
                    error_signal.update(|map| {
                        map.insert((generation, tab), message);
                    });
                }
            }
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
        utility_rail_initials: RwSignal::new(HashMap::new()),
        expanded_epoch: RwSignal::new(None),
        visited,
        cache,
        fetch,
        csv,
        csv_files,
        refresh_error,
        fetch_rev,
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
