# セキュリティポリシー

## 脆弱性の報告

セキュリティに関わる問題を発見した場合は、**公開 Issue ではなく**、以下の手順で報告してください。

### 報告先

GitHub の [Security Advisories](https://github.com/zaichu/shoken-webapp/security/advisories/new) からプライベートレポートを作成してください。

報告内容には以下を含めてください：

- 問題の概要と影響範囲
- 再現手順（できるだけ具体的に）
- 想定される攻撃シナリオ
- 発見バージョン・環境情報

### 対応方針と SLA

| ステップ | 目安 |
|---|---|
| 受領確認 | 3 営業日以内 |
| 初期評価・重大度判定 | 7 日以内 |
| 修正完了 | 重大度 Critical/High: 30 日以内、Medium 以下: 90 日以内 |
| 公開 | 修正リリース後に GitHub Advisory を公開 |

### 対象範囲

- バックエンド API（Rust / Axum）
- フロントエンド（React / TypeScript）
- 認証・セッション管理（Google OAuth2）
- CSV インポート処理

### 対象外

- 開発環境限定の設定ミス
- 第三者サービス（Fly.io・Vercel・Neon・J-Quants API）側の脆弱性

## サポートバージョン

現在は `main` ブランチの最新リリースのみサポートします。

## 謝辞

報告していただいた方には、修正公開後に Advisory の謝辞に名前を掲載します（ご希望の場合）。
