//! `sqlx::QueryBuilder<Postgres>` 相当の SQL 組み立て器。
//! `push` は SQL 断片をそのまま追記し、`push_bind` は `$N` プレースホルダを
//! 追記して値をパラメータとして保持する。

use super::{Bind, FromRow, Query, QueryAs, QueryScalar};
use postgres_types::FromSql;
use std::fmt::Display;
use std::fmt::Write;

pub struct QueryBuilder {
    sql: String,
    params: Vec<Bind>,
}

impl QueryBuilder {
    pub fn new(init: impl Display) -> Self {
        Self {
            sql: init.to_string(),
            params: Vec::new(),
        }
    }

    pub fn push(&mut self, sql: impl Display) -> &mut Self {
        write!(self.sql, "{sql}").expect("error formatting `sql`");
        self
    }

    /// `$N` を追記して値をパラメータとして登録する
    pub fn push_bind(&mut self, value: impl Into<Bind>) -> &mut Self {
        self.params.push(value.into());
        write!(self.sql, "${}", self.params.len()).expect("error formatting placeholder");
        self
    }

    /// 生成した SQL 断片をそのまま返す（テスト・logging 用）
    pub fn sql(&self) -> &str {
        &self.sql
    }

    /// SQL とパラメータを取り出して `Query` に変換する。ビルド後は再利用可能
    pub fn build(&mut self) -> Query {
        Query {
            sql: std::mem::take(&mut self.sql),
            params: std::mem::take(&mut self.params),
        }
    }

    pub fn build_query_as<T: FromRow>(&mut self) -> QueryAs<T> {
        QueryAs::new(self.build())
    }

    pub fn build_query_scalar<T: for<'a> FromSql<'a>>(&mut self) -> QueryScalar<T> {
        QueryScalar::new(self.build())
    }
}
