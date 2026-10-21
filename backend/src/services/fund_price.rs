//! 三菱UFJアセットの投信情報API(公式)から保有ファンドの基準価額を cron で自動更新する。
//! 取得元は `fetch_fund_nav` 1関数に集約し、別の取得元への差し替えは
//! この関数だけを直せばよい形にする。

use crate::db::{Bind, Db};
use crate::errors::{ApiError, UpstreamError};
use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc, Weekday};
use rust_decimal::{Decimal, prelude::Signed};
use serde::Deserialize;
use std::str::FromStr;

/// 三菱UFJアセットマネジメントの投信情報API(公式)。
/// `{BASE}/association_fund_cd/<協会コード>` で最新の基準価額を返す
const API_BASE_URL: &str = "https://developer.am.mufg.jp/fund_information_latest";

/// cron 1回で処理する協会コード数の上限。無料枠の CPU 時間に収まるよう絞る。
/// 残りは次回の cron に委ねる（最終試行時刻が古い順に処理するため循環する）
const MAX_CODES_PER_RUN: i64 = 30;

/// ファンド間の待ち時間。公式 API への負荷を抑えるために少し間を空ける
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

/// 投信情報APIから取り出して保存する共有の基準価額
#[derive(Debug, Clone, PartialEq)]
pub struct FundNav {
    /// 基準価額(1万口あたり・円)
    pub nav: Decimal,
    /// 前日比(1万口あたり・円)。応答の cmp_prev_day
    pub prev_day_change: Decimal,
    /// 基準価額の基準日(応答の base_date)
    pub base_date: NaiveDate,
}

/// 応答は result.status=200 かつ datasets 1件以上のときだけ値を持つ。
/// 値は datasets の最初の要素に入り、数値は文字列・数値のどちらでも来うるため
/// Value で受けて検証時に捌く
#[derive(Deserialize)]
struct MufgNavResponse {
    result: Option<MufgResult>,
    datasets: Option<Vec<MufgNavDataset>>,
}

#[derive(Deserialize)]
struct MufgResult {
    status: Option<i64>,
}

#[derive(Deserialize)]
struct MufgNavDataset {
    nav: Option<serde_json::Value>,
    base_date: Option<serde_json::Value>,
    cmp_prev_day: Option<serde_json::Value>,
    percentage_change: Option<serde_json::Value>,
}

fn as_decimal(v: Option<&serde_json::Value>) -> Option<Decimal> {
    match v? {
        serde_json::Value::Number(n) => n
            .as_i64()
            .map(Decimal::from)
            .or_else(|| n.as_f64().and_then(Decimal::from_f64_retain)),
        serde_json::Value::String(s) => Decimal::from_str(s.trim()).ok(),
        _ => None,
    }
}

/// 応答を検証して保存可能な基準価額にする。検証に通らない応答は一部だけでも
/// 保存しない(別出どころの値の混在・異常値の表示を防ぐ)。規則:
/// nav は正、base_date は日付として解釈できる、cmp_prev_day は符号付き数値、
/// percentage_change と cmp_prev_day の符号が一致、|cmp_prev_day|/nav は 20% 以内。
/// result.status=404(対象外の協会コード)は Ok(None)。エラーではなく前回値を維持する
pub fn parse_mufg_nav(body: &str) -> Result<Option<FundNav>, ApiError> {
    let reject =
        |reason: &str| UpstreamError::Invalid(format!("投信情報APIの応答: {reason}")).into();
    let response: MufgNavResponse = serde_json::from_str(body).map_err(UpstreamError::Decode)?;

    let status = response.result.as_ref().and_then(|r| r.status);
    match status {
        Some(200) => {}
        Some(404) => return Ok(None),
        _ => return Err(reject(&format!("result.status が 200 でない: {status:?}"))),
    }
    let dataset = response
        .datasets
        .as_deref()
        .and_then(|d| d.first())
        .ok_or_else(|| reject("datasets が無いか空"))?;

    let nav = as_decimal(dataset.nav.as_ref()).ok_or_else(|| reject("nav が数値でない"))?;
    if nav <= Decimal::ZERO {
        return Err(reject("nav が 0 以下"));
    }
    let date_str = dataset
        .base_date
        .as_ref()
        .and_then(|v| match v {
            serde_json::Value::String(s) => Some(s.trim().to_string()),
            serde_json::Value::Number(n) => n.as_i64().map(|i| i.to_string()),
            _ => None,
        })
        .ok_or_else(|| reject("base_date がない"))?;
    let base_date = NaiveDate::parse_from_str(&date_str, "%Y%m%d")
        .map_err(|_| reject("base_date が日付でない"))?;
    let prev_day_change = as_decimal(dataset.cmp_prev_day.as_ref())
        .ok_or_else(|| reject("cmp_prev_day が数値でない"))?;
    let percentage_change = as_decimal(dataset.percentage_change.as_ref())
        .ok_or_else(|| reject("percentage_change が数値でない"))?;
    if prev_day_change.signum() != percentage_change.signum() {
        return Err(reject("cmp_prev_day と percentage_change の符号が不一致"));
    }
    if prev_day_change.abs() > nav * Decimal::new(2, 1) {
        return Err(reject("前日比が nav の 20% を超える"));
    }
    Ok(Some(FundNav {
        nav,
        prev_day_change,
        base_date,
    }))
}

