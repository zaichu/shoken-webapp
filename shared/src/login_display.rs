const COUNTRY_TABLE: &[(&str, &str)] = &[
    ("JP", "日本"),
    ("US", "アメリカ"),
    ("CN", "中国"),
    ("KR", "韓国"),
    ("TW", "台湾"),
    ("HK", "香港"),
    ("SG", "シンガポール"),
    ("TH", "タイ"),
    ("VN", "ベトナム"),
    ("PH", "フィリピン"),
    ("AU", "オーストラリア"),
    ("GB", "イギリス"),
    ("DE", "ドイツ"),
    ("FR", "フランス"),
    ("CA", "カナダ"),
    ("BR", "ブラジル"),
    ("IN", "インド"),
    ("IT", "イタリア"),
    ("ES", "スペイン"),
    ("NL", "オランダ"),
];

const REGION_TABLE: &[(&str, &str)] = &[
    ("hokkaido", "北海道"),
    ("aomori", "青森県"),
    ("iwate", "岩手県"),
    ("miyagi", "宮城県"),
    ("akita", "秋田県"),
    ("yamagata", "山形県"),
    ("fukushima", "福島県"),
    ("ibaraki", "茨城県"),
    ("tochigi", "栃木県"),
    ("gunma", "群馬県"),
    ("saitama", "埼玉県"),
    ("chiba", "千葉県"),
    ("tokyo", "東京都"),
    ("kanagawa", "神奈川県"),
    ("niigata", "新潟県"),
    ("toyama", "富山県"),
    ("ishikawa", "石川県"),
    ("fukui", "福井県"),
    ("yamanashi", "山梨県"),
    ("nagano", "長野県"),
    ("gifu", "岐阜県"),
    ("shizuoka", "静岡県"),
    ("aichi", "愛知県"),
    ("mie", "三重県"),
    ("shiga", "滋賀県"),
    ("kyoto", "京都府"),
    ("osaka", "大阪府"),
    ("hyogo", "兵庫県"),
    ("nara", "奈良県"),
    ("wakayama", "和歌山県"),
    ("tottori", "鳥取県"),
    ("shimane", "島根県"),
    ("okayama", "岡山県"),
    ("hiroshima", "広島県"),
    ("yamaguchi", "山口県"),
    ("tokushima", "徳島県"),
    ("kagawa", "香川県"),
    ("ehime", "愛媛県"),
    ("kochi", "高知県"),
    ("fukuoka", "福岡県"),
    ("saga", "佐賀県"),
    ("nagasaki", "長崎県"),
    ("kumamoto", "熊本県"),
    ("oita", "大分県"),
    ("miyazaki", "宮崎県"),
    ("kagoshima", "鹿児島県"),
    ("okinawa", "沖縄県"),
];

const PREFECTURE_TABLE: [&str; 47] = [
    "北海道",
    "青森県",
    "岩手県",
    "宮城県",
    "秋田県",
    "山形県",
    "福島県",
    "茨城県",
    "栃木県",
    "群馬県",
    "埼玉県",
    "千葉県",
    "東京都",
    "神奈川県",
    "新潟県",
    "富山県",
    "石川県",
    "福井県",
    "山梨県",
    "長野県",
    "岐阜県",
    "静岡県",
    "愛知県",
    "三重県",
    "滋賀県",
    "京都府",
    "大阪府",
    "兵庫県",
    "奈良県",
    "和歌山県",
    "鳥取県",
    "島根県",
    "岡山県",
    "広島県",
    "山口県",
    "徳島県",
    "香川県",
    "愛媛県",
    "高知県",
    "福岡県",
    "佐賀県",
    "長崎県",
    "熊本県",
    "大分県",
    "宮崎県",
    "鹿児島県",
    "沖縄県",
];

const BROWSER_TABLE: &[(&str, &str)] = &[
    ("chrome", "Chrome"),
    ("edge", "Edge"),
    ("firefox", "Firefox"),
    ("safari", "Safari"),
];

