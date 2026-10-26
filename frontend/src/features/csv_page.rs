//! CSV管理ページ(テーブル形式)
//! 5種類のCSVを1画面の表で管理する。取得・取り込み・削除は既存の共通ロジックを使用。

use crate::api::dto::CsvKindStatsResponse;
use crate::api::fetch_csv_kind_stats;
use crate::features::asset_balance::use_asset_balance_state;
use crate::features::csv_sync::{CsvKind as SyncKind, CsvSyncBody, KindStatus, use_csv_sync};
use crate::features::receipts::{ReceiptsTab, use_receipts_data};
use crate::session::use_session;
use crate::ui::csv_page::{CsvKindRowData, CsvKindTable, CsvManageFetchCard};
use crate::ui::format::{days_since_jst_date, format_jst_time, format_short_date};
use crate::ui::labels;
use crate::ui::page_frame::PageFrame;
use leptos::prelude::*;

/// CSV種類の定義(表示順はIssue指定の5種類)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CsvKind {
    AssetStock,
    AssetFund,
    Dividend,
    DomesticStock,
    MutualFund,
}

impl CsvKind {
    pub const ALL: [CsvKind; 5] = [
        Self::AssetStock,
        Self::AssetFund,
        Self::Dividend,
        Self::DomesticStock,
        Self::MutualFund,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::AssetStock => "資産管理(国内株式)",
            Self::AssetFund => "資産管理(投資信託)",
            Self::Dividend => "取引明細(配当金)",
            Self::DomesticStock => "取引明細(国内株式)",
            Self::MutualFund => "取引明細(投資信託)",
        }
    }

    /// API の kind 値。csv-kinds/stats と拡張機能の依頼で同じ値を使う
    pub fn kind_param(self) -> &'static str {
        match self {
            Self::AssetStock => "assetbalance",
            Self::AssetFund => "fundbalance",
            Self::Dividend => "dividend",
            Self::DomesticStock => "domesticstock",
            Self::MutualFund => "mutualfund",
        }
    }

    /// `?kind=` から開く種類を決める。一覧・設定のリンクは旧い種別値も使うため受け付ける
    pub fn from_kind_param(value: Option<&str>) -> Option<Self> {
        match value? {
            "assetbalance" => Some(Self::AssetStock),
            "fundbalance" => Some(Self::AssetFund),
            "dividend" => Some(Self::Dividend),
            "domesticstock" | "stock" => Some(Self::DomesticStock),
            "mutualfund" | "fund" => Some(Self::MutualFund),
            _ => None,
        }
    }

    pub fn receipt_tab(self) -> Option<ReceiptsTab> {
        match self {
            Self::Dividend => Some(ReceiptsTab::Dividend),
            Self::DomesticStock => Some(ReceiptsTab::DomesticStock),
            Self::MutualFund => Some(ReceiptsTab::MutualFund),
            _ => None,
        }
    }

    /// 取得(拡張機能)側の同じ種類
    pub fn sync_kind(self) -> SyncKind {
        match self {
            Self::AssetStock => SyncKind::AssetBalance,
            Self::AssetFund => SyncKind::FundBalance,
            Self::Dividend => SyncKind::Dividend,
            Self::DomesticStock => SyncKind::DomesticStock,
            Self::MutualFund => SyncKind::MutualFund,
        }
    }

    /// 削除確認に出す種類名
    pub fn delete_subject(self) -> &'static str {
        self.label()
    }

    /// 保存後に確認できる一覧ページ
    pub fn list_href(self) -> &'static str {
        match self.receipt_tab() {
            Some(_) => "/receipts",
            None => "/assetbalance",
        }
    }
}

fn row_data(
    kind: CsvKind,
    stats: Option<&CsvKindStatsResponse>,
    sync_kinds: &[(SyncKind, KindStatus)],
) -> CsvKindRowData {
    let stat = stats.and_then(|response| {
        response
            .kinds
            .iter()
            .find(|stat| stat.kind == kind.kind_param())
    });
    CsvKindRowData {
        kind,
        count: stat.map(|stat| stat.count),
        latest_imported_at: stat.and_then(|stat| stat.latest_imported_at.clone()),
        sync_status: sync_kinds
            .iter()
            .find(|(sync_kind, _)| *sync_kind == kind.sync_kind())
            .map(|(_, status)| status.clone()),
    }
}