/// 2xx 以外の応答をエラーに写す。429 はレート制限として区別する
#[cfg(any(target_arch = "wasm32", test))]
fn nav_status_error(status: u16, body: String) -> ApiError {
    if status == 429 {
        UpstreamError::RateLimited.into()
    } else {
        UpstreamError::Http { status, body }.into()
    }
}

/// 投信情報APIから基準価額を取得する。ホスト側では実行経路がないためエラー固定
/// （テストは parse_mufg_nav / apply_fund_nav を直接検証する）
#[cfg(not(target_arch = "wasm32"))]
async fn fetch_fund_nav(
    _base_url: &str,
    _association_fund_cd: &str,
) -> Result<Option<FundNav>, ApiError> {
    Err(ApiError::Internal(
        "投信情報APIへの接続は Workers 環境でのみ利用できます",
    ))
}

#[cfg(target_arch = "wasm32")]
async fn fetch_fund_nav(
    base_url: &str,
    association_fund_cd: &str,
) -> Result<Option<FundNav>, ApiError> {
    use worker::{Fetch, Method, Request, RequestInit};

    let url = format!("{base_url}/association_fund_cd/{association_fund_cd}");
    let mut init = RequestInit::new();
    init.with_method(Method::Get);
    let request = Request::new_with_init(&url, &init).map_err(UpstreamError::Transport)?;
    let mut response = worker::send::SendFuture::new(Fetch::Request(request).send())
        .await
        .map_err(UpstreamError::Transport)?;

    let status = response.status_code();
    if status == 404 {
        // 対象外の協会コード。失敗ではなく前回値を維持する
        return Ok(None);
    }
    if !(200..300).contains(&status) {
        let body = worker::send::SendFuture::new(response.text())
            .await
            .unwrap_or_default();
        worker::console_warn!(
            "投信情報APIエラー: status={}, cd={}",
            status,
            association_fund_cd
        );
        return Err(nav_status_error(status, body));
    }

    let body = worker::send::SendFuture::new(response.text())
        .await
        .map_err(UpstreamError::Transport)?;
    parse_mufg_nav(&body)
}

/// 取得した基準価額を共有の fund_prices 行へ適用する(1コード1行)。
/// 保存済みの基準日より新しい取得日のときだけ更新し、日付の新しいほうを残す。
/// まだ行がないコードは挿入する
// tests/db_integration.rs からの検証用に公開しているため docs には出さない
#[doc(hidden)]
pub async fn apply_fund_nav(
    pool: &Db,
    association_fund_cd: &str,
    nav: &FundNav,
) -> Result<u64, ApiError> {
    crate::db::query(
        r#"
        INSERT INTO fund_prices (association_fund_cd, nav, prev_day_change, price_as_of)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (association_fund_cd) DO UPDATE SET
            nav = EXCLUDED.nav,
            prev_day_change = EXCLUDED.prev_day_change,
            price_as_of = EXCLUDED.price_as_of,
            updated_at = NOW(),
            fetch_error = NULL,
            fetch_failed_at = NULL
        WHERE fund_prices.price_as_of IS NULL
           OR fund_prices.price_as_of <= EXCLUDED.price_as_of
        "#,
        vec![
            Bind::from(association_fund_cd),
            Bind::from(nav.nav),
            Bind::from(nav.prev_day_change),
            Bind::from(nav.base_date),
        ],
    )
    .execute(pool)
    .await
    .map_err(ApiError::from)
}

