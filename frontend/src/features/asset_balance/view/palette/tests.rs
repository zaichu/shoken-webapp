use super::*;

#[test]
fn bar_class_cycles_through_all_tokens_then_wraps() {
    let expected: Vec<String> = (1..=20).map(|n| format!("bg-holding-{n}")).collect();
    for (position, class) in expected.iter().enumerate() {
        assert_eq!(holding_bar_class(position), *class);
    }
    assert_eq!(holding_bar_class(20), "bg-holding-1");
    assert_eq!(holding_bar_class(21), "bg-holding-2");
}

#[test]
fn band_class_matches_the_same_token_as_bar() {
    for position in 0..20 {
        let expected = format!("border-l-holding-{}", position + 1);
        assert_eq!(holding_band_class(position), expected);
    }
    assert_eq!(holding_band_class(20), "border-l-holding-1");
}
