mod guard;
mod import;
mod port;
#[cfg(test)]
mod tests;
mod view;

pub(crate) use view::CsvSyncPanel;

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

    pub fn request_value(self) -> &'static str {
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
    pub fn index(self) -> usize {
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

/// 安全装置や確認で取り込まなかった理由。表示文言は labels の関数が引く
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SkipReason {
    /// 空または見出しだけでデータ行がない
    Empty,
    /// HTML を受け取った(エラーページ・ログイン画面など)
    Html,
    /// デコード失敗の置換文字を含む(文字化け)
    Mojibake,
    /// 見出し行または必須の列が見つからない
    Columns,
    /// 検証で有効な行が0件
    NoValidRows,
    /// 全件置換の対象で行エラーがある(そのまま置き換えると保存済み分が失われる)
    RowErrors,
    /// 件数激減の確認で取り込まない選択をした
    Declined,
    /// 保存済み件数を取れず激減の判定ができなかった
    CountUnknown,
}

impl SkipReason {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Empty => labels::CSV_SKIP_EMPTY,
            Self::Html => labels::CSV_SKIP_HTML,
            Self::Mojibake => labels::CSV_SKIP_MOJIBAKE,
            Self::Columns => labels::CSV_SKIP_COLUMNS,
            Self::NoValidRows => labels::CSV_SKIP_NO_VALID,
            Self::RowErrors => labels::CSV_SKIP_ROW_ERRORS,
            Self::Declined => labels::CSV_SKIP_DECLINED,
            Self::CountUnknown => labels::CSV_SKIP_COUNT_UNKNOWN,
        }
    }
}

/// 進行表示用の種類ごとの状態
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum KindStatus {
    Waiting,
    Fetching,
    Done {
        lines: usize,
    },
    Importing,
    /// 件数激減の確認を出して回答を待っている
    Confirming,
    Imported {
        result: crate::api::dto::CsvUploadResponse,
    },
    Skipped {
        reason: SkipReason,
    },
    Failed {
        reason: String,
    },
}

/// 件数激減の確認表示
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlungeConfirm {
    pub(crate) kind: CsvKind,
    /// 取り込もうとしている行数(検証を通った有効行)
    pub(crate) new_rows: usize,
    /// いま保存されている件数
    pub(crate) saved_rows: usize,
}

/// 取り込み段の画面状態
#[derive(Clone, Default)]
pub(crate) struct ImportProgress {
    /// 激減の確認待ち。回答で消える
    pub(crate) confirm: Option<PlungeConfirm>,
    /// 今回の実行で取り込めた種類。確認待ちで再開を分けても取りこぼさないよう溜める
    pub(crate) imported: Vec<CsvKind>,
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
    pub(crate) import: ImportProgress,
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
                self.finished = true;
                self.login_wait = false;
                // 取り込み・確認が残る間は実行中のままにして、連打で重複送信させない
                self.running = !csvs.is_empty();
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
/// 受け取った CSV 本文は個人情報を含むためシグナルには保持せず、取り込みタスクと
/// 確認待ちの pending にだけ渡す(どちらも処理が終われば破棄される)
#[derive(Clone, Copy)]
pub(crate) struct CsvSyncStore {
    state: RwSignal<SyncState>,
    /// 実行中のポート接続。リセット・次の実行で切断するために保持する。
    /// JS オブジェクトを含み Send/Sync でないため LocalStorage のシグナルに置く
    active: RwSignal<Option<port::ActiveFetch>, LocalStorage>,
    /// 実行ごとの世代。リセット後に古い実行の応答が状態を書き換えないようにする
    epoch: RwSignal<u64>,
    /// 件数激減の確認待ち。回答で残りの取り込みを再開する
    pending: RwSignal<Option<import::PendingImport>, LocalStorage>,
    /// 取り込み後の一覧再取得に使うページ状態とセッション世代
    pages: crate::app::PageStates,
    session: crate::session::SessionStore,
}

impl CsvSyncStore {
    pub(crate) fn new() -> Self {
        Self {
            state: RwSignal::new(SyncState::default()),
            active: RwSignal::new_local(None),
            epoch: RwSignal::new(0),
            pending: RwSignal::new_local(None),
            pages: use_context::<crate::app::PageStates>().unwrap_or_default(),
            session: use_context::<crate::session::SessionStore>().unwrap_or_default(),
        }
    }

    pub(crate) fn running(&self) -> bool {
        self.state.with(|state| state.running)
    }

    pub(crate) fn state(&self) -> SyncState {
        self.state.get()
    }

    /// ボタンの押下。実行中は何もしない(全種類取得)
    pub(crate) fn start(&self) {
        self.start_kinds(&CsvKind::ALL);
    }

