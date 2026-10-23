//! 資産管理・取引明細あわせて5種類の CSV を1画面で扱う統合管理ページ。
//! `/assetbalance/csv`・`/receipts/csv`・`/csv` が同じ画面を開き、
//! `?kind=` で初期タブを選ぶ(旧 URL の種別値もそのまま解釈する)

use crate::app::{current_location, pathname_of, query_param};
use crate::features::asset_balance::{
    AssetBalanceCsvSource, AssetBalanceState, AssetKind, FundCsvPreview, StockCsvPreview,
    use_asset_balance_state,
};
use crate::features::csv_sync::CsvSyncPanel;
use crate::features::receipts::{
    ReceiptCsvSource, ReceiptsCsvPreview, ReceiptsStore, ReceiptsTab, ensure_kind,
    use_receipts_data,
};
use crate::session::use_session;
use crate::ui::csv_page::{CsvActionBar, CsvManageBody};
use crate::ui::page_frame::PageFrame;
use crate::ui::page_skeleton::SkeletonTabs;
use leptos::prelude::*;

/// 統合 CSV ページの5タブ。並びは Issue で指定された5種類の順
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CsvPageTab {
    AssetStock,
    AssetFund,
    Dividend,
    DomesticStock,
    MutualFund,
}

impl CsvPageTab {
    const ALL: [CsvPageTab; 5] = [
        Self::AssetStock,
        Self::AssetFund,
        Self::Dividend,
        Self::DomesticStock,
        Self::MutualFund,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::AssetStock => "資産管理(国内株式)",
            Self::AssetFund => "資産管理(投資信託)",
            Self::Dividend => "取引明細(配当金)",
            Self::DomesticStock => "取引明細(国内株式)",
            Self::MutualFund => "取引明細(投資信託)",
        }
    }

    /// 「対象」欄・保存済みカード・削除モーダルに出すデータ名
    fn subject(self) -> &'static str {
        self.receipt_tab()
            .map_or_else(|| self.label(), |receipt_tab| receipt_tab.label())
    }

    /// `?kind=` に出す値。取引明細は旧 `/receipts/csv` の値(dividend/stock/fund)を継ぐ
    fn kind_param(self) -> &'static str {
        match self {
            Self::AssetStock => "assetbalance",
            Self::AssetFund => "fundbalance",
            Self::Dividend => "dividend",
            Self::DomesticStock => "stock",
            Self::MutualFund => "fund",
        }
    }

    /// タブの id・testid に使う機械向けの値
    fn slug(self) -> &'static str {
        match self {
            Self::AssetStock => "assetbalance",
            Self::AssetFund => "fundbalance",
            Self::Dividend => "dividend",
            Self::DomesticStock => "domesticstock",
            Self::MutualFund => "mutualfund",
        }
    }

    fn receipt_tab(self) -> Option<ReceiptsTab> {
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

    /// 「一覧に戻る」の先
    fn list_href(self) -> &'static str {
        match self.receipt_tab() {
            Some(_) => "/receipts",
            None => "/assetbalance",
        }
    }

    /// パスと `?kind=` から開くタブを決める。
    /// 旧パスでは当時の kind 値をそのまま解釈する
    /// (`/receipts/csv?kind=fund` は取引明細の投資信託、`/assetbalance/csv?kind=fund` は資産管理の投資信託)
    fn from_location(location: &str) -> Self {
        let kind = query_param(location, "kind");
        match pathname_of(location) {
            "/assetbalance/csv" | "/assetbalance/csv/delete" => match kind.as_deref() {
                Some("fund") => Self::AssetFund,
                _ => Self::AssetStock,
            },
            "/receipts/csv" | "/receipts/csv/delete" => {
                match ReceiptsTab::from_kind_param(kind.as_deref()) {
                    ReceiptsTab::Dividend => Self::Dividend,
                    ReceiptsTab::DomesticStock => Self::DomesticStock,
                    ReceiptsTab::MutualFund => Self::MutualFund,
                }
            }
            _ => Self::ALL
                .iter()
                .copied()
                .find(|tab| Some(tab.kind_param()) == kind.as_deref())
                .unwrap_or(Self::AssetStock),
        }
    }
}

// タブ切替は履歴を増やさず URL だけ追従させる。path シグナルは更新しない
// (更新するとページ全体が再マウントされ、タブ移動のたびにフォーカスが本文へ飛ぶ)
fn replace_url_with_tab(tab: CsvPageTab) {
    let Some(history) = web_sys::window().and_then(|window| window.history().ok()) else {
        return;
    };
    let _ = history.replace_state_with_url(
        &wasm_bindgen::JsValue::NULL,
        "",
        Some(&format!("/csv?kind={}", tab.kind_param())),
    );
}

