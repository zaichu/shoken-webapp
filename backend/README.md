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

| メソッド | パス | 説明 |
|---------|------|------|
| POST | `/stock` | 株式情報を追加 |
| GET | `/stock/{query}` | 株式情報を検索 |
| POST | `/jquants/auth` | J-Quants認証 |
| POST | `/jquants/refresh` | J-Quantsトークン更新 |
| GET | `/jquants/fins/statements` | 財務諸表を取得 |
| GET | `/auth/google` | Google OAuth開始 |
| GET | `/auth/google/callback` | OAuthコールバック |
| GET | `/auth/me` | 現在のユーザー情報 |
| POST | `/auth/logout` | ログアウト |
| GET | `/health` | ヘルスチェック |

## 開発

### 前提条件

- Rust 1.70+
- PostgreSQL データベース
- 環境変数の設定

### 環境変数

```bash
DATABASE_URL=postgresql://user:pass@host/db
JQUANTS_EMAIL=your-email
JQUANTS_PASSWORD=your-password
GOOGLE_CLIENT_ID=your-client-id
GOOGLE_CLIENT_SECRET=your-client-secret
FRONTEND_URL=http://localhost:8080
BACKEND_URL=http://localhost:3001  # 本番環境のみ
PORT=3001
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

# Fly.ioへデプロイ
make deploy
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
│   │   ├── auth.rs      # 認証
│   │   ├── jquants.rs   # J-Quants API
│   │   └── stock.rs     # 株式検索
│   ├── models/          # データモデル
│   ├── services/        # ビジネスロジック
│   └── state.rs         # アプリケーション状態
├── migrations/          # SQLxマイグレーション
├── Cargo.toml
├── Dockerfile
├── fly.toml             # Fly.io設定
└── Makefile
```
