use super::*;
use crate::settings;
use crate::settings::AmountUnit;
use leptos::prelude::Set;

#[test]
fn valuation_amount_formats_currency_and_rounding() {
    assert_eq!(format_valuation_amount(Some(60000.0)), "¥60,000");
    assert_eq!(format_valuation_amount(Some(-10000.0)), "-¥10,000");
    assert_eq!(format_valuation_amount(Some(0.0)), "¥0");
    assert_eq!(
        format_valuation_amount(Some(3632999.9999999995)),
        "¥3,633,000"
    );
    assert_eq!(format_valuation_amount(None), "—");
    assert_eq!(format_valuation_amount(Some(f64::NAN)), "—");
    assert_eq!(
        format_valuation_amount(Some(1e29)),
        "¥100,000,000,000,000,000,000,000,000,000"
    );
}

#[test]
fn valuation_amount_formats_each_display_unit() {
    let owner = leptos::prelude::Owner::new();
    owner.with(|| {
        let unit = settings::amount_unit_signal();
        unit.set(AmountUnit::Yen);
        assert_eq!(format_valuation_amount(Some(60000.0)), "¥60,000");
        unit.set(AmountUnit::Sen);
        assert_eq!(format_valuation_amount(Some(60000.0)), "60千円");
        unit.set(AmountUnit::Man);
        assert_eq!(format_valuation_amount(Some(60000.0)), "6万円");
        assert_eq!(format_signed_valuation_amount(Some(60000.0)), "+6万円");
        unit.set(AmountUnit::Yen);
        assert_eq!(format_valuation_amount(Some(-0.4)), "¥0");
        assert_eq!(format_valuation_amount(Some(-0.5)), "-¥1");
    });
}

#[test]
fn row_amount_stays_in_yen_and_formats_signed_values() {
    assert_eq!(format_row_valuation_amount(Some(60000.0)), "¥60,000");
    assert_eq!(format_row_valuation_amount(Some(400.0)), "¥400");
    assert_eq!(format_row_valuation_amount(Some(-400.0)), "-¥400");
    assert_eq!(format_row_valuation_amount(None), "—");
    assert_eq!(format_row_valuation_amount(Some(f64::NAN)), "—");
    assert_eq!(format_signed_row_valuation_amount(Some(400.0)), "+¥400");
    assert_eq!(format_signed_row_valuation_amount(Some(-400.0)), "-¥400");
    assert_eq!(format_signed_row_valuation_amount(Some(23.0)), "+¥23");
    assert_eq!(format_signed_row_valuation_amount(Some(-12.0)), "-¥12");
    assert_eq!(format_signed_row_valuation_amount(Some(0.0)), "¥0");
    assert_eq!(format_signed_row_valuation_amount(None), "—");
}

#[test]
fn valuation_rate_formats_fixed_decimals_and_missing_values() {
    assert_eq!(format_valuation_rate(Some(10.0), 1), "10.0%");
    assert_eq!(format_valuation_rate(Some(-10.0), 1), "-10.0%");
    assert_eq!(format_valuation_rate(Some(0.0), 1), "0.0%");
    assert_eq!(format_valuation_rate(None, 1), "—");
    assert_eq!(format_valuation_rate(Some(f64::NAN), 1), "—");
}