/// 5タブの切り替え行。取引明細ページのタブと同じ見た目・操作(キーボードの矢印移動)にする
#[component]
fn CsvPageTabs(tab: RwSignal<CsvPageTab>) -> impl IntoView {
    view! {
        <nav aria-label="CSV の種類">
            <div class="page-tab-list" role="tablist">
                {CsvPageTab::ALL
                    .iter()
                    .copied()
                    .map(|item| {
                        view! {
                            <button
                                type="button"
                                role="tab"
                                id=format!("csv-tab-{}", item.slug())
                                aria-controls=format!("csv-tabpanel-{}", item.slug())
                                aria-selected=move || (tab.get() == item).to_string()
                                tabindex=move || if tab.get() == item { "0" } else { "-1" }
                                class="page-tab"
                                on:click=move |_| {
                                    tab.set(item);
                                    replace_url_with_tab(item);
                                }
                                on:keydown=move |event| {
                                    let current = CsvPageTab::ALL
                                        .iter()
                                        .position(|entry| *entry == tab.get_untracked())
                                        .unwrap_or(0);
                                    let count = CsvPageTab::ALL.len();
                                    let next = match event.key().as_str() {
                                        "ArrowRight" => Some((current + 1) % count),
                                        "ArrowLeft" => Some((current + count - 1) % count),
                                        "Home" => Some(0),
                                        "End" => Some(count - 1),
                                        _ => None,
                                    };
                                    if let Some(next) = next {
                                        event.prevent_default();
                                        let next_tab = CsvPageTab::ALL[next];
                                        tab.set(next_tab);
                                        replace_url_with_tab(next_tab);
                                        if let Some(element) = web_sys::window()
                                            .and_then(|window| window.document())
                                            .and_then(|document| {
                                                document.get_element_by_id(&format!(
                                                    "csv-tab-{}",
                                                    next_tab.slug()
                                                ))
                                            })
                                        {
                                            use wasm_bindgen::JsCast;
                                            let _ = element
                                                .dyn_ref::<web_sys::HtmlElement>()
                                                .map(|element| element.focus());
                                        }
                                    }
                                }
                            >
                                {item.label()}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
        </nav>
    }
}

/// 「一覧に戻る」が同じ種別のタブを開くよう、一覧側ストアの選択状態を CSV 側のタブにそろえる
fn ensure_list_tab(asset: AssetBalanceState, receipts: ReceiptsStore, tab: CsvPageTab) {
    if let Some(receipt_tab) = tab.receipt_tab() {
        ensure_kind(receipts, receipt_tab);
    } else if let Some(kind) = tab.asset_kind() {
        asset.kind.set(kind);
    }
}

#[component]
fn ReceiptCsvSection(
    store: ReceiptsStore,
    receipt_tab: ReceiptsTab,
    tab: CsvPageTab,
) -> impl IntoView {
    let source = ReceiptCsvSource {
        store,
        tab: receipt_tab,
    };
    view! {
        <CsvManageBody
            source=source
            subject=tab.subject().to_string()
            list_href=tab.list_href()
            preview=view! { <ReceiptsCsvPreview store=store tab=receipt_tab /> }.into_any()
        />
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
    // 再訪ではストアのキャッシュがそのまま出るので、鮮度だけ裏で更新する
    receipts.revisit();
    // ?kind= はページの再マウントでしか変わらないため、マウント時に1回だけ読む
    let tab = RwSignal::new(CsvPageTab::from_location(&current_location()));

    // 一覧側の選択タブを CSV 側の種別にそろえ、「一覧に戻る」が同じタブを開く。
    // 世代リセット(ユーザー切替)後はストア側の Effect が選択済みタブを再取得する
    Effect::new(move |_| {
        ensure_list_tab(asset_state, receipts, tab.get());
    });

    let action_bar = view! {
        {move || match tab.get() {
            CsvPageTab::AssetStock => view! {
                <CsvActionBar source=AssetBalanceCsvSource { store: asset_state.csv, kind: AssetKind::Stock } />
            }
                .into_any(),
            CsvPageTab::AssetFund => view! {
                <CsvActionBar source=AssetBalanceCsvSource { store: asset_state.csv, kind: AssetKind::Fund } />
            }
                .into_any(),
            current => {
                let receipt_tab = current.receipt_tab().unwrap_or(ReceiptsTab::Dividend);
                view! {
                    <CsvActionBar source=ReceiptCsvSource { store: receipts, tab: receipt_tab } />
                }
                    .into_any()
            }
        }}
    }
    .into_any();

    view! {
        <PageFrame title="CSV管理" action_bar=action_bar action_bar_pc_top=true>
            <div class="flex flex-col gap-4">
                <CsvSyncPanel />
                <SkeletonTabs>
                    <CsvPageTabs tab=tab />
                </SkeletonTabs>
                {move || {
                    let current = tab.get();
                    view! {
                        <div
                            id=format!("csv-tabpanel-{}", current.slug())
                            role="tabpanel"
                            aria-labelledby=format!("csv-tab-{}", current.slug())
                        >
                            {match current {
                                CsvPageTab::AssetStock => view! {
                                    <CsvManageBody
                                        source=AssetBalanceCsvSource { store: asset_state.csv, kind: AssetKind::Stock }
                                        subject=current.subject().to_string()
                                        list_href=current.list_href()
                                        preview=view! { <StockCsvPreview store=asset_state.csv /> }.into_any()
                                        delete_button_label="国内株式をすべて削除…".to_string()
                                    />
                                }
                                    .into_any(),
                                CsvPageTab::AssetFund => view! {
                                    <CsvManageBody
                                        source=AssetBalanceCsvSource { store: asset_state.csv, kind: AssetKind::Fund }
                                        subject=current.subject().to_string()
                                        list_href=current.list_href()
                                        preview=view! { <FundCsvPreview store=asset_state.csv /> }.into_any()
                                        delete_button_label="投資信託をすべて削除…".to_string()
                                    />
                                }
                                    .into_any(),
                                _ => {
                                    let receipt_tab = current.receipt_tab().unwrap_or(ReceiptsTab::Dividend);
                                    view! {
                                        <ReceiptCsvSection
                                            store=receipts
                                            receipt_tab=receipt_tab
                                            tab=current
                                        />
                                    }
                                        .into_any()
                                }
                            }}
                        </div>
                    }
                }}
            </div>
        </PageFrame>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::SessionStore;

    /// 引数は `location.pathname + location.search` 相当の文字列
    #[test]
    fn from_location_interprets_legacy_kind_per_path() {
        // 旧 /receipts/csv の kind 値は取引明細の種別
        assert_eq!(
            CsvPageTab::from_location("/receipts/csv"),
            CsvPageTab::Dividend
        );
        assert_eq!(
            CsvPageTab::from_location("/receipts/csv?kind=dividend"),
            CsvPageTab::Dividend
        );
        assert_eq!(
            CsvPageTab::from_location("/receipts/csv?kind=stock"),
            CsvPageTab::DomesticStock
        );
        assert_eq!(
            CsvPageTab::from_location("/receipts/csv?kind=fund"),
            CsvPageTab::MutualFund
        );
        assert_eq!(
            CsvPageTab::from_location("/receipts/csv/delete?kind=fund"),
            CsvPageTab::MutualFund
        );

        // 旧 /assetbalance/csv では kind=fund が資産管理の投資信託
        assert_eq!(
            CsvPageTab::from_location("/assetbalance/csv"),
            CsvPageTab::AssetStock
        );
        assert_eq!(
            CsvPageTab::from_location("/assetbalance/csv?kind=fund"),
            CsvPageTab::AssetFund
        );
        assert_eq!(
            CsvPageTab::from_location("/assetbalance/csv/delete?kind=fund"),
            CsvPageTab::AssetFund
        );
        // 資産管理側にない種別値は国内株式に戻す
        assert_eq!(
            CsvPageTab::from_location("/assetbalance/csv?kind=dividend"),
            CsvPageTab::AssetStock
        );
    }

    #[test]
    fn from_location_matches_canonical_kind_params() {
        assert_eq!(CsvPageTab::from_location("/csv"), CsvPageTab::AssetStock);
        assert_eq!(
            CsvPageTab::from_location("/csv?kind=assetbalance"),
            CsvPageTab::AssetStock
        );
        assert_eq!(
            CsvPageTab::from_location("/csv?kind=fundbalance"),
            CsvPageTab::AssetFund
        );
        assert_eq!(
            CsvPageTab::from_location("/csv?kind=dividend"),
            CsvPageTab::Dividend
        );
        assert_eq!(
            CsvPageTab::from_location("/csv?kind=stock"),
            CsvPageTab::DomesticStock
        );
        assert_eq!(
            CsvPageTab::from_location("/csv?kind=fund"),
            CsvPageTab::MutualFund
        );
        assert_eq!(
            CsvPageTab::from_location("/csv?kind=unknown"),
            CsvPageTab::AssetStock
        );
    }

    /// 「一覧に戻る」が CSV 側で開いていた種別のタブを再現できるよう、
    /// 資産管理タブを開いたときは一覧側の kind も同じ種別にそろう
    #[test]
    fn ensure_list_tab_syncs_asset_kind_for_asset_tabs() {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            let asset = use_asset_balance_state(session);
            let receipts = use_receipts_data(session, RwSignal::new(ReceiptsTab::Dividend));

            asset.kind.set(AssetKind::Stock);
            ensure_list_tab(asset, receipts, CsvPageTab::AssetFund);
            assert_eq!(asset.kind.get_untracked(), AssetKind::Fund);

            asset.kind.set(AssetKind::Fund);
            ensure_list_tab(asset, receipts, CsvPageTab::AssetStock);
            assert_eq!(asset.kind.get_untracked(), AssetKind::Stock);
        });
    }

    /// 取引明細タブでは資産管理側の選択を変えない(取引明細側だけ同期する)
    #[test]
    fn ensure_list_tab_leaves_asset_kind_alone_for_receipt_tabs() {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            let asset = use_asset_balance_state(session);
            let receipts = use_receipts_data(session, RwSignal::new(ReceiptsTab::Dividend));

            asset.kind.set(AssetKind::Fund);
            ensure_list_tab(asset, receipts, CsvPageTab::MutualFund);
            assert_eq!(asset.kind.get_untracked(), AssetKind::Fund);
            assert_eq!(receipts.active_tab.get_untracked(), ReceiptsTab::MutualFund);
        });
    }
}
