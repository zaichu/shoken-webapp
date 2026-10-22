//! 受け取った CSV を取り込みに通す前の安全装置。
//! 取り込むかどうかの判定だけを持ち、通信・画面には触れない

use super::{CsvKind, SkipReason};
use crate::support::csv_flow::{csv_fields, decode_csv_bytes, record_ranges};

/// 種類ごとの見出しの見つけ方と、形式を見分ける必須の列。
/// marker は backend の CsvParserConfig.header_marker と同じ値。
/// 必須列は backend の変換が要求する列のうち、その種類の CSV を見分ける核
struct KindSpec {
    /// 先頭10レコード以内でこの文字で始まるフィールドを持つ行を見出しとする。
    /// None なら先頭レコードが見出し(backend の skip_header_rows=0・marker なしと同じ)
    marker: Option<&'static str>,
    required: &'static [&'static str],
}

fn spec_of(kind: CsvKind) -> KindSpec {
    match kind {
        CsvKind::AssetBalance => KindSpec {
            marker: Some("銘柄コード"),
            required: &["銘柄コード", "銘柄名", "保有数量"],
        },
        CsvKind::FundBalance => KindSpec {
            marker: Some("投資信託種別"),
            required: &["投資信託種別", "ファンド", "保有数量"],
        },
        CsvKind::Dividend => KindSpec {
            marker: Some("入金日"),
            required: &["入金日", "口座", "銘柄"],
        },
        CsvKind::DomesticStock => KindSpec {
            marker: None,
            required: &["約定日", "銘柄コード", "口座"],
        },
        CsvKind::MutualFund => KindSpec {
            marker: None,
            required: &["約定日", "ファンド名", "口座"],
        },
    }
}

fn record_fields(bytes: &[u8], range: (usize, usize)) -> Vec<String> {
    csv_fields(decode_csv_bytes(&bytes[range.0..range.1]).as_ref())
}

/// 取り込み不可なら理由を返す。None なら既存の preview/import へ進んでよい
pub(crate) fn inspect(kind: CsvKind, content: &str) -> Option<SkipReason> {
    // デコード失敗の置換文字を含む(文字化け)内容はどの種類でも通さない
    if content.contains('\u{FFFD}') {
        return Some(SkipReason::Mojibake);
    }
    let body = content.strip_prefix('\u{FEFF}').unwrap_or(content);
    // エラーページやログイン画面の HTML を CSV として受け付けない
    if body.trim_start().starts_with('<') {
        return Some(SkipReason::Html);
    }
    let bytes = body.as_bytes();
    let records = record_ranges(bytes);
    if records.is_empty() {
        return Some(SkipReason::Empty);
    }
    let spec = spec_of(kind);
    let header_index = match spec.marker {
        Some(marker) => match records.iter().take(10).position(|range| {
            record_fields(bytes, *range)
                .iter()
                .any(|field| field.trim().starts_with(marker))
        }) {
            Some(index) => index,
            None => return Some(SkipReason::Columns),
        },
        None => 0,
    };
    let header = record_fields(bytes, records[header_index]);
    let missing = spec
        .required
        .iter()
        .any(|col| !header.iter().any(|field| field.trim().starts_with(col)));
    if missing {
        return Some(SkipReason::Columns);
    }
    let has_data = records[header_index + 1..].iter().any(|range| {
        record_fields(bytes, *range)
            .iter()
            .any(|field| !field.trim().is_empty())
    });
    if !has_data {
        return Some(SkipReason::Empty);
    }
    None
}

/// 「いまの件数の半分未満」の激減判定
pub(crate) fn is_plunge(new_rows: usize, saved_rows: usize) -> bool {
    new_rows * 2 < saved_rows
}

#[cfg(test)]
mod tests;
