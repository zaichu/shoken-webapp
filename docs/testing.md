# テストガイド

## テスト全体像

| 対象 | ツール | 場所 |
|---|---|---|
| Leptos 単体テスト | cargo test | `frontend/src/` |
| バックエンド単体・統合テスト | cargo test | `backend/src/`、`backend/tests/` |
| ブラウザ E2E | Playwright | `frontend/e2e/`、`frontend/e2e/migrated/` |
| Vercel 配信設定 | Playwright | `frontend/e2e-vercel/` |

## Leptos

`frontend/` で実行します。

```bash
npm ci
cargo fmt --check
cargo clippy --all-targets --target wasm32-unknown-unknown -- -D warnings
cargo clippy --all-targets -- -D warnings
cargo test
trunk build --release
```

Playwright は Trunk の開発サーバーを自動起動します。worktree ごとに異なるポートを使ってください。

```bash
env LEPTOS_E2E_PORT=8091 npx playwright test --config playwright.leptos.config.ts
env LEPTOS_E2E_PORT=8091 npx playwright test --config playwright.vercel.config.ts
```

## バックエンド

`backend/` で実行します。

```bash
cargo fmt --check
env SQLX_OFFLINE=true cargo clippy -- -D warnings
env SQLX_OFFLINE=true cargo test
```

DB を使う ignored test は Docker が必要です。外部 API を呼ぶ ignored test は API キーが必要です。

## CI

| ワークフロー | 必須チェック | 内容 |
|---|---|---|
| `frontend.yml` | Check, build, and E2E | fmt、clippy、cargo test、release build、Playwright |
| `deploy-backend.yml` | Test & Build | clippy、cargo test、OpenAPI 同期、Docker build |
| `pr-gate.yml` | PR gate | Issue の紐付け、レビューコメント |

依存関係の監査は `security-audit.yml` で Cargo と `frontend/` の npm を対象に実行します。
