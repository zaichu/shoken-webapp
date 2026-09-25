import { expect, test, type Page } from '@playwright/test';

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

function jsonResponse(body: unknown, status = 200) {
  return {
    status,
    contentType: 'application/json',
    body: JSON.stringify(body),
  };
}

function paginated(data: unknown[]) {
  return { data, total: data.length, page: 1, per_page: data.length };
}

async function mockSession(page: Page) {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill(jsonResponse(MOCK_USER)),
  );
}

async function mockReceipts(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(jsonResponse(paginated([DIVIDEND]))),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(jsonResponse(paginated([]))),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(jsonResponse(paginated([]))),
  );
}

async function mockAssetBalance(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(jsonResponse({ ...paginated([ASSET]), per_page: 1000 })),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(jsonResponse({ items: [] })),
  );
}

test('取引明細の検索カードは 390px では初期折り畳みでトグルで開閉できる', async ({ page }) => {
  await mockReceipts(page);
  await page.setViewportSize({ width: 390, height: 844 });

  await page.goto('/receipts');
  await page.waitForLoadState('networkidle');

  const header = page.getByTestId('search-card-header');
  const body = page.locator('#search-options-body');
  await expect(header).toBeVisible();
  await expect(header).toHaveAttribute('aria-expanded', 'false');
  await expect(body).toBeHidden();

  await header.click();
  await expect(header).toHaveAttribute('aria-expanded', 'true');
  await expect(body).toBeVisible();

  const chevron = page.getByTestId('search-card-chevron-toggle');
  await expect(chevron).toHaveAttribute('aria-hidden', 'true');
  await expect(chevron).toHaveAttribute('tabindex', '-1');
  await chevron.click();
  await expect(header).toHaveAttribute('aria-expanded', 'false');
  await expect(body).toBeHidden();
});

test('取引明細の検索カードは 640px 以上で初期展開', async ({ page }) => {
  await mockReceipts(page);
  await page.setViewportSize({ width: 640, height: 844 });

  await page.goto('/receipts');
  await page.waitForLoadState('networkidle');

  await expect(page.getByTestId('search-card-header')).toHaveAttribute(
    'aria-expanded',
    'true',
  );
  await expect(page.locator('#search-options-body')).toBeVisible();
});

test('資産管理の検索カードは 390px では初期折り畳みでトグルで開閉できる', async ({ page }) => {
  await mockAssetBalance(page);
  await page.setViewportSize({ width: 390, height: 844 });

  await page.goto('/assetbalance');
  await page.waitForLoadState('networkidle');

  const header = page.getByTestId('search-card-header');
  const body = page.locator('#search-options-body');
  await expect(header).toBeVisible();
  await expect(header).toHaveAttribute('aria-expanded', 'false');
  await expect(body).toBeHidden();

  await header.click();
  await expect(header).toHaveAttribute('aria-expanded', 'true');
  await expect(body).toBeVisible();
});

test('資産管理の検索カードは 640px 以上で初期展開', async ({ page }) => {
  await mockAssetBalance(page);
  await page.setViewportSize({ width: 640, height: 844 });

  await page.goto('/assetbalance');
  await page.waitForLoadState('networkidle');

  await expect(page.getByTestId('search-card-header')).toHaveAttribute(
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

  const header = page.getByTestId('search-card-header');
  await header.click();
  await page.getByLabel('銘柄').selectOption('7203');
  await header.click();

  await expect(header).toContainText('適用中');
  await expect(header).toHaveAttribute(
    'aria-label',
    '検索オプション 開く（絞り込み適用中）',
  );
  const clearButton = page.getByTestId('search-clear-button');
  await expect(clearButton).toHaveAttribute('aria-hidden', 'false');
  await clearButton.click();
  await expect(header).not.toContainText('適用中');
  await expect(header).toHaveAttribute('aria-label', '検索オプション 開く');
});

test('検索カードを閉じると年ピッカーも閉じる', async ({ page }) => {
  await mockReceipts(page);
  await page.setViewportSize({ width: 390, height: 844 });

  await page.goto('/receipts');
  await page.waitForLoadState('networkidle');

  const header = page.getByTestId('search-card-header');
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
