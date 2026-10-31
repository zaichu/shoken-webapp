use crate::db::{Bind, Db, DbError};
use crate::models::user::User;
use crate::services::login_event::{ClientInfo, LoginOutcome};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared::value::UserId;
use utoipa::ToSchema;

pub const MAX_SENDS_PER_HOUR: i64 = 3;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NotificationSettingsResponse {
    pub login_notify_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateNotificationSettingsRequest {
    pub login_notify_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LogoutTokenInfoResponse {
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LogoutWithTokenRequest {
    pub token: String,
}

pub fn header_value_safe(value: &str) -> bool {
    !value.is_empty() && !value.bytes().any(|b| b == b'\r' || b == b'\n')
}

pub fn email_safe(email: &str) -> bool {
    if !header_value_safe(email) || email.len() > 254 || email.contains(' ') {
        return false;
    }
    let mut parts = email.split('@');
    matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(_), Some(_), None)
    )
}

const BASE64_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let mut n: u32 = 0;
        for (i, &b) in chunk.iter().enumerate() {
            n |= (b as u32) << (16 - 8 * i);
        }
        let pad = 3 - chunk.len();
        for i in 0..4 - pad {
            let shift = 18 - 6 * i;
            out.push(BASE64_TABLE[((n >> shift) & 0x3F) as usize] as char);
        }
        for _ in 0..pad {
            out.push('=');
        }
    }
    out
}

fn encode_subject(subject: &str) -> String {
    if subject.is_ascii() {
        subject.to_string()
    } else {
        format!("=?UTF-8?B?{}?=", base64_encode(subject.as_bytes()))
    }
}

pub fn build_message(from: &str, to: &str, subject: &str, body: &str) -> Result<String, String> {
    if !email_safe(from) || !email_safe(to) {
        return Err("宛先の形式が不正です".to_string());
    }
    if !header_value_safe(subject) {
        return Err("件名の形式が不正です".to_string());
    }
    let normalized_body = body
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    Ok(format!(
        "From: {from}\r\nTo: {to}\r\nSubject: {}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: 8bit\r\n\r\n{normalized_body}\r\n",
        encode_subject(subject)
    ))
}

pub fn reply_code(line: &str) -> Option<u16> {
    line.get(..3)?.parse().ok()
}

pub fn login_notification_subject() -> &'static str {
    "新しい端末からのログインがありました"
}

pub fn login_notification_body(
    occurred_at: &str,
    place: &str,
    agent: &str,
    logout_url: &str,
) -> String {
    format!(
        "新しい端末または新しい場所からのログインがありました。\n\n日時: {occurred_at}\n場所: {place}\nブラウザ・OS: {agent}\n\n心当たりがない場合は、次のリンクからすべての端末をログアウトしてください(24時間以内・1回限り有効です)。\n{logout_url}\n\nオフでも、ログインの記録は残ります。"
    )
}

pub fn logout_url(frontend_url: &str, token: &str) -> String {
    format!(
        "{}/settings?notify_token={}",
        frontend_url.trim_end_matches('/'),
        token
    )
}

pub fn token_hash(plaintext: &str) -> Vec<u8> {
    Sha256::digest(plaintext.as_bytes()).to_vec()
}

fn new_plaintext_token() -> String {
    let a = uuid::Uuid::new_v4().simple().to_string();
    let b = uuid::Uuid::new_v4().simple().to_string();
    format!("{a}{b}")
}

pub async fn get_setting(pool: &Db, user_id: UserId) -> Result<bool, DbError> {
    crate::db::query_scalar::<bool>(
        "SELECT login_notify_enabled FROM users WHERE id = $1",
        vec![Bind::from(user_id)],
    )
    .fetch_one(pool)
    .await
}

pub async fn set_setting(pool: &Db, user_id: UserId, enabled: bool) -> Result<bool, DbError> {
    crate::db::query_scalar::<bool>(
        "UPDATE users SET login_notify_enabled = $2, updated_at = NOW() WHERE id = $1 RETURNING login_notify_enabled",
        vec![Bind::from(user_id), Bind::from(enabled)],
    )
    .fetch_one(pool)
    .await
}

pub async fn recent_send_count(pool: &Db, user_id: UserId) -> Result<i64, DbError> {
    crate::db::query_scalar::<i64>(
        "SELECT COUNT(*) FROM login_notify_tokens WHERE user_id = $1 AND created_at > NOW() - INTERVAL '1 hour'",
        vec![Bind::from(user_id)],
    )
    .fetch_one(pool)
    .await
}

