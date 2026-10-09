# shoken-webapp バックエンド

日本株の証券情報を管理するRust/Axumバックエンドサービス。

## 技術スタック

- **フレームワーク**: Axum 0.8
- **データベース**: PostgreSQL
- **ORM**: SQLx
- **認証**: Google OAuth 2.0
- **デプロイ**: Cloudflare Workers

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

- Rust（リポジトリルートの `rust-toolchain.toml` を正本とする）
- PostgreSQL データベース
- 環境変数の設定

### 環境変数

バックエンドは `backend/.env` の環境変数を読み込みます。

```bash
# 必須
DATABASE_URL=postgresql://user:pass@host/db
FRONTEND_URL=http://localhost:8081

# Google OAuth（認証機能を使う場合）
GOOGLE_CLIENT_ID=your-client-id
GOOGLE_CLIENT_SECRET=your-client-secret

# J-Quants API（決算サマリー取得に必要）
JQUANTS_API_KEY=your-api-key

# オプション
PORT=3001                          # デフォルト: 3001
BACKEND_URL=https://example.com    # バックエンド自身のURL（OAuth リダイレクト等に使用）
CORS_ORIGINS=http://localhost:8081 # 許可するフロントエンドのオリジン

# 環境判定（fail-safe）
# RUST_ENV / APP_ENV の設定値がすべて開発用の値
# （local / dev / development / test）のときだけ非本番扱い。
# 未設定・不明値・本番値との混在はすべて本番扱いになる。
# ローカル開発では APP_ENV=development を設定すること（.env.example に同梱済み）
APP_ENV=development
# SECURE_COOKIE=true                # 非本番で Secure Cookie を有効化する場合のみ。本番では値に関わらず Secure
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
- 対象: 認証・銘柄検索・配当キャッシュ・advisory lock と、各ドメインの固定 `UNNEST` による一括登録
- 非対象: ドメイン共通の件数取得・検索・集計・facet は、テーブル・列・条件・並び順を `QueryBuilder` で組み立てるため
- 非対象: テストデータの投入・確認 SQL は実 DB テスト用であり、本番クエリのオフラインキャッシュには含めない

`backend/.sqlx/` はリポジトリ管理します。
理由は、CI とローカル検証を `SQLX_OFFLINE=true` で実行し、DB接続なしでも `query!` のメタデータ整合性を検証できるようにするためです。
`query!` を追加・変更した場合は、ローカルDBに最新マイグレーションを適用した上で次を実行します。

```bash
make db-up
make migrate-local
make sqlx-prepare
```

CI は `cargo fmt --check`、`SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings`、`SQLX_OFFLINE=true cargo test`、Worker ビルド（`worker-build --release`）を実行します。
`cargo sqlx prepare --check` はローカルDBの起動とマイグレーション適用が必要なためCIには入れていません。
`query!` 追加・変更時は `make sqlx-prepare` の結果を必ずコミットします。

## デプロイ

Cloudflare Workers へのデプロイは `main` へのマージで自動実行されます（`deploy-cloudflare-worker.yml`）。手動反映とログ確認:

```bash
# デプロイ（secrets/vars の注入を含むため CI 経路を使う）
gh workflow run deploy-cloudflare-worker.yml

# ログ確認（wrangler の認証が必要）
wrangler tail
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
│   │   ├── common.rs    # ハンドラー共通の応答
│   │   └── v1/          # v1 API(auth/・csv_import・dividend_per_share を含む)
│   ├── models/          # リクエスト・応答・DB の型
│   ├── services/        # ビジネスロジック
│   │   ├── domain/      # 4ドメイン共通の検索・一括登録・集計
│   │   ├── csv/         # CSV の解析・検証・取り込み
│   │   ├── jquants.rs   # J-Quants の通信
│   │   └── jquants/     # J-Quants の応答型
│   └── state.rs         # アプリケーション状態
├── migrations/          # SQLxマイグレーション
├── Cargo.toml
├── wrangler.toml        # Cloudflare Workers の設定（Hyperdrive/ratelimits/cron）
└── Makefile
```

### 層の役割と依存の向き

本番コードの依存は handlers → services → models の一方向にする(テストは除く)。

- handlers: リクエストの取り出しと応答の組み立てだけを行い、処理は services に渡す
- services: 業務ロジックと DB・外部 API へのアクセス。handlers には依存しない
- models: 型だけを置く。services・handlers には依存しない