const OS_TABLE: &[(&str, &str)] = &[
    ("windows", "Windows"),
    ("macos", "macOS"),
    ("mac os", "macOS"),
    ("macosx", "macOS"),
    ("ios", "iOS"),
    ("android", "Android"),
    ("linux", "Linux"),
];

const UNKNOWN_LABEL: &str = "—";

fn lookup<'a>(table: &[(&str, &'a str)], key: &str) -> Option<&'a str> {
    table
        .iter()
        .find_map(|(code, name)| key.eq_ignore_ascii_case(code).then_some(*name))
}

fn prefecture_name(trimmed: &str) -> Option<&'static str> {
    if !trimmed.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let number = trimmed.parse::<u32>().ok()?;
    PREFECTURE_TABLE
        .get(number.checked_sub(1)? as usize)
        .copied()
}

#[must_use]
pub fn country_name(code: &str) -> &str {
    lookup(COUNTRY_TABLE, code.trim()).unwrap_or(code)
}

#[must_use]
pub fn region_name(region: &str) -> &str {
    let trimmed = region.trim();
    if let Some(name) = prefecture_name(trimmed) {
        return name;
    }
    lookup(REGION_TABLE, trimmed).unwrap_or(region)
}

fn capitalize_first(value: &str) -> String {
    let mut chars = value.chars();
    chars.next().map_or_else(String::new, |first| {
        let mut out = String::with_capacity(value.len());
        for upper in first.to_uppercase() {
            out.push(upper);
        }
        out.push_str(chars.as_str());
        out
    })
}

fn truncate_with_ellipsis(value: &str, max_chars: usize) -> String {
    if value.chars().count() > max_chars {
        let kept: String = value.chars().take(max_chars).collect();
        format!("{kept}…")
    } else {
        value.to_string()
    }
}

fn mapped_label(table: &[(&str, &str)], value: &str) -> String {
    match lookup(table, &value.to_lowercase()) {
        Some(name) => name.to_string(),
        None => capitalize_first(value.trim()),
    }
}

#[must_use]
pub fn browser_label(browser: &str) -> String {
    truncate_with_ellipsis(&mapped_label(BROWSER_TABLE, browser), 32)
}

#[must_use]
pub fn os_label(os: &str) -> String {
    truncate_with_ellipsis(&mapped_label(OS_TABLE, os), 32)
}

fn present(value: Option<&str>) -> Option<&str> {
    match value {
        Some(text) if !text.trim().is_empty() => Some(text.trim()),
        _ => None,
    }
}

fn join_with(first: Option<String>, second: Option<String>, separator: &str) -> String {
    match (first, second) {
        (Some(first), Some(second)) => format!("{first}{separator}{second}"),
        (Some(only), None) | (None, Some(only)) => only,
        (None, None) => UNKNOWN_LABEL.to_string(),
    }
}

#[must_use]
pub fn format_place(country: Option<&str>, region: Option<&str>) -> String {
    join_with(
        present(region).map(|value| region_name(value).trim().to_string()),
        present(country).map(|code| country_name(code).trim().to_string()),
        ", ",
    )
}

#[must_use]
pub fn format_agent(browser: Option<&str>, os: Option<&str>) -> String {
    // 個別ではなく結合してから省略する。個別に省略すると長い端末名で
    // 「… / …」と省略記号が2つ並ぶため
    const MAX_CHARS: usize = 64;
    let combined = join_with(
        present(browser).map(|value| mapped_label(BROWSER_TABLE, value)),
        present(os).map(|value| mapped_label(OS_TABLE, value)),
        " / ",
    );
    if combined == UNKNOWN_LABEL {
        return combined;
    }
    truncate_with_ellipsis(&combined, MAX_CHARS)
}

const JST_OFFSET_HOURS: u32 = 9;
const HOURS_PER_DAY: u32 = 24;

