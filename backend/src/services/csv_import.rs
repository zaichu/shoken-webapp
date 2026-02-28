use chrono::NaiveDate;
use encoding_rs::{SHIFT_JIS, UTF_8};

/// UTF-8 デコードを試み、失敗時は Shift-JIS にフォールバック
pub fn decode_bytes(bytes: &[u8]) -> String {
    // まず UTF-8 を試みる
    let (result, _, had_errors) = UTF_8.decode(bytes);
    if !had_errors {
        return result.into_owned();
    }
    // Shift-JIS にフォールバック
    let (result, _, _) = SHIFT_JIS.decode(bytes);
    result.into_owned()
}

/// 数値文字列をパース（カンマ区切り・括弧マイナス対応）
/// 例: "1,234" → 1234.0、"(500)" → -500.0
pub fn parse_number(s: &str) -> Result<f64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(0.0);
    }
    // 括弧表記はマイナス
    let (negative, s) = if s.starts_with('(') && s.ends_with(')') {
        (true, &s[1..s.len() - 1])
    } else {
        (false, s)
    };
    let s = s.replace(',', "");
    let value: f64 = s
        .parse()
        .map_err(|_| format!("数値のパースに失敗しました: '{}'", s))?;
    Ok(if negative { -value } else { value })
}

/// 日付文字列をパース（"YYYY/MM/DD" または "YYYY-MM-DD"）
pub fn parse_date(s: &str) -> Result<NaiveDate, String> {
    let s = s.trim();
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y/%m/%d") {
        return Ok(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(d);
    }
    Err(format!("日付のパースに失敗しました: '{}'", s))
}

/// 税金を計算する（特定口座かつ利益がある場合のみ）
pub fn compute_taxes(account: &str, realized_pnl: f64) -> (f64, f64) {
    const TAX_RATE: f64 = 0.20315;
    if account.contains("特定") && realized_pnl > 0.0 {
        let taxes = (realized_pnl * TAX_RATE).floor();
        let after_tax = realized_pnl - taxes;
        (taxes, after_tax)
    } else {
        (0.0, realized_pnl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number_normal() {
        assert_eq!(parse_number("1,234").unwrap(), 1234.0);
        assert_eq!(parse_number("500").unwrap(), 500.0);
        assert_eq!(parse_number("(500)").unwrap(), -500.0);
        assert_eq!(parse_number("").unwrap(), 0.0);
    }

    #[test]
    fn test_parse_date() {
        assert_eq!(
            parse_date("2024/01/15").unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()
        );
        assert_eq!(
            parse_date("2024-01-15").unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()
        );
    }

    #[test]
    fn test_compute_taxes_tokutei_profit() {
        let (taxes, after) = compute_taxes("特定", 10000.0);
        assert_eq!(taxes, 2031.0); // floor(10000 * 0.20315)
        assert_eq!(after, 7969.0);
    }

    #[test]
    fn test_compute_taxes_loss() {
        let (taxes, after) = compute_taxes("特定", -5000.0);
        assert_eq!(taxes, 0.0);
        assert_eq!(after, -5000.0);
    }

    #[test]
    fn test_compute_taxes_nisa() {
        let (taxes, after) = compute_taxes("NISA", 10000.0);
        assert_eq!(taxes, 0.0);
        assert_eq!(after, 10000.0);
    }

    #[test]
    fn test_decode_bytes_utf8() {
        let input = "テスト".as_bytes();
        assert_eq!(decode_bytes(input), "テスト");
    }
}
