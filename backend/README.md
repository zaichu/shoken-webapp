# shoken-webapp バックエンド

日本株の証券情報を管理するRust/Axumバックエンドサービス。

## 技術スタック

- **フレームワーク**: Axum 0.8
- **データベース**: PostgreSQL (Neon)
- **ORM**: SQLx
- **認証**: Google OAuth 2.0
- **デプロイ**: Fly.io

## 機能

- **株式検索API**: 銘柄コード/名前による日本株検索
- **J-Quants連携**: 財務データ取得API
- **Google OAuth認証**: セッションベースの認証
- **ヘルスチェック**: サービス監視用エンドポイント

## APIエンドポイント

### プローブ

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/health` | Liveness チェック |
| GET | `/ready` | 起動完了チェック |

### 認証・セッション

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/v1/session` | 現在のセッション情報取得 |
| DELETE | `/api/v1/session` | ログアウト |
| POST | `/api/v1/account-deletion-confirmations` | アカウント削除確認（短命Cookie発行） |
| DELETE | `/api/v1/account` | アカウント削除（確認Cookie必須） |
| GET | `/api/v1/oauth/google/authorize` | Google OAuth 開始 |
| GET | `/api/v1/oauth/google/callback` | Google OAuth コールバック |

### 証券情報

| メソッド | パス | 説明 | 認証 |
|---------|------|------|------|
| GET | `/api/v1/stocks?query=7203` | 株式情報を検索 | 不要 |
| POST | `/api/v1/stocks` | 株式情報を追加 | **必須** |

### 配当金（すべて認証必須）

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/v1/dividends` | 配当金一覧を取得 |
| DELETE | `/api/v1/dividends` | 配当金を全削除 |
| POST | `/api/v1/dividend-import-validations` | 配当金CSV バリデーション（DB書き込みなし） |
| POST | `/api/v1/dividend-imports` | 配当金CSV 取り込み |
| POST | `/api/v1/dividend-per-share-estimates` | 1株配当一括推計 |

### 国内株式取引（すべて認証必須）

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/v1/domestic-stock-transactions` | 国内株式取引一覧を取得 |
| DELETE | `/api/v1/domestic-stock-transactions` | 国内株式取引を全削除 |
| POST | `/api/v1/domestic-stock-import-validations` | 国内株式CSV バリデーション |
| POST | `/api/v1/domestic-stock-imports` | 国内株式CSV 取り込み |

### 投資信託取引（すべて認証必須）

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/v1/mutual-fund-transactions` | 投資信託取引一覧を取得 |
| DELETE | `/api/v1/mutual-fund-transactions` | 投資信託取引を全削除 |
| POST | `/api/v1/mutual-fund-import-validations` | 投資信託CSV バリデーション |
| POST | `/api/v1/mutual-fund-imports` | 投資信託CSV 取り込み |

### 保有銘柄（すべて認証必須）

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/v1/asset-balances` | 保有銘柄一覧を取得 |
| PUT | `/api/v1/asset-balances` | 保有銘柄を全件置換 |
| DELETE | `/api/v1/asset-balances` | 保有銘柄を全削除 |
| POST | `/api/v1/asset-balance-import-validations` | 保有銘柄CSV バリデーション |
| POST | `/api/v1/asset-balance-imports` | 保有銘柄CSV 取り込み |

### 市場データ（認証必須）

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/v1/financial-statements?code=7203` | 財務サマリーを取得 |

## 開発

### 前提条件

- Rust 1.70+
- PostgreSQL データベース
- 環境変数の設定

### 環境変数

バックエンドは `backend/.env` の環境変数を読み込みます。

```bash
# 必須
DATABASE_URL=postgresql://user:pass@host/db
FRONTEND_URL=http://localhost:5173

# Google OAuth（認証機能を使う場合）
GOOGLE_CLIENT_ID=your-client-id
GOOGLE_CLIENT_SECRET=your-client-secret

# J-Quants API（決算サマリー取得に必要）
JQUANTS_API_KEY=your-api-key

# オプション
PORT=3001                          # デフォルト: 3001
BACKEND_URL=https://example.com    # 本番環境のURL（https://で始まる場合Secure Cookie有効）
CORS_ORIGINS=http://localhost:8080 # 許可するフロントエンドのオリジン

# 本番環境判定（いずれかを設定）
RUST_ENV=production                # または APP_ENV=production
SECURE_COOKIE=true                 # Cookie の Secure 属性を明示的に制御
```

`.env` ファイルの準備（既存があればそのまま使用）:

```bash
if [ -f .env ]; then
  echo ".env already exists. reuse it."
elif [ -f .env.example ]; then
  cp .env.example .env
else
  echo ".env.example not found. create .env manually."
fi
```

### コマンド

```bash
# ローカル開発サーバー起動
make run

# テスト実行
make test

# コンパイルチェック
make check

# マイグレーション実行
make migrate

# ローカルDocker DBにマイグレーション
make migrate-local

# PostgreSQL を Docker で起動
make db-up

# PostgreSQL を停止
make db-down

# Fly.ioへデプロイ
make deploy
```

### Docker で PostgreSQL を起動する

```bash
cd backend
make db-up
```

`.env` の `DATABASE_URL` は以下を想定しています。

```bash
DATABASE_URL=postgresql://user:password@localhost:5432/shoken_db
```

## データベーススキーマ

### テーブル

- `stock`: 銘柄情報
- `users`: ユーザー情報（Google OAuth）
- `sessions`: セッション管理

### マイグレーション

```bash
# マイグレーション実行
cargo sqlx migrate run

# SQLxクエリキャッシュ準備（オフラインビルド用）
cargo sqlx prepare --merged
```

## デプロイ

Fly.ioへのデプロイ:

```bash
# デプロイ
make deploy

# ステータス確認
make status

# ログ確認
make logs
```

## ディレクトリ構成

```
backend/
├── src/
│   ├── main.rs          # エントリーポイント
│   ├── config.rs        # 設定
│   ├── errors.rs        # エラーハンドリング
│   ├── extractors/      # Axumエクストラクター
│   ├── handlers/        # リクエストハンドラー
│   │   ├── v1/          # v1 APIハンドラー
│   │   └── csv_import.rs # CSV取り込み共通処理
│   ├── models/          # データモデル
│   ├── services/        # ビジネスロジック
│   └── state.rs         # アプリケーション状態
├── migrations/          # SQLxマイグレーション
├── Cargo.toml
├── Dockerfile
├── fly.toml             # Fly.io設定
└── Makefile
```
