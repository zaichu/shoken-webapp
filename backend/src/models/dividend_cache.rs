use crate::models::common::validate_length_field;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::{borrow::Cow, collections::BTreeMap};
use utoipa::ToSchema;
use validator::{Validate, ValidationError, ValidationErrors, ValidationErrorsKind};

/// 配当キャッシュレコード
///
/// status × stale_at 整合ルール（is_stale / 再取得対象の判定基準）
/// | status  | stale_at    | is_stale | 再取得? | 理由                         |
/// |---------|-------------|----------|---------|------------------------------|
/// | pending | NULL        | false    | No      | 取得中のため再取得しない     |
/// | ok      | future      | false    | No      | 有効データ                   |
/// | ok      | NULL / past | true     | Yes     | stale（再取得待ち）          |
/// | zero    | future      | false    | No      | 配当なし（有効）             |
/// | zero    | NULL / past | true     | Yes     | stale（再取得待ち）          |
/// | error   | NULL        | true     | Yes     | 即再取得対象                 |
/// | error   | future      | false    | No      | 429 cooldown 中は再取得しない |
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DividendCache {
    pub security_code: String,
    pub dividend_per_share: Option<f64>,
    /// ok: 有配当、zero: ゼロ配当、error: 取得失敗、pending: 未取得/更新待ち
    pub status: String,
    pub fetched_at: Option<DateTime<Utc>>,
    /// TTL期限。この時刻を過ぎると再取得対象（NULL かつ pending 以外 = 即再取得対象）
    pub stale_at: Option<DateTime<Utc>>,
    pub provider: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// バッチリクエスト
#[derive(Debug, Deserialize, ToSchema)]
pub struct DividendPerShareBatchRequest {
    pub security_codes: Vec<String>,
}

impl Validate for DividendPerShareBatchRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        validate_length_field(
            &mut errors,
            "security_codes",
            &self.security_codes,
            Some(1),
            Some(100),
        );
        // 件数自体が不正（空・上限超過）の場合は要素検証を省略する。
        // `security_codes` キーには既に長さエラーが入っており、List 形式で上書きすると
        // 件数エラーが失われるため（`BulkCreateAssetBalanceRequest::validate` と同慣例）。
        if errors.is_empty() {
            let element_errors = self
                .security_codes
                .iter()
                .enumerate()
                .filter_map(|(index, code)| {
                    let mut element = ValidationErrors::new();
                    validate_length_field(&mut element, "security_code", code, Some(1), Some(10));
                    if !code.chars().all(|c| c.is_ascii_alphanumeric() || c == '.') {
                        element.add(
                            "security_code",
                            ValidationError::new("invalid_security_code"),
                        );
                    }
                    if element.is_empty() {
                        None
                    } else {
                        Some((index, Box::new(element)))
                    }
                })
                .collect::<BTreeMap<_, _>>();
            if !element_errors.is_empty() {
                errors.errors_mut().insert(
                    Cow::Borrowed("security_codes"),
                    ValidationErrorsKind::List(element_errors),
                );
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// レスポンス内の1銘柄アイテム
#[derive(Debug, Serialize, ToSchema)]
pub struct DividendPerShareItem {
    pub security_code: String,
    pub dividend_per_share: Option<f64>,
    /// ok / zero / pending / error
    pub status: String,
    pub fetched_at: Option<DateTime<Utc>>,
    pub is_stale: bool,
}

/// バッチレスポンス
#[derive(Debug, Serialize, ToSchema)]
pub struct DividendPerShareBatchResponse {
    pub items: Vec<DividendPerShareItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dividend_per_share_batch_request_validation() {
        // 1件は OK（min = 1）
        assert!(DividendPerShareBatchRequest {
            security_codes: vec!["1234".to_string()],
        }
        .validate()
        .is_ok());

        // 100件は OK（max = 100）
        assert!(DividendPerShareBatchRequest {
            security_codes: vec!["1234".to_string(); 100],
        }
        .validate()
        .is_ok());

        // 0件は NG（min = 1）
        assert!(DividendPerShareBatchRequest {
            security_codes: vec![],
        }
        .validate()
        .is_err());

        // 101件は NG（max = 100）
        assert!(DividendPerShareBatchRequest {
            security_codes: vec!["1234".to_string(); 101],
        }
        .validate()
        .is_err());
    }

    #[test]
    fn test_batch_request_accepts_valid_codes() {
        // 国内株4桁・英字混じり・ドット付きはいずれも OK
        assert!(DividendPerShareBatchRequest {
            security_codes: vec!["7203".to_string(), "AAPL".to_string(), "7203.T".to_string(),],
        }
        .validate()
        .is_ok());
    }

    #[test]
    fn test_batch_request_rejects_invalid_element_with_index() {
        // 2 番目の要素（インデックス 1）に記号を含む
        let request = DividendPerShareBatchRequest {
            security_codes: vec!["7203".to_string(), "7203;DROP".to_string()],
        };
        let errors = request.validate().expect_err("不正な要素は拒否される");
        assert!(
            format!("{errors}").contains("security_codes[1]"),
            "どの要素が不正か分かること: {errors}"
        );

        // 空文字要素（インデックス 0）は長さ min=1 違反で拒否される
        let request = DividendPerShareBatchRequest {
            security_codes: vec![String::new(), "7203".to_string()],
        };
        let errors = request.validate().expect_err("空文字要素は拒否される");
        assert!(format!("{errors}").contains("security_codes[0]"));

        // 11 文字要素は VARCHAR(10) のため拒否される
        let request = DividendPerShareBatchRequest {
            security_codes: vec!["12345678901".to_string()],
        };
        assert!(request.validate().is_err());

        // 10 文字は OK（上限境界）
        assert!(DividendPerShareBatchRequest {
            security_codes: vec!["1234567890".to_string()],
        }
        .validate()
        .is_ok());

        // 日本語・空白を含む要素は拒否される
        assert!(DividendPerShareBatchRequest {
            security_codes: vec!["トヨタ".to_string()],
        }
        .validate()
        .is_err());
        assert!(DividendPerShareBatchRequest {
            security_codes: vec!["7203 ".to_string()],
        }
        .validate()
        .is_err());
    }
}
