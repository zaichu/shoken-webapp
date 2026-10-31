//! SQL クエリのバインド値。実行には `query_typed`/`execute_typed` を使い、各値の
//! Postgres 型を `declared_type()` で明示する（Hyperdrive では untyped prepare の
//! 分割往復が接続を壊すため）。native/wasm で同一の宣言を使う。

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
    Decimal(Decimal),
    Str(String),
    Bytes(Vec<u8>),
    Date(NaiveDate),
    Timestamp(DateTime<Utc>),
    Uuid(Uuid),
    /// variant から宣言型を決められない値（配列・newtype 等）を `ty` 付きで包む
    Custom {
        ty: Type,
        value: Box<dyn ToSql + Sync + Send>,
    },
}

impl Bind {
    /// `Option` をそのまま束縛する。`None` は untyped NULL（列型で推論される）
    pub fn opt<T: Into<Bind>>(value: Option<T>) -> Self {
        value.map(Into::into).unwrap_or(Bind::Null)
    }

    /// `ty` を明示して値を束縛する。`Vec<T>` の配列は専用 `From` impl 側で型を決める
    fn custom(ty: Type, value: impl ToSql + Sync + Send + 'static) -> Self {
        Bind::Custom {
            ty,
            value: Box::new(value),
        }
    }

    /// `query_typed`/`execute_typed` が Parse メッセージでサーバーへ宣言する型。
    /// `Null` は UNKNOWN（oid 0）を送り、従来どおり文脈からの推論に任せる
    pub(crate) fn declared_type(&self) -> Type {
        match self {
            Bind::Null => Type::UNKNOWN,
            Bind::Bool(_) => Type::BOOL,
            Bind::Int(_) => Type::INT8,
            Bind::Decimal(_) => Type::NUMERIC,
            Bind::Str(_) => Type::TEXT,
            Bind::Bytes(_) => Type::BYTEA,
            Bind::Date(_) => Type::DATE,
            Bind::Timestamp(_) => Type::TIMESTAMPTZ,
            Bind::Uuid(_) => Type::UUID,
            Bind::Custom { ty, .. } => ty.clone(),
        }
    }
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
            Bind::Int(v) => ToSql::to_sql(v, ty, out),
            Bind::Decimal(v) => ToSql::to_sql(&Numeric(*v), ty, out),
            Bind::Str(v) => ToSql::to_sql(v, ty, out),
            Bind::Bytes(v) => ToSql::to_sql(&v.as_slice(), ty, out),
            Bind::Date(v) => ToSql::to_sql(v, ty, out),
            Bind::Timestamp(v) => ToSql::to_sql(v, ty, out),
            Bind::Uuid(v) => ToSql::to_sql(v, ty, out),
            Bind::Custom { value, .. } => value.to_sql_checked(ty, out),
        }
    }

    fn accepts(ty: &Type) -> bool {
        // 実値は variant ごとに to_sql 内で検査するため、ここは受け付ける
        let _ = ty;
        true
    }

    to_sql_checked!();
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
from_int!(i32, i64);

