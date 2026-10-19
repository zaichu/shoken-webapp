use crate::models::common::{SearchParamsAccessor, SearchQueryParams, validate_length_field};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared::value::Account;
use validator::{Validate, ValidationErrors};

pub use shared::domain::{FundBalance, FundBalanceSummary};

impl crate::db::FromRow for FundBalance {
    fn from_row(row: &crate::db::Row) -> Result<Self, crate::db::DbError> {
        Ok(Self {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            fund_name: row.try_get("fund_name")?,
            account: row.try_get("account")?,
            dividends_course: row.try_get("dividends_course")?,
            shares: row.try_get::<_, crate::db::Numeric>("shares")?.0,
            average_purchase_price: row
                .try_get::<_, crate::db::Numeric>("average_purchase_price")?
                .0,
            total_purchase_amount: row
                .try_get::<_, crate::db::Numeric>("total_purchase_amount")?
                .0,
            base_price: row.try_get::<_, crate::db::Numeric>("base_price")?.0,
            base_price_day_change: row
                .try_get::<_, Option<crate::db::Numeric>>("base_price_day_change")?
                .map(|n| n.0),
            base_price_month_change: row
                .try_get::<_, Option<crate::db::Numeric>>("base_price_month_change")?
                .map(|n| n.0),
            market_value: row.try_get::<_, crate::db::Numeric>("market_value")?.0,
            profit_loss: row.try_get::<_, crate::db::Numeric>("profit_loss")?.0,
            profit_loss_rate: row
                .try_get::<_, Option<crate::db::Numeric>>("profit_loss_rate")?
                .map(|n| n.0),
            total_return: row
                .try_get::<_, Option<crate::db::Numeric>>("total_return")?
                .map(|n| n.0),
            price_as_of: row.try_get("price_as_of")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl crate::db::FromRow for FundBalanceSummary {
    fn from_row(row: &crate::db::Row) -> Result<Self, crate::db::DbError> {
        Ok(Self {
            total_market_value: row
                .try_get::<_, crate::db::Numeric>("total_market_value")?
                .0,
            total_purchase_amount: row
                .try_get::<_, crate::db::Numeric>("total_purchase_amount")?
                .0,
            total_daily_change: row
                .try_get::<_, crate::db::Numeric>("total_daily_change")?
                .0,
        })
    }
}

/// 投資信託残高作成リクエスト（CSV 1行分）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFundBalanceRequest {
    pub fund_name: String,
    pub account: Account,
    pub dividends_course: Option<String>,
    pub shares: Decimal,
    pub average_purchase_price: Decimal,
    pub total_purchase_amount: Decimal,
    pub base_price: Decimal,
    pub base_price_day_change: Option<Decimal>,
    pub base_price_month_change: Option<Decimal>,
    pub market_value: Decimal,
    pub profit_loss: Decimal,
    pub profit_loss_rate: Option<Decimal>,
    pub total_return: Option<Decimal>,
}

impl Validate for CreateFundBalanceRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        validate_length_field(
            &mut errors,
            "fund_name",
            &self.fund_name,
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

/// 投資信託残高一覧の検索クエリパラメータ
///
/// fund_balances の日付列は price_as_of（取込日）だが日付条件は提供しない
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FundBalanceSearchQueryParams {
    #[serde(flatten)]
    pub search: SearchQueryParams,
    /// ファンド名での絞り込み
    pub fund_name: Option<String>,
    /// 口座区分での絞り込み
    pub account: Option<String>,
}

impl SearchParamsAccessor for FundBalanceSearchQueryParams {
    fn search_params(&self) -> &SearchQueryParams {
        &self.search
    }
}
