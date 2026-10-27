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
pub(crate) enum CsvKind {
    AssetStock,
    AssetFund,
    Dividend,
    DomesticStock,
    MutualFund,
}

impl CsvKind {
    pub(crate) const ALL: [CsvKind; 5] = [
        Self::AssetStock,
        Self::AssetFund,
        Self::Dividend,
        Self::DomesticStock,
        Self::MutualFund,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::AssetStock => "資産管理(国内株式)",
            Self::AssetFund => "資産管理(投資信託)",
            Self::Dividend => "取引明細(配当金)",
            Self::DomesticStock => "取引明細(国内株式)",
            Self::MutualFund => "取引明細(投資信託)",
        }
    }

    /// API の kind 値。csv-kinds/stats と拡張機能の依頼で同じ値を使う
    pub(crate) fn kind_param(self) -> &'static str {
        match self {
            Self::AssetStock => "assetbalance",
            Self::AssetFund => "fundbalance",
            Self::Dividend => "dividend",
            Self::DomesticStock => "domesticstock",
            Self::MutualFund => "mutualfund",
        }
    }

    pub(crate) fn receipt_tab(self) -> Option<ReceiptsTab> {
        match self {
            Self::Dividend => Some(ReceiptsTab::Dividend),
            Self::DomesticStock => Some(ReceiptsTab::DomesticStock),
            Self::MutualFund => Some(ReceiptsTab::MutualFund),
            _ => None,
        }
    }

    /// 取得(拡張機能)側の同じ種類
    pub(crate) fn sync_kind(self) -> SyncKind {
        match self {
            Self::AssetStock => SyncKind::AssetBalance,
            Self::AssetFund => SyncKind::FundBalance,
            Self::Dividend => SyncKind::Dividend,
            Self::DomesticStock => SyncKind::DomesticStock,
            Self::MutualFund => SyncKind::MutualFund,
        }
    }

    /// 残高(資産管理2種)は当日だけが新しく、明細(取引3種)は7日以内が新しい。
    /// 日数の判定は呼び出し側が JST の暦日差で求めた値を渡す
    pub(crate) fn is_fresh(self, days_ago: i64) -> bool {
        match self {
            Self::AssetStock | Self::AssetFund => days_ago <= 0,
            Self::Dividend | Self::DomesticStock | Self::MutualFund => days_ago <= 7,
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
    let sync_status = sync_kinds
        .iter()
        .find(|(sync_kind, _)| *sync_kind == kind.sync_kind())
        .map(|(_, status)| status.clone());
    let (count, latest_imported_at) = match &sync_status {
        Some(KindStatus::Imported { result, fetched_at }) => (
            Some(i64::try_from(result.inserted).unwrap_or(i64::MAX)),
            Some(fetched_at.clone()),
        ),
        _ => (
            stat.map(|stat| stat.count),
            stat.and_then(|stat| stat.latest_imported_at.clone()),
        ),
    };
    CsvKindRowData {
        kind,
        count,
        latest_imported_at,
        sync_status,
    }
}

#[component]
pub(crate) fn CsvPage() -> impl IntoView {
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
    let refresh_stats = move |settle_sync: bool| {
        leptos::task::spawn_local(async move {
            if let Ok(response) = fetch_csv_kind_stats().await {
                stats.set(Some(response));
                if settle_sync {
                    csv_sync.settle_imported_with_stats();
                }
            }
        });
    };
    Effect::new(move |_| refresh_stats(false));

    // 取得前の最終取得時刻表示用
    let last_fetched = RwSignal::new(None::<String>);
    let running = RwSignal::new(false);
    let handled_finished = RwSignal::new(false);
    Effect::new(move |_| running.set(csv_sync.running()));

    // csv_syncの完了を監視して保存済み件数を取り直す
    Effect::new(move |_| {
        let state = csv_sync.state();
        let finished = state.finished && !state.running;
        if finished && !handled_finished.get_untracked() {
            handled_finished.set(true);
            refresh_stats(true);
        } else if state.running {
            handled_finished.set(false);
        }
    });

    // 取得前の最終取得時刻(統計の最新の取込日時)
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

    // 取り込みは行の「⋯」→「ファイルから取り込み」で OS の選択を開く

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
                    last_fetched=last_fetched.read_only()
                    on_fetch=Callback::new(move |_| csv_sync.start())
                />
                <CsvSyncBody />

                {/* 種類ごとのテーブル */}
                <section class="rounded-lg border border-border bg-surface-sunken">
                    <CsvKindTable
                        rows=rows
                        asset_csv=asset_state.csv
                        receipts_store=receipts
                        csv_sync=csv_sync
                        on_stats_changed=Callback::new(move |_| refresh_stats(false))
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
        let params = [
            "assetbalance",
            "fundbalance",
            "dividend",
            "domesticstock",
            "mutualfund",
        ];
        for (kind, expected) in CsvKind::ALL.into_iter().zip(params) {
            assert_eq!(kind.kind_param(), expected);
        }
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

    #[test]
    fn row_data_prefers_import_result_until_stats_refresh() {
        let stats = CsvKindStatsResponse {
            kinds: vec![shared::csv_kind_stats::CsvKindStat {
                kind: "dividend".to_string(),
                label: "取引明細(配当金)".to_string(),
                count: 40,
                latest_imported_at: Some("2026-10-25T12:00:00Z".to_string()),
            }],
        };
        let sync = [(
            SyncKind::Dividend,
            KindStatus::Imported {
                result: crate::api::dto::CsvUploadResponse {
                    inserted: 1,
                    skipped: 0,
                    errors: vec![],
                },
                fetched_at: "2026-10-26T01:02:00Z".to_string(),
            },
        )];

        let row = row_data(CsvKind::Dividend, Some(&stats), &sync);

        assert_eq!(row.count, Some(1));
        assert_eq!(
            row.latest_imported_at.as_deref(),
            Some("2026-10-26T01:02:00Z")
        );
    }

    #[test]
    fn freshness_boundary_differs_between_balance_and_receipts() {
        for kind in [CsvKind::AssetStock, CsvKind::AssetFund] {
            assert!(kind.is_fresh(0));
            assert!(!kind.is_fresh(1));
        }
        for kind in [
            CsvKind::Dividend,
            CsvKind::DomesticStock,
            CsvKind::MutualFund,
        ] {
            assert!(kind.is_fresh(0));
            assert!(kind.is_fresh(7));
            assert!(!kind.is_fresh(8));
            assert!(!kind.is_fresh(40));
        }
    }
}
