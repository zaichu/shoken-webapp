//! 保存済み一覧の CSV エクスポート。証券会社のインポート形式ではなく、
//! 画面の見出しに揃えた列構成で生値(ISO 日付・桁区切りなしの数値)を出す。
//! メールアドレスや認証情報などの個人情報は含めない

#[cfg(test)]
mod tests;

use crate::api::ApiError;
use crate::api::dto::{AssetBalance, Dividend, DomesticStock, Mutualfund};
use crate::features::asset_balance::{ValuationInput, holding_valuation};
use crate::support::pagination::{ListPage, fetch_all_pages_with};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};

// 一覧表示のページ上限より大きく取り、保存上限の変更にも耐える。
// これを超えるデータ量は truncated としてエラーにする
const EXPORT_MAX_PAGES: usize = 1_000;

/// エクスポートの失敗。API 由来はその案内文、件数超過は専用の文で出す
#[derive(Debug)]
pub(crate) enum ExportError {
    Api(ApiError),
    Truncated,
}

impl ExportError {
    pub(crate) fn user_message(&self) -> String {
        match self {
            Self::Api(error) => error.user_message(),
            Self::Truncated => "件数が上限を超えたため、全件を取得できませんでした".to_string(),
        }
    }
}

impl From<ApiError> for ExportError {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ExportKind {
    Dividend,
    AssetBalance,
    Transaction,
}

impl ExportKind {
    pub(crate) const ALL: [Self; 3] = [Self::Dividend, Self::AssetBalance, Self::Transaction];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Dividend => "配当金",
            Self::AssetBalance => "資産残高",
            Self::Transaction => "売買履歴",
        }
    }

    fn stem(self) -> &'static str {
        match self {
            Self::Dividend => "dividends",
            Self::AssetBalance => "asset-balances",
            Self::Transaction => "transactions",
        }
    }

    pub(crate) fn filename(self, date: &str) -> String {
        format!("{}_{date}.csv", self.stem())
    }
}

// 取得上限に達したときは一部だけの CSV を成功扱いにしない
fn rows_of<T, S>(result: ListPage<T, S>) -> Result<Vec<T>, ExportError> {
    if result.truncated {
        Err(ExportError::Truncated)
    } else {
        Ok(result.rows)
    }
}

/// 一覧 API を全ページ取って CSV 化する。ファイル名の日付はローカル日付
pub(crate) async fn fetch_csv(kind: ExportKind) -> Result<(String, String), ExportError> {
    let content = match kind {
        ExportKind::Dividend => dividends_csv(&rows_of(
            fetch_all_pages_with::<Dividend>(EXPORT_MAX_PAGES).await?,
        )?),
        ExportKind::AssetBalance => asset_balances_csv(&rows_of(
            fetch_all_pages_with::<AssetBalance>(EXPORT_MAX_PAGES).await?,
        )?),
        ExportKind::Transaction => transactions_csv(
            &rows_of(fetch_all_pages_with::<DomesticStock>(EXPORT_MAX_PAGES).await?)?,
            &rows_of(fetch_all_pages_with::<Mutualfund>(EXPORT_MAX_PAGES).await?)?,
        ),
    };
    Ok((kind.filename(&today()), content))
}

fn today() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let date = js_sys::Date::new_0();
        format!(
            "{:04}-{:02}-{:02}",
            date.get_full_year(),
            date.get_month() + 1,
            date.get_date()
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // 単体テストでは呼ばれない
        String::new()
    }
}

fn csv_cell(text: &str) -> String {
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_string()
    }
}

/// ユーザー由来の文字列用。`= + - @` に加えタブ・CR 始まりも表計算ソフトで
/// 式として実行されうるため `'` で逃がす。数値フィールドには使わない
fn csv_text(text: &str) -> String {
    match text.chars().next() {
        Some('=' | '+' | '-' | '@' | '\t' | '\r') => format!("'{text}"),
        _ => text.to_string(),
    }
}

fn csv_line(fields: &[String]) -> String {
    fields
        .iter()
        .map(|f| csv_cell(f))
        .collect::<Vec<_>>()
        .join(",")
}

fn header(fields: &[&str]) -> String {
    csv_line(&fields.iter().map(|f| f.to_string()).collect::<Vec<_>>())
}

fn num(value: Decimal) -> String {
    value.normalize().to_string()
}

/// 計算値(評価額・含み損益)は画面と同じ円単位の整数に丸める。算出不能は空欄
fn derived_num(value: Option<f64>) -> String {
    value
        .filter(|value| value.is_finite())
        .and_then(|value| Decimal::from_str_exact(&value.to_string()).ok())
        .map(|value| {
            value
                .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
                .normalize()
                .to_string()
        })
        .unwrap_or_default()
}

