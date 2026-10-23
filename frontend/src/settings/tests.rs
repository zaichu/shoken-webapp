use super::*;
use rust_decimal_macros::dec;

#[test]
fn asset_sort_round_trips_and_rejects_invalid() {
    for sort in AssetSort::ALL {
        assert_eq!(AssetSort::parse(&sort.value()), Some(sort));
    }
    assert_eq!(AssetSort::parse("market_value"), None);
    assert_eq!(AssetSort::parse("bogus:desc"), None);
    assert_eq!(AssetSort::parse("market_value:sideways"), None);
    assert_eq!(AssetSort::parse(""), None);
}

#[test]
fn amount_unit_accepts_only_known_values() {
    for unit in [AmountUnit::Yen, AmountUnit::Sen, AmountUnit::Man] {
        assert_eq!(AmountUnit::from_value(unit.value()), Some(unit));
    }
    assert_eq!(AmountUnit::from_value("百万円"), None);
    assert_eq!(AmountUnit::from_value(""), None);
}

#[test]
fn idle_minutes_accepts_only_the_choices() {
    for minutes in IDLE_LOGOUT_CHOICES {
        assert_eq!(parse_idle_minutes(&minutes.to_string()), Some(minutes));
    }
    assert_eq!(parse_idle_minutes("45"), None);
    assert_eq!(parse_idle_minutes("0"), None);
    assert_eq!(parse_idle_minutes("abc"), None);
}

#[test]
fn receipts_tab_parses_only_known_kinds() {
    for tab in ReceiptsTab::ALL {
        assert_eq!(parse_receipts_tab(tab.kind_param()), Some(tab));
    }
    assert_eq!(parse_receipts_tab("bond"), None);
    assert_eq!(parse_receipts_tab(""), None);
}

#[test]
fn theme_accepts_only_known_values() {
    for theme in [Theme::System, Theme::Light, Theme::Dark] {
        assert_eq!(Theme::from_value(theme.value()), Some(theme));
    }
    assert_eq!(Theme::from_value("auto"), None);
    assert_eq!(Theme::from_value(""), None);
}

#[test]
fn format_amount_follows_the_amount_unit_signal() {
    let unit = amount_unit_signal();
    unit.set(AmountUnit::Yen);
    assert_eq!(format_amount(dec!(123456)), "¥123,456");
    assert_eq!(format_amount(dec!(-12345)), "-¥12,345");
    unit.set(AmountUnit::Sen);
    assert_eq!(format_amount(dec!(123456)), "123.5千円");
    assert_eq!(format_amount(dec!(-12345)), "-12.3千円");
    unit.set(AmountUnit::Man);
    assert_eq!(format_amount(dec!(123456)), "12.3万円");
    assert_eq!(format_amount(dec!(99)), "0万円");
    // 小額の損失がゼロに丸まっても符号は残す
    assert_eq!(format_amount(dec!(-99)), "-0.0万円");
    unit.set(AmountUnit::Yen);
}
