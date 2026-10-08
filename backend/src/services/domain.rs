pub mod bulk;
pub mod facets;
pub mod search;
pub mod search_filters;

use crate::db::{Db, DbError};
use shared::value::UserId;
use std::future::Future;

/// 書き込みで既存行に追記するか、利用者の行を全件置き換えるか
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteMode {
    Append,
    Replace,
}

/// 利用者ごとのデータを持つドメイン(配当金・国内株式・投資信託・保有銘柄)
pub trait Domain: Send + Sync + 'static {
    /// ログに出す名前
    const NAME: &'static str;
    const TABLE: &'static str;
    const WRITE_MODE: WriteMode;

    fn delete_rows(pool: &Db, user_id: UserId)
    -> impl Future<Output = Result<u64, DbError>> + Send;
}
