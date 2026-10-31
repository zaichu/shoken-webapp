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

fn prefecture_by_number(number: u32) -> Option<&'static str> {
    match number {
        1 => Some("北海道"),
        2 => Some("青森県"),
        3 => Some("岩手県"),
        4 => Some("宮城県"),
        5 => Some("秋田県"),
        6 => Some("山形県"),
        7 => Some("福島県"),
        8 => Some("茨城県"),
        9 => Some("栃木県"),
        10 => Some("群馬県"),
        11 => Some("埼玉県"),
        12 => Some("千葉県"),
        13 => Some("東京都"),
        14 => Some("神奈川県"),
        15 => Some("新潟県"),
        16 => Some("富山県"),
        17 => Some("石川県"),
        18 => Some("福井県"),
        19 => Some("山梨県"),
        20 => Some("長野県"),
        21 => Some("岐阜県"),
        22 => Some("静岡県"),
        23 => Some("愛知県"),
        24 => Some("三重県"),
        25 => Some("滋賀県"),
        26 => Some("京都府"),
        27 => Some("大阪府"),
        28 => Some("兵庫県"),
        29 => Some("奈良県"),
        30 => Some("和歌山県"),
        31 => Some("鳥取県"),
        32 => Some("島根県"),
        33 => Some("岡山県"),
        34 => Some("広島県"),
        35 => Some("山口県"),
        36 => Some("徳島県"),
        37 => Some("香川県"),
        38 => Some("愛媛県"),
        39 => Some("高知県"),
        40 => Some("福岡県"),
        41 => Some("佐賀県"),
        42 => Some("長崎県"),
        43 => Some("熊本県"),
        44 => Some("大分県"),
        45 => Some("宮崎県"),
        46 => Some("鹿児島県"),
        47 => Some("沖縄県"),
        _ => None,
    }
}

#[must_use]
pub fn country_name(code: &str) -> &str {
    let trimmed = code.trim();
    for (key, name) in COUNTRY_TABLE {
        if trimmed.eq_ignore_ascii_case(key) {
            return name;
        }
    }
    code
}

#[must_use]
pub fn region_name(region: &str) -> &str {
    let trimmed = region.trim();
    if !trimmed.is_empty()
        && trimmed.bytes().all(|b| b.is_ascii_digit())
        && let Ok(number) = trimmed.parse::<u32>()
        && let Some(name) = prefecture_by_number(number)
    {
        return name;
    }
    for (key, name) in REGION_TABLE {
        if trimmed.eq_ignore_ascii_case(key) {
            return name;
        }
    }
    region
}

fn capitalize_first(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let mut out = String::with_capacity(value.len());
            for upper in first.to_uppercase() {
                out.push(upper);
            }
            out.push_str(chars.as_str());
            out
        }
    }
}

#[must_use]
pub fn browser_label(browser: &str) -> String {
    let mut lower = String::with_capacity(browser.len());
    for c in browser.chars() {
        for l in c.to_lowercase() {
            lower.push(l);
        }
    }
    match lower.as_str() {
        "chrome" => "Chrome".to_string(),
        "edge" => "Edge".to_string(),
        "firefox" => "Firefox".to_string(),
        "safari" => "Safari".to_string(),
        _ => capitalize_first(browser.trim()),
    }
}

#[must_use]
pub fn os_label(os: &str) -> String {
    let mut lower = String::with_capacity(os.len());
    for c in os.chars() {
        for l in c.to_lowercase() {
            lower.push(l);
        }
    }
    match lower.as_str() {
        "windows" => "Windows".to_string(),
        "macos" | "mac os" | "macosx" => "macOS".to_string(),
        "ios" => "iOS".to_string(),
        "android" => "Android".to_string(),
        "linux" => "Linux".to_string(),
        _ => capitalize_first(os.trim()),
    }
}

fn present(value: Option<&str>) -> Option<&str> {
    match value {
        Some(text) if !text.trim().is_empty() => Some(text.trim()),
        _ => None,
    }
}

#[must_use]
pub fn format_place(country: Option<&str>, region: Option<&str>) -> String {
    match (present(country), present(region)) {
        (Some(country), Some(region)) => {
            format!("{}, {}", region_name(region), country_name(country))
        }
        (Some(country), None) => country_name(country).trim().to_string(),
        (None, Some(region)) => region_name(region).trim().to_string(),
        (None, None) => "—".to_string(),
    }
}

#[must_use]
pub fn format_agent(browser: Option<&str>, os: Option<&str>) -> String {
    match (present(browser), present(os)) {
        (Some(browser), Some(os)) => {
            format!("{} / {}", browser_label(browser), os_label(os))
        }
        (Some(browser), None) => browser_label(browser),
        (None, Some(os)) => os_label(os),
        (None, None) => "—".to_string(),
    }
}

#[must_use]
pub fn days_in_month(year: i32, month: u32) -> u32 {
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        _ => 28,
    }
}

#[must_use]
pub fn shift_utc_to_jst(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
) -> (i32, u32, u32, u32, u32) {
    let mut year = year;
    let mut month = month;
    let mut day = day;
    let mut hour = hour + 9;
    if hour >= 24 {
        hour -= 24;
        day += 1;
        if day > days_in_month(year, month) {
            day = 1;
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }
    }
    (year, month, day, hour, minute)
}

#[must_use]
pub fn format_ymd_hm(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> String {
    format!("{year:04}/{month:02}/{day:02} {hour:02}:{minute:02}")
}

fn parse_ymd_hm(iso: &str) -> Option<(i32, u32, u32, u32, u32)> {
    let bytes = iso.as_bytes();
    if bytes.len() < 16 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let sep = bytes[10];
    if sep != b'T' && sep != b' ' {
        return None;
    }
    if bytes[13] != b':' {
        return None;
    }
    let year: i32 = iso[..4].parse().ok()?;
    let month: u32 = iso[5..7].parse().ok()?;
    let day: u32 = iso[8..10].parse().ok()?;
    let hour: u32 = iso[11..13].parse().ok()?;
    let minute: u32 = iso[14..16].parse().ok()?;
    if !(1..=12).contains(&month) || day == 0 || day > 31 || hour > 23 || minute > 59 {
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
}
