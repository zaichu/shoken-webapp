import { expect, test, type Page } from '@playwright/test';
import * as path from 'path';

/**
 * CSV取込結果・プレビューのエラー詳細表示の E2E。
 * - 取込結果の行エラーは <details> 折りたたみ(初期は閉、summary クリックで開く)
 * - プレビューの行エラーは操作なしで常時表示
 */

function fixtureDir(): string {
  return path.resolve(
    test.info().project.testDir,
    '../../frontend/e2e/__fixtures__/csv',
  );
}

const USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const KDDI = {
  settlement_date: '2025-12-08',
  product: '国内株式',
  account: '特定・一般',
  security_code: '9433',
  security_name: 'ＫＤＤＩ',
  unit_price: 40,
  shares: 600,
  dividends_before_tax: 24000,
  taxes: 4875,
  net_amount_received: 19125,
};
const NTT = {
  settlement_date: '2025-12-08',
  product: '国内株式',
  account: '特定・一般',
  security_code: '9432',
  security_name: '日本電信電話',
  unit_price: 5,
  shares: 2000,
  dividends_before_tax: 10000,
  taxes: 2031,
  net_amount_received: 7969,
};
const PREVIEW_ROWS = [KDDI, NTT];

const ROW_ERRORS = [
  { row: 2, message: '銘柄コードが不正です' },
  { row: 4, message: '株数が不正です' },
];

interface RowError {
  row: number;
  message: string;
}

interface MockOptions {
  previewErrors?: RowError[];
  importErrors?: RowError[];
}

async function setupMocks(page: Page, options: MockOptions) {
  const db: unknown[] = [];
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(USER),
    }),
  );
  const listBody = (data: unknown[]) =>
    JSON.stringify({ data, total: data.length, page: 1, per_page: data.length });
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: listBody(db),
    }),
  );
  for (const pathRe of [
    /\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/,
    /\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/,
  ]) {
    await page.route(pathRe, (route) =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: listBody([]),
      }),
    );
  }
  const previewErrors = options.previewErrors ?? [];
  await page.route(/\/api\/v1\/dividend-import-validations$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        total_rows: PREVIEW_ROWS.length + previewErrors.length,
        valid_rows: PREVIEW_ROWS.length,
        errors: previewErrors,
        rows: PREVIEW_ROWS,
      }),
    }),
  );
  const importErrors = options.importErrors ?? [];
  await page.route(/\/api\/v1\/dividend-imports$/, (route) => {
    db.push(...PREVIEW_ROWS);
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        inserted: PREVIEW_ROWS.length,
        skipped: 0,
        errors: importErrors,
      }),
    });
  });
}

async function selectCsvAndSave(page: Page, validRows: number) {
  await page.goto('/receipts');
  const fileInput = page.getByTestId('csv-file-input');
  await expect(fileInput).toBeAttached();
  await fileInput.setInputFiles(path.join(fixtureDir(), 'dividend-base.csv'));
  await expect(
    page.getByText(`${validRows}件 追加で保存されます`),
  ).toBeVisible();
  await page
    .getByRole('button', { name: `${validRows}件 追加で保存` })
    .click();
}

test.describe('取込結果のエラー詳細表示', () => {
  test('取込エラーは <details> 折りたたみ(初期は閉)で summary クリックで開く', async ({
    page,
  }) => {
    await setupMocks(page, { importErrors: ROW_ERRORS });
    await selectCsvAndSave(page, PREVIEW_ROWS.length);

    const notice = page.getByTestId('csv-save-result-notice');
    await expect(notice).toBeVisible();
    await expect(notice).toContainText(`${PREVIEW_ROWS.length}件反映`);
    await expect(notice).toContainText(`${ROW_ERRORS.length}件エラー`);

    const details = notice.locator('details');
    await expect(details).toHaveCount(1);
    // デフォルトは閉: open 属性・プロパティともに無い
    await expect(details).toHaveJSProperty('open', false);

    const summary = details.locator('summary');
    await expect(summary).toBeVisible();
    await expect(summary).toHaveText('エラー詳細を表示');

    const items = details.locator('li');
    await expect(items).toHaveCount(ROW_ERRORS.length);
    await expect(items.first()).toBeHidden();
    await expect(items.nth(1)).toBeHidden();

    await summary.click();
    await expect(details).toHaveJSProperty('open', true);
    await expect(items.first()).toBeVisible();
    await expect(items.first()).toHaveText('2行目: 銘柄コードが不正です');
    await expect(items.nth(1)).toHaveText('4行目: 株数が不正です');
  });

  test('プレビューの行エラーは操作なしで表示される', async ({ page }) => {
    await setupMocks(page, { previewErrors: ROW_ERRORS });
    await page.goto('/receipts');
    const fileInput = page.getByTestId('csv-file-input');
    await expect(fileInput).toBeAttached();
    await fileInput.setInputFiles(path.join(fixtureDir(), 'dividend-base.csv'));

    const previewAlert = page.locator('section[role="status"]', {
      hasText: '追加で保存されます',
    });
    await expect(previewAlert).toContainText(
      `${PREVIEW_ROWS.length}件 追加で保存されます`,
    );
    await expect(previewAlert).toContainText(`${ROW_ERRORS.length}件エラー`);

    const items = previewAlert.locator('li');
    await expect(items).toHaveCount(ROW_ERRORS.length);
    // 折りたたみ操作なしで最初から見える
    await expect(items.first()).toBeVisible();
    await expect(items.first()).toHaveText('2行目: 銘柄コードが不正です');
    await expect(items.nth(1)).toBeVisible();
    await expect(items.nth(1)).toHaveText('4行目: 株数が不正です');
  });
});
