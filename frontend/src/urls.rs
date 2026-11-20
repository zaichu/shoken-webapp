//! 画面 URL の正本(詳細パスの判定とリンク生成)。挙動は変えない。

pub(crate) const HOLDINGS_PREFIX: &str = "/assetbalance/holdings/";
pub(crate) const FUND_HOLDINGS_PREFIX: &str = "/assetbalance/funds/";

/// 詳細パスから ID だけを抜く。空・2段目は受け付けない
pub(crate) fn detail_id<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    let id = path.strip_prefix(prefix)?.split('#').next()?;
    (!id.is_empty() && !id.contains('/')).then_some(id)
}

/// 投信の詳細ページへのパス。リンク側は直書きせずこれを使う
pub(crate) fn fund_holding_href(id: impl std::fmt::Display) -> String {
    format!("{FUND_HOLDINGS_PREFIX}{id}")
}

/// `?tab=` など1つのクエリ値を抜く。`path` は pathname+search。純粋関数
pub(crate) fn tab_query_value(path: &str, key: &str) -> Option<String> {
    let query = path.split('?').nth(1)?.split('#').next()?;
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if parts.next() == Some(key) {
            let raw = parts.next().unwrap_or("").replace('+', " ");
            return urlencoding::decode(&raw)
                .ok()
                .map(std::borrow::Cow::into_owned);
        }
    }
    None
}

/// 取引明細のタブ付きパス。ホームなどタブを指定して開くリンクはこれを使う
pub(crate) fn receipts_tab_href(tab: &str) -> String {
    format!("/receipts?tab={tab}")
}

/// 資産管理のタブ付きパス。ホームなどタブを指定して開くリンクはこれを使う
pub(crate) fn asset_balance_tab_href(kind: &str) -> String {
    format!("/assetbalance?tab={kind}")
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
        assert_eq!(
            detail_id("/assetbalance/holdings/abc-123#summary", HOLDINGS_PREFIX),
            Some("abc-123")
        );
        assert_eq!(
            detail_id("/assetbalance/funds/fund-1#details", FUND_HOLDINGS_PREFIX),
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

    #[test]
    fn tab_query_value_extracts_single_param() {
        assert_eq!(
            tab_query_value("/receipts?tab=stock", "tab").as_deref(),
            Some("stock")
        );
        assert_eq!(
            tab_query_value("/assetbalance?tab=fund", "tab").as_deref(),
            Some("fund")
        );
    }

    #[test]
    fn tab_query_value_returns_none_without_param() {
        assert_eq!(tab_query_value("/receipts", "tab"), None);
        assert_eq!(tab_query_value("", "tab"), None);
        assert_eq!(tab_query_value("/receipts?kind=stock", "tab"), None);
    }

    #[test]
    fn tab_query_value_reads_first_of_many_params() {
        assert_eq!(
            tab_query_value("/receipts?kind=stock&tab=fund&page=2", "tab").as_deref(),
            Some("fund")
        );
        assert_eq!(
            tab_query_value("/receipts?tab=stock&tab=fund", "tab").as_deref(),
            Some("stock")
        );
    }

    #[test]
    fn tab_query_value_decodes_value() {
        assert_eq!(
            tab_query_value("/receipts?tab=", "tab").as_deref(),
            Some("")
        );
        assert_eq!(
            tab_query_value("/receipts?tab=st%6Fck", "tab").as_deref(),
            Some("stock")
        );
    }

    #[test]
    fn tab_hrefs_carry_tab_param() {
        assert_eq!(receipts_tab_href("dividend"), "/receipts?tab=dividend");
        assert_eq!(asset_balance_tab_href("all"), "/assetbalance?tab=all");
    }
}
