---
name: oauth-auth
description: |
  Google OAuth 認証とセッション管理の実装パターン。
  CSRF対策、Cookie設定、セッション検証を含む。
  Use when: 認証フロー修正、OAuth実装、セッション管理、Cookie設定を依頼された時。
---

# OAuth/認証 実装ガイド

`oauth2`/`openidconnect` クレートは wasm 非対応のため使わない。現行実装は
`services/auth/google_common.rs`（認可 URL・state/nonce・PKCE・tokeninfo クレーム検証）と
`services/auth/google_worker.rs`（`worker::Fetch` でトークン交換・tokeninfo 検証）、
Cookie・リダイレクトは `handlers/v1/auth/oauth.rs`。

## 目的

- OAuth 2.0 フロー（Google）を安全に実装する
- CSRF/state 検証を正しく行う
- Cookie 設定を環境に応じて適切に行う
- セッション管理を安全に実装する

## 適用場面

- OAuth 認証フローの新規実装・修正
- セッション管理の実装
- Cookie 設定の調整
- 認証関連のセキュリティ修正

## OAuth フロー

### 1. 認証開始（`/api/v1/oauth/google/authorize`）

`begin_google_auth` が認可 URL・state・nonce・PKCE verifier を一式生成する。
OAuth クライアントは起動時に `create_oauth_client` で構築して `state.google_oauth` に保持する。

```rust
pub async fn google_auth(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), ApiError> {
    let client = google_oauth_client(&state)?;

    let flow = auth_service::begin_google_auth(client);

    // state / PKCE verifier / nonce を短命（10分）Cookie に保存
    let is_secure = state.config.secure_cookie;
    let jar = jar
        .add(build_state_cookie(&flow.state, is_secure))
        .add(build_oauth_cookie(OAUTH_PKCE_VERIFIER_COOKIE_NAME, &flow.pkce_verifier, is_secure))
        .add(build_oauth_cookie(OAUTH_NONCE_COOKIE_NAME, &flow.nonce, is_secure));

    Ok((jar, Redirect::to(flow.authorize_url.as_str())))
}
```

### 2. コールバック（`/api/v1/oauth/google/callback`）

state を照合し、oauth 系 Cookie を削除してから `authenticate_with_google_code` で
コード検証・ユーザー upsert・セッション発行まで行う（`handlers/v1/auth/oauth.rs`）。

```rust
// CSRF トークンを検証（必須）
let stored_state = jar
    .get(OAUTH_STATE_COOKIE_NAME)
    .map(|c| c.value().to_string())
    .ok_or_else(|| ApiError::Unauthorized("OAuth state が見つかりません"))?;
if query.state != stored_state {
    return Err(ApiError::Unauthorized("OAuth state が一致しません"));
}

// state / pkce_verifier / nonce の Cookie を取得して削除
let is_secure = state.config.secure_cookie;
let pkce_verifier = jar.get(OAUTH_PKCE_VERIFIER_COOKIE_NAME).map(|c| c.value().to_string());
let nonce = jar.get(OAUTH_NONCE_COOKIE_NAME).map(|c| c.value().to_string());
let jar = jar
    .remove(clear_state_cookie(is_secure))
    .remove(clear_oauth_cookie(OAUTH_PKCE_VERIFIER_COOKIE_NAME, is_secure))
    .remove(clear_oauth_cookie(OAUTH_NONCE_COOKIE_NAME, is_secure));

// Option を Unauthorized に変換してから渡す
let pkce_verifier = pkce_verifier
    .ok_or_else(|| ApiError::Unauthorized("OAuth の PKCE verifier が見つかりません"))?;
let nonce = nonce.ok_or_else(|| ApiError::Unauthorized("OAuth nonce が見つかりません"))?;

// トークン交換 + tokeninfo で nonce/aud/exp 検証 + セッション発行
let session_token = auth_service::authenticate_with_google_code(
    &state.pool, client, query.code, pkce_verifier, &nonce,
).await?;
let jar = jar.add(build_session_cookie(session_token, is_secure));
```

## Cookie 設定

### 環境判定

```rust
// state.config.secure_cookie を使う（独自の環境判定を実装しない）
// 値は起動時に Config へ解決済み: RUST_ENV / APP_ENV がすべて開発用の値
// （local / dev / development / test）のときだけ非本番。未設定・不明値は
// fail-safe で本番扱い。本番では SECURE_COOKIE の値に関わらず true。
let is_secure = state.config.secure_cookie;
```

### Cookie 属性