#[component]
pub fn CsvPage() -> impl IntoView {
    let session = use_session();
    let asset_state = use_asset_balance_state(session);
    let receipts = use_receipts_data(
        session,
        crate::settings::try_use_settings().receipts_initial_tab,
    );
    receipts.revisit();

    let csv_sync = use_csv_sync();

    let stats = RwSignal::new(None::<CsvKindStatsResponse>);
    // 取得完了・取り込み・削除のあとに件数と状態を取り直す。失敗しても前回の表示は残す
    let refresh_stats = move || {
        leptos::task::spawn_local(async move {
            if let Ok(response) = fetch_csv_kind_stats().await {
                stats.set(Some(response));
            }
        });
    };
    Effect::new(move |_| refresh_stats());

    // 取得結果のメッセージ(取得完了時に表示)
    let fetch_result = RwSignal::new(None::<String>);
    // 取得前の最終取得時刻表示用
    let last_fetched = RwSignal::new(None::<String>);
    let running = RwSignal::new(false);
    Effect::new(move |_| running.set(csv_sync.running()));

    // csv_syncの完了を監視して結果メッセージを更新し、保存済み件数を取り直す
    Effect::new(move |_| {
        let state = csv_sync.state();
        if state.finished && !state.running {
            let mut updated = 0;
            let mut failed = 0;
            for (_, status) in &state.kinds {
                match status {
                    KindStatus::Imported { .. } => updated += 1,
                    KindStatus::Failed { .. } | KindStatus::Skipped { .. } => failed += 1,
                    _ => {}
                }
            }
            if updated > 0 || failed > 0 {
                fetch_result.set(Some(labels::csv_manage_fetch_result(updated, failed)));
            }
            last_fetched.set(None);
            refresh_stats();
        } else if state.running {
            fetch_result.set(None);
        }
    });

    // 取得前の最終取得時刻(統計の最新の取込日時)。統計が変わるたびに再計算し、
    // 該当する時刻がなければ表示を消す
    Effect::new(move |_| {
        let latest = stats.get().and_then(|response| {
            response
                .kinds
                .iter()
                .filter_map(|stat| stat.latest_imported_at.clone())
                .max()
        });
        match latest {
            Some(ts)
                if days_since_jst_date(&ts)
                    .map(|days| days <= 0)
                    .unwrap_or(false) =>
            {
                let date = format_short_date(&ts);
                let time = format_jst_time(&ts);
                last_fetched.set(Some(labels::csv_manage_last_fetched(&date, &time)));
            }
            _ => last_fetched.set(None),
        }
    });

    // 表は5種類を常に出し、統計が未取得なら件数は「—」で削除も選べない
    let rows = Memo::new(move |_| {
        let stats = stats.get();
        let sync_kinds = csv_sync.state().kinds;
        CsvKind::ALL
            .into_iter()
            .map(|kind| row_data(kind, stats.as_ref(), &sync_kinds))
            .collect::<Vec<_>>()
    });

    // 一覧・設定からの `?kind=` は同じ種類の取り込みを開く。指定が無いときは閉じておく
    let open_import = RwSignal::new(CsvKind::from_kind_param(
        crate::app::query_param(&crate::app::current_location(), "kind").as_deref(),
    ));

    view! {
        <PageFrame title="CSV管理">
            <div class="flex flex-col gap-4">
                {/* パンくず + 見出し */}
                <nav class="flex items-center gap-2 text-sm text-text-soft" aria-label="パンくず">
                    <a href="/assetbalance" class="hover:underline">"資産管理"</a>
                    <span aria-hidden="true">"/"</span>
                    <span class="text-text" aria-current="page">"CSV管理"</span>
                </nav>

                {/* 取得カード */}
                <CsvManageFetchCard
                    running=running.read_only()
                    fetch_result=fetch_result.read_only()
                    last_fetched=last_fetched.read_only()
                    on_fetch=Callback::new(move |_| csv_sync.start())
                />
                <CsvSyncBody />

                {/* 種類ごとのテーブル */}
                <section class="rounded-lg border border-border bg-surface-sunken overflow-hidden">
                    <CsvKindTable
                        rows=rows
                        asset_csv=asset_state.csv
                        receipts_store=receipts
                        csv_sync=csv_sync
                        open_import=open_import
                        on_stats_changed=Callback::new(move |_| refresh_stats())
                    />
                </section>
            </div>
        </PageFrame>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_kind_all_order() {
        let kinds = CsvKind::ALL;
        assert_eq!(kinds[0].label(), "資産管理(国内株式)");
        assert_eq!(kinds[1].label(), "資産管理(投資信託)");
        assert_eq!(kinds[2].label(), "取引明細(配当金)");
        assert_eq!(kinds[3].label(), "取引明細(国内株式)");
        assert_eq!(kinds[4].label(), "取引明細(投資信託)");
    }

    #[test]
    fn csv_kind_params_match_api() {
        for kind in CsvKind::ALL {
            // APIのkind値と一致すること
            assert!(!kind.kind_param().is_empty());
            assert_eq!(
                CsvKind::from_kind_param(Some(kind.kind_param())),
                Some(kind)
            );
        }
    }

    #[test]
    fn from_kind_param_accepts_legacy_values() {
        assert_eq!(
            CsvKind::from_kind_param(Some("stock")),
            Some(CsvKind::DomesticStock)
        );
        assert_eq!(
            CsvKind::from_kind_param(Some("fund")),
            Some(CsvKind::MutualFund)
        );
        assert_eq!(CsvKind::from_kind_param(Some("unknown")), None);
        assert_eq!(CsvKind::from_kind_param(None), None);
    }

    #[test]
    fn row_data_takes_count_and_time_of_same_kind() {
        let stats = CsvKindStatsResponse {
            kinds: vec![shared::csv_kind_stats::CsvKindStat {
                kind: "dividend".to_string(),
                label: "取引明細(配当金)".to_string(),
                count: 77,
                latest_imported_at: Some("2026-10-25T12:39:00Z".to_string()),
            }],
        };
        let row = row_data(CsvKind::Dividend, Some(&stats), &[]);
        assert_eq!(row.count, Some(77));
        assert_eq!(
            row.latest_imported_at.as_deref(),
            Some("2026-10-25T12:39:00Z")
        );

        // 統計に無い種類は件数未取得として扱う
        let other = row_data(CsvKind::AssetStock, Some(&stats), &[]);
        assert_eq!(other.count, None);
    }

    #[test]
    fn row_data_maps_sync_status_by_kind() {
        let sync = [(SyncKind::FundBalance, KindStatus::Fetching)];
        let row = row_data(CsvKind::AssetFund, None, &sync);
        assert_eq!(row.sync_status, Some(KindStatus::Fetching));
        let asset = row_data(CsvKind::AssetStock, None, &sync);
        assert_eq!(asset.sync_status, None);
    }
}
