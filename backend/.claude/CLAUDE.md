# CLAUDE.md

日本語で必ず回答してください。
このファイルは Claude Code (claude.ai/code) がこのリポジトリで作業する際のガイダンスを提供します。

## コマンド

### 開発用コマンド
- `make run` - ローカルでサーバーを起動（cargo run）
- `make build` - cargo buildでバックエンドをビルド
- `make build-release` - リリースビルドを作成
- `make check` - ビルドなしでコンパイルチェックを実行
- `make test` - cargo testですべてのテストを実行

### データベースコマンド
- `make migrate` - データベースマイグレーションを実行
- `cargo sqlx migrate add <name>` - 新しいマイグレーションを作成
- `make sqlx-prepare` - デプロイ用SQLxクエリキャッシュを準備

### デプロイコマンド
- `make deploy` - Fly.ioへデプロイ
- `make status` - Fly.ioサービスステータスを確認
- `make logs` - アプリケーションログを表示
- `make docker-build` - Dockerイメージをビルド

### ユーティリティコマンド
- `make clean` - ビルド成果物をクリーン
- `make clean-build` - 完全クリーンと再ビルド
- `make help` - 利用可能なすべてのコマンドを表示

## アーキテクチャ概要

### 技術スタック
- **Rust** と **Axum 0.8.1** Webフレームワーク
- **Fly.io** デプロイプラットフォーム
- **Neon** PostgreSQLデータベース
- **SQLx 0.8.3** 型安全なデータベース操作
- **OAuth2** 認証（Google OAuth）
- **Tower-HTTP** CORS とミドルウェア

### プロジェクト構造
モジュラーRustパターンに従った**日本株取引アプリケーションバックエンド**：

- `src/main.rs` - アプリケーションエントリーポイントとルート設定
- `src/lib.rs` - モジュールエクスポートとパブリックAPI
- `src/state.rs` - データベースプール、secrets、HTTPクライアント付きAppState
- `src/errors.rs` - 構造化APIレスポンス付き一元エラーハンドリング
- `src/handlers/` - ドメイン別に整理されたビジネスロジック（stock、jquants、auth）
- `src/models/` - バリデーション付きデータ構造（serde + validator）
- `src/extractors/` - リクエスト処理用カスタムAxumエクストラクター
- `migrations/` - SQLxデータベーススキーママイグレーション
- `tests/` - ハンドラーとサービスの統合テスト

### コアアーキテクチャパターン

**ドメイン駆動構造**: ドメイン別に分離されたビジネスロジック（株式操作、JQuants API統合、認証）

**型安全なデータベース層**: SQLxを使用したデータベース操作、自動マッピング用`FromRow`派生

**包括的エラーハンドリング**: バリデーション、データベース、ネットワーク、OAuthエラーをカバーする構造化`ApiError` enum、自動HTTPステータスマッピング

**カスタムリクエスト処理**:
- `ValidatedJson<T>` エクストラクター：自動JSON解析 + バリデーション
- `char_width_converter`：日本語テキスト処理（半角→全角変換）

**外部API統合**: OAuth2認証とbearer token管理付きJQuants金融データAPI

### 主要APIエンドポイント

**株式操作**:
- `GET /stock/{query}` - コードまたは名前で株式検索（ILIKEパターンマッチング）
- `POST /stock` - 新しい株式情報を追加

**JQuants金融データ**:
- `POST /jquants/auth` - JQuants APIで認証
- `POST /jquants/refresh` - 認証トークンをリフレッシュ
- `GET /jquants/fins/statements` - 財務諸表を取得

**Google OAuth認証**:
- `GET /auth/google` - Google認証URLを取得
- `GET /auth/google/callback` - OAuthコールバック処理
- `GET /auth/me` - 現在のユーザー情報を取得
- `POST /auth/logout` - ログアウト処理

### データベーススキーマ

**株式モデル** (`migrations/0001_init.sql`):
```sql
stocks (
    date DATE NOT NULL,
    code VARCHAR(10) PRIMARY KEY,
    name CITEXT NOT NULL,
    market_category VARCHAR(50),
)
```

**ユーザーモデル** (`migrations/0002_create_users.sql`):
```sql
users (
    id UUID PRIMARY KEY,
    google_id VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    picture_url TEXT,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
)
```

### 設定

**環境変数** (.env または Fly.io Secrets):
- `DATABASE_URL` - PostgreSQL接続文字列（Neon）
- `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET` - OAuth認証情報
- `FRONTEND_URL` - フロントエンドURL（CORS origin設定）
- `BACKEND_URL` - バックエンドURL（本番環境判定用）

**Fly.io Secrets設定**:
```bash
fly secrets set DATABASE_URL="postgres://..."
fly secrets set GOOGLE_CLIENT_ID="..."
fly secrets set GOOGLE_CLIENT_SECRET="..."
fly secrets set FRONTEND_URL="https://..."
fly secrets set BACKEND_URL="https://..."
```

**データベース接続**: 最大5接続のPostgreSQLプール

### 日本語テキスト処理
- 大文字小文字区別なし日本語テキスト検索用CITEXTカラム
- 検索機能用文字幅変換ユーティリティ
- 標準化入力用半角→全角カタカナ変換

### 開発ガイドライン
- すべてのリクエストモデルで入力バリデーション用`validator`クレートの派生を使用
- ビジネスロジックはドメイン固有のハンドラーモジュールに配置
- エラーレスポンスはエラーコード付き構造化JSON形式に従う
- CORSは特定のフロントエンドoriginのみに設定（credentials許可）
- スキーマ変更にはデータベースマイグレーションが必要
- mod.rsは古い書き方なので非推奨

### Cookie設定（クロスオリジン認証）
フロントエンド（Vercel）とバックエンド（Fly.io）が異なるドメインのため：
- `SameSite=None` + `Secure` が必須（本番環境）
- `HttpOnly` でXSS対策
- ローカル開発では `SameSite=Lax`

### テスト戦略
- すべてのHTTPエンドポイントの統合テスト
- ビジネスロジックとユーティリティの単体テスト
- 外部依存関係のモック（JQuants API）
- フィクスチャ付きテストデータベースセットアップ

### デプロイ注意事項
- Fly.io + Neon PostgreSQLの構成
- Dockerfileでマルチステージビルド
- fly.tomlでデプロイ設定（東京リージョン）
- CORS originはデプロイ済みフロントエンドURLと一致させる必要がある
- 起動時にデータベースマイグレーションを自動適用
