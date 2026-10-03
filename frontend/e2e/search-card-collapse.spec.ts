import type { Page } from '@playwright/test';
import { expect, test, json, paginated } from './support/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000011',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const DIVIDEND = {
  id: '00000000-0000-0000-0000-000000000001',
  settlement_date: '2024-03-01',
  product: '特定口座',
  account: 'SBI証券',
  security_code: '7203',
  security_name: 'トヨタ自動車',
  unit_price: 30.0,
  shares: 100,
  dividends_before_tax: 3000,
  taxes: 609,
  net_amount_received: 2391,
  created_at: '2024-03-01T00:00:00Z',
  updated_at: '2024-03-01T00:00:00Z',
};

const ASSET = {
  id: 'asset-balance-1',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '7203',
  security_name: 'トヨタ自動車',
  shares: 100,
  executing_shares: 0,
  average_purchase_price: 2500,
  total_purchase_amount: 250000,
  current_price: 2600,
  daily_change: 50,
  market_value: 260000,
  profit_loss_rate: 4.0,
};



async function mockSession(page: Page) {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill(json(MOCK_USER)),
  );
}

async function mockReceipts(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([DIVIDEND]))),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([]))),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([]))),
  );
}

async function mockAssetBalance(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json({ ...paginated([ASSET]), per_page: 1000 })),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

// 狭い帯ではパネルが畳んで始まるので、検索カードに触れる前にドロワーを開ける
async function openReceiptPanel(page: Page) {
  const toggle = page.getByTestId('receipt-utility-toggle');
  await expect(toggle).toBeVisible();
  if ((await toggle.getAttribute('aria-expanded')) !== 'true') {
    await toggle.click();
  }
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
}

async function openAssetPanel(page: Page) {
  const toggle = page.getByTestId('asset-utility-toggle');
  await expect(toggle).toBeVisible();
  if ((await toggle.getAttribute('aria-expanded')) !== 'true') {
    await toggle.click();
  }
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
}

test('取引明細の検索カードは 390px では初期折り畳みでトグルで開閉できる', async ({ page }) => {
  await mockReceipts(page);
  await page.setViewportSize({ width: 390, height: 844 });

  await page.goto('/receipts');
  await page.waitForLoadState('networkidle');
  await openReceiptPanel(page);

  const header = page.locator('[data-testid="search-card-header"]:visible, [data-testid="receipt-search-toggle"]:visible');
  const body = page.locator('#search-options-body');
  await expect(header).toBeVisible();
  await expect(header).toHaveAttribute('aria-expanded', 'false');
  await expect(body).toBeHidden();

  await header.click();
  await expect(header).toHaveAttribute('aria-expanded', 'true');
  await expect(body).toBeVisible();

  await header.click();
  await expect(header).toHaveAttribute('aria-expanded', 'false');
  await expect(body).toBeHidden();
});

test('取引明細の検索カードは 640px 以上で初期展開', async ({ page }) => {
  await mockReceipts(page);
  await page.setViewportSize({ width: 640, height: 844 });

  await page.goto('/receipts');
  await page.waitForLoadState('networkidle');
  await openReceiptPanel(page);

  await expect(page.locator('[data-testid="search-card-header"]:visible, [data-testid="receipt-search-toggle"]:visible')).toHaveAttribute(
    'aria-expanded',
    'true',
  );
  await expect(page.locator('#search-options-body')).toBeVisible();
});

test('資産管理の検索カードは 390px でも初期展開でトグルで開閉できる', async ({ page }) => {
  await mockAssetBalance(page);
  await page.setViewportSize({ width: 390, height: 844 });

  await page.goto('/assetbalance');
  await page.waitForLoadState('networkidle');
  await openAssetPanel(page);

  const header = page.locator('[data-testid="search-card-header"]:visible, [data-testid="receipt-search-toggle"]:visible');
  const body = page.locator('#search-options-body');
  await expect(header).toBeVisible();
  await expect(header).toHaveAttribute('aria-expanded', 'true');
  await expect(body).toBeVisible();

  await header.click();
  await expect(header).toHaveAttribute('aria-expanded', 'false');
  await expect(body).toBeHidden();
});

test('資産管理の検索カードは 640px 以上で初期展開', async ({ page }) => {
  await mockAssetBalance(page);
  await page.setViewportSize({ width: 640, height: 844 });

  await page.goto('/assetbalance');
  await page.waitForLoadState('networkidle');
  await openAssetPanel(page);

  await expect(page.locator('[data-testid="search-card-header"]:visible, [data-testid="receipt-search-toggle"]:visible')).toHaveAttribute(
    'aria-expanded',
    'true',
  );
  await expect(page.locator('#search-options-body')).toBeVisible();
});

test('絞り込み中は折り畳み状態で適用中バッジが出てクリアできる', async ({ page }) => {
  await mockReceipts(page);
  await page.setViewportSize({ width: 390, height: 844 });

  await page.goto('/receipts');
  await page.waitForLoadState('networkidle');
  await openReceiptPanel(page);

  const header = page.locator('[data-testid="search-card-header"]:visible, [data-testid="receipt-search-toggle"]:visible');
  await header.click();
  await page.getByLabel('銘柄').selectOption('7203');
  await header.click();

  await expect(header).toContainText('適用中');
  await expect(header).toHaveAttribute(
    'aria-label',
    '検索オプション 開く（絞り込み適用中）',
  );
  await header.click();
  const clearButton = page.getByTestId('receipt-search-clear-button');
  await expect(clearButton).toBeEnabled();
  await clearButton.click();
  await header.click();
  await expect(header).not.toContainText('適用中');
  await expect(header).toHaveAttribute('aria-label', '検索オプション 開く');
});

test('検索カードを閉じると年ピッカーも閉じる', async ({ page }) => {
  await mockReceipts(page);
  await page.setViewportSize({ width: 390, height: 844 });

  await page.goto('/receipts');
  await page.waitForLoadState('networkidle');
  await openReceiptPanel(page);

  const header = page.locator('[data-testid="search-card-header"]:visible, [data-testid="receipt-search-toggle"]:visible');
  await header.click();

  const picker = page.getByLabel('年を選択');
  await picker.click();
  await expect(page.getByRole('listbox')).toBeVisible();

  await header.click();
  await header.click();

  await expect(page.getByRole('listbox')).toBeHidden();
});

test('不明パスは /404 に置き換わる', async ({ page }) => {
  await mockSession(page);

  await page.goto('/no-such-page');

  await expect(page).toHaveURL(/\/404$/);
  await expect(page.getByText('404 - ページが見つかりません')).toBeVisible();
});
