/// 保存済み(DB 行)の一覧に載せるための区別。
/// 以前は CSV プレビュー(未保存)も同じ一覧に載せていたが、直接保存に変えたため保存行だけ残す
#[derive(Clone, Debug, PartialEq)]
pub enum Row<S> {
    Saved(S),
}

impl<S> Row<S> {
    pub fn saved(&self) -> Option<&S> {
        match self {
            Self::Saved(row) => Some(row),
        }
    }
}
