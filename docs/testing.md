# テストガイド

## テスト全体像

| 種別 | 対象 | ツール | 場所 |
|---|---|---|---|
| 単体テスト（フロント） | hooks / utils / api | Vitest + RTL | `frontend/src/**/__tests__/` |
| 単体テスト（バックエンド） | ハンドラー / サービス / ミドルウェア | cargo test | 各ファイル末尾の `#[cfg(test)] mod tests` |
| 統合テスト（バックエンド） | DB アクセス（testcontainers） | cargo test | `backend/tests/` |
| E2E テスト（正常系） | 主要ページ / CSV CRUD | Playwright | `frontend/e2e/` |
| E2E テスト（失敗系） | API 401 / CSV 行エラー | Playwright + page.route() | `frontend/e2e/error-scenarios.spec.ts` |

## フロントエンドテスト

```bash
cd frontend

# 全テスト実行
npm test

# ウォッチモード
npm run test:watch

# 単一ファイル指定
npm test -- src/lib/api/__tests__/client.test.ts
```

### テスト方針

- ユーザー視点で書く: `getByRole` / `getByText` を優先、`data-testid` は最終手段
- 非同期処理: `waitFor` または `findBy*` を使用
- テストファイルの配置: 各機能配下の `__tests__/` ディレクトリ
- フィクスチャ: `__fixtures__/` に配置

## バックエンドテスト

```bash
cd backend

# 全テスト実行
cargo test

# lib テストのみ
cargo test --lib

# 特定モジュールのテスト
cargo test --lib -- middleware::tests

# Docker が必要な統合テスト（通常は ignore）
cargo test -- --ignored
```

### テスト方針

- 単体テスト: `#[cfg(test)] mod tests` を各ファイルの末尾に配置
- DB が不要なテスト: `connect_pool_lazy` を使用して遅延初期化
- DB が必要なテスト: testcontainers を使用し `#[ignore = "requires Docker"]` を付与
- 外部 API 呼び出しテスト: `#[ignore = "requires API key"]` を付与

## E2E テスト

### 前提

1. ローカル環境を起動（DB → バックエンド → フロントエンド の順）
2. ログイン状態を保存（初回のみ）

```bash
# 環境起動 + 認証保存 + スクリーンショット取得
./scripts/run-ui-e2e.sh --save-auth

# 主要ページのみ
./scripts/run-ui-e2e.sh --skip-csv

# CSV CRUD フローのみ
./scripts/run-ui-e2e.sh --skip-main

# 実 backend 接続の配当金一覧スモーク
./scripts/run-real-backend-receipts-smoke.sh
```

### 実 backend 接続スモーク

`./scripts/run-real-backend-receipts-smoke.sh` は local PostgreSQL、backend、frontend を起動し、認証済み session と配当金 fixture を投入してから Playwright で `/receipts` を確認します。

このスモークは `page.route()` の mock を使わないため、backend の query deserialize、認証 cookie、CORS、migration、DB schema の不整合を検出できます。
検証対象は `/api/v1/dividends?per_page=1000&page=1` と summary/facets 付き API、および配当金タブの銘柄表示です。

### 失敗系テスト（API モック使用）

```bash
# フロントエンドサーバーのみ起動でテスト可能
cd frontend && npm run dev -- --host 127.0.0.1 --port 8080 &
cd frontend && npx playwright test e2e/error-scenarios.spec.ts
```

`error-scenarios.spec.ts` は `page.route()` でバックエンド API をモックするため、
実際のバックエンドサーバーは不要です。

### E2E 用スクリーンショット

スクリーンショットは `.playwright-mcp/` に保存されます（gitignore 対象）。

## CI での実行

GitHub Actions で以下が自動実行されます（PR 時）:

| ワークフロー | ファイル | ステップ |
|---|---|---|
| フロント | `deploy-frontend.yml` | lint / typecheck / test（1回） / build / E2E |
| バックエンド | `deploy-backend.yml` | clippy / test / OpenAPI 同期確認 / frontend typecheck |

### フロントエンド CI の方針

- PR CI では unit test を **1回だけ** 実行する。
- flaky detection（unit test 2回目実行）は `workflow_dispatch` で手動トリガーする。
  - GitHub Actions の「Run workflow」ボタン、または `gh workflow run deploy-frontend.yml` で実行できる。
- E2E テストは PR CI に常時含める。

### セキュリティ監査（週次）

`.github/workflows/security-audit.yml` で毎週月曜日に自動実行:

```bash
cargo audit    # Rust 依存関係の脆弱性
(cd frontend && node ../scripts/npm-audit-allowlist.mjs)  # npm 依存関係の脆弱性
```

## テストカバレッジ確認（任意）

```bash
# バックエンド（tarpaulin 要インストール）
(cd backend && cargo tarpaulin --out Html)

# フロントエンド
(cd frontend && npm test -- --coverage)
```
