use crate::value::Account;
use rust_decimal::Decimal;

/// 税率（所得税15.315% + 住民税5%）
pub const TAX_RATE: Decimal = Decimal::from_parts(20315, 0, 0, false, 5);

/// 特定口座を示すキーワード
pub const SPECIFIC_ACCOUNT_KEYWORD: &str = "特定";

/// 税額を計算（実現損益が正の場合のみ課税）
#[must_use]
pub fn tax_amount(realized_pnl: Decimal) -> Decimal {
    if realized_pnl <= Decimal::ZERO {
        return Decimal::ZERO;
    }
    (realized_pnl * TAX_RATE).floor()
}

/// 税額と税引後損益の計算結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaxBreakdown {
    pub taxes: Decimal,
    pub realized_profit_and_loss_after_tax: Decimal,
}

/// 税額と税引後損益を計算
#[must_use]
pub fn compute_taxes(account: &Account, realized_pnl: Decimal) -> TaxBreakdown {
    let taxes = if account.is_specific() {
        tax_amount(realized_pnl)
    } else {
        Decimal::ZERO
    };
    TaxBreakdown {
        taxes,
        realized_profit_and_loss_after_tax: realized_pnl - taxes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn account(s: &str) -> Account {
        Account::try_from(s).expect("test account")
    }

    #[test]
    fn is_specific_matches_keyword() {
        assert!(account("特定").is_specific());
        assert!(account("特定口座").is_specific());
        assert!(account("特定・一般").is_specific());
        assert!(!account("NISA").is_specific());
        assert!(!account("一般").is_specific());
    }

    #[test]
    fn tax_amount_only_on_positive() {
        assert_eq!(tax_amount(dec!(10000)), dec!(2031));
        assert_eq!(tax_amount(dec!(0)), dec!(0));
        assert_eq!(tax_amount(dec!(-500)), dec!(0));
    }

    #[test]
    fn compute_taxes_returns_breakdown() {
        assert_eq!(
            compute_taxes(&account("特定口座"), dec!(10000)),
            TaxBreakdown {
                taxes: dec!(2031),
                realized_profit_and_loss_after_tax: dec!(7969),
            }
        );
        assert_eq!(
            compute_taxes(&account("NISA"), dec!(10000)),
            TaxBreakdown {
                taxes: dec!(0),
                realized_profit_and_loss_after_tax: dec!(10000),
            }
        );
        assert_eq!(
            compute_taxes(&account("特定"), dec!(-500)),
            TaxBreakdown {
                taxes: dec!(0),
                realized_profit_and_loss_after_tax: dec!(-500),
            }
        );
    }
}
