use super::*;

#[test]
fn place_and_agent_match_shared_tables() {
    let country = Some("JP".to_string());
    let region = Some("Tokyo".to_string());
    assert_eq!(
        format_place(&country, &region),
        shared::login_display::format_place(Some("JP"), Some("Tokyo"))
    );
    assert_eq!(format_place(&country, &region), "東京都, 日本");
    let browser = Some("chrome".to_string());
    let os = Some("android".to_string());
    assert_eq!(
        format_agent(&browser, &os),
        shared::login_display::format_agent(Some("chrome"), Some("android"))
    );
    assert_eq!(format_agent(&browser, &os), "Chrome / Android");
}

#[test]
fn unknown_values_pass_through_without_inventing() {
    let country = Some("XX".to_string());
    let region = Some("Atlantis".to_string());
    assert_eq!(format_place(&country, &region), "Atlantis, XX");
    assert_eq!(format_place(&None, &None), "—");
    assert_eq!(format_agent(&None, &None), "—");
}

#[test]
fn raw_user_agent_rounds_to_unknown_browser() {
    let raw = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".to_string();
    assert_eq!(
        format_agent(&Some(raw.clone()), &Some(raw.clone())),
        "不明なブラウザ(Windows)"
    );
    assert_eq!(
        format_place(&Some("JP".to_string()), &Some("Tokyo".to_string())),
        "東京都, 日本"
    );
}

#[test]
fn datetime_shows_jst_with_year() {
    assert_eq!(format_datetime("2026-10-31T07:38:00Z"), "2026/10/31 16:38");
    assert_eq!(format_datetime("not-a-date"), "not-a-date");
}

#[test]
fn result_shows_only_success_or_failure() {
    assert_eq!(result_label("success"), labels::LOGIN_RESULT_SUCCESS);
    assert_eq!(result_label("new_device"), labels::LOGIN_RESULT_SUCCESS);
    assert_eq!(
        result_label("location_changed"),
        labels::LOGIN_RESULT_SUCCESS
    );
    assert_eq!(result_label("failure"), labels::LOGIN_RESULT_FAILURE);
    assert_eq!(
        result_badge_variant("failure"),
        crate::ui::badge::BadgeVariant::Danger
    );
    assert_eq!(
        result_badge_variant("new_device"),
        crate::ui::badge::BadgeVariant::Positive
    );
}

#[test]
fn attention_marks_only_records_to_check() {
    let (label, _, note) = attention_of("new_device").expect("新しい端末は注意");
    assert_eq!(label, labels::LOGIN_RESULT_NEW_DEVICE);
    assert_eq!(note, labels::LOGIN_ATTENTION_NEW_DEVICE_NOTE);
    let (label, _, note) = attention_of("location_changed").expect("場所の変化は注意");
    assert_eq!(label, labels::LOGIN_RESULT_LOCATION_CHANGED);
    assert_eq!(note, labels::LOGIN_ATTENTION_LOCATION_CHANGED_NOTE);
    assert!(attention_of("success").is_none());
    assert!(attention_of("failure").is_none());
}
