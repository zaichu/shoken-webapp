/// 明細一覧 API のページ結合状態。
/// API は 1 ページあたり最大 `per_page` 件しか返さないため、
/// 返却件数が `per_page` 未満のページ(=末尾)まで逐次取得して 1 つの Vec に結合する。
pub struct PageCollector<T> {
    per_page: usize,
    max_pages: usize,
    rows: Vec<T>,
    fetched: usize,
    last_len: usize,
    total: Option<usize>,
}

impl<T> PageCollector<T> {
    pub fn new(per_page: usize, max_pages: usize) -> Self {
        Self {
            per_page,
            max_pages,
            rows: Vec::new(),
            fetched: 0,
            last_len: 0,
            total: None,
        }
    }

    pub fn next_page(&self) -> usize {
        self.fetched + 1
    }

    /// 1 ページ分を結合し、次ページを取得すべきなら true を返す。
    /// `total` が取得済み件数以下なら、ちょうど `per_page` 件で終わる場合の
    /// 空振りリクエストを省いて打ち切る。`total` が実データより大きい等の
    /// API 不整合で終端に達しない場合に備え、`max_pages` を上限とする。
    pub fn push(&mut self, page: Vec<T>, total: i64) -> bool {
        self.fetched += 1;
        self.last_len = page.len();
        self.rows.extend(page);
        if total >= 0 {
            self.total = Some(total as usize);
        }
        self.last_len >= self.per_page
            && self.total.is_none_or(|total| self.rows.len() < total)
            && self.fetched < self.max_pages
    }

    pub fn into_rows(self) -> Vec<T> {
        self.rows
    }
}

#[cfg(test)]
pub(crate) mod tests;
