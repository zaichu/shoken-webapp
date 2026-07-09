use crate::models::common::{validate_length_field, SearchQueryParams};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

/// 投資信託モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Mutualfund {
    pub id: Uuid,
    #[serde(skip_serializing)]
    #[allow(dead_code)] // SELECT * で取得されるがRust側では参照しない行所有者ID
    pub user_id: Uuid,
    pub trade_date: NaiveDate,
    pub settlement_date: NaiveDate,
    pub fund_name: String,
    pub dividends: Option<String>,
    pub account: String,
    #[schema(value_type = f64)]
    pub shares: Decimal,
    #[schema(value_type = f64)]
    pub exchange_rate: Decimal,
    #[schema(value_type = f64)]
    pub cancellation_unit_price_yen: Decimal,
    #[schema(value_type = f64)]
    pub cancellation_amount_yen: Decimal,
    #[schema(value_type = f64)]
    pub average_acquisition_price_yen: Decimal,
    #[schema(value_type = f64)]
    pub realized_profit_and_loss: Decimal,
    #[schema(value_type = f64)]
    pub taxes: Decimal,
    #[schema(value_type = f64)]
    pub realized_profit_and_loss_after_tax: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 投資信託作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMutualfundRequest {
    pub trade_date: NaiveDate,
    pub settlement_date: NaiveDate,
    pub fund_name: String,
    pub dividends: Option<String>,
    pub account: String,
    #[schema(value_type = f64)]
    pub shares: Decimal,
    #[schema(value_type = f64)]
    pub exchange_rate: Decimal,
    #[schema(value_type = f64)]
    pub cancellation_unit_price_yen: Decimal,
    #[schema(value_type = f64)]
    pub cancellation_amount_yen: Decimal,
    #[schema(value_type = f64)]
    pub average_acquisition_price_yen: Decimal,
    #[schema(value_type = f64)]
    pub realized_profit_and_loss: Decimal,
    #[schema(value_type = f64)]
    pub taxes: Decimal,
    #[schema(value_type = f64)]
    pub realized_profit_and_loss_after_tax: Decimal,
}

/// 投資信託一覧の検索クエリパラメータ（共通 `SearchQueryParams` + 投資信託固有の絞り込み）
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct MutualfundSearchQueryParams {
    #[serde(flatten)]
    pub search: SearchQueryParams,
    /// 口座での絞り込み
    pub account: Option<String>,
    /// ファンド名での絞り込み
    pub fund_name: Option<String>,
    /// 分配金での絞り込み
    pub dividends: Option<String>,
}

impl MutualfundSearchQueryParams {
    pub fn page(&self) -> i64 {
        self.search.page()
    }

    pub fn per_page(&self) -> i64 {
        self.search.per_page()
    }

    pub fn offset(&self) -> i64 {
        self.search.offset()
    }

    pub fn should_include_summary(&self) -> bool {
        self.search.should_include_summary()
    }

    pub fn should_include_facets(&self) -> bool {
        self.search.should_include_facets()
    }
}

/// 投資信託 検索条件全体の集計
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct MutualfundSummary {
    #[schema(value_type = f64)]
    pub total_realized_profit_and_loss: Decimal,
    #[schema(value_type = f64)]
    pub total_taxes: Decimal,
    #[schema(value_type = f64)]
    pub total_realized_profit_and_loss_after_tax: Decimal,
}

impl Validate for CreateMutualfundRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        validate_length_field(
            &mut errors,
            "fund_name",
            &self.fund_name,
            Some(1),
            Some(300),
        );
        validate_length_field(&mut errors, "dividends", &self.dividends, None, Some(100));
        validate_length_field(&mut errors, "account", &self.account, Some(1), Some(100));
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
#[cfg(test)]
mod tests {
    use {super::*, rust_decimal_macros::dec};

    fn base() -> CreateMutualfundRequest {
        CreateMutualfundRequest {
            trade_date: NaiveDate::from_ymd_opt(2024, 1, 15).expect("有効な日付 2024-01-15"),
            settlement_date: NaiveDate::from_ymd_opt(2024, 1, 19).expect("有効な日付 2024-01-19"),
            fund_name: "テストファンド".to_string(),
            dividends: Some("再投資型".to_string()),
            account: "特定".to_string(),
            shares: dec!(10000),
            exchange_rate: dec!(1),
            cancellation_unit_price_yen: dec!(15000),
            cancellation_amount_yen: dec!(150000),
            average_acquisition_price_yen: dec!(14000),
            realized_profit_and_loss: dec!(10000),
            taxes: dec!(2000),
            realized_profit_and_loss_after_tax: dec!(8000),
        }
    }

    #[test]
    fn test_create_mutualfund_request_validation() {
        assert!(base().validate().is_ok());

        // fund_name は min=1 のため空文字は NG
        assert!(CreateMutualfundRequest {
            fund_name: String::new(),
            ..base()
        }
        .validate()
        .is_err());

        // account は min=1 のため空文字は NG
        assert!(CreateMutualfundRequest {
            account: String::new(),
            ..base()
        }
        .validate()
        .is_err());

        // dividends は max=100 のため 101 文字は NG
        assert!(CreateMutualfundRequest {
            dividends: Some("あ".repeat(101)),
            ..base()
        }
        .validate()
        .is_err());

        // dividends は None でも OK
        assert!(CreateMutualfundRequest {
            dividends: None,
            ..base()
        }
        .validate()
        .is_ok());
    }

    #[test]
    fn test_mutualfund_search_query_params_delegate_include_flags() {
        let mut params = MutualfundSearchQueryParams::default();
        assert!(!params.should_include_summary());
        assert!(!params.should_include_facets());

        params.search.include_summary = Some(true);
        params.search.include_facets = Some(true);
        assert!(params.should_include_summary());
        assert!(params.should_include_facets());
    }
}
