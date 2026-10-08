//! SQL クエリのバインド値。パラメータ型はサーバーがクエリ文脈から推論する
//! （`client.query` は untyped prepare）ため、`Int` は推論された型に合わせて encode する。

use crate::db::numeric::Numeric;
use bytes::BytesMut;
use chrono::{DateTime, NaiveDate, Utc};
use postgres_types::{IsNull, ToSql, Type, to_sql_checked};
use rust_decimal::Decimal;
use std::error::Error;
use uuid::Uuid;

/// `$n` プレースホルダに渡す値。配列やドメイン newtype は `Custom` に包む
#[derive(Debug)]
pub enum Bind {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Decimal(Decimal),
    Str(String),
    Bytes(Vec<u8>),
    Date(NaiveDate),
    Timestamp(DateTime<Utc>),
    Uuid(Uuid),
    Json(serde_json::Value),
    Custom(Box<dyn ToSql + Sync + Send>),
}

impl Bind {
    /// `Option` をそのまま束縛する。`None` は untyped NULL（列型で推論される）
    pub fn opt<T: Into<Bind>>(value: Option<T>) -> Self {
        value.map(Into::into).unwrap_or(Bind::Null)
    }
}

fn err(message: impl Into<String>) -> Box<dyn Error + Sync + Send> {
    message.into().into()
}

impl ToSql for Bind {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        match self {
            Bind::Null => Ok(IsNull::Yes),
            Bind::Bool(v) => ToSql::to_sql(v, ty, out),
            Bind::Int(v) => {
                if *ty == Type::INT2 {
                    ToSql::to_sql(
                        &i16::try_from(*v).map_err(|_| err("Int が i16 範囲外"))?,
                        ty,
                        out,
                    )
                } else if *ty == Type::INT4 {
                    ToSql::to_sql(
                        &i32::try_from(*v).map_err(|_| err("Int が i32 範囲外"))?,
                        ty,
                        out,
                    )
                } else if *ty == Type::INT8 {
                    ToSql::to_sql(v, ty, out)
                } else if *ty == Type::OID {
                    ToSql::to_sql(
                        &u32::try_from(*v).map_err(|_| err("Int が u32 範囲外"))?,
                        ty,
                        out,
                    )
                } else if *ty == Type::FLOAT4 {
                    ToSql::to_sql(&(*v as f32), ty, out)
                } else if *ty == Type::FLOAT8 {
                    ToSql::to_sql(&(*v as f64), ty, out)
                } else if *ty == Type::NUMERIC {
                    ToSql::to_sql(&Numeric(Decimal::from(*v)), ty, out)
                } else {
                    Err(err(format!("Int を {ty} として encode できない")))
                }
            }
            Bind::Float(v) => {
                if *ty == Type::FLOAT4 {
                    ToSql::to_sql(&(*v as f32), ty, out)
                } else if *ty == Type::NUMERIC {
                    let d = Decimal::try_from(*v)
                        .map_err(|_| err("Float を NUMERIC に変換できない"))?;
                    ToSql::to_sql(&Numeric(d), ty, out)
                } else {
                    ToSql::to_sql(v, ty, out)
                }
            }
            Bind::Decimal(v) => ToSql::to_sql(&Numeric(*v), ty, out),
            Bind::Str(v) => ToSql::to_sql(v, ty, out),
            Bind::Bytes(v) => ToSql::to_sql(&v.as_slice(), ty, out),
            Bind::Date(v) => ToSql::to_sql(v, ty, out),
            Bind::Timestamp(v) => ToSql::to_sql(v, ty, out),
            Bind::Uuid(v) => ToSql::to_sql(v, ty, out),
            Bind::Json(v) => ToSql::to_sql(v, ty, out),
            Bind::Custom(v) => v.to_sql_checked(ty, out),
        }
    }

    fn accepts(ty: &Type) -> bool {
        // 実値は variant ごとに to_sql 内で検査するため、ここは受け付ける
        let _ = ty;
        true
    }

    to_sql_checked!();
}

impl From<bool> for Bind {
    fn from(v: bool) -> Self {
        Bind::Bool(v)
    }
}

