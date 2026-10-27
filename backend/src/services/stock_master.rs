//! stock(銘柄マスタ)を J-Quants 上場銘柄一覧から全置換する。
//! scheduled イベントから週1回(または表が空のとき=初期投入)実行する。

use crate::db::{Bind, Db};
use crate::errors::ApiError;
use crate::services::jquants::{EquitiesMasterResponse, JQuantsClient};
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::HashSet;

/// market_data_provider_rate_control.provider の銘柄マスタ更新用キー
const MASTER_PROVIDER: &str = "jquants-equities-master";

/// stock テーブルの1行分。J-Quants の5桁コードは先頭4桁へ縮約済み
pub struct StockRow {
    pub date: NaiveDate,
    pub code: String,
    pub name: String,
    pub market_category: String,
    pub industry_category_33: Option<String>,
    pub industry_category_17: Option<String>,
    pub size_category: Option<String>,
}

/// 上場銘柄一覧を stock 行へ写す。
/// 5桁コードのうち末尾 `0` の行だけが普通株（末尾 5/6 は優先株式）なので採用し、
/// `code` は先頭4桁とする。同一の4桁 code が複数来た場合は先着を残す（防御）
fn map_master(response: &EquitiesMasterResponse) -> Vec<StockRow> {
    let mut seen = HashSet::new();
    let non_empty = |s: &Option<String>| s.as_deref().filter(|v| !v.is_empty()).map(str::to_owned);
    response
        .data
        .iter()
        .filter(|row| row.code.len() == 5 && row.code.is_ascii() && row.code.ends_with('0'))
        .filter(|row| seen.insert(row.code[..4].to_string()))
        .filter_map(|row| {
            Some(StockRow {
                date: NaiveDate::parse_from_str(&row.date, "%Y-%m-%d").ok()?,
                code: row.code[..4].to_string(),
                name: row.company_name.clone(),
                market_category: row.market_name.clone(),
                industry_category_33: non_empty(&row.sector33_name),
                industry_category_17: non_empty(&row.sector17_name),
                size_category: non_empty(&row.scale_category),
            })
        })
        .collect()
}

/// stock を1トランザクションで全置換する。戻り値は投入件数
pub async fn replace_master(pool: &Db, rows: &[StockRow]) -> Result<usize, ApiError> {
    let tx = pool.begin().await?;
    let result = async {
        crate::db::query("DELETE FROM public.stock", vec![])
            .execute(&tx)
            .await?;
        let inserted = crate::db::query(
            r#"
            INSERT INTO public.stock
                (date, code, name, market_category,
                 industry_category_33, industry_category_17, size_category)
            SELECT * FROM unnest(
                $1::date[], $2::text[], $3::text[], $4::text[], $5::text[], $6::text[], $7::text[]
            )
            "#,
            vec![
                Bind::from(rows.iter().map(|r| r.date).collect::<Vec<_>>()),
                Bind::from(rows.iter().map(|r| r.code.clone()).collect::<Vec<_>>()),
                Bind::from(rows.iter().map(|r| r.name.clone()).collect::<Vec<_>>()),
                Bind::from(
                    rows.iter()
                        .map(|r| r.market_category.clone())
                        .collect::<Vec<_>>(),
                ),
                Bind::from(
                    rows.iter()
                        .map(|r| r.industry_category_33.clone())
                        .collect::<Vec<_>>(),
                ),
                Bind::from(
                    rows.iter()
                        .map(|r| r.industry_category_17.clone())
                        .collect::<Vec<_>>(),
                ),
                Bind::from(
                    rows.iter()
                        .map(|r| r.size_category.clone())
                        .collect::<Vec<_>>(),
                ),
            ],
        )
        .execute(&tx)
        .await?;
        Ok::<u64, ApiError>(inserted)
    }
    .await;

    match result {
        Ok(inserted) => {
            tx.commit().await?;
            Ok(usize::try_from(inserted).unwrap_or(usize::MAX))
        }
        Err(e) => {
            tx.rollback().await?;
            Err(e)
        }
    }
}

