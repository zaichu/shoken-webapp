import type { Locator, Page } from '@playwright/test';
import { expect, test, json, mockSession } from '../support/test';

// 1株配当が取得できる銘柄
const WITH_DIVIDEND = {
  id: 'asset-balance-1',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '9904',
  security_name: '架空自動車',
  account: '特定',
  shares: 100,
  average_purchase_price: 2500,
  total_purchase_amount: 250000,
  current_price: 2600,
  daily_change: 50,
  price_as_of: '2026-01-01',
};

// 1株配当が取得できる銘柄(2件目)
const SECOND_WITH_DIVIDEND = {
  ...WITH_DIVIDEND,
  id: 'asset-balance-2',
  security_code: '9914',
  security_name: '架空エレクトロニクス',
  shares: 50,
  average_purchase_price: 3000,
  total_purchase_amount: 150000,
  current_price: 3200,
};

// 1株配当の取得に失敗した銘柄
const FAILED_DIVIDEND = {
  ...WITH_DIVIDEND,
  id: 'asset-balance-3',
  security_code: '9920',
  security_name: '架空精密機器',
};

// 無配(0円)の銘柄
const ZERO_DIVIDEND = {
  ...WITH_DIVIDEND,
  id: 'asset-balance-4',
  security_code: '9921',
  security_name: '架空食品工業',
};

// 1株配当の応答に含まれない銘柄
const NO_DIVIDEND = {
  ...WITH_DIVIDEND,
  id: 'asset-balance-5',
  security_code: '9922',
  security_name: '架空総合化学',
};

const STOCKS = [
  WITH_DIVIDEND,
  SECOND_WITH_DIVIDEND,
  FAILED_DIVIDEND,
  ZERO_DIVIDEND,
  NO_DIVIDEND,
];

const DIVIDEND_ITEMS = [
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
  {
    security_code: '9920',
    dividend_per_share: null,
    status: 'error',
    fetched_at: null,
    is_stale: false,
  },
  {
    security_code: '9921',
    dividend_per_share: 0,
    status: 'zero',
    fetched_at: '2026-01-01T00:00:00Z',
    is_stale: false,
  },
];

async function setupMocks(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(
      json({
        data: STOCKS,
        total: STOCKS.length,
        page: 1,
        per_page: 1000,
      }),
    ),
  );
  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json({ data: [], total: 0, page: 1, per_page: 0 })),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: DIVIDEND_ITEMS })),
  );
}

// 見出しの文字から列の位置を引く(列の並びが変わっても値を特定できるように)
async function columnIndex(table: Locator, label: string) {
  const headers = await table.getByRole('columnheader').allTextContents();
  const index = headers.findIndex((text) => text.includes(label));
  expect(index).toBeGreaterThanOrEqual(0);
  return index;
}

test('国内株式の表に1株配当の列があり、1株配当と年間配当が整合する', async ({ page }) => {
  await setupMocks(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/assetbalance');
  await page.getByTestId('asset-kind-tab-stock').click();

  const table = page.getByTestId('asset-holdings-table');
  await expect(table).toBeVisible();

  // 1株配当は年間配当の左の独立した列
  const perShareIndex = await columnIndex(table, '1株配当');
  const annualIndex = await columnIndex(table, '年間配当');
  expect(annualIndex).toBeGreaterThan(perShareIndex);

  // 取得できた銘柄は1株配当と年間配当(1株配当×数量)が出る
  const priced = table.getByRole('row', { name: /架空自動車/ });
  await expect(priced.getByRole('cell').nth(perShareIndex)).toHaveText('¥90');
  await expect(priced.getByRole('cell').nth(annualIndex)).toHaveText('¥9,000');

  // 無配は0円
  const zero = table.getByRole('row', { name: /架空食品工業/ });
  await expect(zero.getByRole('cell').nth(perShareIndex)).toHaveText('¥0');

  // 取得失敗と、応答に無い銘柄は数字を出さない
  const failed = table.getByRole('row', { name: /架空精密機器/ });
  await expect(failed.getByRole('cell').nth(perShareIndex)).toHaveText('取得失敗');
  const missing = table.getByRole('row', { name: /架空総合化学/ });
  await expect(missing.getByRole('cell').nth(perShareIndex)).toHaveText('—');

  // 合計は取得できた銘柄だけ(90×100 + 70×50)
  const kpi = page.getByTestId('portfolio-kpi-grid');
  await expect(kpi).toContainText('¥12,500');
});

test('スマホの行は年間配当の隣に1株配当を出す', async ({ page }) => {
  await setupMocks(page);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/assetbalance');
  await page.getByTestId('asset-kind-tab-stock').click();

  const row = page.getByTestId('asset-compact-row').filter({ hasText: '架空自動車' });
  await expect(row).toBeVisible();

  // 配当で並べたときだけ行に配当を出す
  await page.getByTestId('asset-sort-select').selectOption('dividend');
  await expect(row).toContainText('1株 ¥90');
  // 年間配当(1株配当×数量)も一緒に出る
  await expect(row).toContainText('¥9,000');
});

test('「すべて」タブの株式行にも1株配当が出る', async ({ page }) => {
  await setupMocks(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/assetbalance');
  await page.getByTestId('asset-kind-tab-all').click();

  const table = page.getByTestId('asset-merged-table');
  await expect(table).toBeVisible();

  const index = await columnIndex(table, '1株配当');
  const row = table.getByRole('row', { name: /架空自動車/ });
  await expect(row.getByRole('cell').nth(index)).toHaveText('¥90');
});
