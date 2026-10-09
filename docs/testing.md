# テストガイド

## テスト全体像

| 対象 | ツール | 場所 |
|---|---|---|
| Leptos 単体テスト | cargo test | `frontend/src/` |
| バックエンド単体・統合テスト | cargo test | `backend/src/`、`backend/tests/` |
| ブラウザ E2E | Playwright | `frontend/e2e/<機能>/` (`deploy/` を除く) |
| Pages 配信設定 | Playwright | `frontend/e2e/deploy/` |

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
env LEPTOS_E2E_PORT=8091 npx playwright test --config playwright.deploy.config.ts
```

## バックエンド

`backend/` で実行します。

```bash
cargo fmt --check
env SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings
env SQLX_OFFLINE=true cargo test
```

DB を使う ignored test は Docker が必要です。外部 API を呼ぶ ignored test は API キーが必要です。

## CI

| ワークフロー | チェック名 | 内容 |
|---|---|---|
| `frontend.yml` | Check, build, and E2E | fmt、CSS トークン、テスト配置、clippy、cargo test、release build、Playwright（E2E 3 シャード + Pages 配信）、Cloudflare Pages デプロイ |
| `backend.yml` | Test & Build | fmt、clippy（全ターゲット）、cargo test、shared クレート、OpenAPI 同期、Worker build |
| `mutation-testing.yml` | cargo-mutants (backend/frontend/shared diff) | PR 差分の変異テスト |
| `scripts-test.yml` | Test gate scripts | ゲート系スクリプトのテスト |
| `security-audit.yml` | cargo audit (Rust) / npm audit (Node.js) | 依存関係の脆弱性監査 |
| `pr-gate.yml` | Evaluate PR gate | Issue の紐付け、未解決レビューコメント |

Markdown 文書だけの変更では、アプリのビルド・E2E・配信を省略します。PR の必須チェック名は維持し、Frontend CI の集約チェックは変更判定の成功を確認します。手動実行は変更の有無によらず検証します。`shared/Cargo.toml` の変更は既存の Rust audit の対象です。

`backend-db-integration.yml` は定期・手動実行のみで、Docker 必須の ignored DB 統合テストを実行します。
