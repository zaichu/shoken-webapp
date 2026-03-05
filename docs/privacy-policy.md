# プライバシーポリシー

最終更新日: 2026年3月

## 1. 事業者情報

本サービス「証券情報ウェブアプリケーション (shoken-webapp)」は個人が運営しています。

## 2. 収集する情報

本サービスでは以下の情報を収集します。

### 2.1 Google OAuth 認証情報

Google アカウントでログインする際、Google が提供する以下の情報を取得します。

- Google アカウントのメールアドレス
- Google アカウントの表示名
- Google が発行する一意の識別子（Google ID）

これらの情報はサービス利用者の識別および認証にのみ使用します。

### 2.2 利用者が入力するデータ

- CSV ファイルにより登録した証券明細（配当金・国内株式・投資信託・資産残高）

これらのデータは利用者のポートフォリオ表示や明細管理のために使用します。

### 2.3 ログ情報

- リクエストの HTTP メソッド・パス・ステータスコード・処理時間
- x-request-id（リクエスト追跡用）

クエリパラメータ（OAuth コードを含む）はログに記録しません。

## 3. 情報の利用目的

収集した情報は以下の目的にのみ使用します。

- サービスへのログイン・認証
- 利用者ごとのデータ管理
- サービス改善のための統計分析（個人を特定しない形式）

## 4. 第三者提供

収集した個人情報を第三者へ提供することはありません。ただし以下を除きます。

- 法令に基づく開示が必要な場合

## 5. 利用する外部サービス

| サービス | 用途 | プライバシーポリシー |
|---|---|---|
| Google OAuth 2.0 | 認証 | https://policies.google.com/privacy |
| Fly.io | バックエンドホスティング | https://fly.io/legal/privacy-policy/ |
| Vercel | フロントエンドホスティング | https://vercel.com/legal/privacy-policy |
| J-Quants API | 日本株情報取得 | https://jpx-jquants.com/support/privacy/ |

## 6. Cookie の利用

本サービスが使用する Cookie については [Cookie ポリシー](cookie-policy.md) を参照してください。

## 7. データの保管・削除

- 認証情報（Google ID・メールアドレス・表示名）はサービスのデータベースに保存します
- 利用者はアカウント削除機能により、自身のすべてのデータを削除できます
- アカウント削除後、データは即時削除されます

## 8. セキュリティ

- 通信は HTTPS で暗号化されます
- セッション Cookie は `HttpOnly` および `Secure` 属性で保護されます
- セキュリティ上の問題を発見した場合は [SECURITY.md](../SECURITY.md) の手順で報告してください

## 9. プライバシーポリシーの変更

本ポリシーを変更する場合は、このページにて告知します。

## 10. お問い合わせ

ご質問は GitHub の [Issues](https://github.com/zaichu/shoken-webapp/issues) または [Security Advisories](https://github.com/zaichu/shoken-webapp/security/advisories/new) までお寄せください。
