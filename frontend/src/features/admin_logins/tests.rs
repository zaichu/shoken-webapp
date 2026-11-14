use super::*;

#[test]
fn more_button_needs_successful_first_load_and_remaining_rows() {
    assert!(!can_load_more(false, 50, Some(64)));
    assert!(!can_load_more(true, 50, None));
    assert!(can_load_more(true, 50, Some(64)));
    assert!(!can_load_more(true, 64, Some(64)));
    assert!(!can_load_more(true, 70, Some(64)));
    assert!(!can_load_more(
        true,
        usize::try_from(i64::MAX).unwrap_or(usize::MAX),
        Some(i64::MAX)
    ));
}

#[test]
fn sort_order_words_follow_the_target_kind() {
    assert_ne!(
        sort_order_label("occurred_at", true),
        sort_order_label("occurred_at", false)
    );
    assert_ne!(
        sort_order_label("occurred_at", true),
        sort_order_label("email", true)
    );
    for column in ["email", "result", "place"] {
        assert_eq!(
            sort_order_label(column, true),
            sort_order_label("place", true)
        );
        assert_eq!(
            sort_order_label(column, false),
            sort_order_label("place", false)
        );
    }
}

#[test]
fn sort_changes_reset_page_and_toggle_direction() {
    let mut query = ListQuery {
        page: 3,
        ..Default::default()
    };
    query.select_sort("email");
    assert_eq!(
        (query.page, query.sort_by, query.descending),
        (1, "email", true)
    );
    query.select_sort("email");
    assert!(!query.descending);
    query.select_sort("place");
    assert!(query.descending);
}

#[test]
fn filters_reset_page_and_are_sent_with_pagination_and_sort() {
    let mut query = ListQuery {
        page: 3,
        ..Default::default()
    };
    query.set_filter("email", "long@example.com".into());
    query.set_filter("place", "Tokyo".into());
    query.set_filter("q", "JP".into());
    assert_eq!(query.page, 1);
    query.page = 2;
    let pairs = query.pairs();
    for expected in [
        ("page", "2"),
        ("per_page", "50"),
        ("email", "long@example.com"),
        ("place", "Tokyo"),
        ("q", "JP"),
        ("sort_by", "occurred_at"),
        ("sort_order", "desc"),
    ] {
        assert!(
            pairs
                .iter()
                .any(|(key, value)| *key == expected.0 && value == expected.1)
        );
    }
}
