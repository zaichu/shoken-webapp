//! 保有銘柄の現在値を日次 cron で自動更新する。
//! 取得元は `fetch_close_price` 1関数に集約し、別の取得元への差し替えは
//! この関数だけを直せばよい形にする（Yahoo Finance chart API は非公式）。

use crate::db::{Bind, Db};
use crate::errors::{ApiError, UpstreamError};
use chrono::{DateTime, NaiveDate};
use rust_decimal::Decimal;
use serde::Deserialize;
use shared::value::SecurityCode;

/// Yahoo Finance chart API（非公式）。`<code>.T` は東証銘柄。
const CHART_BASE_URL: &str = "https://query2.finance.yahoo.com/v8/finance/chart";

/// 日次 cron 1回で処理する銘柄数の上限。無料枠の CPU 時間に収まるよう絞る。
/// 残りは翌日の cron に委ねる（古い price_as_of から優先して処理するため循環する）
const MAX_CODES_PER_RUN: i64 = 15;

/// 銘柄間の待ち時間。非公式 API への負荷を抑えるために少し間を空ける
const FETCH_GAP_MS: u64 = 500;

/// Yahoo chart API から取り出した終値
#[derive(Debug, Clone, PartialEq)]
pub struct ClosePrice {
    pub close: Decimal,
    /// 終値の基準日（JST）
    pub as_of: NaiveDate,
    /// 直前の取引日の終値。取得できたときだけ daily_change を更新する
    pub prev_close: Option<Decimal>,
}

#[derive(Deserialize)]
struct ChartResponse {
    chart: ChartBody,
}