/// 次に更新してよい時刻(`next_available_at`)と現在時刻から更新要否を判定する純関数。
/// `stock.date`(上流の基準日)は無料プランでは常に約12週間遅れで経過時間の代わりに
/// ならないため、こちらで記録する時刻で判定する。枠の行が無い(None)は初回として
/// 実行する。stock が空の初回投入もこの経路で起きる(更新は成功すれば必ず行を残すため、
/// stock 空のまま枠が未来なのは失敗待ち・実行中だけで、待つのが正しい)
fn should_refresh(next_available_at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    next_available_at.is_none_or(|at| at <= now)
}

/// 更新枠の「次に更新してよい時刻」を読む。行が無ければ None(=初回)
async fn refresh_gate_next_available_at(pool: &Db) -> Result<Option<DateTime<Utc>>, ApiError> {
    crate::db::query_scalar::<DateTime<Utc>>(
        "SELECT next_available_at FROM market_data_provider_rate_control WHERE provider = $1",
        vec![Bind::from(MASTER_PROVIDER)],
    )
    .fetch_optional(pool)
    .await
    .map_err(ApiError::from)
}

/// 更新の実行権を原子的に取る。取れた(true)回だけ更新を実行するため、
/// 同時に動いても1回だけになる。行が無ければ INSERT して初回として取る
async fn claim_refresh_slot(pool: &Db) -> Result<bool, ApiError> {
    crate::db::query_scalar::<i32>(
        r#"
        INSERT INTO market_data_provider_rate_control (provider, next_available_at)
            VALUES ($1, NOW() + INTERVAL '7 days')
        ON CONFLICT (provider) DO UPDATE
            SET next_available_at = NOW() + INTERVAL '7 days'
            WHERE market_data_provider_rate_control.next_available_at <= NOW()
        RETURNING 1
        "#,
        vec![Bind::from(MASTER_PROVIDER)],
    )
    .fetch_optional(pool)
    .await
    .map(|v| v.is_some())
    .map_err(ApiError::from)
}