const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[must_use]
pub const fn days_in_month(year: i32, month: u32) -> u32 {
    if month == 2 && is_leap_year(year) {
        return 29;
    }
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => 28,
    }
}

const fn next_day(year: i32, month: u32, day: u32) -> (i32, u32, u32) {
    if day < days_in_month(year, month) {
        return (year, month, day + 1);
    }
    if month < 12 {
        return (year, month + 1, 1);
    }
    (year + 1, 1, 1)
}

#[must_use]
pub const fn shift_utc_to_jst(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
) -> (i32, u32, u32, u32, u32) {
    let shifted = hour + JST_OFFSET_HOURS;
    if shifted < HOURS_PER_DAY {
        return (year, month, day, shifted, minute);
    }
    let (year, month, day) = next_day(year, month, day);
    (year, month, day, shifted - HOURS_PER_DAY, minute)
}

#[must_use]
pub fn format_ymd_hm(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> String {
    format!("{year:04}/{month:02}/{day:02} {hour:02}:{minute:02}")
}

const fn has_iso_separators(bytes: &[u8]) -> bool {
    bytes.len() >= 16
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && (bytes[10] == b'T' || bytes[10] == b' ')
        && bytes[13] == b':'
}

fn in_ymd_hm_range(month: u32, day: u32, hour: u32, minute: u32) -> bool {
    (1..=12).contains(&month)
        && (1..=31).contains(&day)
        && (0..=23).contains(&hour)
        && (0..=59).contains(&minute)
}

fn parse_ymd_hm(iso: &str) -> Option<(i32, u32, u32, u32, u32)> {
    if !has_iso_separators(iso.as_bytes()) {
        return None;
    }
    let year: i32 = iso[..4].parse().ok()?;
    let month: u32 = iso[5..7].parse().ok()?;
    let day: u32 = iso[8..10].parse().ok()?;
    let hour: u32 = iso[11..13].parse().ok()?;
    let minute: u32 = iso[14..16].parse().ok()?;
    if !in_ymd_hm_range(month, day, hour, minute) {
        return None;
    }
    Some((year, month, day, hour, minute))
}

#[must_use]
pub fn format_iso_utc_as_jst(iso: &str) -> Option<String> {
    let (year, month, day, hour, minute) = parse_ymd_hm(iso.trim())?;
    let (year, month, day, hour, minute) = shift_utc_to_jst(year, month, day, hour, minute);
    Some(format_ymd_hm(year, month, day, hour, minute))
}

#[must_use]
pub fn format_iso_utc_as_jst_with_zone(iso: &str) -> Option<String> {
    format_iso_utc_as_jst(iso).map(|body| format!("{body} JST"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_places_use_japanese_names() {
        assert_eq!(format_place(Some("JP"), Some("Tokyo")), "東京都, 日本");
        assert_eq!(format_place(Some("JP"), Some("Osaka")), "大阪府, 日本");
        assert_eq!(
            format_place(Some("US"), Some("California")),
            "California, アメリカ"
        );
    }

    #[test]
    fn numeric_region_codes_map_to_prefectures() {
        assert_eq!(format_place(Some("JP"), Some("13")), "東京都, 日本");
        assert_eq!(format_place(Some("JP"), Some("27")), "大阪府, 日本");
    }

    #[test]
    fn unknown_values_pass_through() {
        assert_eq!(format_place(Some("XX"), Some("Atlantis")), "Atlantis, XX");
        assert_eq!(format_place(Some("JP"), None), "日本");
        assert_eq!(format_place(None, Some("Tokyo")), "東京都");
        assert_eq!(format_place(None, None), "—");
        assert_eq!(format_place(Some(""), Some("  ")), "—");
    }

    #[test]
    fn agents_capitalize_and_use_official_os_names() {
        assert_eq!(
            format_agent(Some("chrome"), Some("windows")),
            "Chrome / Windows"
        );
        assert_eq!(format_agent(Some("safari"), Some("ios")), "Safari / iOS");
        assert_eq!(
            format_agent(Some("chrome"), Some("macos")),
            "Chrome / macOS"
        );
        assert_eq!(
            format_agent(Some("edge"), Some("android")),
            "Edge / Android"
        );
        assert_eq!(
            format_agent(Some("firefox"), Some("linux")),
            "Firefox / Linux"
        );
        assert_eq!(format_agent(Some("chrome"), None), "Chrome");
        assert_eq!(format_agent(None, Some("windows")), "Windows");
        assert_eq!(format_agent(None, None), "—");
    }

    #[test]
    fn unknown_browser_keeps_value_with_leading_capital() {
        assert_eq!(
            format_agent(Some("vivaldi"), Some("windows")),
            "Vivaldi / Windows"
        );
    }

    #[test]
    fn raw_ua_is_shortened_and_place_stays_intact() {
        let raw = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
        let agent = format_agent(Some(raw), Some(raw));
        assert!(agent.ends_with('…'));
        assert!(agent.chars().count() < raw.chars().count() * 2);
        assert_eq!(format_place(Some("JP"), Some("Tokyo")), "東京都, 日本");
    }

    #[test]
    fn long_agent_uses_single_ellipsis() {
        let raw = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
        let agent = format_agent(Some(raw), Some(raw));
        assert!(agent.ends_with('…'));
        assert_eq!(agent.chars().filter(|c| *c == '…').count(), 1);
    }

    #[test]
    fn single_labels_truncate_at_boundary() {
        let exact: String = "A".repeat(32);
        assert_eq!(browser_label(&exact), exact);
        assert_eq!(os_label(&exact), exact);
        let over: String = "A".repeat(33);
        assert_eq!(browser_label(&over), format!("{}…", "A".repeat(32)));
        assert_eq!(os_label(&over), format!("{}…", "A".repeat(32)));
    }

    #[test]
    fn single_labels_keep_known_names() {
        assert_eq!(browser_label("chrome"), "Chrome");
        assert_eq!(browser_label("Vivaldi"), "Vivaldi");
        assert_eq!(os_label("Mac OS"), "macOS");
        assert_eq!(os_label("macosx"), "macOS");
        assert_eq!(os_label("plan9"), "Plan9");
    }

    #[test]
    fn combined_agent_truncates_at_boundary() {
        // 64文字ちょうどは省略しない
        let browser = "B".repeat(30);
        let os = "C".repeat(31);
        assert_eq!(browser.len() + " / ".len() + os.len(), 64);
        assert_eq!(
            format_agent(Some(&browser), Some(&os)),
            format!("{browser} / {os}")
        );
        let long_os = "C".repeat(32);
        let agent = format_agent(Some(&browser), Some(&long_os));
        assert!(agent.ends_with('…'));
        assert_eq!(agent.chars().filter(|c| *c == '…').count(), 1);
    }

    #[test]
    fn iso_utc_converts_to_jst_with_year() {
        assert_eq!(
            format_iso_utc_as_jst("2026-10-31T07:38:00Z"),
            Some("2026/10/31 16:38".to_string())
        );
        assert_eq!(
            format_iso_utc_as_jst_with_zone("2026-10-31T07:38:00Z"),
            Some("2026/10/31 16:38 JST".to_string())
        );
    }

    #[test]
    fn iso_utc_rolls_over_past_midnight() {
        assert_eq!(
            format_iso_utc_as_jst("2026-12-31T16:00:00Z"),
            Some("2027/01/01 01:00".to_string())
        );
    }

    #[test]
    fn invalid_iso_returns_none() {
        assert_eq!(format_iso_utc_as_jst("not-a-date"), None);
        assert_eq!(format_iso_utc_as_jst(""), None);
    }

    #[test]
    fn unknown_os_keeps_value_with_leading_capital() {
        assert_eq!(format_agent(None, Some("plan9")), "Plan9");
        assert_eq!(
            format_agent(Some("vivaldi"), Some("haiku")),
            "Vivaldi / Haiku"
        );
    }

    #[test]
    fn os_aliases_use_official_name() {
        assert_eq!(
            format_agent(Some("chrome"), Some("Mac OS")),
            "Chrome / macOS"
        );
        assert_eq!(
            format_agent(Some("chrome"), Some("MACOSX")),
            "Chrome / macOS"
        );
    }

    #[test]
    fn browser_names_match_case_insensitively() {
        assert_eq!(
            format_agent(Some("CHROME"), Some("WINDOWS")),
            "Chrome / Windows"
        );
    }

    #[test]
    fn place_trims_and_matches_case_insensitively() {
        assert_eq!(format_place(Some(" jp "), Some(" tokyo ")), "東京都, 日本");
        assert_eq!(format_place(Some("JP"), Some(" 13 ")), "東京都, 日本");
    }

    #[test]
    fn blank_place_parts_become_placeholder() {
        assert_eq!(format_place(Some("  "), None), "—");
        assert_eq!(format_place(None, Some("  ")), "—");
        assert_eq!(format_place(Some("JP"), Some("  ")), "日本");
    }

    #[test]
    fn out_of_range_region_numbers_pass_through() {
        assert_eq!(format_place(Some("JP"), Some("0")), "0, 日本");
        assert_eq!(format_place(Some("JP"), Some("48")), "48, 日本");
    }

    #[test]
    fn prefecture_number_boundaries_resolve() {
        assert_eq!(format_place(Some("JP"), Some("1")), "北海道, 日本");
        assert_eq!(format_place(Some("JP"), Some("47")), "沖縄県, 日本");
    }

    #[test]
    fn iso_utc_rolls_over_month_end() {
        assert_eq!(
            format_iso_utc_as_jst("2026-01-31T15:00:00Z"),
            Some("2026/02/01 00:00".to_string())
        );
    }

    #[test]
    fn iso_utc_handles_february_boundaries() {
        assert_eq!(
            format_iso_utc_as_jst("2023-02-28T15:00:00Z"),
            Some("2023/03/01 00:00".to_string())
        );
        assert_eq!(
            format_iso_utc_as_jst("2024-02-28T15:00:00Z"),
            Some("2024/02/29 00:00".to_string())
        );
        assert_eq!(
            format_iso_utc_as_jst("2024-02-29T15:00:00Z"),
            Some("2024/03/01 00:00".to_string())
        );
    }

    #[test]
    fn iso_with_space_separator_converts_to_jst() {
        assert_eq!(
            format_iso_utc_as_jst("2026-10-31 07:38:00"),
            Some("2026/10/31 16:38".to_string())
        );
    }

    #[test]
    fn iso_out_of_range_returns_none() {
        assert_eq!(format_iso_utc_as_jst("2026-13-01T00:00:00Z"), None);
        assert_eq!(format_iso_utc_as_jst("2026-01-00T00:00:00Z"), None);
        assert_eq!(format_iso_utc_as_jst("2026-01-01T24:00:00Z"), None);
        assert_eq!(format_iso_utc_as_jst("2026-01-01T00:60:00Z"), None);
        assert_eq!(format_iso_utc_as_jst_with_zone("not-a-date"), None);
    }

    #[test]
    fn days_in_month_handles_leap_years() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2000, 2), 29);
        assert_eq!(days_in_month(1900, 2), 28);
    }

    #[test]
    fn days_in_month_returns_fixed_lengths() {
        assert_eq!(days_in_month(2026, 1), 31);
        assert_eq!(days_in_month(2026, 4), 30);
        assert_eq!(days_in_month(2026, 2), 28);
        assert_eq!(days_in_month(2026, 0), 28);
        assert_eq!(days_in_month(2026, 13), 28);
    }
}
