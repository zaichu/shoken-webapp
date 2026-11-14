use super::*;

#[test]
fn columns_reason_includes_received_first_line() {
    let content = "約定日,受渡日,銘柄コード,口座\n2026/10/01,2026/10/03,9901,特定\n";
    assert_eq!(
        guard::inspect(CsvKind::MutualFund, content),
        Some(SkipReason::Columns)
    );
    let reason = columns_failure_reason(CsvKind::MutualFund, content);
    assert!(
        reason.contains("対応形式: 投資信託の取引履歴CSV"),
        "{reason}"
    );
    assert!(
        reason.contains("受け取った先頭行: 「約定日,受渡日,銘柄コード,口座」"),
        "{reason}"
    );
}

#[test]
fn columns_reason_omits_numeric_first_line() {
    let content = "123,456\n2026/10/01,国内株式,特定,架空銘柄\n";
    assert_eq!(
        guard::inspect(CsvKind::Dividend, content),
        Some(SkipReason::Columns)
    );
    let reason = columns_failure_reason(CsvKind::Dividend, content);
    assert!(!reason.contains("受け取った先頭行"), "{reason}");
}

#[test]
fn columns_reason_truncates_first_line_at_80_chars() {
    let long = format!("約定日,{}", "あ".repeat(100));
    let content = format!("{long}\n2026/10/01,xxx\n");
    assert_eq!(
        guard::inspect(CsvKind::MutualFund, &content),
        Some(SkipReason::Columns)
    );
    let reason = columns_failure_reason(CsvKind::MutualFund, &content);
    let expected: String = long.chars().take(80).collect();
    assert!(
        reason.contains(&format!("受け取った先頭行: 「{expected}…」")),
        "{reason}"
    );
    assert!(!reason.contains(&long), "{reason}");
}

#[test]
fn other_skip_reasons_add_no_first_line() {
    assert_eq!(
        guard::inspect(
            CsvKind::Dividend,
            "入金日,商品,口座,銘柄\n2026/10/01,国内株式,特定,架空�\n"
        ),
        Some(SkipReason::Mojibake)
    );
    assert_eq!(
        guard::inspect(CsvKind::Dividend, "<html><body>login</body></html>"),
        Some(SkipReason::Html)
    );
    assert_eq!(
        guard::inspect(CsvKind::Dividend, ""),
        Some(SkipReason::Empty)
    );
    assert!(!columns_failure_reason(CsvKind::Dividend, "").contains("受け取った先頭行"));
}