/// 更新失敗時の再試行を1時間後に遅らせる(毎分のやり直しを止める)
async fn postpone_refresh_slot(pool: &Db) -> Result<(), ApiError> {
    crate::db::query(
        "UPDATE market_data_provider_rate_control
         SET next_available_at = NOW() + INTERVAL '1 hour'
         WHERE provider = $1",
        vec![Bind::from(MASTER_PROVIDER)],
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// `scheduled_ms`(UTC エポックミリ秒)から UTC 時刻を得る。
/// `Utc::now()` には依存しない(テスト可能なように scheduled イベント時刻のみを使う)
fn scheduled_time(scheduled_ms: f64) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp_millis(scheduled_ms as i64)
}

/// market_data_provider_rate_control の行(次に更新してよい時刻)で更新要否を判定し、
/// 枠を取れた回だけ stock を全置換する。stock が空の初回は枠の行が無いため投入される。
/// 失敗時は次回を1時間後に遅らせる。戻り値は `Some(投入件数)`、スキップ時は `None`
pub async fn maybe_refresh(
    pool: &Db,
    client: &JQuantsClient,
    scheduled_ms: f64,
) -> Result<Option<usize>, ApiError> {
    let now = match scheduled_time(scheduled_ms) {
        Some(t) => t,
        None => return Ok(None),
    };
    let next_available_at = refresh_gate_next_available_at(pool).await?;
    if !should_refresh(next_available_at, now) {
        return Ok(None);
    }
    // 判定の後に他インスタンスが枠を取っても、claim が false で打ち止まる
    if !claim_refresh_slot(pool).await? {
        return Ok(None);
    }

    let result = async {
        let response = client.get_equities_master().await?;
        let rows = map_master(&response);
        // 上流の応答が0件(契約変更・フィルタ全落ち)なら、マスタを空にしない
        if rows.is_empty() {
            return Err(ApiError::Internal("J-Quants 上場銘柄一覧が0件"));
        }
        replace_master(pool, &rows).await
    }
    .await;
    match result {
        Ok(inserted) => Ok(Some(inserted)),
        Err(e) => {
            if let Err(pe) = postpone_refresh_slot(pool).await {
                tracing::error!("銘柄マスタ更新の再試行時刻の記録に失敗: {pe}");
            }
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::jquants::EquitiesMasterData;
    use chrono::TimeZone;

    fn master_row(code: &str, name: &str) -> EquitiesMasterData {
        EquitiesMasterData {
            date: "2026-10-15".to_string(),
            code: code.to_string(),
            company_name: name.to_string(),
            market_name: "プライム".to_string(),
            sector17_name: Some("情報通信・サービスその他".to_string()),
            sector33_name: Some("情報・通信業".to_string()),
            scale_category: Some("TOPIX Large70".to_string()),
        }
    }

    fn response(rows: Vec<EquitiesMasterData>) -> EquitiesMasterResponse {
        EquitiesMasterResponse { data: rows }
    }

    #[test]
    fn test_map_master_keeps_only_zero_tail_and_truncates_to_4_digits() {
        let rows = map_master(&response(vec![
            master_row("13010", "極洋"),
            master_row("25935", "伊藤園 優先株式"),
            master_row("25936", "日本航空 優先株式"),
            master_row("25930", "伊藤園"),
        ]));
        let codes: Vec<&str> = rows.iter().map(|r| r.code.as_str()).collect();
        assert_eq!(codes, ["1301", "2593"]);
        // 優先株式が先に来ても採用されるのは普通株(末尾0)の行だけ
        let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["極洋", "伊藤園"]);
    }

    #[test]
    fn test_map_master_dedupes_same_4_digit_code() {
        // 末尾 0 だけでは衝突しないため、防御として先着を残す経路を検証する
        let rows = map_master(&response(vec![
            master_row("13010", "先の行"),
            master_row("13010", "後の行"),
        ]));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "先の行");
    }

    #[test]
    fn test_map_master_skips_malformed_rows() {
        for code in ["1234", "123456", "1234A", "１２３４０"] {
            assert!(
                map_master(&response(vec![master_row(code, "x")])).is_empty(),
                "{code}"
            );
        }
        let mut bad_date = master_row("13010", "x");
        bad_date.date = "2026/10/15".to_string();
        assert!(map_master(&response(vec![bad_date])).is_empty());
    }

    #[test]
    fn test_map_master_maps_fields_and_none_if_empty() {
        let mut row = master_row("13010", "極洋");
        row.sector17_name = Some(String::new());
        row.sector33_name = None;
        let rows = map_master(&response(vec![row]));
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert_eq!(r.date, NaiveDate::from_ymd_opt(2026, 10, 15).unwrap());
        assert_eq!(r.name, "極洋");
        assert_eq!(r.market_category, "プライム");
        assert_eq!(r.industry_category_17, None);
        assert_eq!(r.industry_category_33, None);
        assert_eq!(r.size_category.as_deref(), Some("TOPIX Large70"));
    }

    fn fixed_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 18, 12, 0, 0)
            .single()
            .expect("固定時刻の生成に失敗")
    }

    #[test]
    fn test_should_refresh_no_gate_row_runs() {
        // 枠の行が無い = 初回(stock が空の初回投入を含む)なので実行する
        assert!(should_refresh(None, fixed_now()));
    }

    #[test]
    fn test_should_refresh_future_slot_skips() {
        let now = fixed_now();
        // 未来の枠は失敗待ち・実行中・直前の成功いずれかなので待つ
        assert!(!should_refresh(
            Some(now + chrono::Duration::minutes(1)),
            now
        ));
        assert!(!should_refresh(Some(now + chrono::Duration::days(7)), now));
    }

    #[test]
    fn test_should_refresh_due_slot_runs() {
        let now = fixed_now();
        // ちょうど・過去は実行する
        assert!(should_refresh(Some(now), now));
        assert!(should_refresh(
            Some(now - chrono::Duration::seconds(1)),
            now
        ));
    }
}