pub async fn issue_token(pool: &Db, user_id: UserId) -> Result<String, DbError> {
    let plaintext = new_plaintext_token();
    crate::db::query(
        "INSERT INTO login_notify_tokens (user_id, token_hash) VALUES ($1, $2)",
        vec![Bind::from(user_id), Bind::Bytes(token_hash(&plaintext))],
    )
    .execute(pool)
    .await?;
    Ok(plaintext)
}

pub async fn peek_token(
    pool: &Db,
    plaintext: &str,
) -> Result<Option<(UserId, DateTime<Utc>)>, DbError> {
    crate::db::query(
        "SELECT user_id, expires_at FROM login_notify_tokens WHERE token_hash = $1 AND used_at IS NULL AND expires_at > NOW()",
        vec![Bind::Bytes(token_hash(plaintext))],
    )
    .fetch_optional::<(UserId, DateTime<Utc>), _>(pool)
    .await
}

pub async fn consume_token(pool: &Db, plaintext: &str) -> Result<Option<UserId>, DbError> {
    crate::db::query_scalar::<UserId>(
        "UPDATE login_notify_tokens SET used_at = NOW() WHERE token_hash = $1 AND used_at IS NULL AND expires_at > NOW() RETURNING user_id",
        vec![Bind::Bytes(token_hash(plaintext))],
    )
    .fetch_optional(pool)
    .await
}

#[allow(async_fn_in_trait)]
pub trait MailSender {
    async fn send_raw(&self, from: &str, to: &str, message: &str) -> Result<(), String>;
}

pub struct NoopSender;

impl MailSender for NoopSender {
    async fn send_raw(&self, _from: &str, _to: &str, _message: &str) -> Result<(), String> {
        Ok(())
    }
}

pub struct FailingSender;

impl MailSender for FailingSender {
    async fn send_raw(&self, _from: &str, _to: &str, _message: &str) -> Result<(), String> {
        Err("送信に失敗しました".to_string())
    }
}

#[cfg(target_arch = "wasm32")]
pub struct WorkerSmtpSender {
    pub smtp_user: String,
    pub smtp_pass: String,
}

#[cfg(target_arch = "wasm32")]
impl MailSender for WorkerSmtpSender {
    async fn send_raw(&self, from: &str, to: &str, message: &str) -> Result<(), String> {
        send_via_gmail(&self.smtp_user, &self.smtp_pass, from, to, message).await
    }
}

