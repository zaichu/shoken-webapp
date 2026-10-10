mod csv;

#[cfg(test)]
use super::ReceiptRow;
use super::{ReceiptTabData, ReceiptsTab, TabState};
use crate::api::ApiError;
use crate::features::receipts::csv::CsvPreviewRow;
use crate::features::receipts::filter::ReceiptSearch;
use crate::session::{Generation, SessionStore};
use crate::support::csv_flow::CsvTabState;
use crate::ui::workspace_shell::workspace_panel_default_open;
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};

// 一覧・CSV・裏再取得の状態は同じ世代×タブのキーで管理するため1エントリにまとめる。
// list が None のエントリは CSV 操作だけが先行した未取得タブ
#[derive(Default)]
pub(crate) struct TabCacheEntry {
    pub(crate) list: Option<TabState>,
    pub(crate) csv: CsvTabState<CsvPreviewRow>,
    pub(crate) csv_file: Option<web_sys::File>,
    // 裏再取得の失敗は表示済みの一覧とは別に持ち、行を消さない
    pub(crate) refresh_error: Option<String>,
    // 裏再取得の遅れ応答が全件削除や CSV 保存の結果を上書きしないための世代内リビジョン
    pub(crate) fetch_rev: u64,
}

impl From<TabState> for TabCacheEntry {
    fn from(list: TabState) -> Self {
        Self {
            list: Some(list),
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy)]
pub struct ReceiptsStore {
    pub(crate) session: SessionStore,
    pub active_tab: RwSignal<ReceiptsTab>,
    pub search: RwSignal<ReceiptSearch>,
    // 開閉状態は検索変更や一覧再描画でビューが作り直されても消えないようストア側に持つ
    pub expanded: RwSignal<HashSet<String>>,
    pub mobile_summary_expanded: RwSignal<bool>,
    // Cookie ポリシーへの記載を避けるため localStorage には保存せず、
    // セッション内だけの画面状態として持つ。開閉はユーザー操作だけが変える
    pub utility_rail_open: RwSignal<bool>,
    // 再マウントなしのアカウント切替で前のユーザーの開閉状態を残さないためのセッション世代
    pub(crate) expanded_epoch: RwSignal<Option<Generation>>,
    pub(crate) visited: RwSignal<HashSet<ReceiptsTab>>,
    // 一覧キャッシュと同じく世代で区切り、ログアウト・ユーザー切替で自動的に無効化する
    pub(crate) tabs: RwSignal<HashMap<(Generation, ReceiptsTab), TabCacheEntry>>,
    pub(crate) fetch: Action<(Generation, ReceiptsTab), ()>,
}

fn tab_list(
    map: &HashMap<(Generation, ReceiptsTab), TabCacheEntry>,
    generation: Generation,
    tab: ReceiptsTab,
) -> Option<&TabState> {
    map.get(&(generation, tab))
        .and_then(|entry| entry.list.as_ref())
}

impl ReceiptsStore {
    #[cfg(test)]
    pub fn rows(&self, tab: ReceiptsTab) -> Vec<ReceiptRow> {
        let generation = self.session.generation.get();
        self.tabs.with(|map| match tab_list(map, generation, tab) {
            Some(TabState::Ready(data)) => data.rows.clone(),
            _ => Vec::new(),
        })
    }

    pub fn count(&self, tab: ReceiptsTab) -> usize {
        let generation = self.session.generation.get();
        self.tabs.with(|map| match tab_list(map, generation, tab) {
            Some(TabState::Ready(data)) => data.rows.len(),
            _ => 0,
        })
    }

