//! 保有銘柄の現在値を cron で自動更新する。
//! 取得元は `fetch_close_price` 1関数に集約し、別の取得元への差し替えは
//! この関数だけを直せばよい形にする（Yahoo Finance chart API は非公式）。

use crate::db::{Bind, Db};
use crate::errors::{ApiError, UpstreamError};
use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc, Weekday};
use rust_decimal::Decimal;
use serde::Deserialize;
use shared::value::SecurityCode;

/// Yahoo Finance chart API（非公式）。`<code>.T` は東証銘柄。
const CHART_BASE_URL: &str = "https://query2.finance.yahoo.com/v8/finance/chart";

/// User-Agent を付けない要求は Yahoo 側が 429 で拒否するため、ブラウザ相当の値を送る
#[cfg(any(target_arch = "wasm32", test))]
const CHART_REQUEST_HEADERS: &[(&str, &str)] = &[("User-Agent", "Mozilla/5.0")];

/// cron 1回で処理する銘柄数の上限。無料枠の CPU 時間に収まるよう絞る。
/// 残りは次回の cron に委ねる（最終試行時刻が古い順に処理するため循環する）
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
/// 日中バーを含まない日足（interval=1d）を想定する。場中は当日バーに暫定値が
/// 乗るため、max_as_of 以前の確定バーのうち null でない終値を持つ最後のバーを採用する
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
            let as_of = DateTime::from_timestamp(*ts, 0)
                .map(|t| (t + chrono::Duration::hours(9)).date_naive())?;
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

/// 取得した終値を共有の security_prices 行へ適用する(1銘柄1行)。
/// 保存済みの基準日より新しい取得日のときだけ更新し、日付の新しいほうを残す。
/// まだ行がない銘柄(マイグレーションとデプロイの隙間に取り込まれた保有など)は挿入する
// tests/db_integration.rs からの検証用に公開しているため docs には出さない
#[doc(hidden)]
pub async fn apply_close_price(
    pool: &Db,
    code: &SecurityCode,
    price: &ClosePrice,
) -> Result<u64, ApiError> {
    crate::db::query(
        r#"
        INSERT INTO security_prices (security_code, current_price, prev_close, price_as_of)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (security_code) DO UPDATE SET
            current_price = EXCLUDED.current_price,
            -- 前日終値は同一基準日なら既知の値を維持し、基準日が変われば取得値で置き換える
            prev_close = CASE
                WHEN security_prices.price_as_of = EXCLUDED.price_as_of
                    THEN COALESCE(EXCLUDED.prev_close, security_prices.prev_close)
                ELSE EXCLUDED.prev_close
            END,
            price_as_of = EXCLUDED.price_as_of,
            updated_at = NOW()
        WHERE security_prices.price_as_of IS NULL
           OR security_prices.price_as_of <= EXCLUDED.price_as_of
        "#,
        vec![
            Bind::from(code.as_str()),
            Bind::from(price.close),
            Bind::opt(price.prev_close),
            Bind::from(price.as_of),
        ],
    )
    .execute(pool)
    .await
    .map_err(ApiError::from)
}

/// 最新の終値が確定している基準日（JST 15:30 以降なら当日、それより前なら前日。
/// 土日は直前の金曜に寄せる。祝日は扱わない）
fn latest_close_date(now: DateTime<Utc>) -> NaiveDate {
    let jst = now + chrono::Duration::hours(9);
    let mut date = jst.date_naive();
    if (jst.hour(), jst.minute()) < (15, 30) {
        date -= chrono::Duration::days(1);
    }
    date - chrono::Duration::days(match date.weekday() {
        Weekday::Sat => 1,
        Weekday::Sun => 2,
        _ => 0,
    })
}

