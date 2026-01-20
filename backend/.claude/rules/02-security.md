# バックエンドセキュリティルール

## 環境変数

```toml
# Secrets.toml (gitignore)
API_KEY = "xxx"
DATABASE_URL = "postgres://..."
```

Shuttle SecretStore で管理。

## 認証・認可

### Google OAuth2

- `oauth2` クレートを使用
- PKCE フローを推奨
- トークンは安全に保管

### セッション管理

- HTTPOnly Cookie を使用
- Secure フラグを有効化（本番環境）
- 適切な有効期限を設定

## 入力バリデーション

```rust
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct StockRequest {
    #[validate(length(equal = 4))]
    #[validate(regex = "[0-9]{4}")]
    pub code: String,
}
```

## SQLインジェクション対策

```rust
// 安全（パラメータバインディング）
sqlx::query!("SELECT * FROM stocks WHERE code = $1", code)

// 危険（文字列結合）- 絶対に使用しない
format!("SELECT * FROM stocks WHERE code = '{}'", code)
```

## CORS設定

```rust
let cors = CorsLayer::new()
    .allow_origin([
        "https://your-domain.vercel.app".parse().unwrap(),
    ])
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([CONTENT_TYPE, AUTHORIZATION])
    .allow_credentials(true);
```

開発時のみ `localhost` を許可:

```rust
#[cfg(debug_assertions)]
.allow_origin("http://localhost:8080".parse().unwrap())
```

## 依存関係のセキュリティ

```bash
cargo audit            # 脆弱性チェック
cargo update           # 依存関係更新
```

## ログ出力

```rust
use tracing::{info, warn, error};

// ユーザー情報は最小限に
info!(user_id = %user.id, "ログイン成功");

// エラーは詳細に（機密情報除く）
error!(error = ?e, "データベース接続エラー");
```

## エラーハンドリング

内部エラーの詳細をクライアントに返さない:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Internal(e) => {
                // ログには詳細を出力
                error!(error = ?e, "内部エラー");
                // クライアントには汎用メッセージ
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            // ...
        }
    }
}
```

## レートリミット

```rust
// J-Quants API 等の呼び出し制限を遵守
// 必要に応じてキャッシュを活用
```
