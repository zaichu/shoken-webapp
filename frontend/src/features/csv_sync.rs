mod port;
#[cfg(test)]
mod tests;
mod view;

pub(crate) use view::{CsvSyncButton, CsvSyncPanel, CsvSyncStatus};

use crate::ui::labels;
use leptos::prelude::*;

/// 拡張機能が受け付ける CSV の種類。kind 値は拡張機能側の CsvDownloadType と同じ
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CsvKind {
    /// 国内株式の保有残高
    AssetBalance,
    /// 投資信託の保有残高
    FundBalance,
    /// 配当金・分配金
    Dividend,
    /// 国内株式の取引履歴
    DomesticStock,
    /// 投資信託の取引履歴
    MutualFund,
}

impl CsvKind {
    /// 依頼する順序(Issue の5種類)
    pub(crate) const ALL: [CsvKind; 5] = [
        Self::AssetBalance,
        Self::FundBalance,
        Self::Dividend,
        Self::DomesticStock,
        Self::MutualFund,
    ];

    fn request_value(self) -> &'static str {
        match self {
            Self::AssetBalance => "assetbalance",
            Self::FundBalance => "fundbalance",
            Self::Dividend => "dividend",
            Self::DomesticStock => "domesticstock",
            Self::MutualFund => "mutualfund",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::AssetBalance => labels::CSV_KIND_STOCK_BALANCE,
            Self::FundBalance => labels::CSV_KIND_FUND_BALANCE,
            Self::Dividend => labels::DIVIDEND_AMOUNT,
            Self::DomesticStock => labels::CSV_KIND_STOCK_TRADE,
            Self::MutualFund => labels::CSV_KIND_FUND_TRADE,
        }
    }

    fn from_request_value(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.request_value() == value)
    }

    /// ALL 内の位置。進行表示の配列添字に使う
    fn index(self) -> usize {
        Self::ALL.iter().position(|kind| *kind == self).unwrap()
    }
}

/// 拡張機能から受け取った CSV 本文(段階2の取り込みに渡す。画面には出さない)
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ReceivedCsv {
    pub(crate) kind: CsvKind,
    pub(crate) content: String,
    pub(crate) fetched_at: String,
}

/// 種類ごとの取得失敗
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FetchFailure {
    pub(crate) kind: CsvKind,
    pub(crate) reason: String,
    /// 失敗を区別するコード('file-access-disabled' など拡張機能の定義どおり)
    pub(crate) code: Option<String>,
}

/// 進行表示用の種類ごとの状態
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum KindStatus {
    Waiting,
    Fetching,
    Done { lines: usize },
    Failed { reason: String },
}

/// 進行・結果の画面状態。受け取った CSV 本文は表示に不要なので store が別に持つ
#[derive(Clone, Default)]
pub(crate) struct SyncState {
    pub(crate) running: bool,
    pub(crate) login_wait: bool,
    pub(crate) finished: bool,
    /// どの拡張機能 ID にも接続できなかった(未導入・古い)
    pub(crate) missing: bool,
    /// {type:'error'} や通信切断の表示文言
    pub(crate) error: Option<String>,
    pub(crate) kinds: Vec<(CsvKind, KindStatus)>,
    pub(crate) failures: Vec<FetchFailure>,
}

impl SyncState {
    fn initial() -> Self {
        Self {
            running: true,
            kinds: CsvKind::ALL
                .into_iter()
                .map(|kind| (kind, KindStatus::Waiting))
                .collect(),
            ..Self::default()
        }
    }

    fn set_kind(&mut self, kind: CsvKind, status: KindStatus) {
        // 実行前(未初期化)の状態へ来た応答は進行表示を持たないので無視する
        if let Some(slot) = self.kinds.get_mut(kind.index()) {
            *slot = (kind, status);
        }
    }

    /// 「ファイルの URL へのアクセス」の案内を出すか
    pub(crate) fn needs_file_access_guide(&self) -> bool {
        self.failures
            .iter()
            .any(|failure| failure.code.as_deref() == Some("file-access-disabled"))
    }

