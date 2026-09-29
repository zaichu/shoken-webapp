use super::*;

#[test]
fn bar_class_cycles_through_all_tokens_then_wraps() {
    let expected = [
        "bg-holding-1",
        "bg-holding-2",
        "bg-holding-3",
        "bg-holding-4",
        "bg-holding-5",
        "bg-holding-6",
        "bg-holding-7",
        "bg-holding-8",
        "bg-holding-9",
        "bg-holding-10",
    ];
    for (position, class) in expected.iter().enumerate() {
        assert_eq!(holding_bar_class(position), *class);
    }
    assert_eq!(holding_bar_class(10), "bg-holding-1");
    assert_eq!(holding_bar_class(11), "bg-holding-2");
}

#[test]
fn band_class_matches_the_same_token_as_bar() {
    for position in 0..10 {
        let expected = format!("border-l-holding-{}", position + 1);
        assert_eq!(holding_band_class(position), expected);
    }
    assert_eq!(holding_band_class(10), "border-l-holding-1");
}
