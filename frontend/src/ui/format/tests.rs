use super::*;

#[test]
fn intl_fixed_matches_number_format_boundary_cases() {
    assert_eq!(intl_fixed(0.0, 2), 0.0);
    assert_eq!(intl_fixed(0.125, 2), 0.13);
    assert_eq!(intl_fixed(-0.125, 2), -0.13);
    assert_eq!(intl_fixed(1.005, 2), 1.01);
    assert_eq!(intl_fixed(2.675, 2), 2.68);
    assert_eq!(intl_fixed(-1.005, 2), -1.01);
    assert_eq!(intl_fixed(-2.675, 2), -2.68);
    assert_eq!(intl_fixed(0.995, 2), 1.0);
    assert_eq!(intl_fixed(1.004, 2), 1.0);
    assert_eq!(intl_fixed(201.0 / 20000.0 * 100.0, 2), 1.01);
    assert_eq!(intl_fixed(1.5, 2), 1.5);
    assert_eq!(intl_fixed(100.0, 2), 100.0);
    assert_eq!(intl_fixed(12.5, 0), 13.0);
    assert_eq!(intl_fixed(-12.5, 0), -13.0);
    assert_eq!(intl_fixed(0.75, 0), 1.0);
    assert_eq!(intl_fixed(-0.75, 0), -1.0);
    assert_eq!(intl_fixed(0.1 + 0.2, 10), 0.3);
    assert_eq!(intl_fixed(f64::from_bits(1), 2), 0.0);
    assert!(intl_fixed(-f64::from_bits(1), 2).is_sign_negative());
    let negative_zero = intl_fixed(-0.001, 2);
    assert_eq!(negative_zero, 0.0);
    assert!(negative_zero.is_sign_negative());
    assert_eq!(intl_fixed(1e30, 2), 1e30);
    assert_eq!(intl_fixed(f64::INFINITY, 2), f64::INFINITY);
    assert!(intl_fixed(f64::NAN, 2).is_nan());
}

#[test]
fn rounding_display_locked_before_unification() {
    // 0・負数・x.5・桁の多い値・小数第2位の表示を固定する
    assert_eq!(format_number_value(0.0), "0");
    assert_eq!(format_number_value(-0.0), "0");
    assert_eq!(format_number_value(-0.001), "-0");
    assert_eq!(format_number_value(-12345.678), "-12,345.68");
    assert_eq!(format_number_value(1.005), "1.01");
    assert_eq!(format_number_value(2.675), "2.68");
    assert_eq!(format_number_value(0.995), "1");
    assert_eq!(format_number_value(1.2345), "1.23");
    assert_eq!(format_fixed_percent(1.005, 2), "1.01%");
    assert_eq!(format_fixed_percent(0.0, 2), "0.00%");
    assert_eq!(format_currency(0.0), "¥0");
    assert_eq!(format_currency(-0.0), "¥0");
    assert_eq!(
        format_currency(1e29),
        "¥100,000,000,000,000,000,000,000,000,000"
    );
    assert_eq!(format_number_value(f64::NAN), "—");
    assert_eq!(format_number_value(f64::INFINITY), "—");
    assert_eq!(format_fixed_percent(f64::NAN, 1), "—");
    assert_eq!(format_fixed_percent(f64::INFINITY, 1), "—");
}

