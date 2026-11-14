use axum::http::HeaderMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClientInfo {
    pub country: Option<String>,
    pub region: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

fn normalize_country(value: &str) -> Option<String> {
    let upper = value.trim().to_ascii_uppercase();
    (upper.len() == 2 && upper.bytes().all(|b| b.is_ascii_alphabetic())).then_some(upper)
}

fn normalize_region(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.chars().take(64).collect())
}

fn detect_browser(lower_ua: &str) -> Option<&'static str> {
    if lower_ua.contains("edg/") {
        Some("edge")
    } else if lower_ua.contains("firefox/") || lower_ua.contains("fxios/") {
        Some("firefox")
    } else if lower_ua.contains("chrome/") || lower_ua.contains("crios/") {
        Some("chrome")
    } else if lower_ua.contains("safari/") {
        Some("safari")
    } else {
        None
    }
}

fn detect_os(lower_ua: &str) -> Option<&'static str> {
    if lower_ua.contains("windows nt") {
        Some("windows")
    } else if lower_ua.contains("iphone") || lower_ua.contains("ipad") {
        Some("ios")
    } else if lower_ua.contains("android") {
        Some("android")
    } else if lower_ua.contains("mac os x") || lower_ua.contains("macintosh") {
        Some("macos")
    } else if lower_ua.contains("linux") {
        Some("linux")
    } else {
        None
    }
}

pub fn parse_user_agent(ua: &str) -> (Option<String>, Option<String>) {
    let lower = ua.to_ascii_lowercase();
    (
        detect_browser(&lower).map(str::to_owned),
        detect_os(&lower).map(str::to_owned),
    )
}

fn country_from_headers(headers: &HeaderMap) -> Option<String> {
    header_value(headers, "cf-ipcountry").and_then(normalize_country)
}

fn region_from_headers(headers: &HeaderMap) -> Option<String> {
    header_value(headers, "cf-region-code")
        .or_else(|| header_value(headers, "cf-region"))
        .and_then(normalize_region)
}

pub fn client_info_from_headers(headers: &HeaderMap) -> ClientInfo {
    let (browser, os) = header_value(headers, "user-agent").map_or((None, None), parse_user_agent);
    ClientInfo {
        country: country_from_headers(headers),
        region: region_from_headers(headers),
        browser,
        os,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn user_agent_yields_kinds_only() {
        for (ua, browser, os) in [
            (
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0 Safari/537.36",
                Some("chrome"),
                Some("windows"),
            ),
            (
                "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
                Some("safari"),
                Some("ios"),
            ),
            (
                "Mozilla/5.0 (X11; Linux x86_64; rv:132.0) Gecko/20100101 Firefox/132.0",
                Some("firefox"),
                Some("linux"),
            ),
            (
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0 Safari/537.36 Edg/131.0",
                Some("edge"),
                Some("windows"),
            ),
            ("", None, None),
            ("curl/8.0", None, None),
        ] {
            assert_eq!(
                parse_user_agent(ua),
                (browser.map(str::to_owned), os.map(str::to_owned)),
                "{ua}"
            );
        }
    }

    #[test]
    fn ios_and_android_apps_resolve_to_known_kinds() {
        assert_eq!(
            parse_user_agent(
                "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) CriOS/131.0 Mobile/15E148 Safari/604.1"
            ),
            (Some("chrome".to_owned()), Some("ios".to_owned())),
        );
        assert_eq!(
            parse_user_agent(
                "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) FxiOS/132.0 Mobile/15E148 Safari/605.1.15"
            ),
            (Some("firefox".to_owned()), Some("ios".to_owned())),
        );
        assert_eq!(
            parse_user_agent(
                "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 Chrome/131.0 Mobile Safari/537.36"
            ),
            (Some("chrome".to_owned()), Some("android".to_owned())),
        );
    }

    #[test]
    fn edge_wins_over_chrome_when_both_tokens_present() {
        assert_eq!(
            detect_browser("mozilla/5.0 chrome/131.0 safari/537.36 edg/131.0"),
            Some("edge")
        );
    }

    #[test]
    fn os_tokens_match_independently() {
        assert_eq!(detect_os("customagent/1.0 (macintosh)"), Some("macos"));
        assert_eq!(detect_os("something (mac os x 10_15)"), Some("macos"));
        assert_eq!(
            parse_user_agent("CustomAgent/1.0 (Macintosh)").1,
            Some("macos".to_owned()),
        );
    }

    #[test]
    fn country_accepts_two_letter_codes_only() {
        assert_eq!(normalize_country("jp"), Some("JP".to_string()));
        assert_eq!(normalize_country("JP "), Some("JP".to_string()));
        for bad in ["", "JPN", "J1", "1"] {
            assert_eq!(normalize_country(bad), None, "{bad}");
        }
    }

    #[test]
    fn region_is_trimmed_and_capped_at_64_chars() {
        assert_eq!(normalize_region("  Tokyo  "), Some("Tokyo".to_string()));
        assert_eq!(normalize_region("   "), None);
        let long = "あ".repeat(100);
        assert_eq!(normalize_region(&long).map(|s| s.chars().count()), Some(64));
    }

    #[test]
    fn headers_never_carry_raw_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("cf-connecting-ip", "203.0.113.10".parse().unwrap());
        headers.insert("x-forwarded-for", "203.0.113.10".parse().unwrap());
        headers.insert("cf-ipcountry", "JP".parse().unwrap());
        headers.insert(
            "user-agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/131.0"
                .parse()
                .unwrap(),
        );
        let info = client_info_from_headers(&headers);
        assert_eq!(info.country.as_deref(), Some("JP"));
        assert_eq!(info.browser.as_deref(), Some("chrome"));
        assert!(
            !format!("{info:?}").contains("203.0.113"),
            "IP が記録対象に含まれないこと"
        );
    }

    #[test]
    fn region_prefers_code_header_and_ignores_blank() {
        let mut headers = HeaderMap::new();
        headers.insert("cf-region-code", "Tokyo".parse().unwrap());
        headers.insert("cf-region", "Osaka".parse().unwrap());
        assert_eq!(
            client_info_from_headers(&headers).region.as_deref(),
            Some("Tokyo")
        );
        let mut headers = HeaderMap::new();
        headers.insert("cf-region", "Osaka".parse().unwrap());
        assert_eq!(
            client_info_from_headers(&headers).region.as_deref(),
            Some("Osaka")
        );
        let mut headers = HeaderMap::new();
        headers.insert("cf-region", "   ".parse().unwrap());
        assert_eq!(client_info_from_headers(&headers).region, None);
    }
}
