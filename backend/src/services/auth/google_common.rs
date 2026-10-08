//! Google OAuth のターゲット共通ロジック。
//! 乱数生成・PKCE・認可 URL 構築・tokeninfo クレーム検証をここに置き、
//! HTTP 経路のみがターゲット別実装（google_native / google_worker）を持つ。

use sha2::{Digest, Sha256};

use crate::errors::ApiError;
use crate::models::user::GoogleUserInfo;

pub const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
pub const GOOGLE_ISSUER: &str = "https://accounts.google.com";
pub const GOOGLE_TOKENINFO_URL: &str = "https://oauth2.googleapis.com/tokeninfo";

/// RFC 3986 unreserved 以外を %XX にする最小のパーセントエンコード。
/// tokeninfo のクエリと token 交換のフォームボディで使う
pub fn url_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for &b in value.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push('%');
                out.push_str(&format!("{b:02X}"));
            }
        }
    }
    out
}

/// base64url エンコード（パディングなし）。PKCE の code_challenge 用
pub fn base64url_nopad(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(bytes.len() * 4 / 3);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(TABLE[(n >> 18) as usize & 0x3f] as char);
        out.push(TABLE[(n >> 12) as usize & 0x3f] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(n >> 6) as usize & 0x3f] as char);
        }
        if chunk.len() > 2 {
            out.push(TABLE[n as usize & 0x3f] as char);
        }
    }
    out
}

/// state/nonce 用のランダム値（UUID v4 の hex 32 文字）
pub fn new_oauth_random() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

/// PKCE の (verifier, S256 challenge)。verifier は 64 文字の hex で
/// RFC 7636 の 43..=128 文字・unreserved 制約を満たす
pub fn new_pkce_pair() -> (String, String) {
    let verifier = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let challenge = base64url_nopad(&Sha256::digest(verifier.as_bytes()));
    (verifier, challenge)
}

/// Google 認可エンドポイントへのリダイレクト URL を組み立てる。
/// scope は native 側と同じ `openid email profile`、nonce は extra param
pub fn build_authorize_url(
    client_id: &str,
    redirect_uri: &str,
    state: &str,
    nonce: &str,
    code_challenge: &str,
) -> String {
    let params = [
        ("client_id", client_id),
        ("redirect_uri", redirect_uri),
        ("response_type", "code"),
        ("scope", "openid email profile"),
        ("state", state),
        ("nonce", nonce),
        ("code_challenge", code_challenge),
        ("code_challenge_method", "S256"),
    ];
    let query = params
        .iter()
        .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    format!("{GOOGLE_AUTH_URL}?{query}")
}

/// tokeninfo の exp/iat フィールドは文字列数値だが、防御的に数値も受け付ける
fn json_i64(value: &serde_json::Value, field: &str) -> Option<i64> {
    match value.get(field)? {
        serde_json::Value::String(s) => s.parse().ok(),
        serde_json::Value::Number(n) => n.as_i64(),
        _ => None,
    }
}

