# 運用ランブック

## ローカル開発環境の起動

```bash
# 1. リポジトリをクローン
git clone https://github.com/zaichu/shoken-webapp.git
cd shoken-webapp

# 2. 環境変数を設定
(cd backend && cp .env.example .env)  # DATABASE_URL 等を設定
echo "VITE_SHOKEN_WEBAPI_API_URL=http://127.0.0.1:3001" > frontend/.env.development.local

# 3. 一括起動（推奨）
./scripts/start-local.sh
```

起動後のアクセス先:
- フロントエンド: http://127.0.0.1:8080
- バックエンド: http://127.0.0.1:3001
- DB: `postgresql://user:password@localhost:5432/shoken_db`

停止: `./scripts/stop-local.sh`

## デプロイ

`main` ブランチへのマージで CI/CD が自動実行されます。

| サービス | デプロイ先 | トリガー |
|---|---|---|
| フロントエンド | Vercel | main push |
| バックエンド | Fly.io | main push |

### 手動デプロイ（緊急時）

```bash
# バックエンド
(cd backend && make deploy)

# フロントエンド（Vercel CLI）
(cd frontend && vercel --prod)
```

## ヘルスチェック

```bash
# バックエンド
curl https://shoken-webapp-backend.fly.dev/health

# ログ確認
fly logs --app shoken-webapp-backend
```

## データベースマイグレーション

```bash
# マイグレーション作成
(cd backend && sqlx migrate add <migration_name>)

# マイグレーション実行（起動時に自動実行）
(cd backend && make run)

# オフラインモード用 sqlx-data 更新
(cd backend && make sqlx-prepare)
```

## 環境変数（本番）

Fly.io Secrets で管理:

```bash
fly secrets set DATABASE_URL="postgresql://..."
fly secrets set GOOGLE_CLIENT_ID="..."
fly secrets set GOOGLE_CLIENT_SECRET="..."
fly secrets set FRONTEND_URL="https://shoken-webapp.vercel.app"
fly secrets set BACKEND_URL="https://shoken-webapp-backend.fly.dev"
```

## セキュリティインシデント対応

1. [SECURITY.md](../SECURITY.md) の手順に従って報告を受理
2. 重大度に応じて 30〜90 日以内に修正

脆弱性が発見された場合:
```bash
# 依存関係の脆弱性チェック
(cd backend && cargo audit)
(cd frontend && npm audit)
```

## Dependabot PR 対応

毎週月曜日に Dependabot PR が作成されます。

1. CI が green の PR: `gh pr merge <PR番号> --squash --delete-branch`
2. CI が red の PR: 失敗原因を調査して対応方針を決定（修正 / 保留 / close）

## よくあるトラブル

### DB 接続失敗

```bash
(cd backend && make db-up)  # Docker で PostgreSQL を起動
```

### バックエンドビルドエラー

```bash
(cd backend && cargo check)  # コンパイルエラー確認
(cd backend && cargo fmt)    # フォーマット修正
```

### フロントエンドの型エラー

```bash
(cd frontend && npx tsc --noEmit)  # 型エラー確認
(cd frontend && npm run lint)      # Lint エラー確認
```

### OpenAPI スキーマと api.ts の不一致

```bash
bash scripts/check-openapi.sh  # 再生成して差分確認
```
