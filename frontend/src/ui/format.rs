//! 画面に出す金額・率・日付の書式。書式は1か所に集める。
//! 金額(円マーク・桁区切り・符号)、小数桁、率、日付の関数をここに置き、
//! 画面ごとの独自の書式関数はここへ寄せる( docs/ui-design.md の「用語と書式」)

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};
use shared::format::{
    format_currency as format_currency_decimal, format_currency_with_options,
    format_percentage_value as format_percentage_value_decimal,
};

use crate::support::list_search::is_valid_iso_date;

/// 1株・1口あたりの金額(取得単価・現在値・基準価額・1株配当)の小数桁の上限
const UNIT_PRICE_DECIMALS_MAX: u32 = 2;

/// `Intl.NumberFormat`(maximumFractionDigits) 相当の丸め。
/// f64 の最短10進表記を10進の値として半分は遠ざけて丸める
/// (`1.005` → `1.01`、`2.675` → `2.68`)。
/// 非有限値はそのまま返し、巨大値は丸めずに返し、微小値は符号付きゼロにする。
pub fn intl_fixed(value: f64, decimals: u32) -> f64 {
    if !value.is_finite() {
        return value;
    }
    let negative = value.is_sign_negative();
    let fallback = || {
        if value.abs() < 1.0 {
            if negative { -0.0 } else { 0.0 }
        } else {
            value
        }
    };
    let Ok(decimal) = Decimal::from_str_exact(&value.abs().to_string()) else {
        return fallback();
    };
    let Some(result) = decimal
        .round_dp_with_strategy(decimals, RoundingStrategy::MidpointAwayFromZero)
        .to_f64()
    else {
        return fallback();
    };
    if negative { -result } else { result }
}

fn group_thousands(digits: &str) -> String {
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, byte) in digits.bytes().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(byte as char);
    }
    grouped
}

// 符号は呼び出し側が丸め前の値で `value < 0` と判定する（`-0.0` は負としない）。
// この関数は絶対値の桁区切りだけを返す。
pub fn format_abs_number(value: f64) -> Option<String> {
    if !value.is_finite() {
        return None;
    }
    let text = value.abs().to_string();
    let (integer, fraction) = match text.split_once('.') {
        Some((integer, fraction)) => (integer, Some(fraction)),
        None => (text.as_str(), None),
    };
    let grouped = group_thousands(integer);
    Some(match fraction {
        Some(fraction) => format!("{grouped}.{fraction}"),
        None => grouped,
    })
}

/// 数量などの数値(小数2桁・桁区切り)
pub fn format_number_value(value: f64) -> String {
    match format_abs_number(intl_fixed(value, 2)) {
        None => "—".to_string(),
        Some(body) if value < 0.0 => format!("-{body}"),
        Some(body) => body,
    }
}

/// f64 を Decimal に載せる。載らない値は None
pub fn f64_to_decimal(value: f64) -> Option<Decimal> {
    Decimal::from_str_exact(&value.to_string())
        .ok()
        .map(|value| value.normalize())
}

// Decimal に収まらない合計でも、f64 のまま同じ書式で出す
pub fn format_currency_beyond_decimal(value: f64) -> String {
    match format_abs_number(intl_fixed(value, 0)) {
        Some(body) if value < 0.0 => format!("-¥{body}"),
        Some(body) => format!("¥{body}"),
        None => "—".to_string(),
    }
}

/// 金額(円マーク・桁区切り・符号)
pub fn format_currency(value: f64) -> String {
    f64_to_decimal(value).map_or_else(
        || format_currency_beyond_decimal(value),
        format_currency_decimal,
    )
}

/// 列の中の値の小数桁(最大2桁)。同じ列の中の桁をそろえるための値で、
/// 整数円の値しかない列は0桁になる(不要な .00 を付けない)
pub fn unit_price_decimals(values: impl Iterator<Item = f64>) -> u32 {
    values
        .filter_map(|value| decimal_places(&format_currency(value)))
        .max()
        .unwrap_or(0)
        .min(UNIT_PRICE_DECIMALS_MAX)
}