macro_rules! from_int {
    ($($t:ty),*) => {$(
        impl From<$t> for Bind {
            fn from(v: $t) -> Self {
                Bind::Int(i64::from(v))
            }
        }
    )*};
}
from_int!(i8, i16, i32, i64, u8, u16, u32);

impl From<usize> for Bind {
    fn from(v: usize) -> Self {
        Bind::Int(i64::try_from(v).unwrap_or(i64::MAX))
    }
}

impl From<f32> for Bind {
    fn from(v: f32) -> Self {
        Bind::Float(f64::from(v))
    }
}

impl From<f64> for Bind {
    fn from(v: f64) -> Self {
        Bind::Float(v)
    }
}

impl From<Decimal> for Bind {
    fn from(v: Decimal) -> Self {
        Bind::Decimal(v)
    }
}

impl From<String> for Bind {
    fn from(v: String) -> Self {
        Bind::Str(v)
    }
}

impl From<&str> for Bind {
    fn from(v: &str) -> Self {
        Bind::Str(v.to_owned())
    }
}

impl From<&String> for Bind {
    fn from(v: &String) -> Self {
        Bind::Str(v.clone())
    }
}

impl From<&[u8]> for Bind {
    fn from(v: &[u8]) -> Self {
        Bind::Bytes(v.to_vec())
    }
}

impl From<NaiveDate> for Bind {
    fn from(v: NaiveDate) -> Self {
        Bind::Date(v)
    }
}

impl From<DateTime<Utc>> for Bind {
    fn from(v: DateTime<Utc>) -> Self {
        Bind::Timestamp(v)
    }
}

impl From<Uuid> for Bind {
    fn from(v: Uuid) -> Self {
        Bind::Uuid(v)
    }
}

impl From<serde_json::Value> for Bind {
    fn from(v: serde_json::Value) -> Self {
        Bind::Json(v)
    }
}

/// Vec<T>（配列）・newtype・enum など、`ToSql` を実装済みの型はそのまま Custom に包む。
/// `Vec<u8>` は `u8` が ToSql を持たないため対象外で、`Bind::Bytes` / `From<&[u8]>` を使う。
impl<T> From<Vec<T>> for Bind
where
    T: ToSql + Sync + Send + 'static,
{
    fn from(v: Vec<T>) -> Self {
        Bind::Custom(Box::new(v))
    }
}

impl Bind {
    /// `Decimal` の配列バインド。`Decimal` は `ToSql` を持たない
    /// （backend は `Numeric` 経由で encode する）ため汎用 `From<Vec<T>>` では
    /// 扱えず、要素を `Numeric` に詰め替える専用コンストラクタを用意する
    pub fn decimal_vec(v: Vec<Decimal>) -> Self {
        Bind::Custom(Box::new(
            v.into_iter().map(Numeric).collect::<Vec<Numeric>>(),
        ))
    }
}

impl<T: Into<Bind>> From<Option<T>> for Bind {
    fn from(v: Option<T>) -> Self {
        Bind::opt(v)
    }
}

// ---- shared の newtype/enum ----

impl From<shared::value::UserId> for Bind {
    fn from(v: shared::value::UserId) -> Self {
        Bind::Uuid(v.get())
    }
}

impl From<shared::value::RecordId> for Bind {
    fn from(v: shared::value::RecordId) -> Self {
        Bind::Uuid(v.get())
    }
}

impl From<shared::value::SecurityCode> for Bind {
    fn from(v: shared::value::SecurityCode) -> Self {
        Bind::Str(v.as_str().to_owned())
    }
}

impl From<&shared::value::SecurityCode> for Bind {
    fn from(v: &shared::value::SecurityCode) -> Self {
        Bind::Str(v.as_str().to_owned())
    }
}

impl From<shared::value::Account> for Bind {
    fn from(v: shared::value::Account) -> Self {
        Bind::Str(v.as_str().to_owned())
    }
}

impl From<&shared::value::Account> for Bind {
    fn from(v: &shared::value::Account) -> Self {
        Bind::Str(v.as_str().to_owned())
    }
}

impl From<shared::dividend_per_share::DividendCacheStatus> for Bind {
    fn from(v: shared::dividend_per_share::DividendCacheStatus) -> Self {
        Bind::Custom(Box::new(v))
    }
}
