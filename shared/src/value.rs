//! 意味のある値を包む newtype。
//! serde の wire 形と DB 表現はいずれも内側の型のまま（serde transparent / sqlx transparent）。
//! 不正な値は `TryFrom` / `serde(try_from)`（JSON 入力）または CSV 変換時の `try_from` で拒否する。
//! sqlx のデコード（DB → モデル）は検証しない。既存行に対する後方互換のため透過的に包むだけ。

use crate::domain::Uuid;
use crate::tax::SPECIFIC_ACCOUNT_KEYWORD;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// 銘柄コードの検証失敗（1〜10文字の ASCII 英数字またはドット以外）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidSecurityCode;

impl fmt::Display for InvalidSecurityCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("1〜10文字の半角英数字またはドットで指定してください")
    }
}

impl std::error::Error for InvalidSecurityCode {}

/// 銘柄コード（ASCII 英数字とドット、1〜10文字）
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct SecurityCode(String);

impl SecurityCode {
    pub const MAX_LEN: usize = 10;

    pub fn is_valid(value: &str) -> bool {
        !value.is_empty()
            && value.len() <= Self::MAX_LEN
            && value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'.')
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SecurityCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for SecurityCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<SecurityCode> for String {
    fn from(code: SecurityCode) -> Self {
        code.0
    }
}

impl TryFrom<String> for SecurityCode {
    type Error = InvalidSecurityCode;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::is_valid(&value)
            .then_some(Self(value))
            .ok_or(InvalidSecurityCode)
    }
}

impl TryFrom<&str> for SecurityCode {
    type Error = InvalidSecurityCode;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl FromStr for SecurityCode {
    type Err = InvalidSecurityCode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_string())
    }
}

/// レスポンスの JSON を読むときは、DB からのデコードと同じく検証せずに包む。
/// 旧仕様で保存された値(例: `7203-1`)が1件あっても一覧全体を読めなくしないため
pub mod unchecked {
    use super::{Account, SecurityCode};
    use serde::{Deserialize, Deserializer};

    pub fn security_code<'de, D: Deserializer<'de>>(d: D) -> Result<SecurityCode, D::Error> {
        String::deserialize(d).map(SecurityCode)
    }

    pub fn account<'de, D: Deserializer<'de>>(d: D) -> Result<Account, D::Error> {
        String::deserialize(d).map(Account)
    }
}

/// 口座名の検証失敗（1〜100文字以外）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidAccount;

impl fmt::Display for InvalidAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("1〜100文字で指定してください")
    }
}

impl std::error::Error for InvalidAccount {}

/// 口座名（1〜100文字）。特定/NISA/一般 等の区分は文字列のまま保持し、
/// 課税口座かどうかの判定だけを `is_specific` に集約する
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct Account(String);

impl Account {
    pub const MAX_LEN: usize = 100;

    pub fn is_valid(value: &str) -> bool {
        let len = value.chars().count();
        (1..=Self::MAX_LEN).contains(&len)
    }

    /// 特定口座（源泉徴収ありの課税対象口座）か
    pub fn is_specific(&self) -> bool {
        self.0.contains(SPECIFIC_ACCOUNT_KEYWORD)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Account {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<Account> for String {
    fn from(account: Account) -> Self {
        account.0
    }
}

impl TryFrom<String> for Account {
    type Error = InvalidAccount;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::is_valid(&value)
            .then_some(Self(value))
            .ok_or(InvalidAccount)
    }
}

impl TryFrom<&str> for Account {
    type Error = InvalidAccount;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl FromStr for Account {
    type Err = InvalidAccount;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_string())
    }
}

/// DB 行の主キー。wire 上は UUID 文字列のまま
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct RecordId(Uuid);

impl RecordId {
    #[cfg(feature = "typed")]
    pub fn get(self) -> Uuid {
        self.0
    }

    #[cfg(not(feature = "typed"))]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[cfg(not(feature = "typed"))]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Uuid> for RecordId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl fmt::Display for RecordId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// ユーザー ID。wire には出ないため typed 時のみ存在する
#[cfg(feature = "typed")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct UserId(Uuid);

#[cfg(feature = "typed")]
impl UserId {
    pub fn get(self) -> Uuid {
        self.0
    }
}

#[cfg(feature = "typed")]
impl From<Uuid> for UserId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

#[cfg(feature = "typed")]
impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn security_code_accepts_ascii_alnum_and_dot() {
        for valid in ["7203", "AAPL", "7203.T", "BRK.B", "1234567890", "a"] {
            assert!(SecurityCode::is_valid(valid), "{valid}");
            let code = SecurityCode::try_from(valid).unwrap();
            assert_eq!(code.as_str(), valid);
            assert_eq!(
                serde_json::to_string(&code).unwrap(),
                format!("\"{valid}\"")
            );
        }
    }

    #[test]
    fn security_code_rejects_invalid() {
        for invalid in [
            "",
            "12345678901",
            "トヨタ",
            "7203 ",
            "7203;DROP",
            "7203-1",
            "720３",
        ] {
            assert!(!SecurityCode::is_valid(invalid), "{invalid}");
            assert!(SecurityCode::try_from(invalid).is_err(), "{invalid}");
            assert!(invalid.parse::<SecurityCode>().is_err(), "{invalid}");
            assert!(
                serde_json::from_str::<SecurityCode>(&format!("\"{invalid}\"")).is_err(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn account_accepts_1_to_100_chars() {
        assert!(Account::try_from("特定").is_ok());
        assert!(Account::try_from("NISA").is_ok());
        assert!(Account::try_from("あ".repeat(100)).is_ok());
    }

    #[test]
    fn account_rejects_empty_and_over_100() {
        assert!(Account::try_from("").is_err());
        assert!(Account::try_from("あ".repeat(101)).is_err());
        assert!(serde_json::from_str::<Account>("\"\"").is_err());
        assert_eq!(
            Account::try_from("").unwrap_err().to_string(),
            "1〜100文字で指定してください"
        );
    }

    #[test]
    fn record_id_is_transparent_on_wire() {
        let raw = "\"550e8400-e29b-41d4-a716-446655440000\"";
        let id: RecordId = serde_json::from_str(raw).unwrap();
        assert_eq!(serde_json::to_string(&id).unwrap(), raw);
    }
}
