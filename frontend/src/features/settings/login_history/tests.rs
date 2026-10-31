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
fn datetime_shows_jst_with_year() {
    assert_eq!(format_datetime("2026-10-31T07:38:00Z"), "2026/10/31 16:38");
    assert_eq!(format_datetime("not-a-date"), "not-a-date");
}