/// 取得失敗の理由と時刻を共有値の行に残す。直前の失敗理由を上書きする。
/// 記録に失敗しても他のファンドの取得は止めない(ログに残すだけ)
async fn record_fetch_failure(pool: &Db, association_fund_cd: &str, reason: &str) {
    if let Err(e) = crate::db::query(
        r#"
        INSERT INTO fund_prices (association_fund_cd, updated_at, fetch_error, fetch_failed_at)
        VALUES ($1, NOW(), $2, NOW())
        ON CONFLICT (association_fund_cd) DO UPDATE SET
            fetch_error = EXCLUDED.fetch_error,
            fetch_failed_at = EXCLUDED.fetch_failed_at
        "#,
        vec![Bind::from(association_fund_cd), Bind::from(reason)],
    )
    .execute(pool)
    .await
    {
        svc_log!(
            error,
            "基準価額の失敗記録に失敗: cd={}, err={}",
            association_fund_cd,
            e
        )
    }
}

/// 失敗記録を消す。取得できたとき・対象外のとき使う
async fn clear_fetch_error(pool: &Db, association_fund_cd: &str) {
    if let Err(e) = crate::db::query(
        r#"
        UPDATE fund_prices
           SET fetch_error = NULL, fetch_failed_at = NULL
         WHERE association_fund_cd = $1
        "#,
        vec![Bind::from(association_fund_cd)],
    )
    .execute(pool)
    .await
    {
        svc_log!(
            warn,
            "基準価額の失敗記録の削除に失敗: cd={}, err={}",
            association_fund_cd,
            e
        )
    }
}

/// 失敗の理由。用户に見せる短い文に丸める。URL・応答本文・内部 Details は出さない
fn short_failure_reason(error: &ApiError) -> String {
    match error {
        ApiError::Upstream(UpstreamError::Http { status, .. }) => format!("HTTP {status}"),
        ApiError::Upstream(UpstreamError::RateLimited) => "レート制限".to_string(),
        ApiError::Upstream(UpstreamError::Invalid(_)) => "検証エラー".to_string(),
        ApiError::Upstream(UpstreamError::Decode(_)) => "応答の解析に失敗".to_string(),
        #[cfg(target_arch = "wasm32")]
        ApiError::Upstream(UpstreamError::Transport(_)) => "通信エラー".to_string(),
        ApiError::Database(_) => "保存エラー".to_string(),
        _ => "取得エラー".to_string(),
    }
}

/// 「いま公表済みのはずの、直近の基準価額の日」(E)。JST 23:00 を過ぎた平日は当日、
/// それより前は前の平日(土日は金曜)。祝日は扱わない。
/// SQL の fund_expected_nav_date と同じ規則。表示側の「最新/更新待ち」判定は
/// SQL 側が行い、こちらは更新対象の選定とテスト用
pub fn expected_nav_date(now: DateTime<Utc>) -> NaiveDate {
    let jst = now + chrono::Duration::hours(9);
    let mut date = jst.date_naive();
    if jst.hour() < 23 {
        date -= chrono::Duration::days(1);
    }
    date - chrono::Duration::days(match date.weekday() {
        Weekday::Sat => 1,
        Weekday::Sun => 2,
        _ => 0,
    })
}

/// cron 更新で狙う基準日。基準価額の公表は当日の夕方なので、
/// JST 19:00 過ぎの平日は当日を狙う(取れなければ次の回で拾う)
fn nav_publication_date(now: DateTime<Utc>) -> NaiveDate {
    let jst = now + chrono::Duration::hours(9);
    let mut date = jst.date_naive();
    if jst.hour() < 19 {
        date -= chrono::Duration::days(1);
    }
    date - chrono::Duration::days(match date.weekday() {
        Weekday::Sat => 1,
        Weekday::Sun => 2,
        _ => 0,
    })
}