impl From<bool> for Bind {
    fn from(v: bool) -> Self {
        Bind::Bool(v)
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

/// 配列は要素型に応じて宣言型が必要なため、使用する `Vec<T>` ごとに `From` を定義する。
/// 宣言型は各 INSERT の `unnest($n::<type>[])` キャストおよび比較対象の列型と一致させる。
impl From<Vec<shared::value::UserId>> for Bind {
    fn from(v: Vec<shared::value::UserId>) -> Self {
        Bind::custom(Type::UUID_ARRAY, v)
    }
}

impl From<Vec<NaiveDate>> for Bind {
    fn from(v: Vec<NaiveDate>) -> Self {
        Bind::custom(Type::DATE_ARRAY, v)
    }
}

impl From<Vec<String>> for Bind {
    fn from(v: Vec<String>) -> Self {
        Bind::custom(Type::TEXT_ARRAY, v)
    }
}

impl From<Vec<Option<String>>> for Bind {
    fn from(v: Vec<Option<String>>) -> Self {
        Bind::custom(Type::TEXT_ARRAY, v)
    }
}

impl From<Vec<shared::value::SecurityCode>> for Bind {
    fn from(v: Vec<shared::value::SecurityCode>) -> Self {
        Bind::custom(Type::VARCHAR_ARRAY, v)
    }
}

impl Bind {
    /// `Decimal` の配列バインド。`Decimal` は `ToSql` を持たない
    /// （backend は `Numeric` 経由で encode する）ため要素を `Numeric` に詰め替える
    pub fn decimal_vec(v: Vec<Decimal>) -> Self {
        Bind::custom(
            Type::NUMERIC_ARRAY,
            v.into_iter().map(Numeric).collect::<Vec<Numeric>>(),
        )
    }

    /// `Option<Decimal>` の配列バインド。値なし要素は NULL として挿入する
    pub fn optional_decimal_vec(v: Vec<Option<Decimal>>) -> Self {
        Bind::custom(
            Type::NUMERIC_ARRAY,
            v.into_iter()
                .map(|v| v.map(Numeric))
                .collect::<Vec<Option<Numeric>>>(),
        )
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

impl From<shared::dividend_per_share::DividendCacheStatus> for Bind {
    fn from(v: shared::dividend_per_share::DividendCacheStatus) -> Self {
        Bind::custom(Type::VARCHAR, v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::dividend_per_share::DividendCacheStatus;
    use shared::value::{SecurityCode, UserId};

    /// 宣言型の対応表。SQL 側の `unnest($n::<type>[])` キャストや列型と一致しないと
    /// encode/実行時に失敗するため、variant→Type の写像を固定する
    #[test]
    fn declared_type_maps_each_variant() {
        assert_eq!(Bind::Null.declared_type(), Type::UNKNOWN);
        assert_eq!(Bind::from(true).declared_type(), Type::BOOL);
        assert_eq!(Bind::from(1i64).declared_type(), Type::INT8);
        assert_eq!(Bind::from(Decimal::ONE).declared_type(), Type::NUMERIC);
        assert_eq!(Bind::from("x").declared_type(), Type::TEXT);
        assert_eq!(Bind::Bytes(vec![1]).declared_type(), Type::BYTEA);
        assert_eq!(
            Bind::from(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()).declared_type(),
            Type::DATE
        );
        assert_eq!(Bind::from(Utc::now()).declared_type(), Type::TIMESTAMPTZ);
        assert_eq!(Bind::from(Uuid::nil()).declared_type(), Type::UUID);
    }

    /// 配列・newtype の宣言型は unnest のキャスト先や列型に合わせる。
    /// `Option<T>` / `None` は UNKNOWN で推論に任せる
    #[test]
    fn declared_type_maps_custom_and_arrays() {
        let user_ids: Vec<UserId> = vec![UserId::from(Uuid::nil())];
        assert_eq!(Bind::from(user_ids).declared_type(), Type::UUID_ARRAY);
        assert_eq!(
            Bind::from(vec![NaiveDate::MAX]).declared_type(),
            Type::DATE_ARRAY
        );
        assert_eq!(
            Bind::from(vec!["a".to_string()]).declared_type(),
            Type::TEXT_ARRAY
        );
        assert_eq!(
            Bind::from(vec![Some("a".to_string()), None]).declared_type(),
            Type::TEXT_ARRAY
        );
        assert_eq!(
            Bind::from(vec![SecurityCode::from_raw("1234".into())]).declared_type(),
            Type::VARCHAR_ARRAY
        );
        assert_eq!(
            Bind::decimal_vec(vec![Decimal::ONE]).declared_type(),
            Type::NUMERIC_ARRAY
        );
        assert_eq!(
            Bind::from(DividendCacheStatus::Ok).declared_type(),
            Type::VARCHAR
        );
        assert_eq!(Bind::opt::<&str>(None).declared_type(), Type::UNKNOWN);
    }
}
