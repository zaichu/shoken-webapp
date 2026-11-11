//! 画面 URL の正本(詳細パスの判定とリンク生成)。挙動は変えない。

pub(crate) const HOLDINGS_PREFIX: &str = "/assetbalance/holdings/";
pub(crate) const FUND_HOLDINGS_PREFIX: &str = "/assetbalance/funds/";

/// 詳細パスから ID だけを抜く。空・2段目は受け付けない
pub(crate) fn detail_id<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    let id = path.strip_prefix(prefix)?;
    (!id.is_empty() && !id.contains('/')).then_some(id)
}

/// 投信の詳細ページへのパス。リンク側は直書きせずこれを使う
pub(crate) fn fund_holding_href(id: impl std::fmt::Display) -> String {
    format!("{FUND_HOLDINGS_PREFIX}{id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detail_id_extracts_single_segment() {
        assert_eq!(
            detail_id("/assetbalance/holdings/abc-123", HOLDINGS_PREFIX),
            Some("abc-123")
        );
        assert_eq!(
            detail_id("/assetbalance/funds/fund-1", FUND_HOLDINGS_PREFIX),
            Some("fund-1")
        );
    }

    #[test]
    fn detail_id_rejects_empty_and_nested() {
        assert_eq!(detail_id("/assetbalance/holdings/", HOLDINGS_PREFIX), None);
        assert_eq!(
            detail_id("/assetbalance/holdings/a/b", HOLDINGS_PREFIX),
            None
        );
        assert_eq!(
            detail_id("/assetbalance/funds/", FUND_HOLDINGS_PREFIX),
            None
        );
    }

    #[test]
    fn detail_id_rejects_other_prefix() {
        assert_eq!(detail_id("/assetbalance/funds/abc", HOLDINGS_PREFIX), None);
        assert_eq!(detail_id("/receipts", HOLDINGS_PREFIX), None);
    }

    #[test]
    fn fund_holding_href_uses_funds_prefix() {
        assert_eq!(fund_holding_href("fund-1"), "/assetbalance/funds/fund-1");
    }
}
