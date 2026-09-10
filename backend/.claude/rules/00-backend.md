# バックエンド開発ルール

## 技術スタック

- **Rust**: 最新 stable
- **Axum**: 0.8.x (Webフレームワーク)
- **SQLx**: 0.8.x (型安全DBアクセス)
- **Fly.io**: デプロイプラットフォーム
- **Neon**: PostgreSQL データベース
- **Tokio**: 非同期ランタイム
- **OAuth2**: Google認証

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

## Fly.io デプロイ

### ローカル開発

```bash
make run  # cargo run（環境変数は .env から読み込み）
```

### デプロイ

```bash
make deploy  # fly deploy
make status  # fly status
make logs    # fly logs
```

### 環境変数管理

- `.env` - ローカル開発用（gitignore対象）
- Fly.io Secrets で本番環境の環境変数を管理

```bash
# Fly.io に環境変数を設定
fly secrets set DATABASE_URL="postgres://..."
fly secrets set GOOGLE_CLIENT_ID="..."
fly secrets set GOOGLE_CLIENT_SECRET="..."
fly secrets set FRONTEND_URL="https://..."
fly secrets set BACKEND_URL="https://..."
```

### Docker ビルド

```bash
make docker-build  # docker build -t shoken-backend .
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

- `wiremock` によるHTTPモックと `testcontainers` による実DBテストを使用する

### 方針

- 単体テストは `#[cfg(test)] mod tests` を基本とする
- 統合テストは `backend/tests/` に配置する
- `cargo test -- --nocapture` で出力確認が可能
- カバレッジ確認が必要な場合は `cargo tarpaulin --out Html` を使用する

## HTTP クライアント

- `reqwest` を使用
- 外部API (J-Quants等) との通信

## コーディング規約

### フォーマット

- `#[rustfmt::skip]` の使用は禁止
- レイアウト調整が必要でも `cargo fmt` に従い、構造やテストデータの持ち方を見直して対応する

### Result/Option

- `?` 演算子を積極的に使用
- `unwrap()` は避け、適切なエラーハンドリングを行う

### 非同期

- `async/await` を使用
- `tokio::spawn` でバックグラウンドタスク

### ログ

- `tracing` クレートを使用
- 適切なログレベルを設定

## セキュリティ

### 認証・セッション

- Google OAuth2 を使用する
- セッション Cookie は `HttpOnly` を必須とする
- 本番環境では `Secure` を有効にする
- クロスオリジン認証では `SameSite=None`、ローカルでは `SameSite=Lax` を使い分ける

### 入力・DB

- 入力バリデーションは `validator` クレートで行う
- SQL は必ずパラメータバインディングを使用し、文字列連結で組み立てない

### CORS・機密情報

- CORS は許可 origin を明示し、必要時のみ credentials を許可する
- `.env` はローカル専用とし、本番環境は Fly.io Secrets で管理する
- トークン、個人情報、接続情報をログに出力しない