#[derive(Deserialize)]
struct ChartBody {
    result: Option<Vec<ChartResult>>,
    error: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct ChartResult {
    timestamp: Option<Vec<i64>>,
    indicators: Indicators,
}

#[derive(Deserialize)]
struct Indicators {
    quote: Vec<Quote>,
}

#[derive(Deserialize)]
struct Quote {
    close: Option<Vec<Option<f64>>>,
}

/// chart API のレスポンス文字列から終値・基準日・前日終値を取り出す。
/// 日中バーを含まない日足（interval=1d）を想定し、null でない終値を持つ
/// 最後のバーを採用する（引け後実行だが、実行タイミングがずれても直近の確定値になる）
pub fn parse_chart_close(body: &str) -> Result<ClosePrice, ApiError> {
    let response: ChartResponse = serde_json::from_str(body).map_err(UpstreamError::Decode)?;
    if response.chart.error.is_some() {
        return Err(ApiError::Internal("chart API がエラーを返した"));
    }
    let no_data = || ApiError::Internal("chart API の応答に終値データがない");
    let result = response
        .chart
        .result
        .as_deref()
        .and_then(|results| results.first())
        .ok_or_else(no_data)?;
    let timestamps = result.timestamp.as_deref().ok_or_else(no_data)?;
    let closes = result
        .indicators
        .quote
        .first()
        .and_then(|quote| quote.close.as_deref())
        .ok_or_else(no_data)?;

    let bars: Vec<(i64, f64)> = timestamps
        .iter()
        .zip(closes.iter())
        .filter_map(|(ts, close)| close.map(|close| (*ts, close)))
        .collect();
    let (ts, close) = bars.last().ok_or_else(no_data)?;
    let as_of = DateTime::from_timestamp(*ts, 0)
        .map(|t| (t + chrono::Duration::hours(9)).date_naive())
        .ok_or_else(no_data)?;
    Ok(ClosePrice {
        close: Decimal::from_f64_retain(*close).ok_or_else(no_data)?,
        as_of,
        prev_close: bars
            .iter()
            .rev()
            .nth(1)
            .and_then(|(_, c)| Decimal::from_f64_retain(*c)),
    })
}

/// Yahoo chart API から終値を取得する。ホスト側では実行経路がないためエラー固定
/// （テストは parse_chart_close / apply_close_price を直接検証する）
#[cfg(not(target_arch = "wasm32"))]
async fn fetch_close_price(_base_url: &str, _code: &SecurityCode) -> Result<ClosePrice, ApiError> {
    Err(ApiError::Internal(
        "Yahoo Finance への接続は Workers 環境でのみ利用できます",
    ))
}

#[cfg(target_arch = "wasm32")]
async fn fetch_close_price(base_url: &str, code: &SecurityCode) -> Result<ClosePrice, ApiError> {
    use worker::{Fetch, Method, Request, RequestInit};

    // 直近5営業日の日足。最後の確定バーが終値、その前が前日終値
    let url = format!("{base_url}/{}.T?range=5d&interval=1d", code.as_str());
    let mut init = RequestInit::new();
    init.with_method(Method::Get);
    let request = Request::new_with_init(&url, &init).map_err(UpstreamError::Transport)?;
    let mut response = worker::send::SendFuture::new(Fetch::Request(request).send())
        .await
        .map_err(UpstreamError::Transport)?;

    let status = response.status_code();
    if !(200..300).contains(&status) {
        let body = worker::send::SendFuture::new(response.text())
            .await
            .unwrap_or_default();
        tracing::error!("Yahoo chart API エラー: status={}, code={}", status, code);
        if status == 429 {
            return Err(UpstreamError::RateLimited.into());
        }
        return Err(UpstreamError::Http { status, body }.into());
    }

    let body = worker::send::SendFuture::new(response.text())
        .await
        .map_err(UpstreamError::Transport)?;
    parse_chart_close(&body)
}

/// 取得した終値を同じ銘柄を持つ全ユーザーの行へ適用する。
/// 保存済みの基準日より新しい取得日のときだけ更新し、日付の新しいほうを残す
// tests/db_integration.rs からの検証用に公開しているため docs には出さない
#[doc(hidden)]
pub async fn apply_close_price(
    pool: &Db,
    code: &SecurityCode,
    price: &ClosePrice,
) -> Result<u64, ApiError> {
    crate::db::query(
        r#"
        UPDATE asset_balances
        SET current_price = $2,
            daily_change  = COALESCE($3, daily_change),
            market_value  = shares * $2,
            price_as_of   = $4,
            updated_at    = NOW()
        WHERE security_code = $1
          AND (price_as_of IS NULL OR price_as_of <= $4)
        "#,
        vec![
            Bind::from(code.as_str()),
            Bind::from(price.close),
            Bind::opt(price.prev_close.map(|prev| price.close - prev)),
            Bind::from(price.as_of),
        ],
    )
    .execute(pool)
    .await
    .map_err(ApiError::from)
}

/// 日次 cron から呼ばれる更新処理。保有銘柄を基準日の古い順に上限件数だけ処理する。
/// 失敗した銘柄は前回値を残し、全件失敗時は取得元の変更を疑ってエラーログに出す。
/// 戻り値は取得に成功した銘柄数（ログ用）
pub async fn refresh_stock_prices(pool: &Db, base_url: Option<String>) -> Result<usize, ApiError> {
    let codes: Vec<SecurityCode> = crate::db::query(
        r#"
        SELECT security_code FROM asset_balances
        GROUP BY security_code
        ORDER BY MIN(price_as_of) ASC NULLS FIRST
        LIMIT $1
        "#,
        vec![Bind::from(MAX_CODES_PER_RUN)],
    )
    .fetch_all::<(SecurityCode,), _>(pool)
    .await?
    .into_iter()
    .map(|(code,)| code)
    .collect();

    let base_url = base_url.unwrap_or_else(|| CHART_BASE_URL.to_string());
    let mut fetched = 0;
    for (i, code) in codes.iter().enumerate() {
        if i > 0 {
            crate::services::dividend_cache::sleep_millis(FETCH_GAP_MS).await;
        }
        match fetch_close_price(&base_url, code).await {
            Ok(price) => {
                fetched += 1;
                match apply_close_price(pool, code, &price).await {
                    Ok(rows) => {
                        tracing::info!(
                            "現在値更新: code={}, as_of={}, rows={}",
                            code,
                            price.as_of,
                            rows
                        )
                    }
                    Err(e) => tracing::error!("現在値の反映に失敗: code={}, err={}", code, e),
                }
            }
            Err(e) => {
                tracing::warn!(
                    "現在値の取得に失敗（前回値を維持）: code={}, err={}",
                    code,
                    e
                )
            }
        }
    }

    if !codes.is_empty() && fetched == 0 {
        // 全銘柄が失敗 = 個別障害ではなく取得元の応答・エンドポイント変更を疑う
        tracing::error!(
            "現在値の自動取得が全銘柄失敗。取得元（Yahoo chart API）の変更の可能性あり"
        );
    }
    Ok(fetched)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    /// 東証の日足タイムスタンプ（9:00 JST = 0:00 UTC）からなる応答の雛形
    fn chart_body(bars: &[(i64, Option<f64>)]) -> String {
        let timestamps: Vec<String> = bars.iter().map(|(ts, _)| ts.to_string()).collect();
        let closes: Vec<String> = bars
            .iter()
            .map(|(_, c)| c.map_or("null".to_string(), |c| c.to_string()))
            .collect();
        format!(
            r#"{{"chart":{{"result":[{{"timestamp":[{}],"indicators":{{"quote":[{{"close":[{}]}}]}}}}],"error":null}}}}"#,
            timestamps.join(","),
            closes.join(",")
        )
    }

    // 2024-01-16 09:00 JST = 2024-01-16 00:00 UTC
    const T20240116: i64 = 1705363200;
    const T20240117: i64 = T20240116 + 86400;
    const T20240118: i64 = T20240116 + 86400 * 2;

    #[test]
    fn test_parse_chart_close_uses_last_non_null_close() {
        let price = parse_chart_close(&chart_body(&[
            (T20240116, Some(3000.0)),
            (T20240117, Some(3100.5)),
            (T20240118, Some(3200.25)),
        ]))
        .expect("parse");

        assert_eq!(price.close, dec!(3200.25));
        assert_eq!(price.prev_close, Some(dec!(3100.5)));
        // JST のバー日付で基準日を記録する
        assert_eq!(price.as_of, NaiveDate::from_ymd_opt(2024, 1, 18).unwrap());
    }

    #[test]
    fn test_parse_chart_close_skips_null_close_bars() {
        // 立会中など末尾バーの終値が null の場合、その前の確定バーを使う
        let price = parse_chart_close(&chart_body(&[
            (T20240116, Some(3000.0)),
            (T20240117, Some(3100.5)),
            (T20240118, None),
        ]))
        .expect("parse");

        assert_eq!(price.close, dec!(3100.5));
        assert_eq!(price.as_of, NaiveDate::from_ymd_opt(2024, 1, 17).unwrap());
        assert_eq!(price.prev_close, Some(dec!(3000.0)));
    }

    #[test]
    fn test_parse_chart_close_single_bar_has_no_prev_close() {
        let price = parse_chart_close(&chart_body(&[(T20240116, Some(3000.0))])).expect("parse");

        assert_eq!(price.close, dec!(3000));
        assert_eq!(price.prev_close, None);
    }

    #[test]
    fn test_parse_chart_close_errors() {
        // チャート API 側のエラー応答
        assert!(
            parse_chart_close(r#"{"chart":{"result":null,"error":{"code":"Not Found"}}}"#).is_err()
        );
        // result 空・timestamp 欠落・close 欠落・全バー null はいずれもエラー
        assert!(parse_chart_close(r#"{"chart":{"result":[],"error":null}}"#).is_err());
        assert!(parse_chart_close(&chart_body(&[])).is_err());
        assert!(parse_chart_close(&chart_body(&[(T20240116, None)])).is_err());
        // JSON として不正
        assert!(parse_chart_close("not json").is_err());
    }
}