#[test]
fn number_and_currency_formatters_match_intl_cases() {
    assert_eq!(format_number_value(100.0), "100");
    assert_eq!(format_number_value(-12345.0), "-12,345");
    assert_eq!(format_number_value(1.5), "1.5");
    assert_eq!(format_number_value(1.2345), "1.23");
    assert_eq!(format_number_value(-0.001), "-0");
    // 符号は丸め前の値の `num < 0` で判定するため -0 は `0` と表示する
    assert_eq!(format_number_value(-0.0), "0");
    assert_eq!(format_currency(-0.0), "¥0");
    assert_eq!(format_number_value(-12345.678), "-12,345.68");
    // Intl.NumberFormat は toFixed と異なり10進の値で半分以上を切り上げる
    assert_eq!(format_number_value(1.005), "1.01");
    assert_eq!(format_number_value(-1.005), "-1.01");
    assert_eq!(format_number_value(2.675), "2.68");
    assert_eq!(format_number_value(0.995), "1");
    assert_eq!(format_currency(850000.0), "¥850,000");
    assert_eq!(format_currency(0.0), "¥0");
    assert_eq!(format_currency(-250000.0), "-¥250,000");
    assert_eq!(format_currency(2600.0), "¥2,600");
    assert_eq!(format_currency(123.456), "¥123.456");
    assert_eq!(format_fixed_percent(1.005, 2), "1.01%");
    assert_eq!(format_currency(f64::NAN), "—");
    assert_eq!(
        format_currency("0.123456789012345678".parse::<f64>().unwrap()),
        "¥0.123456789012346"
    );
    assert_eq!(format_fixed_percent(f64::NAN, 1), "—");
    assert_eq!(
        format_currency(1e29),
        "¥100,000,000,000,000,000,000,000,000,000"
    );
    assert_eq!(
        format_currency(-1e29),
        "-¥100,000,000,000,000,000,000,000,000,000"
    );
    assert_eq!(
        format_fixed_percent(1e30, 1),
        "1000000000000000019884624838656.0%"
    );
    assert_eq!(format_fixed_percent(f64::INFINITY, 1), "—");
}

#[test]
fn percentage_value_formats_with_fixed_decimals() {
    assert_eq!(format_percentage_value(2.0), "2.00%");
    assert_eq!(format_percentage_value(0.9564), "0.96%");
    assert_eq!(format_percentage_value(1.005), "1.01%");
    assert_eq!(format_percentage_value(2.675), "2.68%");
    assert_eq!(format_percentage_value(f64::INFINITY), "—");
    assert_eq!(format_percentage_value(f64::NAN), "—");
}

#[test]
fn integer_percent_rounds_to_whole_numbers() {
    assert_eq!(format_integer_percent(42.4), "42%");
    assert_eq!(format_integer_percent(42.5), "43%");
    assert_eq!(format_integer_percent(0.0), "0%");
}

#[test]
fn unit_price_decimals_aligns_the_column() {
    assert_eq!(unit_price_decimals([100.0, 200.0].into_iter()), 0);
    assert_eq!(unit_price_decimals([100.0, 200.5].into_iter()), 1);
    assert_eq!(unit_price_decimals([100.125, 200.5].into_iter()), 2);
    // 列の中の上限を超える桁は2桁で揃える
    assert_eq!(unit_price_decimals([100.125, 200.567].into_iter()), 2);
    assert_eq!(unit_price_decimals([].into_iter()), 0);
}

#[test]
fn unit_price_keeps_the_column_decimals() {
    assert_eq!(format_unit_price(100.0, 0), "¥100");
    assert_eq!(format_unit_price(100.5, 1), "¥100.5");
    assert_eq!(format_unit_price(100.5, 2), "¥100.50");
    assert_eq!(format_unit_price(-100.5, 2), "-¥100.50");
}

#[test]
fn iso_date_formats_and_rejects_invalid_values() {
    assert_eq!(format_iso_date("2023-12-25"), "2023/12/25");
    assert_eq!(format_iso_date("2023-02-29"), "—");
    assert_eq!(format_iso_date("string"), "—");
    // ハイフン位置だけが違う10桁文字列も日付としては受理しない
    for malformed in [
        "2024-03001",
        "2024X03-01",
        "2024-03X01",
        "202403-01-",
        "X024-03-01",
    ] {
        assert_eq!(format_iso_date(malformed), "—", "{malformed}");
    }
}

#[test]
fn price_date_formats_month_and_day() {
    assert_eq!(format_price_date("2026-10-21"), "10/21");
    assert_eq!(format_price_date("2026-01-05"), "01/05");
    // 想定外の形式はそのまま返す
    assert_eq!(format_price_date("2026/10/21"), "2026/10/21");
}

#[test]
fn nav_updated_at_formats_month_day_and_time() {
    assert_eq!(format_nav_updated_at("2026-10-21 09:30"), "10/21 09:30");
    assert_eq!(format_nav_updated_at("2026-10-21"), "2026-10-21");
}

