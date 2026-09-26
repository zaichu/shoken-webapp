# Cookie ポリシー

最終更新日: 2026年9月26日

## 1. Cookie の利用概要

本サービス「証券情報ウェブアプリケーション (shoken-webapp)」は、認証セッションの維持と認証・セキュリティ処理のために Cookie を使用します。

## 2. 使用する Cookie

すべての Cookie は `HttpOnly`（JavaScript からアクセス不可）で発行されます。本番環境では `Secure`（HTTPS 通信のみ送信）・`SameSite=None`、開発環境では既定で `Secure` なし・`SameSite=Lax` です。

### 2.1 セッション Cookie（認証用）

| 属性 | 値 |
|---|---|
| 名前 | `session_token` |
| 目的 | ログイン状態の維持 |
| パス | `/` |
| 有効期限 | 7日間（発行から7日後に自動期限切れ） |

この Cookie は Google OAuth 認証後に発行され、ログアウトまたはアカウント削除時に無効化されます。

### 2.2 OAuth 認証フロー用 Cookie（一時）

Google OAuth のログイン開始時に発行され、認証に成功したときに削除されます。認証に失敗した場合は、有効期限の 10 分が過ぎると失効します。パスは `/` です。

| 名前 | 目的 |
|---|---|
| `oauth_state` | CSRF 対策の state パラメータ照合 |
| `oauth_pkce_verifier` | PKCE のコード検証子の照合 |
| `oauth_nonce` | OIDC nonce の照合 |

### 2.3 アカウント削除確認用 Cookie（一時）

| 属性 | 値 |
|---|---|
| 名前 | `account_delete_confirmation` |
| 目的 | アカウント削除操作の直前確認（削除実行の直前に発行した署名付き確認トークンの照合） |
| パス | `/api/v1/account` |
| 有効期限 | 10分間 |

### 2.4 サードパーティ Cookie

本サービス自体はトラッキング目的のサードパーティ Cookie を使用しません。

ただし、利用している外部サービス（Vercel・Fly.io 等）がインフラ運用上の Cookie を設定する場合があります。

## 3. ブラウザ内ストレージの利用

認証状態のタブ間同期のために `localStorage` を使用します。保存するのは以下のフラグ・タイムスタンプのみで、証券データや個人情報は保存しません。`sessionStorage` は使用しません。

| キー | 値 | 目的 |
|---|---|---|
| `last_activity_at` | 最終操作の時刻（ミリ秒） | 複数タブ間で最終操作時刻を共有し、無操作時の自動ログアウト判定に使うため |
| `pending_logout` | フラグ | ログアウト処理の再試行中を示し、ページ再読み込み後も再試行を継続するため |
| `logout_at` | ログアウト通知の時刻（ミリ秒） | BroadcastChannel が使えない環境で他タブへログアウトを通知するため |

また、タブ間通信に BroadcastChannel（`shoken_session`）、排他制御に Web Locks（`logout_claim`）を使用しますが、これらは永続化されません。

## 4. Cookie の管理

ブラウザの設定から Cookie を無効化することができます。ただし、認証 Cookie を無効化するとログイン機能が利用できなくなります。

- [Chrome の Cookie 設定](https://support.google.com/chrome/answer/95647)
- [Firefox の Cookie 設定](https://support.mozilla.org/ja/kb/enable-and-disable-cookies-website-preferences)
- [Safari の Cookie 設定](https://support.apple.com/ja-jp/guide/safari/sfri11471/mac)

## 5. お問い合わせ

Cookie の利用に関するご質問は GitHub の [Issues](https://github.com/zaichu/shoken-webapp/issues) までお寄せください。
