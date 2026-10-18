import path from 'node:path';
import type { Page } from '@playwright/test';
import { expect, test, json } from '../support/test';

const CSV_FIXTURE_PATH = path.resolve(process.cwd(), 'e2e/fixtures/csv/assetbalance-base.csv');

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000010',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const ROUTES = {
  authMe: /\/api\/v1\/session$/,
  assetBalances: /\/api\/v1\/asset-balances(?:\?.*)?$/,
  assetBalancePreview: /\/api\/v1\/asset-balance-import-validations$/,
  assetBalanceUpload: /\/api\/v1\/asset-balance-imports$/,
  dividendPerShareBatch: /\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/,
};

const MOCK_ASSET = {
  id: 'asset-balance-1',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '7203',
  security_name: 'トヨタ自動車',
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
  security_code: '6758',
  security_name: 'ソニーグループ',
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

  await page.route(ROUTES.authMe, (route) => route.fulfill(json(MOCK_USER)));

  await page.route(ROUTES.dividendPerShareBatch, (route) =>
    route.fulfill(
      json({
        items: [
          {
            security_code: '7203',
            dividend_per_share: 90,
            status: 'ok',
            fetched_at: '2026-01-01T00:00:00Z',
            is_stale: false,
          },
          {
            security_code: '6758',
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

async function selectCsvFile(page: Page) {
  const input = page.getByLabel('CSVファイルを選択');
  // 取得中・保存中は input が disabled で change イベントが捨てられるため、有効化を待ってから投入する
  await expect(input).toBeEnabled();
  await input.setInputFiles(CSV_FIXTURE_PATH);
}

test('資産管理データがないとき EmptyState が表示される', async ({ page }) => {
  await setupAssetBalanceMocks(page);

  await gotoAssetBalancePage(page);

  await expect(page.getByText('資産管理データがありません')).toBeVisible();
});

test('CSV プレビュー完了後に保存ボタンが活性化する', async ({ page }) => {
  const mocks = await setupAssetBalanceMocks(page);

  await page.goto('/assetbalance/csv');
  await selectCsvFile(page);

  await expect.poll(mocks.getPreviewRequestCount).toBe(1);
  await expect(page.getByTestId('csv-file-trigger')).toContainText(
    'assetbalance-base.csv',
  );
  await expect(page.getByRole('button', { name: '1件 全件置換で保存' })).toBeEnabled();
});

test('エラーのあるCSVは全件置換で保存できない', async ({ page }) => {
  const mocks = await setupAssetBalanceMocks(page, {
    previewErrors: [{ row: 2, message: '株数が数値ではありません' }],
  });

  await page.goto('/assetbalance/csv');
  await selectCsvFile(page);

  // 全件置換はエラー行を含む CSV を backend が拒否するため、有効行だけの保存を約束しない
  const notice = page.getByTestId('csv-preview-notice');
  await expect(notice).toContainText('全件置換で保存できません');
  await expect(
    page.getByRole('button', { name: /全件置換で保存/ }).first(),
  ).toBeDisabled();
  expect(mocks.getUploadRequestCount()).toBe(0);
});

test('保存後に一覧が再取得されて銘柄データが表示される', async ({ page }) => {
  const mocks = await setupAssetBalanceMocks(page);

  await page.goto('/assetbalance/csv');
  await selectCsvFile(page);

  await page.getByRole('button', { name: '1件 全件置換で保存' }).click();

  await expect.poll(mocks.getUploadRequestCount).toBe(1);
  await expect.poll(mocks.getListRequestCount).toBeGreaterThanOrEqual(2);
  await expect(page.getByTestId('csv-save-result-notice')).toContainText('1件反映');
  // 新UI: カードの件数が更新される
  await expect(page.getByText('保存済みの資産管理')).toBeVisible();
  await expect(page.getByTestId('saved-count')).toContainText('1件');
  // 一覧に戻ると保存した銘柄が見える
  await page.getByRole('link', { name: '一覧に戻る' }).click();
  await expect(
    page.getByTestId('asset-holdings-table').getByText('トヨタ自動車'),
  ).toBeVisible();
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

  await expect(list.getByText('トヨタ自動車')).toBeVisible();
  await expect(list.getByText('ソニーグループ')).toBeVisible();

  await page.getByTestId('asset-search-input-pc').fill('ソニー');

  await expect(page.getByTestId('asset-search-count-pc')).toContainText('1 / 2');
  await expect(list.getByText('ソニーグループ')).toBeVisible();
  await expect(list.getByText('トヨタ自動車')).not.toBeVisible();

  await page.getByTestId('asset-search-clear-pc').click();

  await expect(page.getByTestId('asset-search-count-pc')).not.toBeVisible();
  await expect(list.getByText('トヨタ自動車')).toBeVisible();
});

test('全件削除はカード内の「すべて削除…」ボタンから確認モーダルで行う', async ({ page }) => {
  const mocks = await setupAssetBalanceMocks(page, { initialAssetBalances: [MOCK_ASSET] });

  await gotoAssetBalancePage(page);
  await page.getByRole('link', { name: 'CSV管理' }).click();
  await expect(page).toHaveURL('/assetbalance/csv');

  // カード内の「すべて削除…」ボタンをクリック
  const deleteButton = page.getByRole('button', { name: 'すべて削除…' });
  await expect(deleteButton).toBeVisible();
  await deleteButton.click();

  // 確認モーダルが表示される
  const modal = page.getByRole('dialog');
  await expect(modal).toBeVisible();
  await expect(modal).toContainText('資産管理を削除しますか?');
  await expect(modal).toContainText('この操作は取り消せません');
  await expect(modal).toContainText('1件');
  expect(mocks.getDeleteRequestCount()).toBe(0);

  // キャンセルでモーダルが閉じ、DELETE は送られない
  await modal.getByRole('button', { name: 'キャンセル' }).click();
  await expect(modal).toBeHidden();
  expect(mocks.getDeleteRequestCount()).toBe(0);

  // 再度開いて確定する
  await deleteButton.click();
  await expect(modal).toBeVisible();
  await modal.getByRole('button', { name: '1件を削除する' }).click();

  await expect.poll(mocks.getDeleteRequestCount).toBe(1);
  await expect(modal).toBeHidden();
  // CSV管理ページではカードが「0件」になる
  await expect(page.getByTestId('saved-count')).toContainText('0件');
});
