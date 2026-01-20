# バックエンド開発ルール

## 技術スタック

- **Rust**: 最新 stable
- **Axum**: 0.8.x (Webフレームワーク)
- **SQLx**: 0.8.x (型安全DBアクセス)
- **Shuttle**: デプロイプラットフォーム
- **PostgreSQL**: データベース
- **Tokio**: 非同期ランタイム

## ディレクトリ構成

```
backend/src/
├── handlers/     # ドメイン別ハンドラー
├── models/       # データモデル・バリデーション
├── extractors/   # カスタム Axum エクストラクター
├── services/     # ビジネスロジック
├── main.rs       # エントリーポイント
├── lib.rs        # モジュール定義
├── state.rs      # AppState (DB/secrets/クライアント)
└── errors.rs     # 統一エラーハンドリング
```

## SQLx

### コンパイル時クエリ検証

SQLx はコンパイル時にSQLクエリを検証:

```rust
let user = sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE id = $1",
    user_id
)
.fetch_one(&pool)
.await?;
```

### マイグレーション

```bash
# マイグレーション作成
sqlx migrate add <name>

# マイグレーション実行
sqlx migrate run

# オフラインモード用準備
make sqlx-prepare
```

マイグレーションファイルは `migrations/` に配置。

## エラーハンドリング

`errors.rs` で統一エラー型を定義:

```rust
pub enum AppError {
    NotFound,
    BadRequest(String),
    Internal(String),
    // ...
}

impl IntoResponse for AppError {
    // Axum レスポンスへ変換
}
```

## 認証

- Google OAuth2 を使用
- `oauth2` クレート
- セッション管理は `handlers/` 内で実装

## Shuttle デプロイ

### ローカル開発

```bash
make run  # shuttle run --secrets Secrets.dev.toml --port 3001
```

### デプロイ

```bash
make deploy        # クリーンデプロイ
make deploy-dirty  # ダーティデプロイ（未コミット変更あり）
```

### Secrets管理

- `Secrets.dev.toml` - ローカル開発用
- `Secrets.toml` - 本番用（gitignore対象）
- Shuttle SecretStore で注入

```rust
#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> ShuttleAxum {
    // ...
}
```

## バリデーション

`validator` クレートを使用:

```rust
#[derive(Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub name: String,
}
```

## CORS設定

`tower-http` の CorsLayer を使用:

```rust
let cors = CorsLayer::new()
    .allow_origin(/* ... */)
    .allow_methods([Method::GET, Method::POST])
    .allow_headers(Any);
```

## テスト

### 実行

```bash
make test  # cargo test
```

### モック

- `mockall` クレートを使用
- `tokio-test` で非同期テスト

## HTTP クライアント

- `reqwest` を使用
- 外部API (J-Quants等) との通信

## コーディング規約

### Result/Option

- `?` 演算子を積極的に使用
- `unwrap()` は避け、適切なエラーハンドリングを行う

### 非同期

- `async/await` を使用
- `tokio::spawn` でバックグラウンドタスク

### ログ

- `tracing` クレートを使用
- 適切なログレベルを設定