    /// 指定した1種類だけを取得する。行のメニューから呼ぶ
    pub(crate) fn start_kind(&self, kind: CsvKind) {
        self.start_kinds(&[kind]);
    }

    fn start_kinds(&self, kinds: &[CsvKind]) {
        if self.running() {
            return;
        }
        self.stop_active();
        self.epoch.update(|epoch| *epoch += 1);
        let epoch = self.epoch.get_untracked();
        self.state.set(SyncState::initial());
        self.pending.set(None);
        let store = *self;
        match port::start(kinds, move |event| {
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
        self.pending.set(None);
    }

    fn stop_active(&self) {
        if let Some(active) = self.active.try_update(|slot| slot.take()).flatten() {
            active.disconnect();
        }
    }

    fn set_kind(&self, kind: CsvKind, status: KindStatus) {
        self.state.update(|state| state.set_kind(kind, status));
    }

    /// この実行とセッションがまだ有効か。リセット・次の実行・ユーザー切替のあとに
    /// 遅れて届いた旧タスクが状態を書き換えたり変更系リクエストを送らないようにする
    fn alive(&self, epoch: u64, generation: crate::session::Generation) -> bool {
        self.epoch.get_untracked() == epoch && self.session.is_current(generation)
    }

    /// 激減の確認待ちにして取り込みを止める。回答は answer_confirm で受ける。
    /// 実行世代が変わったあとに遅れて届いた確認は捨てる(前の実行の残り物を出さない)
    fn hold_confirm(&self, pending: import::PendingImport) {
        if !self.alive(pending.epoch, pending.generation) {
            return;
        }
        let kind = pending.csv.kind;
        let confirm = PlungeConfirm {
            kind,
            new_rows: pending.new_rows,
            saved_rows: pending.saved_rows,
        };
        self.pending.set(Some(pending));
        self.state.update(|state| {
            state.import.confirm = Some(confirm);
            state.set_kind(kind, KindStatus::Confirming);
        });
    }

    /// 激減の確認への回答。「はい」は件数判定を飛ばして同じ種類から、「いいえ」は
    /// その種類を取り込まず、どちらも残りの取り込みを続ける
    pub(crate) fn answer_confirm(&self, yes: bool) {
        let Some(pending) = self.pending.try_update(|slot| slot.take()).flatten() else {
            return;
        };
        if !self.alive(pending.epoch, pending.generation) {
            return;
        }
        self.state.update(|state| state.import.confirm = None);
        let store = *self;
        let epoch = pending.epoch;
        let generation = pending.generation;
        if yes {
            let mut csvs = vec![pending.csv];
            csvs.extend(pending.rest);
            let confirmed = csvs[0].kind;
            leptos::task::spawn_local(async move {
                import::process(store, epoch, generation, csvs, Some(confirmed)).await;
            });
        } else {
            self.set_kind(
                pending.csv.kind,
                KindStatus::Skipped {
                    reason: SkipReason::Declined,
                },
            );
            leptos::task::spawn_local(async move {
                import::process(store, epoch, generation, pending.rest, None).await;
            });
        }
    }

    /// 全種類の処理が終わった。取り込めた種類の一覧を再取得し、実行中を解除する
    fn finish_import(&self) {
        let imported = self.state.with(|state| state.import.imported.clone());
        self.refresh_lists(&imported);
        self.state.update(|state| state.running = false);
    }

    /// 取り込めた種類の一覧を取り直す。ページ未訪問のスロットは未生成なので何もしない
    fn refresh_lists(&self, imported: &[CsvKind]) {
        let mut stock = false;
        let mut fund = false;
        for kind in imported {
            match kind {
                CsvKind::AssetBalance => stock = true,
                CsvKind::FundBalance => fund = true,
                _ => {
                    if let (Some(tab), Some((_, store))) = (
                        import::receipts_tab_of(*kind),
                        self.pages.receipts.get_untracked(),
                    ) {
                        store.reload(tab);
                    }
                }
            }
        }
        if (stock || fund)
            && let Some((_, state)) = self.pages.asset_balance.get_untracked()
        {
            state.reload_lists(self.session, stock, fund);
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
        // CSV 本文は表示状態に載せず、取り込みタスクへそのまま渡す
        let mut result_csvs = None;
        if let port::Event::Result { ref csvs, .. } = event {
            result_csvs = Some(csvs.clone());
        }
        self.state.update(|state| state.apply(event));
        if finish {
            self.stop_active();
        }
        if let Some(csvs) = result_csvs
            && !csvs.is_empty()
        {
            let epoch = self.epoch.get_untracked();
            let generation = self.session.generation.get_untracked();
            let store = *self;
            leptos::task::spawn_local(async move {
                import::process(store, epoch, generation, csvs, None).await;
            });
        }
    }
}

pub(crate) fn use_csv_sync() -> CsvSyncStore {
    expect_context::<CsvSyncStore>()
}
