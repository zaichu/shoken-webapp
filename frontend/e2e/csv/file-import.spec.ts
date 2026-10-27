import type { Page } from '@playwright/test';
import { expect, test, json, paginated } from '../support/test';
import {
  csvRow,
  importCsvFile,
  mockCsvKindStats,
  rowStatus,
  savedCount,
} from '../support/csv-page';
import * as path from 'path';

/**
 * 行の「⋯」→「ファイルから取り込み」は OS の選択を直接開き、
 * 選んだらプレビューなしでそのまま保存する。結果は行の状態に出る。
 * 安全装置は最新取得と同じ(preview → 有効0件・行エラーで止める → 激減確認 → upload)。
 */

function fixtureDir(): string {
  return path.resolve(test.info().project.testDir, 'fixtures/csv');
}

const USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

async function mockSessionAndLists(page: Page, counts: Record<string, number>) {
  await page.route(/\/api\/v1\/session$/, (route) => route.fulfill(json(USER)));
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([]))),
  );
  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([]))),
  );
  for (const re of [
    /\/api\/v1\/dividends(?:\?.*)?$/,
    /\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/,
    /\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/,
  ]) {
    await page.route(re, (route) => route.fulfill(json(paginated([]))));
  }
  await mockCsvKindStats(page, { counts: () => counts });
}

async function mockPreview(
  page: Page,
  pattern: RegExp,
  response: { total_rows: number; valid_rows: number; errors?: { row: number; message: string }[] },
) {
  await page.route(pattern, (route) =>
    route.fulfill(
      json({
        total_rows: response.total_rows,
        valid_rows: response.valid_rows,
        errors: response.errors ?? [],
        rows: [],
      }),
    ),
  );
}

async function mockUpload(
  page: Page,
  pattern: RegExp,
  response: { inserted: number; skipped?: number; errors?: unknown[] },
) {
  await page.route(pattern, (route) =>
    route.fulfill(
      json({
        inserted: response.inserted,
        skipped: response.skipped ?? 0,
        errors: response.errors ?? [],
      }),
    ),
  );
}

test('資産管理1種は選んだらそのまま保存し行の状態が更新される', async ({ page }) => {
  await mockSessionAndLists(page, { assetbalance: 0 });
  await mockPreview(page, /\/api\/v1\/asset-balance-import-validations$/, {
    total_rows: 2,
    valid_rows: 2,
  });
  await mockUpload(page, /\/api\/v1\/asset-balance-imports$/, { inserted: 2 });
  // 激減確認の保存済み件数は 0 件(確認なし)
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([]))),
  );

  await page.goto('/csv');
  await importCsvFile(page, 'assetbalance', path.join(fixtureDir(), 'assetbalance-base.csv'));

  await expect(rowStatus(page, 'assetbalance')).toContainText('に更新', { timeout: 10000 });
  await expect(rowStatus(page, 'assetbalance')).toHaveAttribute('data-status', 'fresh');
});

test('取引明細1種は選んだらそのまま保存し行の状態が更新される', async ({ page }) => {
  await mockSessionAndLists(page, { dividend: 0 });
  await mockPreview(page, /\/api\/v1\/dividend-import-validations$/, {
    total_rows: 2,
    valid_rows: 2,
  });
  await mockUpload(page, /\/api\/v1\/dividend-imports$/, { inserted: 2 });

  await page.goto('/csv');
  await importCsvFile(page, 'dividend', path.join(fixtureDir(), 'dividend-base.csv'));

  await expect(rowStatus(page, 'dividend')).toContainText('に更新', { timeout: 10000 });
});

test('全件置換で行エラーがあれば保存せず理由を行に出す', async ({ page }) => {
  await mockSessionAndLists(page, { assetbalance: 12 });
  await mockPreview(page, /\/api\/v1\/asset-balance-import-validations$/, {
    total_rows: 2,
    valid_rows: 1,
    errors: [{ row: 2, message: '株数が不正です' }],
  });
  let uploaded = false;
  await page.route(/\/api\/v1\/asset-balance-imports$/, (route) => {
    uploaded = true;
    return route.fulfill(json({ inserted: 0, skipped: 0, errors: [] }));
  });

  await page.goto('/csv');
  await importCsvFile(page, 'assetbalance', path.join(fixtureDir(), 'assetbalance-base.csv'));

  await expect(csvRow(page, 'assetbalance')).toContainText('取り込まなかった', {
    timeout: 10000,
  });
  await expect(csvRow(page, 'assetbalance')).toContainText('エラーのある行が含まれます');
  expect(uploaded).toBe(false);
});

test('件数激減は確認を出してはい・いいえで分岐する', async ({ page }) => {
  await mockSessionAndLists(page, { assetbalance: 100 });
  await mockPreview(page, /\/api\/v1\/asset-balance-import-validations$/, {
    total_rows: 10,
    valid_rows: 10,
  });
  // 保存済み 100 件 → 激減確認が出る
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => {
    const url = new URL(route.request().url());
    if (url.searchParams.get('per_page') === '1') {
      return route.fulfill(json({ data: [], total: 100, page: 1, per_page: 1 }));
    }
    return route.fulfill(json(paginated([])));
  });
  await mockUpload(page, /\/api\/v1\/asset-balance-imports$/, { inserted: 10 });

  await page.goto('/csv');
  await importCsvFile(page, 'assetbalance', path.join(fixtureDir(), 'assetbalance-base.csv'));

  const confirm = page.getByTestId('csv-sync-confirm');
  await expect(confirm).toBeVisible({ timeout: 10000 });
  await expect(rowStatus(page, 'assetbalance')).toContainText('確認待ち');

  // いいえ → 取り込まず理由を行に出す
  await page.getByTestId('csv-sync-confirm-no').click();
  await expect(csvRow(page, 'assetbalance')).toContainText('取り込まなかった', {
    timeout: 10000,
  });
  await expect(confirm).toBeHidden();
});

test('ファイル選択のキャンセルは何も出さない', async ({ page }) => {
  await mockSessionAndLists(page, { dividend: 0 });

  await page.goto('/csv');
  await expect(csvRow(page, 'dividend')).toContainText('まだ取得していません');

  await csvRow(page, 'dividend').getByRole('button', { name: 'メニュー' }).click();
  const chooser = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: 'ファイルから取り込み' }).click();
  await (await chooser).setFiles([]);

  await expect(savedCount(page, 'dividend')).toContainText('0件');
  await expect(csvRow(page, 'dividend')).toContainText('まだ取得していません');
});