#[test]
fn jst_date_rolls_over_at_nine_am_boundary() {
    assert_eq!(jst_date("2026-10-17T15:00:00Z"), Some((2026, 10, 18)));
    assert_eq!(jst_date("2026-10-17T14:59:59Z"), Some((2026, 10, 17)));
    assert_eq!(jst_date("2026-10-31T20:00:00Z"), Some((2026, 11, 1)));
    assert_eq!(jst_date("2025-12-31T18:30:00Z"), Some((2026, 1, 1)));
    assert_eq!(jst_date("2026-10-18"), Some((2026, 10, 18)));
    assert_eq!(jst_date("not-a-date"), None);
    assert_eq!(jst_date(""), None);
}

#[test]
fn short_date_drops_the_zero_padding() {
    assert_eq!(format_short_date("2026-10-09"), "10/9");
    assert_eq!(format_short_date("2026-10-21"), "10/21");
    assert_eq!(format_short_date("not-a-date"), "—");
}

#[test]
fn format_jst_time_adds_nine_hours() {
    assert_eq!(format_jst_time("2026-10-25T12:39:00+00:00"), "21:39");
    // 日付を跨ぐ時刻は 24 時間で戻る(日付側は jst_date が繰り上げる)
    assert_eq!(format_jst_time("2026-10-25T20:05:00Z"), "05:05");
    assert_eq!(format_jst_time("not-a-timestamp"), "—");
}

#[test]
fn calendar_days_between_counts_calendar_days() {
    // 同じ日は 0
    assert_eq!(calendar_days_between((2026, 10, 20), (2026, 10, 20)), 0);
    // 1日後・2日後
    assert_eq!(calendar_days_between((2026, 10, 20), (2026, 10, 21)), 1);
    assert_eq!(calendar_days_between((2026, 10, 20), (2026, 10, 22)), 2);
    // 月をまたぐ
    assert_eq!(calendar_days_between((2026, 10, 31), (2026, 11, 1)), 1);
    assert_eq!(calendar_days_between((2026, 1, 31), (2026, 2, 1)), 1);
    // 2月の日数は年で変わる(うるう年)
    assert_eq!(calendar_days_between((2024, 2, 28), (2024, 3, 1)), 2);
    assert_eq!(calendar_days_between((2026, 2, 28), (2026, 3, 1)), 1);
    // 過去の日付なら負
    assert_eq!(calendar_days_between((2026, 10, 22), (2026, 10, 20)), -2);
}

#[test]
fn latest_import_date_picks_newest() {
    let stamps = [
        "2026-10-10T00:00:00Z",
        "2026-10-18T14:00:00Z",
        "2026-10-18T16:00:00Z", // JST では 10/19
        "garbage",
    ];
    assert_eq!(
        format_latest_import_date(stamps.iter().copied()),
        Some("2026/10/19".to_string())
    );
    assert_eq!(format_latest_import_date([].iter().copied()), None);
}

proptest::proptest! {
    #[test]
    fn prop_group_thousands_positions(digits in "[0-9]{1,15}") {
        let grouped = group_thousands(&digits);
        proptest::prop_assert_eq!(grouped.replace(',', ""), digits);
        for (position, character) in grouped.chars().enumerate() {
            if character == ',' {
                proptest::prop_assert_eq!((grouped.len() - position) % 4, 0);
            }
        }
    }

    #[test]
    fn prop_intl_fixed_symmetric_and_quantized(
        mantissa in -9_999_999_999_999i64..9_999_999_999_999i64,
        divisor in 1u32..=6u32,
        decimals in 0u32..=5u32,
    ) {
        let value = mantissa as f64 / 10f64.powi(divisor as i32);
        let result = intl_fixed(value, decimals);
        proptest::prop_assert_eq!(intl_fixed(-value, decimals), -result);
        let unit = 10f64.powi(-(decimals as i32));
        proptest::prop_assert!((result - value).abs() <= 0.6 * unit);
    }

    #[test]
    fn prop_iso_date_matches_naive_model(
        year in 0i32..10000i32,
        month in 0u32..=13u32,
        day in 0u32..=32u32,
    ) {
        let input = format!("{year:04}-{month:02}-{day:02}");
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let max_day = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => 0,
        };
        let expected = (1..=max_day).contains(&day) && (1..=12).contains(&month);
        if expected {
            proptest::prop_assert_eq!(format_iso_date(&input), input.replace('-', "/"));
        } else {
            proptest::prop_assert_eq!(format_iso_date(&input), "—");
        }
    }
}
