use super::*;

#[test]
fn allows_google_image_host_only() {
    assert_eq!(
        allowed_avatar_url(Some("https://lh3.googleusercontent.com/a-/avatar")).as_deref(),
        Some("https://lh3.googleusercontent.com/a-/avatar")
    );
    assert_eq!(
        allowed_avatar_url(Some("https://a.b.googleusercontent.com/x")).as_deref(),
        Some("https://a.b.googleusercontent.com/x")
    );
    for bad in [
        None,
        Some(""),
        Some("https://example.com/avatar.png"),
        Some("http://lh3.googleusercontent.com/a"),
        Some("not-a-url"),
        Some("https://googleusercontent.com.evil.example.com/a"),
        Some("https://evilgoogleusercontent.com/a"),
    ] {
        assert!(allowed_avatar_url(bad).is_none(), "{bad:?}");
    }
}

#[test]
fn initials_falls_back_without_image() {
    assert_eq!(avatar_initials(Some("Taro Yamada"), None), "TY");
    assert_eq!(avatar_initials(None, Some("user@example.com")), "US");
    assert_eq!(avatar_initials(None, None), "U");
}