/// cron から呼ばれる更新処理。保有中かつ対応表を持つ協会コードのうち、狙う基準日より
/// 共有値の price_as_of が古いものを最終試行時刻の古い順に上限件数だけ処理する。
/// 1ファンドの失敗はほかの処理を止めず、前回値を残す。
/// 戻り値は取得に成功したコード数（ログ用）
pub async fn refresh_fund_prices(pool: &Db, base_url: Option<String>) -> Result<usize, ApiError> {
    let target = nav_publication_date(Utc::now());
    let codes: Vec<String> = crate::db::query(
        r#"
        SELECT fc.association_fund_cd
        FROM fund_balances fb
        JOIN fund_codes fc ON fc.normalized_name = fb.normalized_fund_name
        LEFT JOIN fund_prices fp ON fp.association_fund_cd = fc.association_fund_cd
        GROUP BY fc.association_fund_cd
        HAVING BOOL_OR(fp.price_as_of IS NULL) OR MIN(fp.price_as_of) < $1
        ORDER BY MIN(fp.updated_at) ASC NULLS FIRST
        LIMIT $2
        "#,
        vec![Bind::from(target), Bind::from(MAX_CODES_PER_RUN)],
    )
    .fetch_all::<(String,), _>(pool)
    .await?
    .into_iter()
    .map(|(code,)| code)
    .collect();

    // 選んだコードの試行時刻を先に進めておく。取得に失敗しても順位が末尾へ回るため、
    // 連続して失敗するコードが後続の更新を永久に塞ぐことを防ぐ。
    // 未取得(行なし)のコードには値なしの行を作り、次回以降も対象のままにする
    if !codes.is_empty() {
        crate::db::query(
            r#"
            INSERT INTO fund_prices (association_fund_cd, updated_at)
            SELECT code, NOW() FROM UNNEST($1::text[]) AS t(code)
            ON CONFLICT (association_fund_cd) DO UPDATE SET updated_at = NOW()
            "#,
            vec![Bind::from(codes.clone())],
        )
        .execute(pool)
        .await?;
    }

    let base_url = base_url.unwrap_or_else(|| API_BASE_URL.to_string());
    let mut fetched = 0;
    let mut excluded = 0;
    let mut failed = 0;
    for (i, code) in codes.iter().enumerate() {
        if i > 0 {
            crate::services::dividend_cache::sleep_millis(FETCH_GAP_MS).await;
        }
        match fetch_fund_nav(&base_url, code).await {
            Ok(Some(nav)) => {
                fetched += 1;
                match apply_fund_nav(pool, code, &nav).await {
                    Ok(rows) => {
                        // 共有値が古く更新されなかったときだけ失敗記録が残りうるので消す
                        if rows == 0 {
                            clear_fetch_error(pool, code).await;
                        }
                        svc_log!(
                            info,
                            "基準価額更新: cd={}, base_date={}, rows={}",
                            code,
                            nav.base_date,
                            rows
                        );
                    }
                    Err(e) => {
                        failed += 1;
                        record_fetch_failure(pool, code, &short_failure_reason(&e)).await;
                        svc_log!(error, "基準価額の反映に失敗: cd={}, err={}", code, e)
                    }
                }
            }
            Ok(None) => {
                excluded += 1;
                // 対象外のコードは取得失敗ではないので、直前の失敗記録を消す
                clear_fetch_error(pool, code).await;
                svc_log!(info, "投信情報APIの対象外(前回値を維持): cd={}", code)
            }
            Err(e) => {
                failed += 1;
                record_fetch_failure(pool, code, &short_failure_reason(&e)).await;
                svc_log!(
                    warn,
                    "基準価額の取得に失敗（前回値を維持）: cd={}, err={}",
                    code,
                    e
                )
            }
        }
    }

    svc_log!(
        info,
        "基準価額更新の結果: 対象={}件, 取得={}件, 対象外={}件, 失敗={}件",
        codes.len(),
        fetched,
        excluded,
        failed
    );
    if !codes.is_empty() && fetched + excluded == 0 {
        // 1件も正常な応答がない = 個別障害ではなく取得元の応答・エンドポイント変更を疑う
        svc_log!(
            error,
            "基準価額の自動取得が全ファンド失敗。取得元（投信情報API）の変更の可能性あり"
        );
    }
    Ok(fetched)
}

