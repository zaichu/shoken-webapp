# Cookie ポリシー

最終更新日: 2026年3月

## 1. Cookie の利用概要

本サービス「証券情報ウェブアプリケーション (shoken-webapp)」は、認証セッションの維持のために Cookie を使用します。

## 2. 使用する Cookie

### 2.1 セッション Cookie（認証用）

| 属性 | 値 |
|---|---|
| 名前 | `session_id` |
| 目的 | ログイン状態の維持 |
| 種類 | セッション Cookie（ブラウザを閉じると削除） |
| HttpOnly | ✅（JavaScript からアクセス不可） |
| Secure | ✅（HTTPS 通信のみ送信） |
| SameSite | `None`（本番環境）/ `Lax`（開発環境） |

この Cookie は Google OAuth 認証後に発行され、ログアウトまたはアカウント削除時に無効化されます。

### 2.2 サードパーティ Cookie

本サービス自体はトラッキング目的のサードパーティ Cookie を使用しません。

ただし、利用している外部サービス（Vercel・Fly.io 等）がインフラ運用上の Cookie を設定する場合があります。

## 3. Cookie の管理

ブラウザの設定から Cookie を無効化することができます。ただし、認証 Cookie を無効化するとログイン機能が利用できなくなります。

- [Chrome の Cookie 設定](https://support.google.com/chrome/answer/95647)
- [Firefox の Cookie 設定](https://support.mozilla.org/ja/kb/enable-and-disable-cookies-website-preferences)
- [Safari の Cookie 設定](https://support.apple.com/ja-jp/guide/safari/sfri11471/mac)

## 4. お問い合わせ

Cookie の利用に関するご質問は GitHub の [Issues](https://github.com/zaichu/shoken-webapp/issues) までお寄せください。
