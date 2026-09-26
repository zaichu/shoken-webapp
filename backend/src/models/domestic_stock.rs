use crate::models::common::{validate_length_field, SearchParamsAccessor, SearchQueryParams};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationErrors};

pub use shared::domain::{DomesticStock, DomesticStockSummary};

/// 国内株式取引一覧の検索クエリパラメータ（共通 `SearchQueryParams` + 国内株式固有の絞り込み）
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct DomesticStockSearchQueryParams {
    #[serde(flatten)]
    pub search: SearchQueryParams,
    /// 口座での絞り込み
    pub account: Option<String>,
    /// 銘柄コードでの絞り込み
    pub security_code: Option<String>,
    /// 銘柄名での絞り込み
    pub security_name: Option<String>,
}

impl SearchParamsAccessor for DomesticStockSearchQueryParams {
    fn search_params(&self) -> &SearchQueryParams {
        &self.search
    }
}

/// 国内株式取引作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateDomesticStockRequest {
    pub trade_date: NaiveDate,
    pub settlement_date: NaiveDate,
    pub security_code: String,
    pub security_name: String,
    pub account: String,
    #[schema(value_type = f64)]
    pub shares: Decimal,
    #[schema(value_type = f64)]
    pub asked_price: Decimal,
    #[schema(value_type = f64)]
    pub proceeds: Decimal,
    #[schema(value_type = f64)]
    pub purchase_price: Decimal,
    #[schema(value_type = f64)]
    pub realized_profit_and_loss: Decimal,
    #[schema(value_type = f64)]
    pub taxes: Decimal,
    #[schema(value_type = f64)]
    pub realized_profit_and_loss_after_tax: Decimal,
}

impl Validate for CreateDomesticStockRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        validate_length_field(
            &mut errors,
            "security_code",
            &self.security_code,
            Some(1),
            Some(10),
        );
        validate_length_field(
            &mut errors,
            "security_name",
            &self.security_name,
            Some(1),
            Some(200),
        );
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

    fn base() -> CreateDomesticStockRequest {
        CreateDomesticStockRequest {
            trade_date: NaiveDate::from_ymd_opt(2024, 1, 15).expect("有効な日付 2024-01-15"),
            settlement_date: NaiveDate::from_ymd_opt(2024, 1, 17).expect("有効な日付 2024-01-17"),
            security_code: "1234".to_string(),
            security_name: "テスト株式会社".to_string(),
            account: "特定".to_string(),
            shares: dec!(100),
            asked_price: dec!(1500),
            proceeds: dec!(150000),
            purchase_price: dec!(1400),
            realized_profit_and_loss: dec!(10000),
            taxes: dec!(2000),
            realized_profit_and_loss_after_tax: dec!(8000),
        }
    }

    #[test]
    fn test_create_domestic_stock_request_validation() {
        assert!(base().validate().is_ok());

        // security_code は max=10 のため 11 文字は NG
        assert!(CreateDomesticStockRequest {
            security_code: "12345678901".to_string(),
            ..base()
        }
        .validate()
        .is_err());

        // security_name は min=1 のため空文字は NG
        assert!(CreateDomesticStockRequest {
            security_name: String::new(),
            ..base()
        }
        .validate()
        .is_err());

        // account は min=1 のため空文字は NG
        assert!(CreateDomesticStockRequest {
            account: String::new(),
            ..base()
        }
        .validate()
        .is_err());
    }

    #[test]
    fn test_domestic_stock_search_query_params_delegate_include_flags() {
        let mut params = DomesticStockSearchQueryParams::default();
        assert!(!params.should_include_summary());
        assert!(!params.should_include_facets());

        params.search.include_summary = Some(true);
        params.search.include_facets = Some(true);
        assert!(params.should_include_summary());
        assert!(params.should_include_facets());
    }
}
