use super::format_number;
use rust_decimal_macros::dec;

#[test]
fn format_number_groups_digits_and_keeps_sign() {
    assert_eq!(format_number(dec!(1000)), "1,000");
    assert_eq!(format_number(dec!(1000000)), "1,000,000");
    assert_eq!(format_number(dec!(1234.56)), "1,234.56");
    assert_eq!(format_number(dec!(0)), "0");
    assert_eq!(format_number(dec!(0.123)), "0.123");
    assert_eq!(format_number(dec!(-1000)), "-1,000");
    assert_eq!(format_number(dec!(-1234.56)), "-1,234.56");
    assert_eq!(format_number(dec!(-0.123)), "-0.123");
}
