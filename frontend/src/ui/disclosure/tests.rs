use super::*;

#[test]
fn disclosure_styles_are_distinguishable() {
    let styles = [
        DisclosureStyle::GroupCard,
        DisclosureStyle::HeaderFlat,
        DisclosureStyle::RowToggle,
    ];
    let classes: Vec<&str> = styles.iter().map(|style| style.class()).collect();
    for (i, class) in classes.iter().enumerate() {
        assert!(!class.is_empty(), "{:?}", styles[i]);
        assert!(!classes[..i].contains(class), "{:?}", styles[i]);
    }
}
