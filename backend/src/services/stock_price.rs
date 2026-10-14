//! 保有銘柄の現在値を cron で自動更新する。毎分起動し、「最新の終値の日」より
//! price_as_of が古い銘柄だけを取り直す（前回の問い合わせから1時間以上たつものに限る）。
//! 取得元は `fetch_close_price` 1関数に集約し、別の取得元への差し替えは
//! この関数だけを直せばよい形にする（Yahoo Finance chart API は非公式）。

use crate::db::{Bind, Db};
use crate::errors::{ApiError, UpstreamError};
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc, Weekday};
use rust_decimal::Decimal;
use serde::Deserialize;
use shared::value::SecurityCode;

/// Yahoo Finance chart API（非公式）。`<code>.T` は東証銘柄。
const CHART_BASE_URL: &str = "https://query2.finance.yahoo.com/v8/finance/chart";

/// User-Agent を付けない要求は Yahoo 側が 429 で拒否するため、ブラウザ相当の値を送る
#[cfg(any(target_arch = "wasm32", test))]
const CHART_REQUEST_HEADERS: &[(&str, &str)] = &[("User-Agent", "Mozilla/5.0")];

/// cron 1回で処理する銘柄数の上限。無料枠の CPU 時間に収まるよう絞る。
/// 残りは次回の cron に委ねる（古い price_as_of から優先して処理するため循環する）
const MAX_CODES_PER_RUN: i64 = 15;

/// 銘柄間の待ち時間。非公式 API への負荷を抑えるために少し間を空ける
const FETCH_GAP_MS: u64 = 500;

