use crate::api::{ApiClient, ApiError};
use crate::dto::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
use crate::receipts::ReceiptsTab;
use crate::receipts_domain::{format_currency, format_date, format_number};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CsvPreview {
    pub total_rows: usize,
    pub valid_rows: usize,
    pub errors: Vec<CsvRowError>,
    pub rows: Vec<CsvPreviewRow>,
}

// 画面への接続は層2
#[allow(dead_code)]
impl CsvPreview {
    pub fn summary_text(&self) -> String {
        let base = format!("{}件 追加で保存されます", self.valid_rows);
        if self.errors.is_empty() {
            base
        } else {
            format!("{base} / {}件エラー", self.errors.len())
        }
    }
}

// プレビュー行は backend が Create*Request をシリアライズしたもので、id・タイムスタンプを持たない。
// React の transformDB* が欠損を 0/空文字で埋めるのと同じく、全フィールドを lenient に受け取る
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct DividendCsvRow {
    #[serde(default)]
    pub settlement_date: String,
    #[serde(default)]
    pub product: String,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub security_code: String,
    #[serde(default)]
    pub security_name: String,
    #[serde(default)]
    pub unit_price: Decimal,
    #[serde(default)]
    pub shares: Decimal,
    #[serde(default)]
    pub dividends_before_tax: Decimal,
    #[serde(default)]
    pub taxes: Decimal,
    #[serde(default)]
    pub net_amount_received: Decimal,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct DomesticStockCsvRow {
    #[serde(default)]
    pub trade_date: String,
    #[serde(default)]
    pub settlement_date: String,
    #[serde(default)]
    pub security_code: String,
    #[serde(default)]
    pub security_name: String,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub shares: Decimal,
    #[serde(default)]
    pub asked_price: Decimal,
    #[serde(default)]
    pub proceeds: Decimal,
    #[serde(default)]
    pub purchase_price: Decimal,
    #[serde(default)]
    pub realized_profit_and_loss: Decimal,
    #[serde(default)]
    pub taxes: Decimal,
    #[serde(default)]
    pub realized_profit_and_loss_after_tax: Decimal,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct MutualfundCsvRow {
    #[serde(default)]
    pub trade_date: String,
    #[serde(default)]
    pub settlement_date: String,
    #[serde(default)]
    pub fund_name: String,
    #[serde(default)]
    pub dividends: Option<String>,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub shares: Decimal,
    #[serde(default)]
    pub exchange_rate: Decimal,
    #[serde(default)]
    pub cancellation_unit_price_yen: Decimal,
    #[serde(default)]
    pub cancellation_amount_yen: Decimal,
    #[serde(default)]
    pub average_acquisition_price_yen: Decimal,
    #[serde(default)]
    pub realized_profit_and_loss: Decimal,
    #[serde(default)]
    pub taxes: Decimal,
    #[serde(default)]
    pub realized_profit_and_loss_after_tax: Decimal,
}

// 画面への接続は層2(層1では variant を構築する箇所が非同期境界の内側だけ)
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum CsvPreviewRow {
    Dividend(DividendCsvRow),
    DomesticStock(DomesticStockCsvRow),
    MutualFund(MutualfundCsvRow),
}

// 画面への接続は層2
#[allow(dead_code)]
impl CsvPreviewRow {
    // ReceiptItem::cells と同じ列順。プレビュー表は一覧と同じカラムで表示する
    pub fn cells(&self) -> Vec<String> {
        match self {
            CsvPreviewRow::Dividend(row) => vec![
                format_date(&row.settlement_date),
                row.product.clone(),
                row.account.clone(),
                row.security_code.clone(),
                row.security_name.clone(),
                format_currency(row.unit_price),
                format_number(row.shares, 2),
                format_currency(row.dividends_before_tax),
                format_currency(row.taxes),
                format_currency(row.net_amount_received),
            ],
            CsvPreviewRow::DomesticStock(row) => vec![
                format_date(&row.trade_date),
                row.security_code.clone(),
                row.security_name.clone(),
                row.account.clone(),
                format_number(row.shares, 2),
                format_currency(row.asked_price),
                format_currency(row.proceeds),
                format_currency(row.purchase_price),
                format_currency(row.realized_profit_and_loss),
                format_currency(row.taxes),
                format_currency(row.realized_profit_and_loss_after_tax),
            ],
            CsvPreviewRow::MutualFund(row) => vec![
                format_date(&row.trade_date),
                row.fund_name.clone(),
                row.account.clone(),
                format_number(row.shares, 2),
                format_currency(row.cancellation_unit_price_yen),
                format_currency(row.cancellation_amount_yen),
                format_currency(row.average_acquisition_price_yen),
                format_currency(row.realized_profit_and_loss),
                format_currency(row.taxes),
                format_currency(row.realized_profit_and_loss_after_tax),
            ],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CsvTabState {
    pub file_name: Option<String>,
    pub preview: Option<CsvPreview>,
    pub import_result: Option<CsvUploadResponse>,
    pub error: Option<String>,
    pub previewing: bool,
    pub saving: bool,
    pub deleting: bool,
    pub show_delete_confirm: bool,
}

// 画面への接続は層2
#[allow(dead_code)]
impl CsvTabState {
    pub fn busy(&self) -> bool {
        self.previewing || self.saving || self.deleting
    }

    pub fn begin_preview(&mut self, file_name: String) -> bool {
        if self.busy() {
            return false;
        }
        self.file_name = Some(file_name);
        self.preview = None;
        self.import_result = None;
        self.error = None;
        self.previewing = true;
        true
    }

    // React はプレビュー失敗を画面に出さない(ファイル選択は残り preview のみ未設定)
    pub fn finish_preview(&mut self, preview: Option<CsvPreview>) {
        self.previewing = false;
        if let Some(preview) = preview {
            self.preview = Some(preview);
        }
    }

    pub fn begin_save(&mut self) -> bool {
        if self.busy() {
            return false;
        }
        self.error = None;
        self.saving = true;
        true
    }

    pub fn finish_save(&mut self, result: Result<CsvUploadResponse, String>) {
        self.saving = false;
        match result {
            Ok(result) => {
                self.file_name = None;
                // React の SET_IMPORT_RESULT はプレビュー表示を残す
                self.import_result = Some(result);
            }
            Err(message) => self.error = Some(message),
        }
    }

    pub fn open_delete_confirm(&mut self) {
        self.show_delete_confirm = true;
    }

    pub fn close_delete_confirm(&mut self) {
        self.show_delete_confirm = false;
    }

    // React はモーダル内の確定ボタンからのみ呼ぶ。画面配線前でも誤起動しないよう確認表示中だけ開始する
    pub fn begin_delete(&mut self) -> bool {
        if self.deleting || !self.show_delete_confirm {
            return false;
        }
        self.show_delete_confirm = false;
        self.error = None;
        self.deleting = true;
        true
    }

    pub fn finish_delete(&mut self, result: Result<(), String>) {
        self.deleting = false;
        match result {
            Ok(()) => self.import_result = None,
            Err(message) => self.error = Some(message),
        }
    }

    pub fn save_label(&self) -> String {
        if self.saving {
            "保存中...".to_string()
        } else if self.previewing {
            "解析中...".to_string()
        } else {
            self.preview
                .as_ref()
                .map(|preview| format!("{}件 追加で保存", preview.valid_rows))
                .unwrap_or_else(|| "追加で保存".to_string())
        }
    }

    pub fn delete_label(&self, db_count: usize) -> String {
        if self.deleting {
            "削除中...".to_string()
        } else {
            format!("全件削除 ({db_count}件)")
        }
    }
}

// 画面への接続は層2
#[allow(dead_code)]
impl CsvUploadResponse {
    pub fn inserted_text(&self) -> String {
        format!("{}件反映", self.inserted)
    }

    pub fn skipped_text(&self) -> Option<String> {
        (self.skipped > 0).then(|| format!("{}件スキップ", self.skipped))
    }

    pub fn error_count_text(&self) -> Option<String> {
        (!self.errors.is_empty()).then(|| format!("{}件エラー", self.errors.len()))
    }
}

// 画面への接続は層2
#[allow(dead_code)]
pub fn row_error_text(error: &CsvRowError) -> String {
    format!("{}行目: {}", error.row, error.message)
}

// 画面への接続は層2
#[allow(dead_code)]
pub fn to_preview(tab: ReceiptsTab, response: CsvPreviewResponse) -> CsvPreview {
    let rows = response
        .rows
        .into_iter()
        .map(|row| match tab {
            ReceiptsTab::Dividend => {
                CsvPreviewRow::Dividend(serde_json::from_value(row).unwrap_or_default())
            }
            ReceiptsTab::DomesticStock => {
                CsvPreviewRow::DomesticStock(serde_json::from_value(row).unwrap_or_default())
            }
            ReceiptsTab::MutualFund => {
                CsvPreviewRow::MutualFund(serde_json::from_value(row).unwrap_or_default())
            }
        })
        .collect();
    CsvPreview {
        total_rows: response.total_rows,
        valid_rows: response.valid_rows,
        errors: response.errors,
        rows,
    }
}

// React は mutation エラーに ApiError.message(HTTP ステータス別の既定文)を使う。
// user_message() とは別の文言体系なので専用に写す
// 画面への接続は層2
#[allow(dead_code)]
pub fn csv_error_message(error: &ApiError) -> String {
    match error {
        ApiError::Network => "ネットワークエラーが発生しました".to_string(),
        ApiError::Timeout => "リクエストがタイムアウトしました".to_string(),
        ApiError::Parse => "応答の解析に失敗しました".to_string(),
        ApiError::Http { status } => match status {
            400 => "リクエストが不正です".to_string(),
            401 => "認証が必要です".to_string(),
            403 => "アクセスが拒否されました".to_string(),
            404 => "リソースが見つかりません".to_string(),
            500..=599 => "サーバーエラーが発生しました".to_string(),
            _ => format!("エラーが発生しました (ステータス: {status})"),
        },
    }
}

// 画面への接続は層2
#[allow(dead_code)]
fn csv_form_data(file: &web_sys::File) -> Result<web_sys::FormData, ApiError> {
    let form = web_sys::FormData::new().map_err(|_| ApiError::Network)?;
    form.append_with_blob("file", file)
        .map_err(|_| ApiError::Network)?;
    Ok(form)
}

// 画面への接続は層2
#[allow(dead_code)]
pub async fn preview_csv(
    tab: ReceiptsTab,
    file: &web_sys::File,
) -> Result<CsvPreviewResponse, ApiError> {
    ApiClient::default_client()
        .post_multipart(tab.preview_path(), &csv_form_data(file)?)
        .await
}

// 画面への接続は層2
#[allow(dead_code)]
pub async fn upload_csv(
    tab: ReceiptsTab,
    file: &web_sys::File,
) -> Result<CsvUploadResponse, ApiError> {
    ApiClient::default_client()
        .post_multipart(tab.import_path(), &csv_form_data(file)?)
        .await
}

// 画面への接続は層2
#[allow(dead_code)]
pub async fn delete_all(tab: ReceiptsTab) -> Result<(), ApiError> {
    ApiClient::default_client()
        .delete_empty(tab.list_path())
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn preview_response(rows: Vec<serde_json::Value>) -> CsvPreviewResponse {
        CsvPreviewResponse {
            total_rows: 2,
            valid_rows: 1,
            errors: vec![CsvRowError {
                row: 2,
                message: "入金日の形式が不正です".to_string(),
            }],
            rows,
        }
    }

    #[test]
    fn preview_response_maps_to_typed_rows_per_tab() {
        let dividend = preview_response(vec![serde_json::json!({
            "settlement_date": "2024-03-01",
            "product": "特定口座",
            "account": "SBI証券",
            "security_code": "7203",
            "security_name": "トヨタ自動車",
            "unit_price": 30.0,
            "shares": 100,
            "dividends_before_tax": 3000,
            "taxes": 609,
            "net_amount_received": 2391
        })]);
        let preview = to_preview(ReceiptsTab::Dividend, dividend);
        assert_eq!((preview.total_rows, preview.valid_rows), (2, 1));
        assert_eq!(preview.errors.len(), 1);
        let CsvPreviewRow::Dividend(row) = &preview.rows[0] else {
            panic!("dividend row expected")
        };
        assert_eq!(row.security_name, "トヨタ自動車");
        assert_eq!(row.unit_price, dec!(30.0));
        assert_eq!(row.shares, dec!(100));

        let domestic = preview_response(vec![serde_json::json!({
            "trade_date": "2024-01-15",
            "settlement_date": "2024-01-17",
            "security_code": "1301",
            "security_name": "極洋",
            "account": "特定",
            "shares": 100,
            "asked_price": 1500.5,
            "proceeds": 150050,
            "purchase_price": 1400.25,
            "realized_profit_and_loss": 10000.1,
            "taxes": 2031.5,
            "realized_profit_and_loss_after_tax": 7968.6
        })]);
        let preview = to_preview(ReceiptsTab::DomesticStock, domestic);
        let CsvPreviewRow::DomesticStock(row) = &preview.rows[0] else {
            panic!("domestic stock row expected")
        };
        assert_eq!(row.security_code, "1301");
        assert_eq!(row.proceeds, dec!(150050));

        let mutualfund = preview_response(vec![serde_json::json!({
            "trade_date": "2024-01-15",
            "settlement_date": "2024-01-17",
            "fund_name": "eMAXIS Slim 全世界株式",
            "dividends": "再投資型",
            "account": "楽天証券",
            "shares": 10000,
            "exchange_rate": 150.25,
            "cancellation_unit_price_yen": 12345,
            "cancellation_amount_yen": 120000,
            "average_acquisition_price_yen": 11000.5,
            "realized_profit_and_loss": 12000,
            "taxes": 2437,
            "realized_profit_and_loss_after_tax": 9563
        })]);
        let preview = to_preview(ReceiptsTab::MutualFund, mutualfund);
        let CsvPreviewRow::MutualFund(row) = &preview.rows[0] else {
            panic!("mutualfund row expected")
        };
        assert_eq!(row.fund_name, "eMAXIS Slim 全世界株式");
        assert_eq!(row.dividends.as_deref(), Some("再投資型"));
        assert_eq!(row.cancellation_amount_yen, dec!(120000));
    }

    #[test]
    fn preview_row_with_missing_fields_falls_back_like_react() {
        let response = preview_response(vec![
            serde_json::Value::Null,
            serde_json::json!({"security_name": "トヨタ自動車"}),
        ]);
        let preview = to_preview(ReceiptsTab::Dividend, response);
        assert_eq!(preview.rows.len(), 2);
        let CsvPreviewRow::Dividend(row) = &preview.rows[1] else {
            panic!("dividend row expected")
        };
        assert_eq!(row.security_name, "トヨタ自動車");
        assert_eq!(row.unit_price, dec!(0));
    }

    #[test]
    fn preview_row_cells_match_list_columns() {
        let row = CsvPreviewRow::Dividend(DividendCsvRow {
            settlement_date: "2024-03-01".to_string(),
            product: "特定口座".to_string(),
            account: "SBI証券".to_string(),
            security_code: "7203".to_string(),
            security_name: "トヨタ自動車".to_string(),
            unit_price: dec!(30.0),
            shares: dec!(100),
            dividends_before_tax: dec!(3000),
            taxes: dec!(609),
            net_amount_received: dec!(2391),
        });
        assert_eq!(
            row.cells(),
            vec![
                "2024/03/01",
                "特定口座",
                "SBI証券",
                "7203",
                "トヨタ自動車",
                "¥ 30",
                "100",
                "¥ 3,000",
                "¥ 609",
                "¥ 2,391",
            ]
        );
    }

    #[test]
    fn upload_response_deserializes() {
        let response: CsvUploadResponse = serde_json::from_str(
            r#"{"inserted":2,"skipped":1,"errors":[{"row":5,"message":"重複"}]}"#,
        )
        .expect("deserialize");
        assert_eq!(
            (response.inserted, response.skipped, response.errors.len()),
            (2, 1, 1)
        );
        assert_eq!(response.errors[0].row, 5);
    }

    #[test]
    fn csv_error_message_matches_react() {
        for (error, expected) in [
            (ApiError::Network, "ネットワークエラーが発生しました"),
            (ApiError::Timeout, "リクエストがタイムアウトしました"),
            (ApiError::Parse, "応答の解析に失敗しました"),
            (ApiError::Http { status: 400 }, "リクエストが不正です"),
            (ApiError::Http { status: 401 }, "認証が必要です"),
            (ApiError::Http { status: 403 }, "アクセスが拒否されました"),
            (ApiError::Http { status: 404 }, "リソースが見つかりません"),
            (
                ApiError::Http { status: 500 },
                "サーバーエラーが発生しました",
            ),
            (
                ApiError::Http { status: 503 },
                "サーバーエラーが発生しました",
            ),
            (
                ApiError::Http { status: 418 },
                "エラーが発生しました (ステータス: 418)",
            ),
        ] {
            assert_eq!(csv_error_message(&error), expected);
        }
    }

    #[test]
    fn save_label_matches_react() {
        let mut state = CsvTabState::default();
        assert_eq!(state.save_label(), "追加で保存");
        state.preview = Some(CsvPreview {
            valid_rows: 3,
            ..Default::default()
        });
        assert_eq!(state.save_label(), "3件 追加で保存");
        state.previewing = true;
        assert_eq!(state.save_label(), "解析中...");
        state.saving = true;
        assert_eq!(state.save_label(), "保存中...");
    }

    #[test]
    fn delete_label_matches_react() {
        let mut state = CsvTabState::default();
        assert_eq!(state.delete_label(5), "全件削除 (5件)");
        state.deleting = true;
        assert_eq!(state.delete_label(5), "削除中...");
    }

    #[test]
    fn select_file_resets_preview_result_and_error() {
        let mut state = CsvTabState {
            preview: Some(CsvPreview::default()),
            import_result: Some(CsvUploadResponse {
                inserted: 1,
                skipped: 0,
                errors: vec![],
            }),
            error: Some("失敗".to_string()),
            ..Default::default()
        };
        assert!(state.begin_preview("new.csv".to_string()));
        assert_eq!(state.file_name.as_deref(), Some("new.csv"));
        assert!(state.preview.is_none());
        assert!(state.import_result.is_none());
        assert!(state.error.is_none());
        assert!(state.previewing);
    }

    #[test]
    fn busy_state_blocks_begin_operations() {
        for busy in [
            CsvTabState {
                previewing: true,
                ..Default::default()
            },
            CsvTabState {
                saving: true,
                ..Default::default()
            },
            CsvTabState {
                deleting: true,
                ..Default::default()
            },
        ] {
            let mut state = busy;
            assert!(!state.begin_preview("a.csv".to_string()));
            assert!(!state.begin_save());
        }
    }

    #[test]
    fn save_success_keeps_preview_and_sets_result() {
        let mut state = CsvTabState {
            file_name: Some("a.csv".to_string()),
            preview: Some(CsvPreview::default()),
            saving: true,
            ..Default::default()
        };
        state.finish_save(Ok(CsvUploadResponse {
            inserted: 2,
            skipped: 1,
            errors: vec![],
        }));
        assert!(!state.saving);
        assert!(state.file_name.is_none());
        assert!(
            state.preview.is_some(),
            "React の SET_IMPORT_RESULT はプレビューを残す"
        );
        assert_eq!(
            state.import_result.as_ref().map(|result| result.inserted),
            Some(2)
        );
    }

    #[test]
    fn save_error_keeps_file_and_sets_message() {
        let mut state = CsvTabState {
            file_name: Some("a.csv".to_string()),
            preview: Some(CsvPreview::default()),
            saving: true,
            ..Default::default()
        };
        state.finish_save(Err("認証が必要です".to_string()));
        assert!(!state.saving);
        assert_eq!(state.file_name.as_deref(), Some("a.csv"));
        assert!(state.preview.is_some());
        assert_eq!(state.error.as_deref(), Some("認証が必要です"));
    }

    #[test]
    fn delete_confirm_state_transitions() {
        let mut state = CsvTabState {
            import_result: Some(CsvUploadResponse {
                inserted: 1,
                skipped: 0,
                errors: vec![],
            }),
            ..Default::default()
        };
        state.open_delete_confirm();
        assert!(state.show_delete_confirm);
        state.close_delete_confirm();
        assert!(!state.show_delete_confirm);

        state.open_delete_confirm();
        assert!(state.begin_delete());
        assert!(!state.show_delete_confirm);
        assert!(state.deleting);
        assert!(!state.begin_delete(), "deleting 中は再開始しない");

        state.finish_delete(Ok(()));
        assert!(!state.deleting);
        assert!(state.import_result.is_none());
    }

    #[test]
    fn begin_delete_requires_open_confirmation() {
        // 全件削除はデータ消失の操作なので、確認表示が開いていなければ開始しない
        let mut state = CsvTabState::default();
        assert!(!state.begin_delete());
        assert!(!state.deleting);
        assert!(!state.show_delete_confirm);
    }

    #[test]
    fn delete_error_keeps_result_and_sets_message() {
        let mut state = CsvTabState {
            import_result: Some(CsvUploadResponse {
                inserted: 1,
                skipped: 0,
                errors: vec![],
            }),
            deleting: true,
            ..Default::default()
        };
        state.finish_delete(Err("サーバーエラーが発生しました".to_string()));
        assert!(!state.deleting);
        assert!(state.import_result.is_some());
        assert_eq!(state.error.as_deref(), Some("サーバーエラーが発生しました"));
    }

    #[test]
    fn notice_texts_match_react() {
        let preview = CsvPreview {
            valid_rows: 3,
            errors: vec![],
            ..Default::default()
        };
        assert_eq!(preview.summary_text(), "3件 追加で保存されます");
        let with_errors = CsvPreview {
            valid_rows: 3,
            errors: vec![CsvRowError {
                row: 2,
                message: "形式エラー".to_string(),
            }],
            ..Default::default()
        };
        assert_eq!(
            with_errors.summary_text(),
            "3件 追加で保存されます / 1件エラー"
        );

        let result = CsvUploadResponse {
            inserted: 2,
            skipped: 1,
            errors: vec![CsvRowError {
                row: 5,
                message: "重複".to_string(),
            }],
        };
        assert_eq!(result.inserted_text(), "2件反映");
        assert_eq!(result.skipped_text().as_deref(), Some("1件スキップ"));
        assert_eq!(result.error_count_text().as_deref(), Some("1件エラー"));
        assert_eq!(row_error_text(&result.errors[0]), "5行目: 重複");

        let clean = CsvUploadResponse {
            inserted: 2,
            skipped: 0,
            errors: vec![],
        };
        assert_eq!(clean.skipped_text(), None);
        assert_eq!(clean.error_count_text(), None);
    }
}
