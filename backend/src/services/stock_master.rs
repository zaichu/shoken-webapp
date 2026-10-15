//! stock(銘柄マスタ)を J-Quants 上場銘柄一覧から全置換する。
//! scheduled イベントから週1回(または表が空のとき=初期投入)実行する。

use crate::db::{Bind, Db};
use crate::errors::ApiError;
use crate::services::jquants::{EquitiesMasterResponse, JQuantsClient};
use chrono::{DateTime, Datelike, NaiveDate, Timelike, Weekday};
use std::collections::HashSet;

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
pub fn map_master(response: &EquitiesMasterResponse) -> Vec<StockRow> {
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

/// 予定時刻(UTC エポックミリ秒)が週次の更新時刻(日曜 20:00 UTC = 月曜 05:00 JST)なら true
pub fn should_refresh_at(scheduled_ms: f64) -> bool {
    DateTime::from_timestamp_millis(scheduled_ms as i64)
        .is_some_and(|t| t.weekday() == Weekday::Sun && t.hour() == 20 && t.minute() == 0)
}

/// stock が空なら頻度に関わらず実行(初期投入)、行があれば週次時刻のみ実行する。
/// 戻り値は `Some(投入件数)`、スキップ時は `None`
pub async fn maybe_refresh(
    pool: &Db,
    client: &JQuantsClient,
    scheduled_ms: f64,
) -> Result<Option<usize>, ApiError> {
    let has_rows = crate::db::query_scalar::<bool>("SELECT EXISTS(SELECT 1 FROM stock)", vec![])
        .fetch_one(pool)
        .await?;
    if has_rows && !should_refresh_at(scheduled_ms) {
        return Ok(None);
    }

    let response = client.get_equities_master().await?;
    let rows = map_master(&response);
    // 上流の応答が0件(契約変更・フィルタ全落ち)なら、マスタを空にしない
    if rows.is_empty() {
        return Err(ApiError::Internal("J-Quants 上場銘柄一覧が0件"));
    }
    Ok(Some(replace_master(pool, &rows).await?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::jquants::EquitiesMasterData;
    use chrono::NaiveTime;

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

    #[test]
    fn test_should_refresh_at_runs_weekly_sunday_20_utc() {
        let ms = |y: i32, m: u32, d: u32, h: u32, mi: u32| {
            NaiveDate::from_ymd_opt(y, m, d)
                .unwrap()
                .and_time(NaiveTime::from_hms_opt(h, mi, 0).unwrap())
                .and_utc()
                .timestamp_millis() as f64
        };
        // 2026-10-18 は日曜
        assert!(should_refresh_at(ms(2026, 10, 18, 20, 0)));
        assert!(!should_refresh_at(ms(2026, 10, 18, 20, 1)));
        assert!(!should_refresh_at(ms(2026, 10, 18, 19, 0)));
        // 2026-10-19 は月曜(JST の月曜朝でも UTC 基準で判定)
        assert!(!should_refresh_at(ms(2026, 10, 19, 20, 0)));
        assert!(!should_refresh_at(ms(2026, 10, 16, 20, 0)));
    }
}
