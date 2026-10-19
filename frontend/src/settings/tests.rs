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
fn review_prompt_pref_rejects_broken_json() {
    for raw in ["", "{", "[]", "\"text\"", "null", "42"] {
        assert_eq!(ReviewPromptPref::parse_json(raw), None, "raw={raw}");
    }
}

#[test]
fn review_prompt_pref_defaults_columns_when_key_missing() {
    let pref = ReviewPromptPref::parse_json(r#"{"preamble":"よろしく"}"#).unwrap();
    assert_eq!(pref.preamble, "よろしく");
    assert_eq!(pref.columns, default_prompt_columns());
}

#[test]
fn review_prompt_pref_drops_unknown_columns_and_normalizes_order() {
    let pref = ReviewPromptPref::parse_json(
        r#"{"preamble":"","columns":["bogus","current_price","shares","shares"]}"#,
    )
    .unwrap();
    // 未定義名は捨て、選択順ではなく既定の列順に揃える
    assert_eq!(
        pref.columns,
        vec![PromptColumn::Shares, PromptColumn::CurrentPrice]
    );
}

#[test]
fn review_prompt_pref_round_trips() {
    let pref = ReviewPromptPref {
        preamble: "テスト用の前置き".to_string(),
        columns: vec![PromptColumn::TotalPurchase],
    };
    assert_eq!(ReviewPromptPref::parse_json(&pref.to_json()), Some(pref));
}

#[test]
fn review_prompt_set_column_toggles_membership() {
    let mut pref = ReviewPromptPref::default();
    pref.set_column(PromptColumn::CurrentPrice, true);
    pref.set_column(PromptColumn::Shares, false);
    assert_eq!(
        pref.columns,
        vec![PromptColumn::AveragePrice, PromptColumn::CurrentPrice]
    );
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
    unit.set(AmountUnit::Yen);
}
