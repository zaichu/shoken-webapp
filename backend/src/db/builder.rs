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

    /// 区切り文字を挟んで要素を追記する。sqlx と同様、
    /// `push`/`push_bind` の呼び出し時に区切りを挟む方式
    pub fn separated<Sep: Display>(&mut self, separator: Sep) -> Separated<'_, Sep> {
        Separated {
            builder: self,
            separator,
            push_separator: false,
        }
    }

    /// `VALUES (...), (...), ...` を組み立てる。sqlx と同様、
    /// 各タプルは `Separated` をクロージャに渡して構築する
    pub fn push_values<I, F>(&mut self, tuples: I, mut push_tuple: F) -> &mut Self
    where
        I: IntoIterator,
        F: FnMut(Separated<'_, &'static str>, I::Item),
    {
        self.push("VALUES ");
        let mut separated = self.separated(", ");
        for tuple in tuples {
            separated.push("(");
            push_tuple(separated.builder.separated(", "), tuple);
            separated.push_unseparated(")");
        }
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

/// `QueryBuilder::separated` で返る区切り管理ビルダ
pub struct Separated<'a, Sep> {
    builder: &'a mut QueryBuilder,
    separator: Sep,
    push_separator: bool,
}

impl<Sep: Display> Separated<'_, Sep> {
    /// 先頭要素以外で区切りを挟んで SQL 断片を追記する
    pub fn push(&mut self, sql: impl Display) -> &mut Self {
        if self.push_separator {
            self.builder.push(&self.separator);
        }
        self.builder.push(sql);
        self.push_separator = true;
        self
    }

    /// 区切りを挟まず SQL 断片を追記する（`push_separator` は変えない）
    pub fn push_unseparated(&mut self, sql: impl Display) -> &mut Self {
        self.builder.push(sql);
        self
    }

    /// 先頭要素以外で区切りを挟んで `$N` プレースホルダを追記する
    pub fn push_bind(&mut self, value: impl Into<Bind>) -> &mut Self {
        if self.push_separator {
            self.builder.push(&self.separator);
        }
        self.builder.push_bind(value);
        self.push_separator = true;
        self
    }

    /// 区切りを挟まず `$N` プレースホルダを追記する（`push_separator` は変えない）
    pub fn push_bind_unseparated(&mut self, value: impl Into<Bind>) -> &mut Self {
        self.builder.push_bind(value);
        self
    }
}