    fn apply(&mut self, event: port::Event) {
        match event {
            port::Event::Progress {
                state,
                kind,
                reason,
            } => match state {
                port::ProgressState::LoginWait => self.login_wait = true,
                port::ProgressState::Fetching => {
                    self.login_wait = false;
                    if let Some(kind) = kind {
                        self.set_kind(kind, KindStatus::Fetching);
                    }
                }
                port::ProgressState::Completed => {
                    if let Some(kind) = kind {
                        self.set_kind(kind, KindStatus::Done { lines: 0 });
                    }
                }
                port::ProgressState::Failed => {
                    if let Some(kind) = kind {
                        self.set_kind(
                            kind,
                            KindStatus::Failed {
                                reason: reason.unwrap_or_default(),
                            },
                        );
                    }
                }
            },
            port::Event::Result { csvs, failures } => {
                self.running = false;
                self.finished = true;
                self.login_wait = false;
                for csv in &csvs {
                    let lines =
                        crate::support::csv_flow::record_ranges(csv.content.as_bytes()).len();
                    self.set_kind(csv.kind, KindStatus::Done { lines });
                }
                for failure in &failures {
                    self.set_kind(
                        failure.kind,
                        KindStatus::Failed {
                            reason: failure.reason.clone(),
                        },
                    );
                }
                self.failures = failures;
            }
            port::Event::ErrorMessage(message) => {
                self.running = false;
                self.login_wait = false;
                self.error = Some(message);
            }
            port::Event::Disconnected => {
                self.running = false;
                self.login_wait = false;
                self.error = Some(labels::CSV_SYNC_DISCONNECTED.to_string());
            }
            port::Event::Unavailable => {
                self.running = false;
                self.login_wait = false;
                self.missing = true;
            }
        }
    }
}

/// 「CSV 最新取得」の実行状態。App で context 提供し、ページをまたいで進行を見せる。
/// 受け取った CSV 本文は個人情報を含むため表示用シグナルには入れず、段階2へ渡す口として別に保持する
#[derive(Clone, Copy)]
pub(crate) struct CsvSyncStore {
    state: RwSignal<SyncState>,
    /// 実行中のポート接続。リセット・次の実行で切断するために保持する。
    /// JS オブジェクトを含み Send/Sync でないため LocalStorage のシグナルに置く
    active: RwSignal<Option<port::ActiveFetch>, LocalStorage>,
    /// 実行ごとの世代。リセット後に古い実行の応答が状態を書き換えないようにする
    epoch: RwSignal<u64>,
    /// 受け取った CSV 本文(段階2の取り込みが読む。表示には使わない)
    received: RwSignal<Vec<ReceivedCsv>>,
}

impl CsvSyncStore {
    pub(crate) fn new() -> Self {
        Self {
            state: RwSignal::new(SyncState::default()),
            active: RwSignal::new_local(None),
            epoch: RwSignal::new(0),
            received: RwSignal::new(Vec::new()),
        }
    }

    pub(crate) fn running(&self) -> bool {
        self.state.with(|state| state.running)
    }

    /// 一覧画面に進行を出すか。CSV 管理では常時パネルに置くためこれは使わない
    pub(crate) fn visible(&self) -> bool {
        self.state
            .with(|state| state.running || state.finished || state.missing || state.error.is_some())
    }

    pub(crate) fn state(&self) -> SyncState {
        self.state.get()
    }

    #[cfg(test)]
    pub(crate) fn received(&self) -> Vec<ReceivedCsv> {
        self.received.get()
    }

    /// ボタンの押下。実行中は何もしない
    pub(crate) fn start(&self) {
        if self.running() {
            return;
        }
        self.stop_active();
        self.epoch.update(|epoch| *epoch += 1);
        let epoch = self.epoch.get_untracked();
        self.state.set(SyncState::initial());
        self.received.set(Vec::new());
        let store = *self;
        match port::start(move |event| {
            if store.epoch.get_untracked() == epoch {
                store.apply(event);
            }
        }) {
            Some(active) => self.active.set(Some(active)),
            None => self.apply(port::Event::Unavailable),
        }
    }

    /// 実行をやめて初期状態に戻す(ログアウト時。表示中の結果も消す)
    pub(crate) fn reset(&self) {
        self.stop_active();
        self.epoch.update(|epoch| *epoch += 1);
        self.state.set(SyncState::default());
        self.received.set(Vec::new());
    }

    fn stop_active(&self) {
        if let Some(active) = self.active.try_update(|slot| slot.take()).flatten() {
            active.disconnect();
        }
    }

    fn apply(&self, event: port::Event) {
        let finish = matches!(
            event,
            port::Event::Result { .. }
                | port::Event::ErrorMessage(_)
                | port::Event::Disconnected
                | port::Event::Unavailable
        );
        // CSV 本文は表示状態に載せず、段階2の取り込みが読む受け取り口にだけ置く
        if let port::Event::Result { ref csvs, .. } = event {
            self.received.set(csvs.clone());
        }
        self.state.update(|state| state.apply(event));
        if finish {
            self.stop_active();
        }
    }
}

pub(crate) fn use_csv_sync() -> CsvSyncStore {
    expect_context::<CsvSyncStore>()
}