fn decimal_places(text: &str) -> Option<u32> {
    let (_, fraction) = text.split_once('.')?;
    Some(fraction.len() as u32)
}

/// 1株・1口あたりの金額(取得単価・現在値・基準価額・1株配当)。
/// 小数点以下は列ごとに決めた桁数(decimals)で固定する
pub fn format_unit_price(value: f64, decimals: u32) -> String {
    f64_to_decimal(value).map_or_else(
        || format_unit_price_beyond_decimal(value, decimals),
        |value| format_currency_with_options(value, "¥", decimals, decimals),
    )
}

// Decimal に収まらない値でも小数桁をそろえて出す
fn format_unit_price_beyond_decimal(value: f64, decimals: u32) -> String {
    let rounded = intl_fixed(value, decimals);
    let Some(body) = format_abs_number(rounded) else {
        return "—".to_string();
    };
    let (integer, fraction) = body
        .split_once('.')
        .map_or((body.as_str(), ""), |(i, f)| (i, f));
    let fraction = format!("{fraction:0<width$}", width = decimals as usize);
    if value < 0.0 {
        format!("-¥{integer}.{fraction}")
    } else {
        format!("¥{integer}.{fraction}")
    }
}

/// 率(%)
pub fn format_fixed_percent(value: f64, decimals: u32) -> String {
    if !value.is_finite() {
        return "—".to_string();
    }
    f64_to_decimal(value).map_or_else(
        || {
            format!(
                "{:.prec$}%",
                intl_fixed(value, decimals),
                prec = decimals as usize
            )
        },
        |value| format_percentage_value_decimal(value, decimals),
    )
}

/// 率(小数2桁)
pub fn format_percentage_value(value: f64) -> String {
    format_fixed_percent(value, 2)
}

/// 基準日("YYYY-MM-DD")を "MM/DD" にする。想定外の形式はそのまま返す
pub fn format_price_date(as_of: &str) -> String {
    match as_of.split_once('-') {
        Some((year, month_day)) if year.len() == 4 && !month_day.is_empty() => {
            month_day.replace('-', "/")
        }
        _ => as_of.to_string(),
    }
}

/// 基準価額の自動更新の時刻("YYYY-MM-DD HH:MM")を "MM/DD HH:MM" にする。
/// 想定外の形式はそのまま返す
pub fn format_nav_updated_at(updated_at: &str) -> String {
    let Some((date, time)) = updated_at.split_once(' ') else {
        return updated_at.to_string();
    };
    if date.len() != 10 || time.len() < 5 {
        return updated_at.to_string();
    }
    format!("{} {}", format_price_date(date), &time[..5])
}

/// 日付("YYYY-MM-DD")を "YYYY/MM/DD" にする。日付として読めない値は「—」
pub fn format_iso_date(value: &str) -> String {
    if is_valid_iso_date(value) {
        format!("{}/{}/{}", &value[..4], &value[5..7], &value[8..])
    } else {
        "—".to_string()
    }
}

/// 率(整数%)
pub fn format_integer_percent(value: f64) -> String {
    format!("{}%", value.round() as i64)
}

/// 日付の年月日を取り出す。日付として読めない値は None
pub fn date_parts(date: &str) -> Option<(i32, u32, u32)> {
    if !date.is_ascii()
        || date.len() < 10
        || date.as_bytes()[4] != b'-'
        || date.as_bytes()[7] != b'-'
    {
        return None;
    }
    let year: i32 = date[..4].parse().ok()?;
    let month: u32 = date[5..7].parse().ok()?;
    let day: u32 = date[8..10].parse().ok()?;
    if !(1..=12).contains(&month) || day == 0 {
        return None;
    }
    Some((year, month, day))
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        _ => 28,
    }
}

