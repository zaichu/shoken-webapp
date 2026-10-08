use crate::models::common::{SearchParamsAccessor, SearchQueryParams, validate_length_field};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared::value::Account;
use utoipa::ToSchema;
use validator::{Validate, ValidationErrors};

pub use shared::domain::{Dividend, DividendSummary};

impl crate::db::FromRow for Dividend {
    fn from_row(row: &crate::db::Row) -> Result<Self, crate::db::DbError> {
        Ok(Self {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            settlement_date: row.try_get("settlement_date")?,
            product: row.try_get("product")?,
            account: row.try_get("account")?,
            security_code: row.try_get("security_code")?,
            security_name: row.try_get("security_name")?,
            unit_price: row.try_get::<_, crate::db::Numeric>("unit_price")?.0,
            shares: row.try_get::<_, crate::db::Numeric>("shares")?.0,
            dividends_before_tax: row
                .try_get::<_, crate::db::Numeric>("dividends_before_tax")?
                .0,
            taxes: row.try_get::<_, crate::db::Numeric>("taxes")?.0,
            net_amount_received: row
                .try_get::<_, crate::db::Numeric>("net_amount_received")?
                .0,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl crate::db::FromRow for DividendSummary {
    fn from_row(row: &crate::db::Row) -> Result<Self, crate::db::DbError> {
        Ok(Self {
            total_dividends_before_tax: row
                .try_get::<_, crate::db::Numeric>("total_dividends_before_tax")?
                .0,
            total_taxes: row.try_get::<_, crate::db::Numeric>("total_taxes")?.0,
            total_net_amount_received: row
                .try_get::<_, crate::db::Numeric>("total_net_amount_received")?
                .0,
        })
    }
}

/// 配当金一覧の検索クエリパラメータ（共通 `SearchQueryParams` + 配当金固有の絞り込み）
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct DividendSearchQueryParams {
    #[serde(flatten)]
    pub search: SearchQueryParams,
    /// 商品での絞り込み
    pub product: Option<String>,
    /// 口座での絞り込み
    pub account: Option<String>,
    /// 銘柄コードでの絞り込み
    pub security_code: Option<String>,
    /// 銘柄名での絞り込み
    pub security_name: Option<String>,
}

impl SearchParamsAccessor for DividendSearchQueryParams {
    fn search_params(&self) -> &SearchQueryParams {
        &self.search
    }
}

/// 配当金作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateDividendRequest {
    pub settlement_date: NaiveDate,
    pub product: String,
    #[schema(value_type = String)]
    pub account: Account,
    // 配当の銘柄コードは空文字を許容する既存仕様のため String のまま
    pub security_code: String,
    pub security_name: String,
    #[schema(value_type = f64)]
    pub unit_price: Decimal,
    #[schema(value_type = f64)]
    pub shares: Decimal,
    #[schema(value_type = f64)]
    pub dividends_before_tax: Decimal,
    #[schema(value_type = f64)]
    pub taxes: Decimal,
    #[schema(value_type = f64)]
    pub net_amount_received: Decimal,
}

impl Validate for CreateDividendRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        validate_length_field(&mut errors, "product", &self.product, Some(1), Some(100));
        validate_length_field(
            &mut errors,
            "security_code",
            &self.security_code,
            None,
            Some(10),
        );
        validate_length_field(
            &mut errors,
            "security_name",
            &self.security_name,
            Some(1),
            Some(200),
        );
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
    fn base() -> CreateDividendRequest {
        CreateDividendRequest {
            settlement_date: NaiveDate::from_ymd_opt(2024, 1, 15).expect("有効な日付 2024-01-15"),
            product: "国内株式".to_string(),
            account: "特定".parse().expect("valid account"),
            security_code: "1234".to_string(),
            security_name: "テスト株式会社".to_string(),
            unit_price: dec!(100),
            shares: dec!(100),
            dividends_before_tax: dec!(1000),
            taxes: dec!(200),
            net_amount_received: dec!(800),
        }
    }

    #[test]
    fn test_create_dividend_request_validation() {
        assert!(base().validate().is_ok());

        // security_code は max=10 のため 11 文字は NG
        assert!(
            CreateDividendRequest {
                security_code: "12345678901".to_string(),
                ..base()
            }
            .validate()
            .is_err()
        );

        // product は min=1 のため空文字は NG
        assert!(
            CreateDividendRequest {
                product: String::new(),
                ..base()
            }
            .validate()
            .is_err()
        );

        // account の形式は Account の serde(try_from) が JSON 入力時に検証する
        assert!("".parse::<Account>().is_err());

        // security_name は min=1 のため空文字は NG
        assert!(
            CreateDividendRequest {
                security_name: String::new(),
                ..base()
            }
            .validate()
            .is_err()
        );

        // security_code は空文字でも OK（max=10 のみ）
        assert!(
            CreateDividendRequest {
                security_code: String::new(),
                ..base()
            }
            .validate()
            .is_ok()
        );
    }
}