    pub fn tab_state(&self, tab: ReceiptsTab) -> TabState {
        let generation = self.session.generation.get();
        self.tabs.with(|map| {
            tab_list(map, generation, tab)
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

    pub fn toggle_utility_rail(&self) {
        self.utility_rail_open.update(|open| *open = !*open);
    }

    /// 開いたままなら絞り込みはレール内で確認できるので、表の上への件数バッジは畳んだときだけ出す
    pub fn utility_filter_badge_visible(&self) -> bool {
        !self.utility_rail_open.get() && !self.search.get().is_default()
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
        // 前の裏再取得が残っている往復では要求を重ねない
        if self.fetch.pending().get_untracked() {
            return;
        }
        let Some(generation) = self.current_generation() else {
            return;
        };
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
        let Some(generation) = self.current_generation() else {
            return;
        };
        let mut tab = tab;
        if self.expanded_epoch.get_untracked() != Some(generation) {
            self.expanded_epoch.set(Some(generation));
            self.expanded.update(|set| set.clear());
            self.mobile_summary_expanded.set(false);
            self.utility_rail_open.set(workspace_panel_default_open());
            // ユーザーが変わっても前の検索語・選択タブ・訪問済みを持ち越さない
            self.search.set(ReceiptSearch::default());
            self.active_tab.set(ReceiptsTab::Dividend);
            self.visited.set(HashSet::new());
            tab = ReceiptsTab::Dividend;
        }
        let needs_prune = self
            .tabs
            .with_untracked(|map| map.keys().any(|(cached, _)| *cached != generation));
        if needs_prune {
            self.tabs
                .update(|map| map.retain(|key, _| key.0 == generation));
        }
        let already_requested = self
            .tabs
            .with_untracked(|map| tab_list(map, generation, tab).is_some());
        if already_requested {
            return;
        }
        self.visited.update(|visited| {
            visited.insert(tab);
        });
        self.tabs.update(|map| {
            map.entry((generation, tab)).or_default().list = Some(TabState::Loading);
        });
        self.fetch.dispatch((generation, tab));
    }

    // 裏再取得が失敗したときだけ Some(表示済みの一覧は残る)
    pub fn refresh_error(&self, tab: ReceiptsTab) -> Option<String> {
        let generation = self.session.generation.get();
        self.tabs.with(|map| {
            map.get(&(generation, tab))
                .and_then(|entry| entry.refresh_error.clone())
        })
    }

    pub fn any_tab_fetching(&self) -> bool {
        let generation = self.session.generation.get();
        self.tabs.with(|map| {
            ReceiptsTab::ALL
                .iter()
                .any(|tab| matches!(tab_list(map, generation, *tab), Some(TabState::Loading)))
        })
    }

    fn current_generation(&self) -> Option<Generation> {
        self.session.user.get_untracked()?;
        Some(self.session.generation.get_untracked())
    }

    pub fn reload(&self, tab: ReceiptsTab) {
        let Some(generation) = self.current_generation() else {
            return;
        };
        if self.refresh_tab_list(generation, tab) {
            self.fetch.dispatch((generation, tab));
        }
    }

    // 再取得が必要なら true を返す。fetch の起動(dispatch)は呼び出し側が行う
    pub(crate) fn refresh_tab_list(&self, generation: Generation, tab: ReceiptsTab) -> bool {
        let mut refresh = false;
        self.tabs.update(|map| {
            refresh = mark_tab_for_refresh(map, generation, tab);
        });
        refresh
    }
}

pub(crate) fn tab_settled(store: &ReceiptsStore, generation: Generation, tab: ReceiptsTab) -> bool {
    store.tabs.with(|map| {
        matches!(
            tab_list(map, generation, tab),
            Some(TabState::Ready(_)) | Some(TabState::Failed(_))
        )
    })
}

pub(crate) fn bump_fetch_rev(
    map: &mut HashMap<(Generation, ReceiptsTab), TabCacheEntry>,
    generation: Generation,
    tab: ReceiptsTab,
) -> u64 {
    let entry = map.entry((generation, tab)).or_default();
    entry.fetch_rev += 1;
    entry.fetch_rev
}

pub(crate) fn is_current_fetch(
    map: &HashMap<(Generation, ReceiptsTab), TabCacheEntry>,
    generation: Generation,
    tab: ReceiptsTab,
    rev: u64,
) -> bool {
    map.get(&(generation, tab))
        .is_some_and(|entry| entry.fetch_rev == rev)
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
    map: &mut HashMap<(Generation, ReceiptsTab), TabCacheEntry>,
    generation: Generation,
    tab: ReceiptsTab,
) -> bool {
    let Some(entry) = map.get_mut(&(generation, tab)) else {
        return false;
    };
    if entry.list.is_none() {
        return false;
    }
    entry.list = Some(TabState::Loading);
    true
}

pub fn use_receipts_data(session: SessionStore, initial_tab: ReceiptsTab) -> ReceiptsStore {
    #[cfg(test)]
    {
        // テストでは呼び出しごとに新しいストアを返して session ごとの独立性を保つ
        build_receipts_store(session, initial_tab)
    }
    #[cfg(not(test))]
    {
        // ページ遷移でビューを作り直しても取得済みデータを失わないよう、App コンテキストのスロットに保持する
        let states = use_context::<crate::app::PageStates>().unwrap_or_default();
        crate::app::page_state(states.receipts, || {
            build_receipts_store(session, initial_tab)
        })
    }
}

fn build_receipts_store(session: SessionStore, initial_tab: ReceiptsTab) -> ReceiptsStore {
    let active_tab = RwSignal::new(initial_tab);
    let visited = RwSignal::new(HashSet::from([initial_tab]));
    let tabs = RwSignal::new(HashMap::new());

    let fetch_session = session;
    let fetch = Action::new_unsync(move |(generation, tab): &(Generation, ReceiptsTab)| {
        let (generation, tab) = (*generation, *tab);
        let session = fetch_session;
        // 同じタブの取得が重なったとき、先に始めた取得の結果で新しい結果を上書きしないため
        let rev = tabs
            .try_update(|map| bump_fetch_rev(map, generation, tab))
            .unwrap_or_default();
        async move {
            let result = tab.fetch_list().await;
            if !session.is_current(generation)
                || !tabs.with_untracked(|map| is_current_fetch(map, generation, tab, rev))
            {
                return;
            }
            let current_ready = tabs.with_untracked(|map| {
                matches!(tab_list(map, generation, tab), Some(TabState::Ready(_)))
            });
            match settle_tab_result(current_ready, result) {
                Ok(state) => {
                    tabs.update(|map| {
                        let entry = map.entry((generation, tab)).or_default();
                        entry.list = Some(state);
                        entry.refresh_error = None;
                    });
                }
                Err(message) => {
                    tabs.update(|map| {
                        map.entry((generation, tab)).or_default().refresh_error = Some(message);
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
        expanded_epoch: RwSignal::new(None),
        visited,
        tabs,
        fetch,
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
