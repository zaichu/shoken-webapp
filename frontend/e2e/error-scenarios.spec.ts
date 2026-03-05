/**
 * エラーシナリオ E2E テスト（API モック使用）
 *
 * page.route() でバックエンド API をモックし、
 * 失敗系・境界系の UI 挙動を検証する。
 * フロントエンドサーバー（8080）のみ起動で実行可能。
 *
 * 使用方法:
 *   cd frontend && npx playwright test error-scenarios.spec.ts
 */
import { test, expect, type Page } from '@playwright/test';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const FIXTURE_DIR = path.join(__dirname, '__fixtures__/csv');

const API_BASE = process.env.VITE_SHOKEN_WEBAPI_API_URL ?? 'http://127.0.0.1:3001';
const MOCK_USER = { id: 1, email: 'test@example.com', name: 'テストユーザー' };

async function setupCommonMocks(page: Page) {
  await page.route(`${API_BASE}/auth/me`, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(MOCK_USER) }),
  );
  await page.route(`${API_BASE}/dividends`, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
  await page.route(`${API_BASE}/domestic-stocks`, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
  await page.route(`${API_BASE}/mutualfunds`, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
  await page.route(`${API_BASE}/asset-balances`, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
}

// =====================================================================
// 一覧取得失敗
// =====================================================================

test('配当金一覧取得が 401 のときエラー Alert が表示される', async ({ page }) => {
  await setupCommonMocks(page);
  // 後から登録したハンドラが優先されるため、401 で上書き
  await page.route(`${API_BASE}/dividends`, (route) =>
    route.fulfill({
      status: 401,
      contentType: 'application/json',
      body: JSON.stringify({ error: '認証が必要です' }),
    }),
  );

  await page.goto('/receipts');
  await page.waitForLoadState('networkidle');

  await expect(
    page.getByRole('alert').filter({ hasText: /エラー/ }),
  ).toBeVisible({ timeout: 10000 });
});

test('資産管理一覧取得が 401 のときエラー Alert が表示される', async ({ page }) => {
  await setupCommonMocks(page);
  await page.route(`${API_BASE}/asset-balances`, (route) =>
    route.fulfill({
      status: 401,
      contentType: 'application/json',
      body: JSON.stringify({ error: '認証が必要です' }),
    }),
  );

  await page.goto('/assetbalance');
  await page.waitForLoadState('networkidle');

  await expect(
    page.getByRole('alert').filter({ hasText: /エラー/ }),
  ).toBeVisible({ timeout: 10000 });
});

// =====================================================================
// CSV 行レベルエラー
// =====================================================================

test('CSV プレビューで行エラーが返ったとき warning Alert に一覧表示される', async ({ page }) => {
  await setupCommonMocks(page);
  // プレビューAPI: 行エラーを含む正常レスポンス（HTTP 200）
  await page.route(`${API_BASE}/dividends/csv/preview`, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        total_rows: 3,
        valid_rows: 1,
        errors: [
          { row: 2, message: '受取金額が数値ではありません' },
          { row: 3, message: '受取日の形式が不正です' },
        ],
        rows: [],
      }),
    }),
  );

  await page.goto('/receipts');
  await page.waitForLoadState('networkidle');

  // ファイル選択でプレビュー API が自動的に呼ばれる
  await page.locator('#csv-file-input').setInputFiles(
    path.join(FIXTURE_DIR, 'dividend-base.csv'),
  );

  // warning Alert に行エラー一覧が表示されることを確認
  const previewRegion = page.locator('[role="status"][aria-live="polite"]').first();
  await expect(previewRegion).toContainText('2件エラー', { timeout: 10000 });
  await expect(previewRegion).toContainText('2行目');
  await expect(previewRegion).toContainText('3行目');
});