| 属性 | 開発環境 | 本番環境 | 理由 |
|------|----------|----------|------|
| `Secure` | false | true | HTTPS 必須 |
| `SameSite` | Lax | None | クロスオリジン対応 |
| `HttpOnly` | true | true | XSS 対策 |
| `Path` | "/" | "/" | 全パスで有効 |

## チェックリスト

### 認証開始時
- [ ] state・nonce・PKCE verifier を生成している（`begin_google_auth`）
- [ ] state・nonce・PKCE verifier を Cookie に保存している
- [ ] Cookie の有効期限は短い（10分程度）

### コールバック時
- [ ] state パラメータを受け取っている（Option ではなく必須）
- [ ] Cookie の state と照合している
- [ ] 検証後に state / pkce_verifier / nonce の Cookie をすべて削除している
- [ ] ID トークンの nonce を Cookie の nonce と照合している
- [ ] セッションを DB に保存している

### Cookie 設定
- [ ] `HttpOnly` が true
- [ ] 本番環境で `Secure` が true
- [ ] 本番環境で `SameSite=None`（クロスオリジン時）
- [ ] `state.config.secure_cookie` を使用（独自の環境判定や `BACKEND_URL` 依存の判定を実装しない）

### セッション管理
- [ ] セッション ID は UUID（推測困難）
- [ ] 有効期限を設定している
- [ ] ログアウト時に DB から削除している

## よくある失敗

### 1. state を検証しない

```rust
// NG: state を無視
pub struct AuthCallbackQuery {
    pub code: String,
    #[allow(dead_code)]
    pub state: Option<String>,  // 使わない
}

// OK: state を必須で検証
pub struct AuthCallbackQuery {
    pub code: String,
    pub state: String,  // 必須
}
```

### 2. BACKEND_URL で本番判定

```rust
// NG: BACKEND_URL の有無やスキームで本番判定（設定漏れで本番の保護が外れる）
let is_secure = std::env::var("BACKEND_URL")
    .map(|url| url.starts_with("https://"))
    .unwrap_or(false);

// OK: state.config.secure_cookie（fail-safe: 未設定・不明値は本番扱い）
let is_secure = state.config.secure_cookie;
```

### 3. state Cookie を削除し忘れ

```rust
// NG: 検証後も Cookie が残る
if query.state != stored_state {
    return Err(...);
}
// この後、state Cookie を削除していない

// OK: 検証後に削除（pkce_verifier / nonce も同様に削除する）
let jar = jar.remove(clear_state_cookie(is_secure));
```

## 環境変数

| 変数 | 用途 | 例 |
|------|------|-----|
| `GOOGLE_CLIENT_ID` | OAuth クライアント ID | `xxx.apps.googleusercontent.com` |
| `GOOGLE_CLIENT_SECRET` | OAuth クライアントシークレット | `GOCSPX-xxx` |
| `BACKEND_URL` | バックエンド URL | `https://api.example.com` |
| `FRONTEND_URL` | リダイレクト先 | `https://example.com` |
| `SECURE_COOKIE` | 非本番で Cookie の Secure 属性を有効化（本番では値に関わらず Secure） | `true` |
| `RUST_ENV` / `APP_ENV` | 環境判定（開発用の値 `local`/`dev`/`development`/`test` 以外・未設定は本番扱い） | `development` |

## Google Cloud OAuth 設定

Authorized redirect URI は API バージョン付きのコールバックを登録する。

| 環境 | URI |
|------|-----|
| 本番 | `https://shoken-backend.zaitomo41.workers.dev/api/v1/oauth/google/callback` |
| ローカル | `http://localhost:8787/api/v1/oauth/google/callback` |

旧 `/auth/google/callback` は現行 API では使用しない。旧 URI を案内したり、互換ルートとして復活させたりしない。

## 参考ファイル

- `backend/src/handlers/v1/auth/oauth.rs` - 認証ハンドラー・Cookie ヘルパー
- `backend/src/handlers/v1/auth.rs` - セッション取得・削除
- `backend/src/services/auth.rs` - `begin_google_auth` / `authenticate_with_google_code` / セッション DB 操作
- `backend/src/services/auth/google_common.rs` / `google_worker.rs` - 認可 URL・PKCE・トークン交換・tokeninfo 検証
- `backend/src/extractors/auth.rs` - AuthenticatedUser エクストラクター
- `backend/src/config.rs`（`Config::secure_cookie`）と `backend/src/config/environment.rs`（`RuntimeEnv`。fail-safe 本番判定）
