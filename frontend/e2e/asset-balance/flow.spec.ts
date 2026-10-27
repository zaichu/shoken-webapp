import path from 'node:path';
import type { Page } from '@playwright/test';
import { expect, test, json, mockSession } from '../support/test';
import { mockCsvKindStats, openCsvDelete, savedCount } from '../support/csv-page';

const CSV_FIXTURE_PATH = path.resolve(process.cwd(), 'e2e/fixtures/csv/assetbalance-base.csv');

const ROUTES = {
  assetBalances: /\/api\/v1\/asset-balances(?:\?.*)?$/,
  assetBalancePreview: /\/api\/v1\/asset-balance-import-validations$/,
  assetBalanceUpload: /\/api\/v1\/asset-balance-imports$/,
  dividendPerShareBatch: /\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/,
};

const MOCK_ASSET = {
  id: 'asset-balance-1',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '9904',
  security_name: '架空自動車',
  shares: 100,
  average_purchase_price: 2500,
  total_purchase_amount: 250000,
  current_price: 2600,
  daily_change: 50,
  market_value: 260000,
  profit_loss_rate: 4.0,
};

const SECOND_ASSET = {
  id: 'asset-balance-2',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '9914',
  security_name: '架空エレクトロニクス',
  shares: 50,
  average_purchase_price: 3000,
  total_purchase_amount: 150000,
  current_price: 3200,
  daily_change: -20,
  market_value: 160000,
  profit_loss_rate: 6.67,
};

const ASSET_BALANCES = [MOCK_ASSET, SECOND_ASSET];

const MOCK_PREVIEW = {
  total_rows: 1,
  valid_rows: 1,
  errors: [],
  rows: [MOCK_ASSET],
};

const MOCK_UPLOAD = {
  inserted: 1,
  skipped: 0,
  errors: [],
};

function cloneAssetBalances(assetBalances: typeof ASSET_BALANCES | typeof MOCK_PREVIEW.rows) {
  return assetBalances.map((assetBalance) => ({ ...assetBalance }));
}

interface AssetBalanceMockController {
  getDeleteRequestCount: () => number;
  getListRequestCount: () => number;
  getPreviewRequestCount: () => number;
  getUploadRequestCount: () => number;
}

async function setupAssetBalanceMocks(
  page: Page,
  {
    initialAssetBalances = [],
    previewRows = cloneAssetBalances(MOCK_PREVIEW.rows),
    previewErrors = [] as { row: number; message: string }[],
    uploadedAssetBalances = cloneAssetBalances(MOCK_PREVIEW.rows),
  }: {
    initialAssetBalances?: typeof ASSET_BALANCES;
    previewRows?: typeof MOCK_PREVIEW.rows;
    previewErrors?: { row: number; message: string }[];
    uploadedAssetBalances?: typeof MOCK_PREVIEW.rows;
  } = {},
): Promise<AssetBalanceMockController> {
  let assetBalances = cloneAssetBalances(initialAssetBalances);
  let deleteRequestCount = 0;
  let listRequestCount = 0;
  let previewRequestCount = 0;
  let uploadRequestCount = 0;

  await mockSession(page);
  await mockCsvKindStats(page, { counts: () => ({ assetbalance: assetBalances.length }) });

  await page.route(ROUTES.dividendPerShareBatch, (route) =>
    route.fulfill(
      json({
        items: [
          {
            security_code: '9904',
            dividend_per_share: 90,
            status: 'ok',
            fetched_at: '2026-01-01T00:00:00Z',
            is_stale: false,
          },
          {
            security_code: '9914',
            dividend_per_share: 70,
            status: 'ok',
            fetched_at: '2026-01-01T00:00:00Z',
            is_stale: false,
          },
        ],
      }),
    ),
  );

  await page.route(ROUTES.assetBalancePreview, (route) => {
    previewRequestCount += 1;
    return route.fulfill(
      json({
        ...MOCK_PREVIEW,
        total_rows: previewRows.length + previewErrors.length,
        valid_rows: previewRows.length,
        errors: previewErrors,
        rows: cloneAssetBalances(previewRows),
      }),
    );
  });

  await page.route(ROUTES.assetBalanceUpload, (route) => {
    uploadRequestCount += 1;
    assetBalances = cloneAssetBalances(uploadedAssetBalances);
    return route.fulfill(json(MOCK_UPLOAD));
  });

  await page.route(ROUTES.assetBalances, (route) => {
    const method = route.request().method();

    if (method === 'DELETE') {
      deleteRequestCount += 1;
      assetBalances = [];
      return route.fulfill({ status: 204 });
    }

    if (method === 'GET') {
      listRequestCount += 1;
      const data = cloneAssetBalances(assetBalances);
      return route.fulfill(
        json({ data, total: data.length, page: 1, per_page: data.length }),
      );
    }

    return route.abort();
  });

  return {
    getDeleteRequestCount: () => deleteRequestCount,
    getListRequestCount: () => listRequestCount,
    getPreviewRequestCount: () => previewRequestCount,
    getUploadRequestCount: () => uploadRequestCount,
  };
}