#[cfg(target_arch = "wasm32")]
async fn send_via_gmail(
    smtp_user: &str,
    smtp_pass: &str,
    from: &str,
    to: &str,
    message: &str,
) -> Result<(), String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
    let socket = worker::Socket::builder()
        .secure_transport(worker::SecureTransport::On)
        .connect("smtp.gmail.com", 465)
        .map_err(|e| format!("SMTP connect 失敗: {e:?}"))?;
    let mut stream = tokio::io::BufReader::new(socket);
    async fn read_reply(
        stream: &mut tokio::io::BufReader<worker::Socket>,
    ) -> Result<String, String> {
        let mut last = String::new();
        loop {
            let mut buf = String::new();
            let n = stream
                .read_line(&mut buf)
                .await
                .map_err(|e| format!("SMTP read 失敗: {e}"))?;
            if n == 0 {
                return Err("SMTP 接続が切断されました".to_string());
            }
            last = buf.trim_end().to_string();
            if last.len() >= 4 && last.as_bytes()[3] == b' ' {
                break;
            }
        }
        reply_code(&last)
            .filter(|code| (200..400).contains(code))
            .is_some()
            .then_some(last)
            .ok_or_else(|| "SMTP 応答が不正です".to_string())
    }
    async fn write_cmd(
        stream: &mut tokio::io::BufReader<worker::Socket>,
        cmd: &str,
    ) -> Result<(), String> {
        stream
            .get_mut()
            .write_all(cmd.as_bytes())
            .await
            .map_err(|e| format!("SMTP write 失敗: {e}"))?;
        stream
            .get_mut()
            .flush()
            .await
            .map_err(|e| format!("SMTP write 失敗: {e}"))?;
        Ok(())
    }
    let greeting = read_reply(&mut stream).await?;
    if !greeting.starts_with("220") {
        return Err("SMTP 応答が不正です".to_string());
    }
    write_cmd(&mut stream, "EHLO localhost\r\n").await?;
    let ehlo = read_reply(&mut stream).await?;
    if !ehlo.starts_with("250") {
        return Err("SMTP 応答が不正です".to_string());
    }
    write_cmd(&mut stream, "AUTH LOGIN\r\n").await?;
    let auth = read_reply(&mut stream).await?;
    if !auth.starts_with("334") {
        return Err("SMTP 応答が不正です".to_string());
    }
    write_cmd(
        &mut stream,
        &format!("{}\r\n", base64_encode(smtp_user.as_bytes())),
    )
    .await?;
    let user_reply = read_reply(&mut stream).await?;
    if !user_reply.starts_with("334") {
        return Err("SMTP 応答が不正です".to_string());
    }
    write_cmd(
        &mut stream,
        &format!("{}\r\n", base64_encode(smtp_pass.as_bytes())),
    )
    .await?;
    let pass_reply = read_reply(&mut stream).await?;
    if !pass_reply.starts_with("235") {
        return Err("SMTP 応答が不正です".to_string());
    }
    write_cmd(&mut stream, &format!("MAIL FROM:<{from}>\r\n")).await?;
    let mail = read_reply(&mut stream).await?;
    if !mail.starts_with("250") {
        return Err("SMTP 応答が不正です".to_string());
    }
    write_cmd(&mut stream, &format!("RCPT TO:<{to}>\r\n")).await?;
    let rcpt = read_reply(&mut stream).await?;
    if !(rcpt.starts_with("250") || rcpt.starts_with("251")) {
        return Err("SMTP 応答が不正です".to_string());
    }
    write_cmd(&mut stream, "DATA\r\n").await?;
    let data = read_reply(&mut stream).await?;
    if !data.starts_with("354") {
        return Err("SMTP 応答が不正です".to_string());
    }
    write_cmd(&mut stream, &format!("{message}\r\n.\r\n")).await?;
    let sent = read_reply(&mut stream).await?;
    if !sent.starts_with("250") {
        return Err("SMTP 応答が不正です".to_string());
    }
    let _ = write_cmd(&mut stream, "QUIT\r\n").await;
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn display_place(client: &ClientInfo) -> String {
    match (client.country.as_deref(), client.region.as_deref()) {
        (Some(country), Some(region)) if !region.is_empty() => format!("{country} {region}"),
        (Some(country), _) => country.to_string(),
        (None, Some(region)) if !region.is_empty() => region.to_string(),
        _ => "—".to_string(),
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn display_agent(client: &ClientInfo) -> String {
    match (client.browser.as_deref(), client.os.as_deref()) {
        (Some(browser), Some(os)) => format!("{browser} / {os}"),
        (Some(browser), None) => browser.to_string(),
        (None, Some(os)) => os.to_string(),
        _ => "—".to_string(),
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn maybe_notify_on_login(
    pool: &Db,
    gmail_user: Option<&str>,
    gmail_pass: Option<&str>,
    frontend_url: &str,
    user: &User,
    outcome: LoginOutcome,
    client: &ClientInfo,
) {
    if outcome == LoginOutcome::Success {
        return;
    }
    let enabled = get_setting(pool, user.id).await.unwrap_or(true);
    if !enabled {
        return;
    }
    if !email_safe(&user.email) {
        worker::console_error!("login_notify skip: 宛先の形式が不正です");
        return;
    }
    let (Some(smtp_user), Some(smtp_pass)) = (gmail_user, gmail_pass) else {
        worker::console_log!("login_notify skip: 送信設定がありません");
        return;
    };
    match recent_send_count(pool, user.id).await {
        Ok(count) if count >= MAX_SENDS_PER_HOUR => {
            worker::console_log!("login_notify skip: 頻度の上限に達しました");
            return;
        }
        Err(e) => {
            worker::console_error!("login_notify skip: 頻度の確認に失敗: {e}");
            return;
        }
        _ => {}
    }
    let token = match issue_token(pool, user.id).await {
        Ok(token) => token,
        Err(e) => {
            worker::console_error!("login_notify skip: トークンの発行に失敗: {e}");
            return;
        }
    };
    let occurred = chrono::Utc::now().format("%Y/%m/%d %H:%M UTC").to_string();
    let body = login_notification_body(
        &occurred,
        &display_place(client),
        &display_agent(client),
        &logout_url(frontend_url, &token),
    );
    let message = match build_message(smtp_user, &user.email, login_notification_subject(), &body) {
        Ok(message) => message,
        Err(e) => {
            worker::console_error!("login_notify skip: 送信内容の組み立てに失敗: {e}");
            return;
        }
    };
    let sender = WorkerSmtpSender {
        smtp_user: smtp_user.to_string(),
        smtp_pass: smtp_pass.to_string(),
    };
    match sender.send_raw(smtp_user, &user.email, &message).await {
        Ok(()) => worker::console_log!("login_notify sent"),
        Err(e) => worker::console_error!("login_notify failed: {e}"),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn maybe_notify_on_login(
    _pool: &Db,
    _gmail_user: Option<&str>,
    _gmail_pass: Option<&str>,
    _frontend_url: &str,
    _user: &User,
    _outcome: LoginOutcome,
    _client: &ClientInfo,
) {
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_header_injection() {
        for bad in ["a\r\nBcc: x@example.com", "a\nBcc: x", "\r", "\n"] {
            assert!(!header_value_safe(bad), "{bad:?}");
            assert!(!email_safe(&format!("{bad}@example.com")));
        }
        assert!(email_safe("user@example.com"));
        assert!(!email_safe("not-an-email"));
        assert!(!email_safe("a@b@c.com"));
        assert!(!email_safe("a b@example.com"));
        assert_eq!(
            build_message("a@example.com", "b@example.com", "bad\r\nsubject", "hi").unwrap_err(),
            "件名の形式が不正です"
        );
        assert_eq!(
            build_message("a@example.com", "evil\r\nTo: x", "hi", "body").unwrap_err(),
            "宛先の形式が不正です"
        );
    }

    #[test]
    fn builds_ascii_and_multibyte_subjects() {
        let ascii = build_message("a@example.com", "b@example.com", "hello", "hi").unwrap();
        assert!(ascii.contains("Subject: hello\r\n"));
        let jp = build_message("a@example.com", "b@example.com", "あいう", "本文").unwrap();
        assert!(jp.contains("Subject: =?UTF-8?B?"));
        assert!(jp.contains("Content-Type: text/plain; charset=utf-8"));
        assert!(!jp.contains("宛先"));
    }

    #[test]
    fn reply_codes_parse_three_digits_only() {
        assert_eq!(reply_code("220 smtp ready"), Some(220));
        assert_eq!(reply_code("235 ok"), Some(235));
        assert_eq!(reply_code("22"), None);
        assert_eq!(reply_code("abc"), None);
    }

    #[test]
    fn token_hash_is_stable_and_hides_plaintext() {
        let hash = token_hash("abcdef");
        assert_eq!(hash, token_hash("abcdef"));
        assert_ne!(hash, token_hash("abcdeg"));
        assert!(!hex::encode(&hash).contains("abcdef"));
    }

    #[test]
    fn logout_link_points_to_settings_with_token() {
        let url = logout_url("https://app.example/", "tok123");
        assert_eq!(url, "https://app.example/settings?notify_token=tok123");
    }

    #[test]
    fn notification_body_contains_expected_parts() {
        let body = login_notification_body(
            "2026/10/31 10:00",
            "JP Tokyo",
            "chrome / windows",
            "https://app/settings?notify_token=x",
        );
        for part in [
            "2026/10/31 10:00",
            "JP Tokyo",
            "chrome / windows",
            "https://app/settings?notify_token=x",
            "24時間",
            "1回限り",
        ] {
            assert!(body.contains(part), "{part}");
        }
    }

    #[test]
    fn display_helpers_cover_missing_parts() {
        let full = ClientInfo {
            country: Some("JP".to_string()),
            region: Some("Tokyo".to_string()),
            browser: Some("chrome".to_string()),
            os: Some("windows".to_string()),
        };
        assert_eq!(display_place(&full), "JP Tokyo");
        assert_eq!(display_agent(&full), "chrome / windows");
        assert_eq!(display_place(&ClientInfo::default()), "—");
        assert_eq!(display_agent(&ClientInfo::default()), "—");
    }

    #[tokio::test]
    async fn noop_sender_accepts_valid_message() {
        let message = build_message("a@example.com", "b@example.com", "hi", "body").unwrap();
        NoopSender
            .send_raw("a@example.com", "b@example.com", &message)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn failing_sender_reports_error_without_panic() {
        let message = build_message("a@example.com", "b@example.com", "hi", "body").unwrap();
        assert!(
            FailingSender
                .send_raw("a@example.com", "b@example.com", &message)
                .await
                .is_err()
        );
    }
}
