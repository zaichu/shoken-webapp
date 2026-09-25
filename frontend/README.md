# frontend

Leptos (CSR) のフロントエンドです。単独の Cargo パッケージで、Vercel に配信します。

## 前提

- Rust 1.96.0 と `wasm32-unknown-unknown` ターゲット
- Trunk 0.21.4
- Node.js 22 と npm

## セットアップ

```bash
npm ci
```

Playwright、axe、Tailwind の依存はこのディレクトリの `package.json` と `package-lock.json` で管理します。

## 開発と検証

```bash
trunk serve --port 8081
trunk build --release
cargo fmt --check
cargo clippy --all-targets --target wasm32-unknown-unknown -- -D warnings
cargo clippy --all-targets -- -D warnings
cargo test
npx playwright test --config playwright.leptos.config.ts
npx playwright test --config playwright.receipts.config.ts
npx playwright test --config playwright.vercel.config.ts
```

`playwright.leptos.config.ts` は `e2e/migrated/` の主要画面テストと `e2e/` の Leptos テストを実行します。`playwright.receipts.config.ts` は取引明細の E2E を実行します。E2E は `LEPTOS_E2E_PORT` でポートを指定できます。複数の worktree で並行実行するときは別々のポートを使ってください。

実 backend と DB を使うスモークテストは `scripts/run-real-backend-smoke.sh` を使います。DB → backend → Leptos の順に起動します。

## CSS

`style/input.css` を Trunk の pre_build フックで `style/output.css` に生成し、`index.html` から読み込みます。生成物は Git 管理外です。

## デプロイ

GitHub Actions の `deploy-frontend.yml` が Vercel CLI でビルド・配信します。`VERCEL_TOKEN`、`VERCEL_ORG_ID`、`VERCEL_PROJECT_ID` が必要です。本番デプロイは main の `workflow_dispatch` と `LEPTOS_PRODUCTION_ENABLED=true` で有効になります。
