//! Base64 とパーセントエンコードの単一実装。
//! `login_notify` の標準 Base64 と PKCE 用 base64url、`google_common` の
//! 手書きパーセントエンコードをここに集約し、呼び出し側は薄い参照だけにする。

const STANDARD_TABLE: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const URL_SAFE_TABLE: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn encode_with(input: &[u8], table: &[u8; 64], padded: bool) -> String {
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let n = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        out.push(table[((n >> 18) & 0x3F) as usize] as char);
        out.push(table[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(table[((n >> 6) & 0x3F) as usize] as char);
        } else if padded {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(table[(n & 0x3F) as usize] as char);
        } else if padded {
            out.push('=');
        }
    }
    out
}

/// 標準 Base64（パディングあり）。SMTP 認証・件名エンコード用
#[must_use]
pub fn base64_encode(input: &[u8]) -> String {
    encode_with(input, STANDARD_TABLE, true)
}

/// base64url（パディングなし）。PKCE の code_challenge 用
#[must_use]
pub fn base64url_nopad(input: &[u8]) -> String {
    encode_with(input, URL_SAFE_TABLE, false)
}

/// RFC 3986 unreserved 以外を %XX にする最小のパーセントエンコード。
/// tokeninfo のクエリと token 交換のフォームボディで使う
#[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_base64_matches_rfc_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64_encode("あ".as_bytes()), "44GC");
    }

    #[test]
    fn url_safe_base64_matches_existing_vectors() {
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
    fn url_encode_keeps_unreserved_and_escapes_rest() {
        assert_eq!(url_encode("abcXYZ019-_.~"), "abcXYZ019-_.~");
        assert_eq!(
            url_encode("https://api.example.com/api/v1/oauth/google/callback"),
            "https%3A%2F%2Fapi.example.com%2Fapi%2Fv1%2Foauth%2Fgoogle%2Fcallback"
        );
        assert_eq!(
            url_encode("openid email profile"),
            "openid%20email%20profile"
        );
        assert_eq!(url_encode("あ"), "%E3%81%82");
        assert_eq!(url_encode(""), "");
    }
}
