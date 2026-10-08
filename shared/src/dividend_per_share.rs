//! `/api/v1/dividend-per-share-estimates` のバッチ DTO とキャッシュ状態。
//! backend・frontend 両方が同じ wire 型を使うため shared に置く。

#[cfg(feature = "typed")]
use crate::domain::{DateTime, Utc};
use crate::value::SecurityCode;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 配当キャッシュの状態。DB は VARCHAR(20)、wire は小文字文字列のまま
///
/// | status  | 意味                     |
/// |---------|--------------------------|
/// | ok      | 有配当データあり         |
/// | zero    | 配当なし                 |
/// | error   | 取得失敗                 |
/// | pending | 未取得/更新待ち          |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum DividendCacheStatus {
    Ok,
    Zero,
    Error,
    Pending,
}

impl DividendCacheStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Zero => "zero",
            Self::Error => "error",
            Self::Pending => "pending",
        }
    }
}

impl fmt::Display for DividendCacheStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// DB 上は VARCHAR(20) の小文字文字列。透過 newtype では表せないため手実装
#[cfg(feature = "postgres")]
mod pg_impls {
    use super::DividendCacheStatus;
    use postgres_types::{FromSql, IsNull, ToSql, Type, to_sql_checked};
    use std::error::Error;

    impl ToSql for DividendCacheStatus {
        fn to_sql(
            &self,
            ty: &Type,
            out: &mut bytes::BytesMut,
        ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
            ToSql::to_sql(&self.as_str(), ty, out)
        }

        fn accepts(ty: &Type) -> bool {
            <&str as ToSql>::accepts(ty)
        }

        to_sql_checked!();
    }

    impl<'a> FromSql<'a> for DividendCacheStatus {
        fn from_sql(
            ty: &Type,
            raw: &'a [u8],
        ) -> Result<DividendCacheStatus, Box<dyn Error + Sync + Send>> {
            match <&str as FromSql>::from_sql(ty, raw)? {
                "ok" => Ok(DividendCacheStatus::Ok),
                "zero" => Ok(DividendCacheStatus::Zero),
                "error" => Ok(DividendCacheStatus::Error),
                "pending" => Ok(DividendCacheStatus::Pending),
                other => Err(format!("不明な dividend cache status: {other}").into()),
            }
        }

        fn accepts(ty: &Type) -> bool {
            <&str as FromSql>::accepts(ty)
        }
    }
}

/// バッチリクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct DividendPerShareBatchRequest {
    #[cfg_attr(feature = "utoipa", schema(value_type = Vec<String>))]
    pub security_codes: Vec<SecurityCode>,
}

/// 要素の形式は `SecurityCode` の serde(try_from) が JSON 入力時に検証済みのため、
/// ここでは件数(1〜100)のみ検証する
#[cfg(feature = "validate")]
impl validator::Validate for DividendPerShareBatchRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        use std::borrow::Cow;
        use validator::{ValidateLength, ValidationError, ValidationErrors};

        let mut errors = ValidationErrors::new();
        if !self
            .security_codes
            .validate_length(Some(1), Some(100), None)
        {
            let mut error = ValidationError::new("length");
            error.add_param(Cow::from("min"), &1u64);
            error.add_param(Cow::from("max"), &100u64);
            errors.add("security_codes", error);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// レスポンス内の1銘柄アイテム
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct DividendPerShareItem {
    #[cfg_attr(feature = "utoipa", schema(value_type = String))]
    #[serde(deserialize_with = "crate::value::unchecked::security_code")]
    pub security_code: SecurityCode,
    pub dividend_per_share: Option<f64>,
    /// ok / zero / pending / error
    pub status: DividendCacheStatus,
    #[cfg(feature = "typed")]
    pub fetched_at: Option<DateTime<Utc>>,
    #[cfg(not(feature = "typed"))]
    pub fetched_at: Option<String>,
    pub is_stale: bool,
}

/// バッチレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct DividendPerShareBatchResponse {
    pub items: Vec<DividendPerShareItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_json(codes: &[&str]) -> String {
        let codes: Vec<String> = codes.iter().map(|c| format!("\"{c}\"")).collect();
        format!("{{\"security_codes\":[{}]}}", codes.join(","))
    }

    #[test]
    fn security_code_roundtrips_as_plain_string() {
        let json = request_json(&["7203", "AAPL", "7203.T"]);
        let request: DividendPerShareBatchRequest =
            serde_json::from_str(&json).expect("valid request");
        assert_eq!(request.security_codes.len(), 3);
        assert_eq!(request.security_codes[0].as_str(), "7203");
        assert_eq!(serde_json::to_string(&request).unwrap(), json);
    }

    #[test]
    fn invalid_element_fails_deserialization() {
        for bad in ["7203;DROP", "", "12345678901", "トヨタ", "7203 "] {
            let json = request_json(&["7203", bad]);
            assert!(
                serde_json::from_str::<DividendPerShareBatchRequest>(&json).is_err(),
                "{bad:?} は拒否される"
            );
        }
    }

    #[test]
    fn status_serializes_lowercase() {
        for (status, expected) in [
            (DividendCacheStatus::Ok, "ok"),
            (DividendCacheStatus::Zero, "zero"),
            (DividendCacheStatus::Error, "error"),
            (DividendCacheStatus::Pending, "pending"),
        ] {
            assert_eq!(
                serde_json::to_string(&status).unwrap(),
                format!("\"{expected}\"")
            );
            assert_eq!(
                serde_json::from_str::<DividendCacheStatus>(&format!("\"{expected}\"")).unwrap(),
                status
            );
            assert_eq!(status.as_str(), expected);
        }
    }

    #[cfg(feature = "validate")]
    mod validate_feature {
        use super::*;
        use validator::Validate;

        fn request(count: usize) -> DividendPerShareBatchRequest {
            DividendPerShareBatchRequest {
                security_codes: vec!["1234".parse().unwrap(); count],
            }
        }

        #[test]
        fn batch_size_between_1_and_100() {
            assert!(request(1).validate().is_ok());
            assert!(request(100).validate().is_ok());
            assert!(request(0).validate().is_err());
            assert!(request(101).validate().is_err());
        }
    }
}
