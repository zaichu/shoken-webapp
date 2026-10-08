use crate::models::common::{SearchParamsAccessor, SearchQueryParams, validate_length_field};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared::value::SecurityCode;
use std::{borrow::Cow, collections::BTreeMap};
use utoipa::ToSchema;
use validator::{Validate, ValidationErrors, ValidationErrorsKind};

pub use shared::domain::{AssetBalance, AssetBalanceSummary};

impl crate::db::FromRow for AssetBalance {
    fn from_row(row: &crate::db::Row) -> Result<Self, crate::db::DbError> {
        Ok(Self {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            security_code: row.try_get("security_code")?,
            security_name: row.try_get("security_name")?,
            shares: row.try_get::<_, crate::db::Numeric>("shares")?.0,
            executing_shares: row.try_get::<_, crate::db::Numeric>("executing_shares")?.0,
            average_purchase_price: row
                .try_get::<_, crate::db::Numeric>("average_purchase_price")?
                .0,
            total_purchase_amount: row
                .try_get::<_, crate::db::Numeric>("total_purchase_amount")?
                .0,
            current_price: row.try_get::<_, crate::db::Numeric>("current_price")?.0,
            daily_change: row.try_get::<_, crate::db::Numeric>("daily_change")?.0,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl crate::db::FromRow for AssetBalanceSummary {
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

/// 保有銘柄作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAssetBalanceRequest {
    #[schema(value_type = String)]
    pub security_code: SecurityCode,
    pub security_name: String,
    #[schema(value_type = f64)]
    pub shares: Decimal,
    #[schema(value_type = f64)]
    pub executing_shares: Decimal,
    #[schema(value_type = f64)]
    pub average_purchase_price: Decimal,
    #[schema(value_type = f64)]
    pub total_purchase_amount: Decimal,
    #[schema(value_type = f64)]
    pub current_price: Decimal,
    #[schema(value_type = f64)]
    pub daily_change: Decimal,
    #[schema(value_type = f64)]
    pub market_value: Decimal,
    #[schema(value_type = f64)]
    pub profit_loss_rate: Decimal,
}

impl Validate for CreateAssetBalanceRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
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

/// 保有銘柄一括作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BulkCreateAssetBalanceRequest {
    pub items: Vec<CreateAssetBalanceRequest>,
}

/// 一括作成リクエストの件数上限。
/// 他の一括 JSON エンドポイントに上限の慣例はないため、API 全体の一覧取得上限
/// （`PaginationParams::per_page` の最大 1000）に合わせる。単一 UNNEST INSERT の
/// 規模と、全削除→全挿入の書き込み増幅を抑える目的。
const MAX_BULK_CREATE_ITEMS: u64 = 1000;

impl Validate for BulkCreateAssetBalanceRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        validate_length_field(
            &mut errors,
            "items",
            &self.items,
            Some(1),
            Some(MAX_BULK_CREATE_ITEMS),
        );
        // 件数自体が不正（空・上限超過）の場合は要素検証を省略する。
        // `items` キーには既に長さエラーが入っており、List 形式で上書きすると
        // 件数エラーが失われるため。
        if errors.is_empty() {
            let element_errors = self
                .items
                .iter()
                .enumerate()
                .filter_map(|(index, item)| {
                    item.validate()
                        .err()
                        .map(|element| (index, Box::new(element)))
                })
                .collect::<BTreeMap<_, _>>();
            if !element_errors.is_empty() {
                errors.errors_mut().insert(
                    Cow::Borrowed("items"),
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

/// 保有銘柄一覧の検索クエリパラメータ（共通 `SearchQueryParams` + 保有銘柄固有の絞り込み）
///
/// asset_balances には snapshot 日付がないため、`search` の date 系フィールドは
/// この検索では使用しない（対象外）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct AssetBalanceSearchQueryParams {
    #[serde(flatten)]
    pub search: SearchQueryParams,
    /// 銘柄コードでの絞り込み
    pub security_code: Option<String>,
    /// 銘柄名での絞り込み
    pub security_name: Option<String>,
}

impl SearchParamsAccessor for AssetBalanceSearchQueryParams {
    fn search_params(&self) -> &SearchQueryParams {
        &self.search
    }
}

#[cfg(test)]
mod tests {
    use {super::*, rust_decimal_macros::dec};

    fn base() -> CreateAssetBalanceRequest {
        CreateAssetBalanceRequest {
            security_code: "1234".parse().expect("valid code"),
            security_name: "テスト株式会社".to_string(),
            shares: dec!(100),
            executing_shares: dec!(0),
            average_purchase_price: dec!(1500),
            total_purchase_amount: dec!(150000),
            current_price: dec!(1600),
            daily_change: dec!(10),
            market_value: dec!(160000),
            profit_loss_rate: dec!(6.67),
        }
    }

    #[test]
    fn test_create_asset_balance_request_validation() {
        assert!(base().validate().is_ok());

        // security_code の形式は SecurityCode の serde(try_from) が JSON 入力時に検証する
        assert!("12345678901".parse::<SecurityCode>().is_err());
        assert!("".parse::<SecurityCode>().is_err());

        // security_name は min=1 のため空文字は NG
        assert!(
            CreateAssetBalanceRequest {
                security_name: String::new(),
                ..base()
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn test_bulk_create_asset_balance_request_validation() {
        // items が空の場合は NG（min=1）
        assert!(
            BulkCreateAssetBalanceRequest { items: vec![] }
                .validate()
                .is_err()
        );

        // items が 1 件以上なら OK
        assert!(
            BulkCreateAssetBalanceRequest {
                items: vec![base()]
            }
            .validate()
            .is_ok()
        );
    }

    #[test]
    fn test_bulk_create_rejects_invalid_element_with_index() {
        // security_name 違反（空文字）はインデックス付きで拒否される
        let request = BulkCreateAssetBalanceRequest {
            items: vec![
                base(),
                base(),
                CreateAssetBalanceRequest {
                    security_name: String::new(),
                    ..base()
                },
            ],
        };
        let errors = request.validate().expect_err("不正な要素は拒否される");
        assert!(format!("{errors}").contains("items[2]"));

        // 全要素が正常なら成功する
        assert!(
            BulkCreateAssetBalanceRequest {
                items: vec![base(), base()]
            }
            .validate()
            .is_ok()
        );
    }

    #[test]
    fn test_bulk_create_rejects_too_many_items() {
        // 上限 1000 件は OK
        let items = vec![base(); 1000];
        assert!(BulkCreateAssetBalanceRequest { items }.validate().is_ok());

        // 1001 件は NG
        let items = vec![base(); 1001];
        assert!(BulkCreateAssetBalanceRequest { items }.validate().is_err());
    }

    #[test]
    fn test_asset_balance_search_query_params_delegate_include_flags() {
        let mut params = AssetBalanceSearchQueryParams::default();
        assert!(!params.should_include_summary());
        assert!(!params.should_include_facets());

        params.search.include_summary = Some(true);
        params.search.include_facets = Some(true);
        assert!(params.should_include_summary());
        assert!(params.should_include_facets());
    }
}
