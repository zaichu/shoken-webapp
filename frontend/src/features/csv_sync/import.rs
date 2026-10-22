//! 受け取った CSV の自動取り込み。
//! 既存の CSV 管理と同じ preview → import の API に流す。
//! 画面の保存操作を通らないため、安全装置の判定と件数激減の確認はここで行う

use super::{CsvKind, CsvSyncStore, KindStatus, ReceivedCsv, SkipReason, guard};
use crate::api::ApiClient;
use crate::api::dto::{CsvPreviewResponse, CsvUploadResponse, PaginatedSearchResponse};
use crate::features::asset_balance::{
    CSV_CHUNKING, FUND_CSV_CHUNKING, FUND_IMPORT_PATH, FUND_LIST_PATH, FUND_PREVIEW_PATH,
    IMPORT_PATH, LIST_PATH, PREVIEW_PATH,
};
use crate::features::receipts::{ReceiptsTab, chunking_for};
use crate::support::csv_flow::{CsvChunking, preview_csv, upload_csv_while};
use leptos::prelude::*;

/// 種類ごとの取り込み経路。パス・分割・保存方式は既存の CSV 管理の定義そのまま
struct ImportPlan {
    preview_path: &'static str,
    import_path: &'static str,
    /// 全件置換の対象だけ、激減判定のため保存済み件数を取る一覧 API
    count_path: Option<&'static str>,
    chunking: CsvChunking,
    /// 全件置換か(行エラー時の中止・激減確認の対象)。明細は追加なので対象外
    replace: bool,
    /// 取り込み後に再取得する取引明細タブ
    receipts_tab: Option<ReceiptsTab>,
}

fn plan_of(kind: CsvKind) -> ImportPlan {
    match kind {
        CsvKind::AssetBalance => ImportPlan {
            preview_path: PREVIEW_PATH,
            import_path: IMPORT_PATH,
            count_path: Some(LIST_PATH),
            chunking: CSV_CHUNKING,
            replace: true,
            receipts_tab: None,
        },
        CsvKind::FundBalance => ImportPlan {
            preview_path: FUND_PREVIEW_PATH,
            import_path: FUND_IMPORT_PATH,
            count_path: Some(FUND_LIST_PATH),
            chunking: FUND_CSV_CHUNKING,
            replace: true,
            receipts_tab: None,
        },
        CsvKind::Dividend => receipts_plan(ReceiptsTab::Dividend),
        CsvKind::DomesticStock => receipts_plan(ReceiptsTab::DomesticStock),
        CsvKind::MutualFund => receipts_plan(ReceiptsTab::MutualFund),
    }
}

/// 取り込み後に再取得する明細タブ(保有残高は別の一覧を持つので None)
pub(crate) fn receipts_tab_of(kind: CsvKind) -> Option<ReceiptsTab> {
    plan_of(kind).receipts_tab
}

fn receipts_plan(tab: ReceiptsTab) -> ImportPlan {
    ImportPlan {
        preview_path: tab.preview_path(),
        import_path: tab.import_path(),
        count_path: None,
        chunking: chunking_for(tab),
        replace: false,
        receipts_tab: Some(tab),
    }
}

/// 種類ごとの取り込み結果
enum Outcome {
    Imported(CsvUploadResponse),
    Skipped(SkipReason),
    Failed(String),
    /// 件数激減の確認に移す(検証を通った行数と保存済み件数)
    NeedsConfirm {
        new_rows: usize,
        saved_rows: usize,
    },
    /// 応答待ちの間に実行・セッションが変わった。状態に何も書かず終わる
    Stale,
}

/// 激減の確認待ち。回答あれば残りを続ける
pub(crate) struct PendingImport {
    pub(crate) csv: ReceivedCsv,
    /// 確認の材料になった新しい行数(検証の有効行)と保存済み件数
    pub(crate) new_rows: usize,
    pub(crate) saved_rows: usize,
    pub(crate) rest: Vec<ReceivedCsv>,
    pub(crate) epoch: u64,
    pub(crate) generation: crate::session::Generation,
}