/// 予定時刻（UTC エポックミリ秒）が JST 20:00/23:00 ちょうどなら基準価額更新を
/// 実行するかを返す。cron は1分おきに飛ぶため、回りすぎないよう1日2回の時刻に絞る
pub fn should_fund_refresh_at(scheduled_ms: f64) -> bool {
    DateTime::from_timestamp_millis(scheduled_ms as i64).is_some_and(|t| {
        let jst = t + chrono::Duration::hours(9);
        matches!(jst.hour(), 20 | 23) && jst.minute() == 0
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    /// 実際の応答は { result: {status}, datasets: [ {nav,...} ] } の形
    fn nav_body(nav: &str, base_date: &str, cmp: &str, pct: &str) -> String {
        format!(
            r#"{{"result":{{"status":200}},"datasets":[{{"association_fund_cd":"03311187","fund_name":"テスト","base_date":"{base_date}","nav":"{nav}","cmp_prev_day":"{cmp}","percentage_change":"{pct}"}}]}}"#
        )
    }

    #[test]
    fn test_parse_mufg_nav_valid_response() {
        let nav = parse_mufg_nav(include_str!(
            "../../tests/fixtures/mufg_nav_latest_03311187.json"
        ))
        .expect("parse");
        assert_eq!(
            nav,
            Some(FundNav {
                nav: dec!(44943),
                prev_day_change: dec!(369),
                base_date: NaiveDate::from_ymd_opt(2026, 10, 20).unwrap(),
            })
        );
    }

    #[test]
    fn test_parse_mufg_nav_not_found_is_excluded() {
        // 対象外(status=404、datasets なし)はエラーにせず Ok(None)。前回値を維持し
        // 失敗件数にも含めない
        assert!(matches!(
            parse_mufg_nav(include_str!(
                "../../tests/fixtures/mufg_nav_latest_29313233.json"
            )),
            Ok(None)
        ));
    }

    #[test]
    fn test_parse_mufg_nav_accepts_numeric_fields() {
        let body = r#"{"result":{"status":200},"datasets":[{"nav":44943,"base_date":20261020,"cmp_prev_day":369,"percentage_change":0.828}]}"#;
        assert!(matches!(parse_mufg_nav(body), Ok(Some(_))));
    }

    #[test]
    fn test_parse_mufg_nav_rejects_each_validation_rule() {
        // 有効な応答を基準に、各規則だけを壊した応答がすべて落ちること
        let cases: &[(&str, String)] = &[
            ("nav が 0", nav_body("0", "20261020", "369", "0.8")),
            ("nav が負", nav_body("-10", "20261020", "369", "0.8")),
            (
                "nav が数値でない",
                nav_body("abc", "20261020", "369", "0.8"),
            ),
            (
                "base_date が不正",
                nav_body("44943", "2026-10-20", "369", "0.8"),
            ),
            (
                "cmp_prev_day が数値でない",
                nav_body("44943", "20261020", "---", "0.8"),
            ),
            ("符号が不一致", nav_body("44943", "20261020", "369", "-0.8")),
            (
                "20% 超の変動",
                nav_body("44943", "20261020", "9000", "20.0"),
            ),
        ];
        for (label, body) in cases {
            assert!(parse_mufg_nav(body).is_err(), "{label}");
        }
        // ちょうど 20% は許容し、0 同士の符号は一致として扱う
        assert!(matches!(
            parse_mufg_nav(&nav_body("44943", "20261020", "8988.6", "20.0")),
            Ok(Some(_))
        ));
        assert!(matches!(
            parse_mufg_nav(&nav_body("44943", "20261020", "0", "0")),
            Ok(Some(_))
        ));
        // JSON として不正・項目欠落
        assert!(parse_mufg_nav("not json").is_err());
        assert!(parse_mufg_nav("{}").is_err());
        // エンベロープの不正: status が 200 でない、datasets が無い・空
        assert!(parse_mufg_nav(r#"{"result":{"status":500},"datasets":[]}"#).is_err());
        assert!(parse_mufg_nav(r#"{"result":{"status":200}}"#).is_err());
        assert!(parse_mufg_nav(r#"{"result":{"status":200},"datasets":[]}"#).is_err());
    }

    #[test]
    fn test_nav_status_error_classifies() {
        assert!(matches!(
            nav_status_error(429, "ignored".to_string()),
            ApiError::Upstream(UpstreamError::RateLimited)
        ));
        for status in [404, 500] {
            match nav_status_error(status, "bad".to_string()) {
                ApiError::Upstream(UpstreamError::Http { status: s, .. }) => {
                    assert_eq!(s, status)
                }
                other => panic!("expected Http error, got {other}"),
            }
        }
    }

    #[test]
    fn test_short_failure_reason_hides_details() {
        // 画面に出すのは短い文だけ。応答本文・URL・内部 Details を含めない
        let cases: [(ApiError, &str); 4] = [
            (
                ApiError::Upstream(UpstreamError::Http {
                    status: 403,
                    body: "https://example.invalid/secret".to_string(),
                }),
                "HTTP 403",
            ),
            (ApiError::Upstream(UpstreamError::RateLimited), "レート制限"),
            (
                ApiError::Upstream(UpstreamError::Invalid(
                    "投信情報APIの応答: nav が 0 以下".to_string(),
                )),
                "検証エラー",
            ),
            (
                ApiError::Upstream(UpstreamError::Decode(
                    serde_json::from_str::<serde_json::Value>("bad").unwrap_err(),
                )),
                "応答の解析に失敗",
            ),
        ];
        for (error, expected) in cases {
            let reason = short_failure_reason(&error);
            assert_eq!(reason, expected);
            // 応答本文・URL・検証の内部 Details を含めない
            assert!(!reason.contains("example.invalid"), "{reason}");
            assert!(!reason.contains("nav が"), "{reason}");
        }
        // 想定外のエラーでも空や詳細の無い文言になる
        let fallback = short_failure_reason(&ApiError::NotFound);
        assert!(!fallback.is_empty());
        assert!(!fallback.contains("Not found"));
    }

    #[test]
    fn test_expected_nav_date() {
        let jst = |y: i32, m: u32, d: u32, h: u32, mi: u32| {
            let utc = NaiveDate::from_ymd_opt(y, m, d)
                .unwrap()
                .and_hms_opt(h, mi, 0)
                .unwrap()
                - chrono::Duration::hours(9);
            expected_nav_date(utc.and_utc())
        };
        let date = |y: i32, m: u32, d: u32| NaiveDate::from_ymd_opt(y, m, d).unwrap();

        // 平日は JST 23:00 を境に前日/当日(2026-10-20 は火曜)
        assert_eq!(jst(2026, 10, 20, 9, 0), date(2026, 10, 19));
        assert_eq!(jst(2026, 10, 20, 22, 59), date(2026, 10, 19));
        assert_eq!(jst(2026, 10, 20, 23, 1), date(2026, 10, 20));

        // 金曜 23:00 過ぎは金曜のまま、土日は時刻に依らず金曜
        assert_eq!(jst(2026, 10, 23, 23, 1), date(2026, 10, 23));
        assert_eq!(jst(2026, 10, 24, 9, 0), date(2026, 10, 23));
        assert_eq!(jst(2026, 10, 24, 23, 30), date(2026, 10, 23));
        assert_eq!(jst(2026, 10, 25, 12, 0), date(2026, 10, 23));

        // 月曜は 23:00 前まで前の金曜
        assert_eq!(jst(2026, 10, 26, 9, 0), date(2026, 10, 23));
        assert_eq!(jst(2026, 10, 26, 23, 1), date(2026, 10, 26));
    }

    #[test]
    fn test_nav_publication_date() {
        let jst = |y: i32, m: u32, d: u32, h: u32| {
            let utc = NaiveDate::from_ymd_opt(y, m, d)
                .unwrap()
                .and_hms_opt(h, 0, 0)
                .unwrap()
                - chrono::Duration::hours(9);
            nav_publication_date(utc.and_utc())
        };
        let date = |y: i32, m: u32, d: u32| NaiveDate::from_ymd_opt(y, m, d).unwrap();

        // 更新回(JST 20:00/23:00)では平日当日を狙い、土日の回は金曜を狙う
        assert_eq!(jst(2026, 10, 20, 20), date(2026, 10, 20));
        assert_eq!(jst(2026, 10, 20, 23), date(2026, 10, 20));
        assert_eq!(jst(2026, 10, 24, 20), date(2026, 10, 23));
        assert_eq!(jst(2026, 10, 26, 20), date(2026, 10, 26));
        // 公表前(夕方前)は前の平日を狙う
        assert_eq!(jst(2026, 10, 20, 9), date(2026, 10, 19));
    }

    #[test]
    fn test_should_fund_refresh_at_runs_at_jst_20_and_23() {
        let ms = |y: i32, mo: u32, d: u32, h: u32, mi: u32| {
            (NaiveDate::from_ymd_opt(y, mo, d)
                .unwrap()
                .and_hms_opt(h, mi, 0)
                .unwrap()
                - chrono::Duration::hours(9))
            .and_utc()
            .timestamp_millis() as f64
        };
        // JST 20:00 と 23:00 ちょうどだけ実行
        assert!(should_fund_refresh_at(ms(2026, 10, 20, 20, 0)));
        assert!(should_fund_refresh_at(ms(2026, 10, 20, 23, 0)));
        assert!(!should_fund_refresh_at(ms(2026, 10, 20, 20, 1)));
        assert!(!should_fund_refresh_at(ms(2026, 10, 20, 22, 59)));
        assert!(!should_fund_refresh_at(ms(2026, 10, 20, 23, 1)));
        assert!(!should_fund_refresh_at(ms(2026, 10, 20, 9, 0)));
    }
}
