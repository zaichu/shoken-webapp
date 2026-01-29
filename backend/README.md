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

### 認証

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/auth/google` | Google OAuth開始 |
| GET | `/auth/google/callback` | OAuthコールバック |
| GET | `/auth/me` | 現在のユーザー情報 |
| POST | `/auth/logout` | ログアウト |
| DELETE | `/auth/delete-account` | アカウント削除 |

### 証券情報

| メソッド | パス | 説明 | 認証 |
|---------|------|------|------|
| GET | `/stock/{query}` | 株式情報を検索 | 不要 |
| POST | `/stock` | 株式情報を追加 | **必須** |

### データ管理（すべて認証必須）

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/dividends` | 配当金一覧を取得 |
| POST | `/dividends/bulk` | 配当金を一括追加 |
| DELETE | `/dividends/all` | 配当金を全削除 |
| GET | `/domestic-stocks` | 国内株式一覧を取得 |
| POST | `/domestic-stocks/bulk` | 国内株式を一括追加 |
| DELETE | `/domestic-stocks/all` | 国内株式を全削除 |
| GET | `/mutualfunds` | 投資信託一覧を取得 |
| POST | `/mutualfunds/bulk` | 投資信託を一括追加 |
| DELETE | `/mutualfunds/all` | 投資信託を全削除 |
| GET | `/asset-balances` | 保有銘柄一覧を取得 |
| POST | `/asset-balances/bulk` | 保有銘柄を一括追加（UPSERT） |
| DELETE | `/asset-balances/all` | 保有銘柄を全削除 |

### J-Quants API

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/jquants/fins/statements` | 決算サマリーを取得 |

### その他

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/health` | ヘルスチェック |

## 開発

### 前提条件

- Rust 1.70+
- PostgreSQL データベース
- 環境変数の設定

### 環境変数

```bash
# 必須
DATABASE_URL=postgresql://user:pass@host/db
FRONTEND_URL=http://localhost:8080

# Google OAuth（認証機能を使う場合）
GOOGLE_CLIENT_ID=your-client-id
GOOGLE_CLIENT_SECRET=your-client-secret

# J-Quants API（決算サマリー取得に必要）
JQUANTS_API_KEY=your-api-key

# オプション
PORT=3001                          # デフォルト: 3001
BACKEND_URL=https://example.com    # 本番環境のURL（https://で始まる場合Secure Cookie有効）

# 本番環境判定（いずれかを設定）
RUST_ENV=production                # または APP_ENV=production
SECURE_COOKIE=true                 # Cookie の Secure 属性を明示的に制御
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
