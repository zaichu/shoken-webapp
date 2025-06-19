# CLAUDE.md

日本語で必ず回答してください。
このファイルは Claude Code (claude.ai/code) がこのリポジトリで作業する際のガイダンスを提供します。

## コマンド

### 開発用コマンド
- `make run` - ポート3001でdev secretsを使用してローカルShuttleプロジェクトを実行
- `make build` - cargo buildでバックエンドをビルド
- `make check` - ビルドなしでコンパイルチェックを実行
- `make test` - cargo testですべてのテストを実行
- `cargo run` - 直接実行（make runの代替）

### データベースコマンド
- `cargo sqlx migrate run` - データベースマイグレーションを実行
- `cargo sqlx migrate add <name>` - 新しいマイグレーションを作成
- `make sqlx-prepare` - デプロイ用SQLxクエリキャッシュを準備

### デプロイコマンド
- `make deploy` - Shuttleプラットフォームへデプロイ
- `make deploy-dirty` - 未コミット変更でデプロイ
- `make status` - Shuttleサービスステータスを確認
- `make logs` - デプロイログを表示

### ユーティリティコマンド
- `make clean` - ビルド成果物をクリーン
- `make clean-build` - 完全クリーンと再ビルド
- `make help` - 利用可能なすべてのコマンドを表示

## アーキテクチャ概要

### 技術スタック
- **Rust** と **Axum 0.8.1** Webフレームワーク
- **Shuttle.rs** デプロイプラットフォーム（PostgreSQL付き）
- **SQLx 0.8.3** 型安全なデータベース操作
- **OAuth2** 認証（Google OAuth）
- **Tower-HTTP** CORS とミドルウェア

### プロジェクト構造
モジュラーRustパターンに従った**日本株取引アプリケーションバックエンド**：

- `src/main.rs` - アプリケーションエントリーポイントとルート設定
- `src/lib.rs` - モジュールエクスポートとパブリックAPI
- `src/state.rs` - データベースプール、secrets、HTTPクライアント付きAppState
- `src/errors.rs` - 構造化APIレスポンス付き一元エラーハンドリング
- `src/handlers/` - ドメイン別に整理されたビジネスロジック（stock、jquants）
- `src/models/` - バリデーション付きデータ構造（serde + validator）
- `src/extractors/` - リクエスト処理用カスタムAxumエクストラクター
- `migrations/` - SQLxデータベーススキーママイグレーション
- `tests/` - ハンドラーとサービスの統合テスト

### コアアーキテクチャパターン

**ドメイン駆動構造**: ドメイン別に分離されたビジネスロジック（株式操作、JQuants API統合）

**型安全なデータベース層**: SQLxを使用してコンパイル時に検証される全SQLクエリ、自動マッピング用`FromRow`派生

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

### データベーススキーマ

**株式モデル** (`migrations/0001_init.sql`):
```sql
stocks (
    date DATE NOT NULL,
    code VARCHAR(10) PRIMARY KEY,  -- 株式コード（1-10文字でバリデーション）
    name CITEXT NOT NULL,          -- 会社名（大文字小文字区別なし、1-100文字）
    market_category VARCHAR(50),   -- 市場カテゴリ（1-50文字）
    -- オプション：業界/規模分類フィールド
)
```

### 設定

**環境変数** (Shuttle SecretStore経由):
- `DATABASE_URL` - PostgreSQL接続文字列
- `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET` - OAuth認証情報
- `FRONTEND_URL` - CORS origin設定

**開発用secrets**: ローカル開発用`Secrets.dev.toml`

**データベース接続**: 最大5接続のPostgreSQLプール

### 日本語テキスト処理
- 大文字小文字区別なし日本語テキスト検索用CITEXTカラム
- 検索機能用文字幅変換ユーティリティ
- 標準化入力用半角→全角カタカナ変換

### 開発ガイドライン
- すべてのSQLクエリはSQLxでコンパイル時検証が必要
- すべてのリクエストモデルで入力バリデーション用`validator`クレートの派生を使用
- ビジネスロジックはドメイン固有のハンドラーモジュールに配置
- エラーレスポンスはエラーコード付き構造化JSON形式に従う
- CORSは特定のフロントエンドoriginのみに設定
- スキーマ変更にはデータベースマイグレーションが必要
- mod.rsは古い書き方なので非推奨

### テスト戦略
- すべてのHTTPエンドポイントの統合テスト
- ビジネスロジックとユーティリティの単体テスト
- 外部依存関係のモック（JQuants API）
- フィクスチャ付きテストデータベースセットアップ

### デプロイ注意事項
- ShuttleプラットフォームがPostgreSQLプロビジョニングを処理
- Shuttle SecretStoreでsecrets管理
- CORS originはデプロイ済みフロントエンドURLと一致させる必要がある
- デプロイ時にデータベースマイグレーションを適用