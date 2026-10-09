# バックエンド開発ルール

## 技術スタック

- **Rust**: 最新 stable
- **Axum**: 0.8.x (Webフレームワーク。Cloudflare Workers 上で `worker` クレート経由で動作)
- **tokio-postgres**: DB アクセス（本番は Hyperdrive 経由）
- **Cloudflare Workers**: 唯一のデプロイ・実行プラットフォーム
- **Neon**: PostgreSQL データベース
- **OAuth2**: Google認証

## ディレクトリ構成

親モジュールは `foo.rs`、子モジュールは `foo/bar.rs` に置く（`mod.rs` は使わない）。

```
backend/src/
├── handlers/       # ドメイン別ハンドラー
├── models/         # データモデル・バリデーション
├── extractors/     # カスタム Axum エクストラクター
├── services/       # ビジネスロジック
├── middleware/     # CORS・レート制限などのミドルウェア
├── config/         # 環境ごとの設定
├── bin/            # 補助バイナリ（migrate / generate_openapi。ホスト実行専用）
├── worker_entry.rs # Workers エントリーポイント（fetch/scheduled イベントとルーティング）
├── lib.rs          # モジュール定義
├── state.rs        # AppState (DB/secrets/クライアント)
├── db.rs           # DB アクセス層（wasm は Hyperdrive、ホストは migrate/テスト用直接接続）
├── openapi.rs      # utoipa の OpenAPI 定義
└── errors.rs       # 統一エラーハンドリング
```

## DB アクセスとマイグレーション

クエリは `tokio-postgres` の生 SQL で書く。Worker では Hyperdrive binding 経由、
ホスト側（migrate bin・DB 統合テスト）は `DATABASE_URL` で直接接続する。

```bash
# マイグレーション実行（ホスト側で直接接続）
make migrate        # DATABASE_URL 環境変数または backend/.env を参照
make migrate-local  # ローカル Docker DB 向け
```

マイグレーションファイルは `migrations/` に配置。運用は `migrations/README.md` を参照。

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

## Cloudflare Workers デプロイ

### ローカル開発

```bash
make run  # wrangler dev --env dev（http://localhost:8787、secrets は backend/.dev.vars）
```

### デプロイ

`main` へのマージで `deploy-cloudflare-worker.yml` が自動デプロイする。手動反映は:

```bash
gh workflow run deploy-cloudflare-worker.yml
```

### 環境変数管理

- `wrangler.toml` の vars/bindings - 設定の正本（ローカルは `[env.dev.*]`）
- `.dev.vars` - ローカル開発用シークレット（gitignore対象、雛形は `.dev.vars.example`）
- `.env` - `cargo run --bin migrate` が読む `DATABASE_URL` 専用（gitignore対象）
- 本番の secrets/vars は `deploy-cloudflare-worker.yml` が GitHub secrets/vars から `wrangler secret put` / `wrangler deploy --var` で注入する

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

`tower-http` の CorsLayer を使用（`config/cors.rs` の `build_cors_layer`）:

```rust
let cors = CorsLayer::new()
    // 許可一覧との完全一致のみ通す
    .allow_origin(AllowOrigin::predicate(|origin, _| /* 許可一覧と照合 */))
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
    // CONTENT_TYPE / ACCEPT / ORIGIN / AUTHORIZATION のみ許可
    .allow_headers(allowed_headers)
    .allow_credentials(true)
    .max_age(Duration::from_secs(3600));
```

`allow_headers(Any)` やワイルドカード origin は使わない。本番では localhost オリジンを除外する。

## テスト

### 実行

```bash
make test  # cargo test
```

### モック

- `testcontainers` による実DBテストを使用する（外部 HTTP 呼び出しは Worker 専用のため HTTP モックは置かない）

### 方針

- 単体テストは `#[cfg(test)] mod tests` を基本とする
- 統合テストは `backend/tests/` に配置する
- `cargo test -- --nocapture` で出力確認が可能
- カバレッジ確認が必要な場合は `cargo tarpaulin --out Html` を使用する

## HTTP クライアント

- Workers ランタイムの Fetch API（`worker::Fetch`）を使用
- 外部API (J-Quants、Google OAuth トークン交換等) との通信は wasm ターゲット専用の実装に置く

## コーディング規約

### フォーマット

- `#[rustfmt::skip]` の使用は禁止
- レイアウト調整が必要でも `cargo fmt` に従い、構造やテストデータの持ち方を見直して対応する

### Result/Option

- `?` 演算子を積極的に使用
- `unwrap()` は避け、適切なエラーハンドリングを行う

### 非同期

- `async/await` を使用
- Workers isolate では `tokio::spawn` は使えない。バックグラウンド相当は `ctx.wait_until` か `[triggers]` の scheduled イベントで実現する

### ログ

- `tracing` クレート（worker では tracing-wasm 経由）と `worker::console_log!` を使用
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
- `.dev.vars`・`.env` はローカル専用とし、本番環境は Cloudflare Workers の secrets/vars で管理する
- トークン、個人情報、接続情報をログに出力しない