fn dividends_csv(rows: &[Dividend]) -> String {
    let mut sorted: Vec<&Dividend> = rows.iter().collect();
    // 一覧の既定と同じく新しい入金日から。同日は銘柄コード順で安定させる
    sorted.sort_by(|a, b| {
        b.settlement_date
            .cmp(&a.settlement_date)
            .then_with(|| a.security_code.cmp(&b.security_code))
    });
    let mut lines = vec![header(&[
        "入金日",
        "商品",
        "口座",
        "銘柄コード",
        "銘柄名",
        "1株配当",
        "数量",
        "配当金",
        "税額",
        "税引後",
    ])];
    for row in sorted {
        lines.push(csv_line(&[
            row.settlement_date.to_string(),
            csv_text(&row.product),
            csv_text(row.account.as_str()),
            csv_text(&row.security_code),
            csv_text(&row.security_name),
            num(row.unit_price),
            num(row.shares),
            num(row.dividends_before_tax),
            num(row.taxes),
            num(row.net_amount_received),
        ]));
    }
    lines.join("\n")
}

fn asset_balances_csv(rows: &[AssetBalance]) -> String {
    let valuations: Vec<_> = rows
        .iter()
        .map(|row| {
            holding_valuation(&ValuationInput {
                current_price: row.current_price.to_f64().unwrap_or(0.0),
                shares: row.shares.to_f64().unwrap_or(0.0),
                total_purchase_amount: row.total_purchase_amount.to_f64().unwrap_or(0.0),
            })
        })
        .collect();
    let mut order: Vec<usize> = (0..rows.len()).collect();
    // 一覧の既定と同じく評価額の降順(価格なしは末尾)、同値は銘柄コード順
    order.sort_by(|a, b| {
        let ordering = match (valuations[*a].market_value, valuations[*b].market_value) {
            (Some(x), Some(y)) => y.total_cmp(&x),
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
        };
        ordering.then_with(|| rows[*a].security_code.cmp(&rows[*b].security_code))
    });
    let mut lines = vec![header(&[
        "銘柄コード",
        "銘柄名",
        "保有数",
        "平均取得単価",
        "取得額",
        "現在値",
        "価格基準日",
        "評価額",
        "含み損益",
    ])];
    for index in order {
        let row = &rows[index];
        let valuation = &valuations[index];
        lines.push(csv_line(&[
            csv_text(row.security_code.as_str()),
            csv_text(&row.security_name),
            num(row.shares),
            num(row.average_purchase_price),
            num(row.total_purchase_amount),
            num(row.current_price),
            row.price_as_of
                .as_ref()
                .map(|date| date.to_string())
                .unwrap_or_default(),
            derived_num(valuation.market_value),
            derived_num(valuation.profit_loss.amount),
        ]));
    }
    lines.join("\n")
}

/// 国内株式と投資信託を1つの売買履歴にする。種別で区別し、投資信託の銘柄コードは空欄
fn transactions_csv(stocks: &[DomesticStock], funds: &[Mutualfund]) -> String {
    // (約定日, 種別, CSV 行)。日付の降順で並べるためタプルの先頭を日付にする
    let mut entries: Vec<(String, &str, Vec<String>)> = Vec::new();
    for row in stocks {
        entries.push((
            row.trade_date.to_string(),
            "国内株式",
            vec![
                csv_text(row.security_code.as_str()),
                csv_text(&row.security_name),
                csv_text(row.account.as_str()),
                num(row.shares),
                num(row.asked_price),
                num(row.proceeds),
                num(row.purchase_price),
                num(row.realized_profit_and_loss),
                num(row.taxes),
                num(row.realized_profit_and_loss_after_tax),
            ],
        ));
    }
    for row in funds {
        entries.push((
            row.trade_date.to_string(),
            "投資信託",
            vec![
                String::new(),
                csv_text(&row.fund_name),
                csv_text(row.account.as_str()),
                num(row.shares),
                num(row.cancellation_unit_price_yen),
                num(row.cancellation_amount_yen),
                num(row.average_acquisition_price_yen),
                num(row.realized_profit_and_loss),
                num(row.taxes),
                num(row.realized_profit_and_loss_after_tax),
            ],
        ));
    }
    entries.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
    let mut lines = vec![header(&[
        "種別",
        "約定日",
        "銘柄コード",
        "銘柄名",
        "口座",
        "数量",
        "単価",
        "売却・解約額",
        "取得価額",
        "実現損益",
        "税額",
        "税引後",
    ])];
    for (date, kind, fields) in entries {
        let mut line = vec![kind.to_string(), date];
        line.extend(fields);
        lines.push(csv_line(&line));
    }
    lines.join("\n")
}

/// Blob + a[download] で保存ダイアログに渡す
#[cfg(target_arch = "wasm32")]
pub(crate) fn download_csv(filename: &str, content: &str) {
    use wasm_bindgen::JsCast;
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Ok(blob) = web_sys::Blob::new_with_str_sequence(
        &js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(content)).into(),
    ) else {
        return;
    };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
        return;
    };
    if let Some(anchor) = document
        .create_element("a")
        .ok()
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = anchor.set_attribute("href", &url);
        let _ = anchor.set_attribute("download", filename);
        anchor.click();
    }
    let _ = web_sys::Url::revoke_object_url(&url);
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn download_csv(_filename: &str, _content: &str) {}
