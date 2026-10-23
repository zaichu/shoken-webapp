use super::*;

fn asset_csv(rows: usize) -> String {
    let mut csv = String::from("■特定口座\n\n銘柄コード,銘柄名,保有数量［株］\n");
    for i in 0..rows {
        csv.push_str(&format!("990{i},架空銘柄{i},100\n"));
    }
    csv
}

fn dividend_csv(rows: usize) -> String {
    let mut csv = String::from("入金日,商品,口座,銘柄\n");
    for _ in 0..rows {
        csv.push_str("2026/10/01,国内株式,特定,架空銘柄\n");
    }
    csv
}

#[test]
fn normal_csvs_pass() {
    assert_eq!(inspect(CsvKind::AssetBalance, &asset_csv(2)), None);
    assert_eq!(
        inspect(
            CsvKind::FundBalance,
            "投資信託種別,口座区分,ファンド,保有数量[口]\n投資信託,特定,架空ファンド,100\n"
        ),
        None
    );
    assert_eq!(inspect(CsvKind::Dividend, &dividend_csv(1)), None);
    assert_eq!(
        inspect(
            CsvKind::DomesticStock,
            "約定日,受渡日,銘柄コード,口座\n2026/10/01,2026/10/03,9901,特定\n"
        ),
        None
    );
    assert_eq!(
        inspect(
            CsvKind::MutualFund,
            "約定日,受渡日,ファンド名,口座\n2026/10/01,2026/10/03,架空ファンド,特定\n"
        ),
        None
    );
}

#[test]
fn empty_and_header_only_are_rejected() {
    assert_eq!(inspect(CsvKind::Dividend, ""), Some(SkipReason::Empty));
    assert_eq!(
        inspect(CsvKind::Dividend, "入金日,商品,口座,銘柄\n"),
        Some(SkipReason::Empty)
    );
    assert_eq!(
        inspect(
            CsvKind::AssetBalance,
            "■特定口座\n銘柄コード,銘柄名,保有数量［株］\n"
        ),
        Some(SkipReason::Empty)
    );
    // データが空行だけでも空
    assert_eq!(
        inspect(CsvKind::Dividend, "入金日,商品,口座,銘柄\n\n,,\n"),
        Some(SkipReason::Empty)
    );
}

#[test]
fn missing_required_columns_are_rejected() {
    // ヘッダはあるが必須列が無い
    assert_eq!(
        inspect(
            CsvKind::DomesticStock,
            "約定日,受渡日,取引\n2026/10/01,2026/10/03,売却\n"
        ),
        Some(SkipReason::Columns)
    );
    // 別種類の CSV を受け取った(見出しが違う)
    assert_eq!(
        inspect(
            CsvKind::Dividend,
            &dividend_csv(0).replace("入金日", "約定日")
        ),
        Some(SkipReason::Columns)
    );
    // マーカーの見出し自体が無い
    assert_eq!(
        inspect(CsvKind::AssetBalance, "コード,名称\n9901,架空\n"),
        Some(SkipReason::Columns)
    );
}

#[test]
fn html_is_rejected() {
    for content in [
        "<!DOCTYPE html><html><body>error</body></html>",
        "<html><body>login</body></html>",
        "  <div>not csv</div>",
    ] {
        assert_eq!(
            inspect(CsvKind::Dividend, content),
            Some(SkipReason::Html),
            "{content}"
        );
    }
}

#[test]
fn mojibake_is_rejected() {
    assert_eq!(
        inspect(
            CsvKind::Dividend,
            "入金日,商品,口座,銘柄\n2026/10/01,国内株式,特定,架空�\n"
        ),
        Some(SkipReason::Mojibake)
    );
}

#[test]
fn header_detection_skips_preamble_and_quoted_newlines() {
    // 表題行つき形式は先頭から10行以内の見出しを探す
    let with_title = format!("配当金明細\n\"改行を\n含む表題\"\n{}", dividend_csv(1));
    assert_eq!(inspect(CsvKind::Dividend, &with_title), None);
}

#[test]
fn plunge_threshold() {
    assert!(is_plunge(9, 20)); // 半分未満
    assert!(!is_plunge(10, 20)); // ちょうど半分は激減でない
    assert!(!is_plunge(11, 20));
    assert!(!is_plunge(0, 0));
    assert!(is_plunge(1, 3));
}
