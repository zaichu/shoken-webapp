/**
 * エラーシナリオ E2E テスト（API モック使用）
 *
 * page.route() でバックエンド API をモックし、
 * 失敗系・境界系の UI 挙動を検証する。
 * Leptos の開発サーバーで実行する。
 */
import type { Page } from '@playwright/test';
import { expect, test, MOCK_USER } from '../support/test';
import { csvRow, importCsvFile, rowStatus } from '../support/csv-page';
import * as path from 'path';
const FIXTURE_DIR = path.resolve(process.cwd(), 'e2e/fixtures/csv');

// ホスト非依存のパターン（VITE_SHOKEN_WEBAPI_API_URL の値に関わらず一致する）
const ROUTES = {
  authMe: /\/api\/v1\/session$/,
  dividends: /\/api\/v1\/dividends(?:\?.*)?$/,
  dividendsCsvPreview: /\/api\/v1\/dividend-import-validations$/,
  domesticStocks: /\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/,
  mutualfunds: /\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/,
  assetBalances: /\/api\/v1\/asset-balances(?:\?.*)?$/,
};

async function setupCommonMocks(page: Page) {
  await page.route(ROUTES.authMe, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(MOCK_USER) }),
  );
  await page.route(ROUTES.dividends, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
  await page.route(ROUTES.domesticStocks, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
  await page.route(ROUTES.mutualfunds, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
  await page.route(ROUTES.assetBalances, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
}

// =====================================================================
// 一覧取得失敗
// =====================================================================

test('配当金一覧取得が 401 のときエラー Alert が表示される', async ({ page }) => {
  await setupCommonMocks(page);
  // 後から登録したハンドラが優先されるため、401 で上書き
  await page.route(ROUTES.dividends, (route) =>
    route.fulfill({
      status: 401,
      contentType: 'application/json',
      body: JSON.stringify({ error: '認証が必要です' }),
    }),
  );

  await page.goto('/receipts');

  await expect(
    page.getByRole('alert').filter({ hasText: /エラー/ }),
  ).toBeVisible({ timeout: 10000 });
});

test('資産管理一覧取得が 401 のときエラー Alert が表示される', async ({ page }) => {
  await setupCommonMocks(page);
  await page.route(ROUTES.assetBalances, (route) =>
    route.fulfill({
      status: 401,
      contentType: 'application/json',
      body: JSON.stringify({ error: '認証が必要です' }),
    }),
  );

  await page.goto('/assetbalance');

  await expect(
    page.getByRole('alert').filter({ hasText: /エラー/ }),
  ).toBeVisible({ timeout: 10000 });
});

// =====================================================================
// CSV 行レベルエラー
// =====================================================================

test('検証 API が失敗したとき行に失敗理由が出る', async ({ page }) => {
  await setupCommonMocks(page);
  await page.route(ROUTES.dividendsCsvPreview, (route) =>
    route.fulfill({
      status: 500,
      contentType: 'application/json',
      body: JSON.stringify({ error: 'サーバーエラーが発生しました' }),
    }),
  );

  await page.goto('/csv');
  await importCsvFile(page, 'dividend', path.join(FIXTURE_DIR, 'dividend-base.csv'));

  await expect(rowStatus(page, 'dividend')).toHaveAttribute('data-status', 'failed', {
    timeout: 10000,
  });
});

test('有効な行が0件のときは取り込まず理由を行に出す', async ({ page }) => {
  await setupCommonMocks(page);
  await page.route(ROUTES.dividendsCsvPreview, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        total_rows: 2,
        valid_rows: 0,
        errors: [
          { row: 1, message: '受取金額が数値ではありません' },
          { row: 2, message: '受取日の形式が不正です' },
        ],
        rows: [],
      }),
    }),
  );

  await page.goto('/csv');
  await importCsvFile(page, 'dividend', path.join(FIXTURE_DIR, 'dividend-base.csv'));

  await expect(csvRow(page, 'dividend')).toContainText('取り込まなかった', { timeout: 10000 });
  await expect(csvRow(page, 'dividend')).toContainText('取り込める行がありませんでした');
});

