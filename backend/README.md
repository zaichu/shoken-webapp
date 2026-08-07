# shoken-webapp バックエンド

日本株の証券情報を管理するRust/Axumバックエンドサービス。

## 技術スタック

- **フレームワーク**: Axum 0.8
- **データベース**: PostgreSQL
- **ORM**: SQLx
- **認証**: Google OAuth 2.0
- **デプロイ**: Fly.io

## 機能

- **株式検索API**: 銘柄コード/名前による日本株検索
- **J-Quants連携**: 財務データ取得API
- **Google OAuth認証**: セッションベースの認証
- **ヘルスチェック**: サービス監視用エンドポイント

## API ドキュメント

API の一覧と設計方針は [`../docs/api/v1-rest-design.md`](../docs/api/v1-rest-design.md) に集約しています。
API 契約の正本は [`../docs/openapi.json`](../docs/openapi.json) です。

## 開発

### 前提条件

- Rust 1.96+（`rust-toolchain.toml` を正本とする）
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
make migrate-local

# SQLxクエリキャッシュ準備（オフラインビルド用）
cargo sqlx prepare
```

### SQLx query! 運用方針

固定SQLは `sqlx::query!` / `sqlx::query_as!` を優先し、コンパイル時検証できる形に寄せます。
対象は SQL 文字列と戻り値の形が固定できる処理です。

- 対象: `delete_all_for_user` のような固定テーブルへの `DELETE`
- 対象: 条件と戻り列が固定された単純な `SELECT COUNT(*)`
- 非対象: 検索条件、facet、並び順、対象カラムを `QueryBuilder` で組み立てる動的SQL
- 非対象: バルク `UNNEST` のように列数や bind 配列を共通ヘルパーで扱う処理

`backend/.sqlx/` はリポジトリ管理します。
理由は、CI とローカル検証を `SQLX_OFFLINE=true` で実行し、DB接続なしでも `query!` のメタデータ整合性を検証できるようにするためです。
`query!` を追加・変更した場合は、ローカルDBに最新マイグレーションを適用した上で次を実行します。

```bash
make db-up
make migrate-local
make sqlx-prepare
```

CI は `SQLX_OFFLINE=true cargo clippy`、`SQLX_OFFLINE=true cargo test`、Docker イメージビルドを実行します。
`cargo sqlx prepare --check` はローカルDBの起動とマイグレーション適用が必要なためCIには入れていません。
`query!` 追加・変更時は `make sqlx-prepare` の結果を必ずコミットします。
`backend/Dockerfile` は本番ビルドを `SQLX_OFFLINE=true` かつ `.sqlx/` を build context にコピーして実行するため、DB接続なしでビルドできます。

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