async function gotoAssetBalancePage(page: Page) {
  await page.goto('/assetbalance');
}

test('資産管理データがないとき EmptyState が表示される', async ({ page }) => {
  await setupAssetBalanceMocks(page);

  await gotoAssetBalancePage(page);

  await expect(page.getByText('資産管理データがありません')).toBeVisible();
});







test('資産管理データが2件あるときサマリーと検索入力が表示される', async ({ page }) => {
  await setupAssetBalanceMocks(page, { initialAssetBalances: ASSET_BALANCES });

  await gotoAssetBalancePage(page);

  await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
  await expect(page.getByText('評価額合計', { exact: true })).toBeVisible();
  // 全件削除は一覧には置かず CSV 管理ページへ誘導する
  await expect(page.getByRole('link', { name: 'CSV管理' })).toBeVisible();
  await expect(page.getByTestId('asset-search-input-pc')).toBeVisible();
});

test('銘柄名の一部で絞り込み解除できる', async ({ page }) => {
  await setupAssetBalanceMocks(page, { initialAssetBalances: ASSET_BALANCES });

  await gotoAssetBalancePage(page);

  const list = page.getByTestId('asset-holdings-table');

  await expect(list.getByText('架空自動車')).toBeVisible();
  await expect(list.getByText('架空エレクトロニクス')).toBeVisible();

  await page.getByTestId('asset-search-input-pc').fill('エレクトロ');

  await expect(page.getByTestId('asset-search-count')).toContainText('1件 / 2件');
  await expect(page.getByTestId('page-filtering-count')).toBeVisible();
  await expect(list.getByText('架空エレクトロニクス')).toBeVisible();
  await expect(list.getByText('架空自動車')).not.toBeVisible();

  await page.getByTestId('asset-search-clear').click();

  await expect(page.getByTestId('asset-search-count')).toContainText('2件 / 2件');
  await expect(page.getByTestId('page-filtering-count')).toHaveCount(0);
  await expect(list.getByText('架空自動車')).toBeVisible();
});

test('全件削除は行の⋯メニューから確認モーダルで行う', async ({ page }) => {
  const mocks = await setupAssetBalanceMocks(page, { initialAssetBalances: [MOCK_ASSET] });

  await gotoAssetBalancePage(page);
  await page.getByRole('link', { name: 'CSV管理' }).click();
  await expect(page).toHaveURL('/csv');

  // 国内株式の行の⋯メニューから削除確認を開く
  await openCsvDelete(page, 'assetbalance');

  const modal = page.getByRole('dialog');
  await expect(modal).toContainText('資産管理(国内株式)を削除しますか?');
  await expect(modal).toContainText('元に戻せません');
  await expect(modal).toContainText('1件');
  expect(mocks.getDeleteRequestCount()).toBe(0);

  // キャンセルでモーダルが閉じ、DELETE は送られない
  await modal.getByRole('button', { name: 'キャンセル' }).click();
  await expect(modal).toBeHidden();
  expect(mocks.getDeleteRequestCount()).toBe(0);

  // 再度開いて確定する
  await openCsvDelete(page, 'assetbalance');
  await modal.getByRole('button', { name: '1件を削除する' }).click();

  await expect.poll(mocks.getDeleteRequestCount).toBe(1);
  await expect(modal).toBeHidden();
  await expect(savedCount(page, 'assetbalance')).toContainText('0件');
});