/// cron から呼ばれる更新処理。最新の終値の日より共有価格の price_as_of が
/// 古い保有銘柄を、最終試行時刻の古い順に上限件数だけ処理する（全銘柄が最新なら取得しない）。
/// 失敗した銘柄は前回値を残し、全件失敗時は取得元の変更を疑ってエラーログに出す。
/// 戻り値は取得に成功した銘柄数（ログ用）
pub async fn refresh_stock_prices(pool: &Db, base_url: Option<String>) -> Result<usize, ApiError> {
    let latest = latest_close_date(Utc::now());
    let codes: Vec<SecurityCode> = crate::db::query(
        r#"
        SELECT ab.security_code
        FROM asset_balances ab
        LEFT JOIN security_prices sp ON sp.security_code = ab.security_code
        GROUP BY ab.security_code
        HAVING BOOL_OR(sp.price_as_of IS NULL) OR MIN(sp.price_as_of) < $1
        ORDER BY MIN(sp.updated_at) ASC NULLS FIRST
        LIMIT $2
        "#,
        vec![Bind::from(latest), Bind::from(MAX_CODES_PER_RUN)],
    )
    .fetch_all::<(SecurityCode,), _>(pool)
    .await?
    .into_iter()
    .map(|(code,)| code)
    .collect();

    // 選んだ銘柄の試行時刻を先に進めておく。取得に失敗しても順位が末尾へ回るため、
    // 連続して失敗する銘柄が後続の更新を永久に塞ぐことを防ぐ（#1338）。
    // 未取得(行なし)の銘柄には基準日なしの行を作り、次回以降も対象のままにする
    if !codes.is_empty() {
        crate::db::query(
            r#"
            INSERT INTO security_prices (security_code, updated_at)
            SELECT code, NOW() FROM UNNEST($1::text[]) AS t(code)
            ON CONFLICT (security_code) DO UPDATE SET updated_at = NOW()
            "#,
            vec![Bind::from(
                codes
                    .iter()
                    .map(|code| code.as_str().to_owned())
                    .collect::<Vec<_>>(),
            )],
        )
        .execute(pool)
        .await?;
    }

    let base_url = base_url.unwrap_or_else(|| CHART_BASE_URL.to_string());
    let mut fetched = 0;
    let mut failed = 0;
    for (i, code) in codes.iter().enumerate() {
        if i > 0 {
            crate::services::dividend_cache::sleep_millis(FETCH_GAP_MS).await;
        }
        match fetch_close_price(&base_url, code, latest).await {
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

/// 予定時刻（UTC エポックミリ秒）の分が5の倍数なら現在値更新を実行するかを返す
pub fn should_refresh_at(scheduled_ms: f64) -> bool {
    DateTime::from_timestamp_millis(scheduled_ms as i64).is_some_and(|t| t.minute() % 5 == 0)
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
    const MAX: NaiveDate = NaiveDate::MAX;

    #[test]
    fn test_parse_chart_close_uses_last_non_null_close() {
        let price = parse_chart_close(
            &chart_body(&[
                (T20240116, Some(3000.0)),
                (T20240117, Some(3100.5)),
                (T20240118, Some(3200.25)),
            ]),
            MAX,
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
            MAX,
        )
        .expect("parse");

        assert_eq!(price.close, dec!(3100.5));
        assert_eq!(price.as_of, NaiveDate::from_ymd_opt(2024, 1, 17).unwrap());
        assert_eq!(price.prev_close, Some(dec!(3000.0)));
    }

    #[test]
    fn test_parse_chart_close_ignores_bars_after_max_as_of() {
        // 場中に当日の暫定バーが返ってきても、確定済み(max_as_of 以前)の最後のバーを採る
        let price = parse_chart_close(
            &chart_body(&[
                (T20240116, Some(3000.0)),
                (T20240117, Some(3100.5)),
                (T20240118, Some(3200.25)),
            ]),
            NaiveDate::from_ymd_opt(2024, 1, 17).unwrap(),
        )
        .expect("parse");

        assert_eq!(price.close, dec!(3100.5));
        assert_eq!(price.as_of, NaiveDate::from_ymd_opt(2024, 1, 17).unwrap());
        assert_eq!(price.prev_close, Some(dec!(3000.0)));
    }

    #[test]
    fn test_parse_chart_close_single_bar_has_no_prev_close() {
        let price =
            parse_chart_close(&chart_body(&[(T20240116, Some(3000.0))]), MAX).expect("parse");

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
            MAX,
        )
        .expect("parse");

        assert_eq!(price.close, dec!(716.6));
        assert_eq!(price.prev_close, Some(dec!(2896.5)));
    }

    #[test]
    fn test_parse_chart_close_errors() {
        // チャート API 側のエラー応答
        assert!(
            parse_chart_close(
                r#"{"chart":{"result":null,"error":{"code":"Not Found"}}}"#,
                MAX
            )
            .is_err()
        );
        // result 空・timestamp 欠落・close 欠落・全バー null・全バーが max_as_of より後
        // はいずれもエラー
        assert!(parse_chart_close(r#"{"chart":{"result":[],"error":null}}"#, MAX).is_err());
        assert!(parse_chart_close(&chart_body(&[]), MAX).is_err());
        assert!(parse_chart_close(&chart_body(&[(T20240116, None)]), MAX).is_err());
        assert!(
            parse_chart_close(
                &chart_body(&[(T20240118, Some(3200.25))]),
                NaiveDate::from_ymd_opt(2024, 1, 17).unwrap(),
            )
            .is_err()
        );
        // JSON として不正
        assert!(parse_chart_close("not json", MAX).is_err());
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

    #[test]
    fn test_latest_close_date() {
        // JST の日時を渡す（内部で UTC に直して評価する）
        let jst = |y: i32, m: u32, d: u32, h: u32, mi: u32| {
            let utc = NaiveDate::from_ymd_opt(y, m, d)
                .unwrap()
                .and_hms_opt(h, mi, 0)
                .unwrap()
                - chrono::Duration::hours(9);
            latest_close_date(utc.and_utc())
        };
        let date = |y: i32, m: u32, d: u32| NaiveDate::from_ymd_opt(y, m, d).unwrap();

        // 平日は JST 15:30 を境に前日/当日
        assert_eq!(jst(2026, 10, 14, 15, 29), date(2026, 10, 13));
        assert_eq!(jst(2026, 10, 14, 15, 30), date(2026, 10, 14));

        // 土日は時刻に依らず直前の金曜
        assert_eq!(jst(2026, 10, 17, 9, 0), date(2026, 10, 16));
        assert_eq!(jst(2026, 10, 17, 16, 0), date(2026, 10, 16));
        assert_eq!(jst(2026, 10, 18, 12, 0), date(2026, 10, 16));

        // 月曜の朝は直前の金曜、引け後は当日
        assert_eq!(jst(2026, 10, 19, 9, 0), date(2026, 10, 16));
        assert_eq!(jst(2026, 10, 19, 15, 30), date(2026, 10, 19));
    }

    #[test]
    fn test_should_refresh_at_runs_every_five_minutes() {
        let ms = |h: u32, m: u32, s: u32| {
            NaiveDate::from_ymd_opt(2026, 10, 14)
                .unwrap()
                .and_hms_opt(h, m, s)
                .unwrap()
                .and_utc()
                .timestamp_millis() as f64
        };
        assert!(should_refresh_at(ms(0, 0, 0)));
        assert!(should_refresh_at(ms(15, 5, 0)));
        assert!(should_refresh_at(ms(23, 55, 0)));
        assert!(!should_refresh_at(ms(0, 1, 0)));
        assert!(!should_refresh_at(ms(12, 59, 0)));
        // 予定時刻の秒・ミリ秒がずれていても分で判定する
        assert!(!should_refresh_at(ms(10, 4, 59)));
        assert!(should_refresh_at(ms(10, 5, 30)));
    }
}