/// updated_at(UTC) を JST の取込日として出す。+9h で日付が繰り上がる場合だけ桁を進める
pub fn jst_date(iso: &str) -> Option<(i32, u32, u32)> {
    let (mut year, mut month, mut day) = date_parts(iso)?;
    let hour: u32 = if iso.len() >= 13 && (iso.as_bytes()[10] == b'T' || iso.as_bytes()[10] == b' ')
    {
        iso[11..13].parse().ok()?
    } else {
        0
    };
    if hour + 9 >= 24 {
        day += 1;
        if day > days_in_month(year, month) {
            day = 1;
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }
    }
    Some((year, month, day))
}

/// 日付("YYYY-MM-DD")を "M/D" にする(取込日など、桁を詰めて出す画面用)
pub fn format_short_date(date: &str) -> String {
    match jst_date(date) {
        Some((_, month, day)) => format!("{month}/{day}"),
        None => "—".to_string(),
    }
}

/// updated_at(UTC) を JST の時刻 "HH:MM" で出す。日付は jst_date が返す
pub fn format_jst_time(iso: &str) -> String {
    let bytes = iso.as_bytes();
    if bytes.len() < 16 || bytes[13] != b':' {
        return "—".to_string();
    }
    let (Ok(hour), Ok(minute)) = (iso[11..13].parse::<u32>(), iso[14..16].parse::<u32>()) else {
        return "—".to_string();
    };
    format!("{:02}:{:02}", (hour + 9) % 24, minute)
}

/// 最新の取込日(JST)。"Y/M/D" で返す。有効な日時が無ければ None
pub fn format_latest_import_date<'a>(timestamps: impl Iterator<Item = &'a str>) -> Option<String> {
    timestamps
        .filter_map(jst_date)
        .max()
        .map(|(year, month, day)| format!("{year}/{month}/{day}"))
}

/// 暦日(年・月・日)を「1970-01-01 からの日数」に直す。暦日の差を数えるためだけに使う
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = i64::from(year);
    let month = i64::from(month);
    let day = i64::from(day);
    // 3月を1月とする年(月をまたいでずらす)の実装
    let shifted_year = if month <= 2 { year - 1 } else { year };
    let era = if shifted_year >= 0 {
        shifted_year
    } else {
        shifted_year - 399
    } / 400;
    let year_of_era = shifted_year - era * 400;
    let shifted_month = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// 「1970-01-01 からの日数」を暦日(年・月・日)に戻す
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = if month <= 2 { year + 1 } else { year };
    (year as i32, month as u32, day as u32)
}

/// 2つの暦日の差(to − from)。後なら正、前でなければ負
pub fn calendar_days_between(from: (i32, u32, u32), to: (i32, u32, u32)) -> i64 {
    days_from_civil(to.0, to.1, to.2) - days_from_civil(from.0, from.1, from.2)
}

/// 今日(JST)の年月日
#[cfg(target_arch = "wasm32")]
fn today_jst() -> Option<(i32, u32, u32)> {
    // epoch 秒を9時間進めて JST の暦日を出す
    let jst_seconds = (js_sys::Date::now() / 1000.0) as i64 + 9 * 3600;
    Some(civil_from_days(jst_seconds.div_euclid(86_400)))
}

/// 今日(JST)の年月日。ネイティブでは None
#[cfg(not(target_arch = "wasm32"))]
fn today_jst() -> Option<(i32, u32, u32)> {
    None
}

/// 取込日から今日までの暦日の差。日付か今日が読めないときは None
pub fn days_since_jst_date(timestamp: &str) -> Option<i64> {
    let imported = jst_date(timestamp)?;
    let today = today_jst()?;
    Some(calendar_days_between(imported, today))
}

/// 数値(Decimal)を桁区切りで出す
pub fn format_decimal_number(value: Decimal) -> String {
    let text = value.to_string();
    let (sign, abs) = text
        .strip_prefix('-')
        .map_or(("", text.as_str()), |stripped| ("-", stripped));
    let (integer, fraction) = abs
        .split_once('.')
        .map_or((abs, None), |(i, f)| (i, Some(f)));
    let grouped = group_thousands(integer);
    match fraction {
        Some(fraction) => format!("{sign}{grouped}.{fraction}"),
        None => format!("{sign}{grouped}"),
    }
}

#[cfg(test)]
mod tests;
