# テストガイド

## テスト全体像

| 対象 | ツール | 場所 |
|---|---|---|
| Leptos 単体テスト | cargo test | `frontend/src/` |
| バックエンド単体・統合テスト | cargo test | `backend/src/`、`backend/tests/` |
| ブラウザ E2E | Playwright | `frontend/e2e/<機能>/` (`deploy/` を除く) |
| Vercel 配信設定 | Playwright | `frontend/e2e/deploy/` |

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

| ワークフロー | チェック名 | 内容 |
|---|---|---|
| `frontend.yml` | Check, build, and E2E | fmt、CSS トークン、テスト配置、clippy、cargo test、release build、Playwright（E2E 3 シャード + Vercel 配信） |
| `deploy-backend.yml` | Test & Build | clippy、cargo test、shared クレート、OpenAPI 同期、Docker build |
| `deploy-frontend.yml` | Build and deploy to Vercel | CI からの呼び出しは `vercel-dist` を再利用。PR は `pull_request` で独立ビルド・preview（権限のある投稿者のみ、CI 成功待ちなし）。直接の手動起動は main 限定の独立ビルド |
| `mutation-testing.yml` | cargo-mutants (backend/frontend/shared diff) | PR 差分の変異テスト |
| `scripts-test.yml` | Test gate scripts | ゲート系スクリプトのテスト |
| `security-audit.yml` | cargo audit (Rust) / npm audit (Node.js) | 依存関係の脆弱性監査 |
| `pr-gate.yml` | Evaluate PR gate | Issue の紐付け、未解決レビューコメント |

`backend-db-integration.yml` は定期・手動実行のみで、Docker 必須の ignored DB 統合テストを実行します。
