# セキュリティ共通ルール

現行構成のセキュリティ方針の正本は `SECURITY.md`。ここでは開発時に守るルールを定める。

## 機密情報の管理

### 絶対にコミットしない

- APIキー、シークレット
- パスワード、認証トークン
- 個人情報（メールアドレス等）
- 本番環境の接続情報

秘密情報はローカルの `.env`（git 管理外）と Fly.io Secrets / GitHub Secrets で管理する。

## HTTPS と本番判定

- 本番環境では必ずHTTPS（Fly.io は `force_https`、Vercel は自動で HTTPS）
- 本番かどうかの判定は `RUST_ENV` / `APP_ENV` の値だけで行う（fail-safe）。
  開発用の値（`local` / `dev` / `development` / `test`）がすべてに明示設定されたときだけ非本番。
  未設定・不明値は本番扱いなので、ローカル起動では `APP_ENV=development` を設定する
- `BACKEND_URL` のスキームや有無でセキュリティ設定を変えない

## Cookie

- セッション Cookie は `HttpOnly`・`Path=/`・`Max-Age=7日`
- 本番では `Secure` + `SameSite=None`、ローカルでは `SameSite=Lax`
- Cookie 属性の判定は `config::is_secure_cookie()` を経由し、独自の環境判定を実装しない

## CSRF / CORS

- 状態を変える GET リクエストを新設しない
- Origin / Referer の検証と CORS の許可一覧は完全一致で行い、前方一致（`starts_with`）は使わない
- 本番では Origin と Referer の両方が無い変更系リクエストを拒否する

## レート制限・リソース上限

- 新しい外部公開エンドポイントには、既存の keyed IP limiter（auth / stock_search / csv / data のいずれか）への所属を検討する
- ユーザーあたりの保存行数上限（`USER_ROW_LIMIT`）を超える追加は拒否する
- クライアント IP は `fly-client-ip` を使い、`X-Forwarded-For` を信用しない

## ログ出力

### 出力してはいけない情報

- パスワード、トークン、OAuth の `code` / `state` などクエリパラメータ
- 個人情報
- APIキー、DB エラーの本文（本番）

リクエストログはパスのみ記録する。

## エラー応答

- 本番では DB の詳細や内部情報を応答に含めない
- OAuth エラーの詳細は応答にもログにも出さない

## CI / GitHub Actions

- `uses:` は 40 桁のコミット SHA で固定する
- `pull_request_target` では PR のコードを checkout しない
- デプロイ用 secrets（`FLY_API_TOKEN` / `VERCEL_*`）を PR のジョブに渡さない

## 受け入れたリスク

エージェント権限のリスク受容（Devin の広い許可、Codex `danger-full-access`、統合側 `gh` の admin）は `SECURITY.md` の「受け入れたリスク」を正本とする。

## チェックリスト

新機能開発時のセキュリティチェック:

- [ ] 入力値のバリデーション実装
- [ ] 認証・認可の確認（`AuthenticatedUser` / `user_id` スコープ）
- [ ] 機密情報がログに出力されないか
- [ ] 依存関係の脆弱性チェック（cargo audit / npm audit）
- [ ] 新しい変更系エンドポイントのレート制限帰属を決めたか
