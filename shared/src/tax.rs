use rust_decimal::Decimal;

/// 特定口座の源泉徴収税率（所得税15% + 住民税5% + 復興特別所得税0.315%）
pub const TAX_RATE: Decimal = Decimal::from_parts(20315, 0, 0, false, 5);

/// 口座種別の文字列に含まれる特定口座の判定キーワード（"特定" / "特定口座" など）
pub const SPECIFIC_ACCOUNT_KEYWORD: &str = "特定";

/// account 名に「特定」を含むか（特定口座かどうか）
pub fn is_taxable_account(account: &str) -> bool {
    account.contains(SPECIFIC_ACCOUNT_KEYWORD)
}

/// 利益がプラスの場合だけ floor(利益 * 税率) を返す。損失・ゼロなら 0。
pub fn tax_amount(realized_pnl: Decimal) -> Decimal {
    if realized_pnl.is_sign_positive() {
        (realized_pnl * TAX_RATE).floor()
    } else {
        Decimal::ZERO
    }
}

/// 行単位の税金を計算する（特定口座かつ利益がある場合のみ）。
/// 戻り値は (税額, 税引後損益)。
pub fn compute_taxes(account: &str, realized_pnl: Decimal) -> (Decimal, Decimal) {
    let taxes = if is_taxable_account(account) {
        tax_amount(realized_pnl)
    } else {
        Decimal::ZERO
    };
    (taxes, realized_pnl - taxes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn tax_rate_is_20_315_percent() {
        assert_eq!(TAX_RATE, dec!(0.20315));
    }

    #[test]
    fn compute_taxes_only_taxes_specific_account_profit() {
        for (account, pnl, expected_taxes, expected_after) in [
            ("特定", dec!(10000), dec!(2031), dec!(7969)),
            ("特定口座", dec!(10000), dec!(2031), dec!(7969)),
            ("特定", dec!(-5000), Decimal::ZERO, dec!(-5000)),
            ("特定", dec!(0), Decimal::ZERO, dec!(0)),
            ("NISA", dec!(10000), Decimal::ZERO, dec!(10000)),
            ("NISA口座", dec!(10000), Decimal::ZERO, dec!(10000)),
            ("一般口座", dec!(10000), Decimal::ZERO, dec!(10000)),
        ] {
            assert_eq!(
                compute_taxes(account, pnl),
                (expected_taxes, expected_after)
            );
        }
    }

    #[test]
    fn tax_amount_floors_positive_profit_only() {
        assert_eq!(tax_amount(dec!(10000)), dec!(2031));
        assert_eq!(tax_amount(dec!(1)), dec!(0));
        assert_eq!(tax_amount(dec!(0)), dec!(0));
        assert_eq!(tax_amount(dec!(-1)), dec!(0));
    }
}