/// wasm は tracing subscriber を持たないため、Worker 上では console 系へ出す
#[cfg(target_arch = "wasm32")]
macro_rules! svc_log {
    (info, $($t:tt)*) => { worker::console_log!($($t)*) };
    (warn, $($t:tt)*) => { worker::console_warn!($($t)*) };
    (error, $($t:tt)*) => { worker::console_error!($($t)*) };
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! svc_log {
    (info, $($t:tt)*) => { tracing::info!($($t)*) };
    (warn, $($t:tt)*) => { tracing::warn!($($t)*) };
    (error, $($t:tt)*) => { tracing::error!($($t)*) };
}

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
/// 日中バーを含まない日足（interval=1d）を想定し、基準日が `max_as_of` 以前で
/// null でない終値を持つ最後のバーを採用する（立会中の当日バーを終値にしないため）
pub fn parse_chart_close(body: &str, max_as_of: NaiveDate) -> Result<ClosePrice, ApiError> {
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

    let bars: Vec<(NaiveDate, f64)> = timestamps
        .iter()
        .zip(closes.iter())
        .filter_map(|(ts, close)| {
            let as_of = DateTime::from_timestamp(*ts, 0)?
                .checked_add_signed(Duration::hours(9))?
                .date_naive();
            close.map(|close| (as_of, close))
        })
        .filter(|(as_of, _)| *as_of <= max_as_of)
        .collect();
    let (as_of, close) = bars.last().ok_or_else(no_data)?;
    let to_price = |v: f64| Decimal::from_f64_retain(v).map(|d| d.round_dp(2).normalize());
    Ok(ClosePrice {
        close: to_price(*close).ok_or_else(no_data)?,
        as_of: *as_of,
        prev_close: bars.iter().rev().nth(1).and_then(|(_, c)| to_price(*c)),
    })
}

/// 2xx 以外の応答をエラーに写す。429 はレート制限として区別する
#[cfg(any(target_arch = "wasm32", test))]
fn chart_status_error(status: u16, body: String) -> ApiError {
    if status == 429 {
        UpstreamError::RateLimited.into()
    } else {
        UpstreamError::Http { status, body }.into()
    }
}

/// Yahoo chart API から終値を取得する。ホスト側では実行経路がないためエラー固定
/// （テストは parse_chart_close / apply_close_price を直接検証する）
#[cfg(not(target_arch = "wasm32"))]
async fn fetch_close_price(
    _base_url: &str,
    _code: &SecurityCode,
    _max_as_of: NaiveDate,
) -> Result<ClosePrice, ApiError> {
    Err(ApiError::Internal(
        "Yahoo Finance への接続は Workers 環境でのみ利用できます",
    ))
}

#[cfg(target_arch = "wasm32")]
async fn fetch_close_price(
    base_url: &str,
    code: &SecurityCode,
    max_as_of: NaiveDate,
) -> Result<ClosePrice, ApiError> {
    use worker::{Fetch, Headers, Method, Request, RequestInit};

    // 直近5営業日の日足。最後の確定バーが終値、その前が前日終値
    let url = format!("{base_url}/{}.T?range=5d&interval=1d", code.as_str());
    let headers = Headers::new();
    for &(name, value) in CHART_REQUEST_HEADERS {
        headers.set(name, value).map_err(UpstreamError::Transport)?;
    }
    let mut init = RequestInit::new();
    init.with_method(Method::Get).with_headers(headers);
    let request = Request::new_with_init(&url, &init).map_err(UpstreamError::Transport)?;
    let mut response = worker::send::SendFuture::new(Fetch::Request(request).send())
        .await
        .map_err(UpstreamError::Transport)?;

    let status = response.status_code();
    if !(200..300).contains(&status) {
        let body = worker::send::SendFuture::new(response.text())
            .await
            .unwrap_or_default();
        worker::console_error!("Yahoo chart API エラー: status={}, code={}", status, code);
        return Err(chart_status_error(status, body));
    }

    let body = worker::send::SendFuture::new(response.text())
        .await
        .map_err(UpstreamError::Transport)?;
    parse_chart_close(&body, max_as_of)
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

/// JST 時刻から「最新の終値の日」を返す。引け（15:30）より前は前営業日、
/// 以降は当日。土日は直前の金曜に戻す（祝日は扱わない）
fn latest_close_date(jst_now: NaiveDateTime) -> NaiveDate {
    let mut date = jst_now.date();
    if jst_now.time() < NaiveTime::from_hms_opt(15, 30, 0).unwrap() {
        date -= Duration::days(1);
    }
    while matches!(date.weekday(), Weekday::Sat | Weekday::Sun) {
        date -= Duration::days(1);
    }
    date
}

/// UTC 時刻に対応する「最新の終値の日」。CSV 取り込みの price_as_of 打刻でも使う
// asset_balance と tests/db_integration.rs から使うため docs には出さない
#[doc(hidden)]
pub fn latest_close_date_at(now: DateTime<Utc>) -> NaiveDate {
    latest_close_date((now + Duration::hours(9)).naive_utc())
}

/// 問い合わせ時刻を残す。成否にかかわらず呼び、祝日や取得失敗で
/// price_as_of が進まない銘柄を毎分取り直し続けないための目印にする
async fn mark_price_checked(pool: &Db, code: &SecurityCode) -> Result<(), ApiError> {
    crate::db::query(
        "UPDATE asset_balances SET price_checked_at = NOW() WHERE security_code = $1",
        vec![Bind::from(code)],
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// cron から呼ばれる更新処理。「最新の終値の日」より price_as_of が古く、
/// 前回の問い合わせから1時間以上たった銘柄を基準日の古い順に上限件数だけ処理する。
/// 失敗した銘柄は前回値を残し、全件失敗時は取得元の変更を疑ってエラーログに出す。
/// 戻り値は取得に成功した銘柄数（ログ用）
pub async fn refresh_stock_prices(pool: &Db, base_url: Option<String>) -> Result<usize, ApiError> {
    let now = Utc::now();
    let latest_close = latest_close_date_at(now);
    let recheck_after = now - Duration::hours(1);
    let codes: Vec<SecurityCode> = crate::db::query(
        r#"
        SELECT security_code FROM asset_balances
        WHERE price_as_of IS NULL OR price_as_of < $1
        GROUP BY security_code
        HAVING MAX(price_checked_at) IS NULL OR MAX(price_checked_at) < $2
        ORDER BY MIN(price_as_of) ASC NULLS FIRST
        LIMIT $3
        "#,
        vec![
            Bind::from(latest_close),
            Bind::from(recheck_after),
            Bind::from(MAX_CODES_PER_RUN),
        ],
    )
    .fetch_all::<(SecurityCode,), _>(pool)
    .await?
    .into_iter()
    .map(|(code,)| code)
    .collect();

    let base_url = base_url.unwrap_or_else(|| CHART_BASE_URL.to_string());
    let mut fetched = 0;
    let mut failed = 0;
    for (i, code) in codes.iter().enumerate() {
        if i > 0 {
            crate::services::dividend_cache::sleep_millis(FETCH_GAP_MS).await;
        }
        match fetch_close_price(&base_url, code, latest_close).await {
            Ok(price) => {
                fetched += 1;
                match apply_close_price(pool, code, &price).await {
                    Ok(rows) => svc_log!(
                        info,
                        "現在値更新: code={}, as_of={}, rows={}",
                        code,
                        price.as_of,
                        rows
                    ),
                    Err(e) => {
                        failed += 1;
                        svc_log!(error, "現在値の反映に失敗: code={}, err={}", code, e)
                    }
                }
            }
            Err(e) => {
                failed += 1;
                svc_log!(
                    warn,
                    "現在値の取得に失敗（前回値を維持）: code={}, err={}",
                    code,
                    e
                )
            }
        }
        if let Err(e) = mark_price_checked(pool, code).await {
            svc_log!(warn, "問い合わせ時刻の記録に失敗: code={}, err={}", code, e);
        }
    }

    // 件数の内訳を残し、cron が動いたか・どの段階で失敗したかを判別できるようにする
    svc_log!(
        info,
        "現在値更新の結果: 対象={}件, 成功={}件, 失敗={}件",
        codes.len(),
        codes.len() - failed,
        failed
    );
    if !codes.is_empty() && fetched == 0 {
        // 全銘柄が失敗 = 個別障害ではなく取得元の応答・エンドポイント変更を疑う
        svc_log!(
            error,
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
        let price = parse_chart_close(
            &chart_body(&[
                (T20240116, Some(3000.0)),
                (T20240117, Some(3100.5)),
                (T20240118, Some(3200.25)),
            ]),
            NaiveDate::from_ymd_opt(2024, 1, 18).unwrap(),
        )
        .expect("parse");

        assert_eq!(price.close, dec!(3200.25));
        assert_eq!(price.prev_close, Some(dec!(3100.5)));
        // JST のバー日付で基準日を記録する
        assert_eq!(price.as_of, NaiveDate::from_ymd_opt(2024, 1, 18).unwrap());
    }

    #[test]
    fn test_parse_chart_close_skips_null_close_bars() {
        // 立会中など末尾バーの終値が null の場合、その前の確定バーを使う
        let price = parse_chart_close(
            &chart_body(&[
                (T20240116, Some(3000.0)),
                (T20240117, Some(3100.5)),
                (T20240118, None),
            ]),
            NaiveDate::from_ymd_opt(2024, 1, 18).unwrap(),
        )
        .expect("parse");

        assert_eq!(price.close, dec!(3100.5));
        assert_eq!(price.as_of, NaiveDate::from_ymd_opt(2024, 1, 17).unwrap());
        assert_eq!(price.prev_close, Some(dec!(3000.0)));
    }

    #[test]
    fn test_parse_chart_close_drops_bars_after_max_as_of() {
        // 立会中に当日バーが非 null で返ってきても、確定していない日付のバーは終値にしない
        let price = parse_chart_close(
            &chart_body(&[
                (T20240116, Some(3000.0)),
                (T20240117, Some(3100.5)),
                (T20240118, Some(3200.25)),
            ]),
            NaiveDate::from_ymd_opt(2024, 1, 17).unwrap(),
        )
        .expect("parse");

        assert_eq!(price.as_of, NaiveDate::from_ymd_opt(2024, 1, 17).unwrap());
        assert_eq!(price.close, dec!(3100.5));
        assert_eq!(price.prev_close, Some(dec!(3000.0)));
    }

    #[test]
    fn test_parse_chart_close_single_bar_has_no_prev_close() {
        let price = parse_chart_close(
            &chart_body(&[(T20240116, Some(3000.0))]),
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
        )
        .expect("parse");

        assert_eq!(price.close, dec!(3000));
        assert_eq!(price.prev_close, None);
    }

    #[test]
    fn test_parse_chart_close_rounds_float_error() {
        let price = parse_chart_close(
            &chart_body(&[
                (T20240117, Some(2896.4999961853027)),
                (T20240118, Some(716.5999755859375)),
            ]),
            NaiveDate::from_ymd_opt(2024, 1, 18).unwrap(),
        )
        .expect("parse");

        assert_eq!(price.close, dec!(716.6));
        assert_eq!(price.prev_close, Some(dec!(2896.5)));
    }

    #[test]
    fn test_parse_chart_close_errors() {
        let max = NaiveDate::MAX;
        // チャート API 側のエラー応答
        assert!(
            parse_chart_close(
                r#"{"chart":{"result":null,"error":{"code":"Not Found"}}}"#,
                max
            )
            .is_err()
        );
        // result 空・timestamp 欠落・close 欠落・全バー null はいずれもエラー
        assert!(parse_chart_close(r#"{"chart":{"result":[],"error":null}}"#, max).is_err());
        assert!(parse_chart_close(&chart_body(&[]), max).is_err());
        assert!(parse_chart_close(&chart_body(&[(T20240116, None)]), max).is_err());
        // 全部のバーが max_as_of より後ろでもエラー
        assert!(
            parse_chart_close(
                &chart_body(&[(T20240116, Some(3000.0))]),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            )
            .is_err()
        );
        // JSON として不正
        assert!(parse_chart_close("not json", max).is_err());
    }

    #[test]
    fn test_chart_request_headers_send_user_agent() {
        let user_agent = CHART_REQUEST_HEADERS
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("user-agent"))
            .map(|(_, value)| *value);

        assert!(user_agent.is_some_and(|value| !value.is_empty()));
    }

    #[test]
    fn test_chart_status_error_classifies_429_as_rate_limited() {
        assert!(matches!(
            chart_status_error(429, "ignored".to_string()),
            ApiError::Upstream(UpstreamError::RateLimited)
        ));

        match chart_status_error(503, "bad".to_string()) {
            ApiError::Upstream(UpstreamError::Http { status, body }) => {
                assert_eq!(status, 503);
                assert_eq!(body, "bad");
            }
            other => panic!("expected Http error, got {other}"),
        }
    }

    fn jst(y: i32, m: u32, d: u32, h: u32, min: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(h, min, 0)
            .unwrap()
    }

    #[test]
    fn test_latest_close_date_weekday_before_close_is_prev_day() {
        // 水曜 15:29 は引け前のため、前営業日(火曜)の終値が最新
        assert_eq!(
            latest_close_date(jst(2024, 1, 17, 15, 29)),
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap()
        );
    }

    #[test]
    fn test_latest_close_date_weekday_after_close_is_same_day() {
        // 引け(15:30)以降は当日の終値が確定済み
        assert_eq!(
            latest_close_date(jst(2024, 1, 17, 15, 30)),
            NaiveDate::from_ymd_opt(2024, 1, 17).unwrap()
        );
    }

    #[test]
    fn test_latest_close_date_weekend_is_friday() {
        assert_eq!(
            latest_close_date(jst(2024, 1, 20, 10, 0)),
            NaiveDate::from_ymd_opt(2024, 1, 19).unwrap()
        );
        assert_eq!(
            latest_close_date(jst(2024, 1, 21, 16, 0)),
            NaiveDate::from_ymd_opt(2024, 1, 19).unwrap()
        );
    }

    #[test]
    fn test_latest_close_date_monday_morning_is_friday() {
        // 月曜の立会前に確定している最新の終値は直前金曜のもの
        assert_eq!(
            latest_close_date(jst(2024, 1, 22, 9, 0)),
            NaiveDate::from_ymd_opt(2024, 1, 19).unwrap()
        );
        assert_eq!(
            latest_close_date(jst(2024, 1, 22, 15, 30)),
            NaiveDate::from_ymd_opt(2024, 1, 22).unwrap()
        );
    }
}
