# テストガイド

## テスト全体像

| 種別 | 対象 | ツール | 場所 |
|---|---|---|---|
| 単体テスト（フロント） | hooks / utils / api | Vitest + RTL | `frontend/src/**/__tests__/` |
| 単体テスト（バックエンド） | ハンドラー / サービス / ミドルウェア | cargo test | `backend/src/**/tests` |
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
```

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
| フロント | `deploy-frontend.yml` | lint / test / build |
| バックエンド | `deploy-backend.yml` | clippy / test / check / OpenAPI 同期確認 / tsc |

### セキュリティ監査（週次）

`.github/workflows/security-audit.yml` で毎週月曜日に自動実行:

```bash
cargo audit    # Rust 依存関係の脆弱性
npm audit --audit-level=high  # npm 依存関係の脆弱性
```

## テストカバレッジ確認（任意）

```bash
# バックエンド（tarpaulin 要インストール）
cd backend && cargo tarpaulin --out Html

# フロントエンド
cd frontend && npm test -- --coverage
```