/// Google tokeninfo レスポンスのクレームを検証してユーザー情報を取り出す。
///
/// tokeninfo エンドポイントが署名検証を肩代わりするため、ここでは
/// iss / aud / exp / nonce / email・sub の存在を確認する（native 側の検証と対応）。
/// `now_unix` は検証時刻（UNIX 秒）を外から渡し、時計依存を排除する
pub fn validate_tokeninfo_claims(
    claims: &serde_json::Value,
    client_id: &str,
    expected_nonce: &str,
    now_unix: i64,
) -> Result<GoogleUserInfo, ApiError> {
    let invalid = || ApiError::OAuth("Google IDトークン検証エラー".to_string());
    let get_str = |field: &str| claims.get(field).and_then(|v| v.as_str());

    match get_str("iss") {
        Some("https://accounts.google.com") | Some("accounts.google.com") => {}
        _ => return Err(invalid()),
    }
    if get_str("aud") != Some(client_id) {
        return Err(invalid());
    }
    match json_i64(claims, "exp") {
        Some(exp) if exp > now_unix => {}
        _ => return Err(invalid()),
    }
    if get_str("nonce") != Some(expected_nonce) {
        return Err(invalid());
    }

    let sub = get_str("sub")
        .filter(|s| !s.is_empty())
        .ok_or_else(invalid)?;
    let email = get_str("email")
        .filter(|s| !s.is_empty())
        .ok_or_else(invalid)?;

    Ok(GoogleUserInfo {
        sub: sub.to_string(),
        email: email.to_string(),
        name: get_str("name").map(str::to_string),
        picture: get_str("picture").map(str::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLIENT_ID: &str = "test-client-id";
    const NONCE: &str = "test-nonce";
    const NOW: i64 = 1_700_000_000;

    fn valid_tokeninfo() -> serde_json::Value {
        serde_json::json!({
            "iss": "https://accounts.google.com",
            "aud": CLIENT_ID,
            "sub": "test-subject",
            "email": "oidc@example.com",
            "email_verified": "true",
            "exp": (NOW + 3600).to_string(),
            "iat": NOW.to_string(),
            "nonce": NONCE,
            "name": "テスト利用者",
            "picture": "https://example.com/avatar.png"
        })
    }

    #[test]
    fn test_base64url_nopad() {
        assert_eq!(base64url_nopad(b""), "");
        assert_eq!(base64url_nopad(b"f"), "Zg");
        assert_eq!(base64url_nopad(b"fo"), "Zm8");
        assert_eq!(base64url_nopad(b"foo"), "Zm9v");
        assert_eq!(base64url_nopad(b"foob"), "Zm9vYg");
        assert_eq!(base64url_nopad(b"fooba"), "Zm9vYmE");
        assert_eq!(base64url_nopad(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64url_nopad(&[0xff, 0xfe, 0xfd]), "__79");
    }

    #[test]
    fn test_pkce_pair() {
        let (verifier, challenge) = new_pkce_pair();
        assert_eq!(verifier.len(), 64);
        assert!(verifier.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(challenge.len(), 43);
        assert!(
            challenge
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        assert_eq!(
            challenge,
            base64url_nopad(&Sha256::digest(verifier.as_bytes()))
        );
    }

    #[test]
    fn test_build_authorize_url() {
        let url = build_authorize_url(
            "client-id",
            "https://api.example.com/api/v1/oauth/google/callback",
            "state-1",
            "nonce-1",
            "challenge-1",
        );
        assert!(url.starts_with(&format!("{GOOGLE_AUTH_URL}?")));
        for expected in [
            "client_id=client-id",
            "redirect_uri=https%3A%2F%2Fapi.example.com%2Fapi%2Fv1%2Foauth%2Fgoogle%2Fcallback",
            "response_type=code",
            "scope=openid%20email%20profile",
            "state=state-1",
            "nonce=nonce-1",
            "code_challenge=challenge-1",
            "code_challenge_method=S256",
        ] {
            assert!(url.contains(expected), "{expected} not in {url}");
        }
    }

    #[test]
    fn test_validate_tokeninfo_accepts_valid() {
        let info = validate_tokeninfo_claims(&valid_tokeninfo(), CLIENT_ID, NONCE, NOW).unwrap();
        assert_eq!(info.sub, "test-subject");
        assert_eq!(info.email, "oidc@example.com");
        assert_eq!(info.name.as_deref(), Some("テスト利用者"));
        assert_eq!(
            info.picture.as_deref(),
            Some("https://example.com/avatar.png")
        );
        // iss の短縮形も許容
        let mut claims = valid_tokeninfo();
        claims["iss"] = serde_json::json!("accounts.google.com");
        assert!(validate_tokeninfo_claims(&claims, CLIENT_ID, NONCE, NOW).is_ok());
        // exp が数値でも受理（防御的）
        let mut claims = valid_tokeninfo();
        claims["exp"] = serde_json::json!(NOW + 60);
        assert!(validate_tokeninfo_claims(&claims, CLIENT_ID, NONCE, NOW).is_ok());
        // name/picture は任意
        let mut claims = valid_tokeninfo();
        claims.as_object_mut().unwrap().remove("name");
        claims.as_object_mut().unwrap().remove("picture");
        let info = validate_tokeninfo_claims(&claims, CLIENT_ID, NONCE, NOW).unwrap();
        assert!(info.name.is_none() && info.picture.is_none());
    }

    #[test]
    fn test_validate_tokeninfo_rejects_invalid() {
        let cases: Vec<(&str, serde_json::Value)> = vec![
            ("iss", serde_json::json!("https://attacker.example.com")),
            ("aud", serde_json::json!("other-client")),
            ("exp", serde_json::json!((NOW - 60).to_string())),
            ("nonce", serde_json::json!("unsolicited-nonce")),
            ("email", serde_json::Value::Null),
            ("sub", serde_json::Value::Null),
        ];
        for (field, value) in cases {
            let mut claims = valid_tokeninfo();
            claims[field] = value;
            assert!(
                validate_tokeninfo_claims(&claims, CLIENT_ID, NONCE, NOW).is_err(),
                "{field} should be rejected"
            );
        }
        // 必須フィールド欠落
        for field in ["iss", "aud", "exp", "nonce", "email", "sub"] {
            let mut claims = valid_tokeninfo();
            claims.as_object_mut().unwrap().remove(field);
            assert!(
                validate_tokeninfo_claims(&claims, CLIENT_ID, NONCE, NOW).is_err(),
                "missing {field} should be rejected"
            );
        }
        // exp は境界で拒否（now と同値は期限切れ扱い）
        let mut claims = valid_tokeninfo();
        claims["exp"] = serde_json::json!(NOW.to_string());
        assert!(validate_tokeninfo_claims(&claims, CLIENT_ID, NONCE, NOW).is_err());
    }
}
