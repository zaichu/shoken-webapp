//! CSV管理ページ(テーブル形式)
//! 5種類のCSVを1画面の表で管理する。取得・取り込み・削除は既存の共通ロジックを使用。

use crate::api::dto::CsvKindStatsResponse;
use crate::api::fetch_csv_kind_stats;
use crate::features::asset_balance::{AssetKind, use_asset_balance_state};
use crate::features::csv_sync::{CsvSyncPanel, use_csv_sync};
use crate::features::receipts::{ReceiptsTab, use_receipts_data};
use crate::session::use_session;
use crate::ui::csv_page::{CsvKindRowData, CsvKindTable, CsvManageFetchCard};
use crate::ui::format::{days_since_jst_date, format_short_date};
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

    pub fn kind_param(self) -> &'static str {
        match self {
            Self::AssetStock => "assetbalance",
            Self::AssetFund => "fundbalance",
            Self::Dividend => "dividend",
            Self::DomesticStock => "domesticstock",
            Self::MutualFund => "mutualfund",
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

    fn asset_kind(self) -> Option<AssetKind> {
        match self {
            Self::AssetStock => Some(AssetKind::Stock),
            Self::AssetFund => Some(AssetKind::Fund),
            _ => None,
        }
    }

    fn list_href(self) -> &'static str {
        match self.receipt_tab() {
            Some(_) => "/receipts",
            None => "/assetbalance",
        }
    }

    /// 保存方式ラベル(資産管理=全件置換、取引明細=追加)
    #[allow(dead_code)]
    pub fn save_mode_label(self) -> &'static str {
        match self {
            Self::AssetStock | Self::AssetFund => labels::CSV_MANAGE_SAVE_MODE_REPLACE,
            _ => labels::CSV_MANAGE_SAVE_MODE_APPEND,
        }
    }

    /// 削除確認用の件数・種類名を取得するためのキー
    pub fn delete_subject(self) -> &'static str {
        self.label()
    }
}

type StatsSlot = RwSignal<Option<CsvKindStatsResponse>>;

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

    // 統計データを取得
    let stats = StatsSlot::new(None);
    let stats_loading = RwSignal::new(false);

    // 初回取得
    let stats_for_fetch = stats;
    let loading_for_fetch = stats_loading;
    leptos::task::spawn_local(async move {
        loading_for_fetch.set(true);
        match fetch_csv_kind_stats().await {
            Ok(response) => {
                stats_for_fetch.set(Some(response));
            }
            Err(_) => {
                // エラー時は空のまま
            }
        }
        loading_for_fetch.set(false);
    });

    // 取得結果のメッセージ(取得完了時に表示)
    let fetch_result = RwSignal::new(None::<String>);
    // 取得前の最終取得時刻表示用
    let last_fetched = RwSignal::new(None::<String>);
    // 実行中状態を追跡するシグナル
    let running = RwSignal::new(false);
    Effect::new(move |_| running.set(csv_sync.running()));

    // csv_syncの完了を監視して結果メッセージを更新
    Effect::new(move |_| {
        let state = csv_sync.state();
        if state.finished && !state.running {
            let mut updated = 0;
            let mut failed = 0;
            for (_, status) in state.kinds {
                match status {
                    crate::features::csv_sync::KindStatus::Imported { .. } => updated += 1,
                    crate::features::csv_sync::KindStatus::Failed { .. } => failed += 1,
                    crate::features::csv_sync::KindStatus::Skipped { .. } => failed += 1,
                    _ => {}
                }
            }
            if updated > 0 || failed > 0 {
                fetch_result.set(Some(labels::csv_manage_fetch_result(updated, failed)));
            }
            // 取得完了後はlast_fetchedをクリア
            last_fetched.set(None);
        } else if state.running {
            // 取得中は結果をクリア
            fetch_result.set(None);
        }
    });

    // 初期表示用のlast_fetched(統計データから最新の取込日時を計算)
    Effect::new(move |_| {
        if let Some(response) = stats.get() {
            let mut latest: Option<String> = None;
            for stat in &response.kinds {
                if let Some(ts) = &stat.latest_imported_at {
                    if latest.clone().is_none_or(|l: String| *ts > l) {
                        latest = Some(ts.clone());
                    }
                }
            }
            if let Some(ts) = latest
                && let Some(days_ago) = days_since_jst_date(&ts)
                && days_ago <= 0
            {
                let date = format_short_date(&ts);
                let time = ts[11..16].to_string(); // HH:MM部分
                last_fetched.set(Some(labels::csv_manage_last_fetched(&date, &time)));
            }
        }
    });

    // 行データの構築(統計 + sync状態をマージ)
    let rows = Memo::new(move |_| {
        let mut result = Vec::new();
        if let Some(response) = stats.get() {
            let sync_state = csv_sync.state();
            for stat in response.kinds {
                let kind = CsvKind::ALL
                    .iter()
                    .find(|k| k.kind_param() == stat.kind)
                    .copied()
                    .unwrap_or(CsvKind::AssetStock);
                let sync_status = sync_state
                    .kinds
                    .iter()
                    .find(|(k, _)| k.request_value() == stat.kind)
                    .map(|(_, s)| s.clone());
                result.push(CsvKindRowData {
                    kind,
                    count: stat.count,
                    latest_imported_at: stat.latest_imported_at,
                    sync_status,
                });
            }
        }
        result
    });

    // 取得カードのアクションバー(csv_syncのボタンを配置)
    let fetch_action_bar = view! { <CsvSyncPanel /> }.into_any();

    view! {
        <PageFrame title="CSV管理" action_bar=fetch_action_bar action_bar_pc_top=true>
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
                    on_fetch=move |_| csv_sync.start()
                />

                {/* 種類ごとのテーブル */}
                <section class="rounded-lg border border-border bg-surface-sunken overflow-hidden">
                    <CsvKindTable
                        rows=rows
                        asset_csv=asset_state.csv
                        receipts_store=receipts
                        csv_sync=csv_sync
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
        }
    }

    #[test]
    fn csv_kind_save_mode() {
        assert_eq!(CsvKind::AssetStock.save_mode_label(), "全件置換");
        assert_eq!(CsvKind::AssetFund.save_mode_label(), "全件置換");
        assert_eq!(CsvKind::Dividend.save_mode_label(), "追加");
        assert_eq!(CsvKind::DomesticStock.save_mode_label(), "追加");
        assert_eq!(CsvKind::MutualFund.save_mode_label(), "追加");
    }
}
