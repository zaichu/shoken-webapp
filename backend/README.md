# shoken-webapp バックエンド

日本株の証券情報を管理するRust/Axumバックエンドサービス。

## 技術スタック

- **フレームワーク**: Axum 0.8（Cloudflare Workers 上で `worker` クレート経由で動作）
- **データベース**: PostgreSQL（本番は Hyperdrive、ローカルは wrangler dev の localConnectionString 経由。ドライバは tokio-postgres）
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

実行環境は Cloudflare Workers のみで、設定の正本は `wrangler.toml`（vars/bindings）です。

- ローカル開発: `wrangler dev --env dev` が `[env.dev.vars]`（`APP_ENV`/`BACKEND_URL`/`FRONTEND_URL`）と
  `[env.dev.hyperdrive]` の `localConnectionString` を読みます
- シークレット: `backend/.dev.vars`（gitignore 済み）に置きます。雛形は `.dev.vars.example`

```bash
cp .dev.vars.example .dev.vars
# GOOGLE_CLIENT_ID / GOOGLE_CLIENT_SECRET / JQUANTS_API_KEY を埋める
```

vars で上書きできる主な値:

```bash
BACKEND_URL=http://localhost:8787    # バックエンド自身のURL（OAuth リダイレクト等に使用）
FRONTEND_URL=http://localhost:8081   # フロントエンドのURL
CORS_ORIGINS=http://localhost:8081   # 許可するフロントエンドのオリジン
APP_ENV=development                  # 未設定・不明値・本番値との混在はすべて本番扱い（fail-safe）
# SECURE_COOKIE=true                # 非本番で Secure Cookie を有効化する場合のみ。本番では値に関わらず Secure
```

`backend/.env` は `cargo run --bin migrate`（ホスト側マイグレーション）だけが
dotenvy で読む `DATABASE_URL` 用です（`.env.example` 参照）。

### コマンド

```bash
# ローカル開発サーバー起動（wrangler dev、http://localhost:8787）
make run

# Workers 向け wasm ビルド
make worker-build

# テスト実行（ホスト側。ユニットテストとツール検証）
make test

# コンパイルチェック
make check

# マイグレーション実行（DATABASE_URL 経由で直接接続）
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

`wrangler dev` は `wrangler.toml` の `localConnectionString` でこの DB に接続します
（既定: `postgres://user:password@127.0.0.1:5432/shoken_db?sslmode=disable`）。

## データベーススキーマ

### テーブル

- `stock`: 銘柄情報
- `users`: ユーザー情報（Google OAuth）
- `sessions`: セッション管理

### マイグレーション

```bash
# マイグレーション実行
make migrate-local
```

クエリは `tokio-postgres` の生 SQL で書きます（Workers 実行時は Hyperdrive 経由）。
CI は `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`、
Worker ビルド（`worker-build --release`）を実行します。

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
│   ├── worker_entry.rs  # Workers エントリーポイント（fetch/scheduled イベント）
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
├── migrations/          # DB マイグレーション（`cargo run --bin migrate` で適用）
├── Cargo.toml
├── wrangler.toml        # Cloudflare Workers の設定（Hyperdrive/ratelimits/cron）
└── Makefile
```

### 層の役割と依存の向き

本番コードの依存は handlers → services → models の一方向にする(テストは除く)。

- handlers: リクエストの取り出しと応答の組み立てだけを行い、処理は services に渡す
- services: 業務ロジックと DB・外部 API へのアクセス。handlers には依存しない
- models: 型だけを置く。services・handlers には依存しない
