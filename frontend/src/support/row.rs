/// 保存済み(DB 行)と CSV プレビュー(未保存)を同じ一覧に載せるための区別。
/// プレビュー行に永続化済みの偽 id を持たせないため、保存行とは別バリアントに分ける。
#[derive(Clone, Debug, PartialEq)]
pub enum Row<S, P> {
    Saved(S),
    Preview(P),
}

impl<S, P> Row<S, P> {
    pub fn saved(&self) -> Option<&S> {
        match self {
            Self::Saved(row) => Some(row),
            Self::Preview(_) => None,
        }
    }

    pub fn is_preview(&self) -> bool {
        matches!(self, Self::Preview(_))
    }
}