/// 受け取った全件を種類順に処理する。確認待ちになったら pending に残して止まる
pub(crate) async fn process(
    store: CsvSyncStore,
    epoch: u64,
    generation: crate::session::Generation,
    csvs: Vec<ReceivedCsv>,
    confirmed: Option<CsvKind>,
) {
    for (index, csv) in csvs.iter().enumerate() {
        if !store.alive(epoch, generation) {
            return;
        }
        store.set_kind(csv.kind, KindStatus::Importing);
        let outcome = import_csv(store, epoch, generation, csv, confirmed == Some(csv.kind)).await;
        // 応答待ちの間に実行・セッションが変わっていれば旧側の結果は何も書かない
        if !store.alive(epoch, generation) {
            return;
        }
        match outcome {
            Outcome::Imported(result) => {
                store.state.update(|state| {
                    state.set_kind(csv.kind, KindStatus::Imported { result });
                    state.import.imported.push(csv.kind);
                });
            }
            Outcome::Skipped(reason) => {
                store.set_kind(csv.kind, KindStatus::Skipped { reason });
            }
            Outcome::Failed(message) => {
                store.set_kind(csv.kind, KindStatus::Failed { reason: message });
            }
            Outcome::NeedsConfirm {
                new_rows,
                saved_rows,
            } => {
                store.hold_confirm(PendingImport {
                    csv: csv.clone(),
                    new_rows,
                    saved_rows,
                    rest: csvs[index + 1..].to_vec(),
                    epoch,
                    generation,
                });
                return;
            }
            Outcome::Stale => return,
        }
    }
    if store.alive(epoch, generation) {
        store.finish_import();
    }
}

/// 安全装置 → 検証(preview) → 取り込み(import)。
/// confirmed は激減確認「はい」のあとの再実行で、件数判定を飛ばす。
/// 応答待ちの間に実行やセッションが切り替わると Stale を返し、変更系の
/// upload は送らない(旧ユーザーの CSV が新しいセッションで投げられないように)
async fn import_csv(
    store: CsvSyncStore,
    epoch: u64,
    generation: crate::session::Generation,
    csv: &ReceivedCsv,
    confirmed: bool,
) -> Outcome {
    if let Some(reason) = guard::inspect(csv.kind, &csv.content) {
        return Outcome::Skipped(reason);
    }
    let Some(file) = csv_to_file(csv) else {
        return Outcome::Failed(crate::ui::labels::CSV_SYNC_FILE_FAILED.to_string());
    };
    let plan = plan_of(csv.kind);
    let preview: CsvPreviewResponse =
        match preview_csv(plan.preview_path, &file, plan.chunking).await {
            Ok(preview) => preview,
            Err(error) => return Outcome::Failed(error.message()),
        };
    if !store.alive(epoch, generation) {
        return Outcome::Stale;
    }
    if preview.valid_rows == 0 {
        return Outcome::Skipped(SkipReason::NoValidRows);
    }
    // 全件置換は行エラーのあるファイルで上書きすると保存済み分が消えるため中止する
    if plan.replace && !preview.errors.is_empty() {
        return Outcome::Skipped(SkipReason::RowErrors);
    }
    if plan.replace && !confirmed {
        let Some(count_path) = plan.count_path else {
            return Outcome::Skipped(SkipReason::CountUnknown);
        };
        match saved_total(count_path).await {
            Ok(saved) if guard::is_plunge(preview.valid_rows, saved) => {
                return Outcome::NeedsConfirm {
                    new_rows: preview.valid_rows,
                    saved_rows: saved,
                };
            }
            Ok(_) => {}
            Err(_) => return Outcome::Skipped(SkipReason::CountUnknown),
        }
        if !store.alive(epoch, generation) {
            return Outcome::Stale;
        }
    }
    match upload_csv_while(plan.import_path, &file, plan.chunking, move || {
        store.alive(epoch, generation)
    })
    .await
    {
        Ok(result) => Outcome::Imported(result),
        // 中断(世代切替)で返った分は旧実行の結果なので画面に出さない
        Err(_) if !store.alive(epoch, generation) => Outcome::Stale,
        Err(error) => Outcome::Failed(error.message()),
    }
}

/// 受け取った本文を multipart 送信できる File にする。
/// 拡張子ガードに合わせて名前は必ず .csv を付ける
fn csv_to_file(csv: &ReceivedCsv) -> Option<web_sys::File> {
    web_sys::window()?;
    let array = js_sys::Uint8Array::from(csv.content.as_bytes());
    let parts = js_sys::Array::of1(&array);
    web_sys::File::new_with_u8_array_sequence(&parts, &file_name(csv.kind)).ok()
}

fn file_name(kind: CsvKind) -> String {
    format!("sync-{}.csv", kind.request_value())
}

/// 激減判定のための保存済み件数。一覧 API の total を1行だけ取る
async fn saved_total(path: &'static str) -> Result<usize, crate::api::ApiError> {
    let response: PaginatedSearchResponse<serde_json::Value, serde_json::Value> =
        ApiClient::read_client()
            .get_json(
                path,
                &[
                    ("page", "1"),
                    ("per_page", "1"),
                    ("include_summary", "false"),
                ],
            )
            .await?;
    Ok(response.total.max(0) as usize)
}
