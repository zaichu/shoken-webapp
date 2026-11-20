import { expect, test, json, mockSession, paginated, type Page, type Locator } from '../support/test';

const STOCK_NAME = '架空グローバル先端素材ソリューションホールディングス';
const FUND_NAME = '架空・世界革新テクノロジー株式インデックスファンド＜購入・換金手数料なし＞';

const STOCK = {
  id: 'asset-balance-detail',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '9901',
  security_name: STOCK_NAME,
  account: '特定',
  shares: 120,
  average_purchase_price: 19706.78,
  total_purchase_amount: 2364813.6,
  current_price: 19806.78,
  daily_change: 75,
};

const FUND = {
  id: 'fund-balance-detail',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  fund_name: FUND_NAME,
  account: '特定',
  dividends_course: '再投資型',
  shares: 120000,
  average_purchase_price: 19706.78,
  total_purchase_amount: 236481.36,
  base_price: 19806.78,
  base_price_day_change: 75,
  base_price_month_change: 210,
  market_value: 237681.36,
  profit_loss: 1200,
  profit_loss_rate: 0.51,
  total_return: 0,
  price_as_of: '2026-11-20',
  price_source: 'csv',
  nav_is_latest: true,
  expected_nav_date: null,
};

const RECEIPT_STOCK = {
  id: '00000000-0000-0000-0000-000000000091',
  trade_date: '2026-11-20',
  settlement_date: '2026-11-22',
  account: '特定口座',
  security_code: '9901',
  security_name: '架空グローバル先端素材ソリューションホールディングス',
  shares: 1234,
  asked_price: 19706.78,
  proceeds: 24357579.52,
  purchase_price: 18000,
  realized_profit_and_loss: 2103188.52,
  taxes: 427217,
  realized_profit_and_loss_after_tax: 1675971.52,
  created_at: '2026-11-20T00:00:00Z',
  updated_at: '2026-11-20T00:00:00Z',
};

const RECEIPT_FUND = {
  id: '00000000-0000-0000-0000-000000000092',
  trade_date: '2026-11-20',
  settlement_date: '2026-11-22',
  account: '特定口座',
  fund_name: FUND.fund_name,
  shares: 1234567,
  redemption_price: 19706.78,
  redemption_amount: 2435757.95,
  purchase_price: 18000,
  realized_profit_and_loss: 210318.85,
  taxes: 42722,
  realized_profit_and_loss_after_tax: 167596.85,
  created_at: '2026-11-20T00:00:00Z',
  updated_at: '2026-11-20T00:00:00Z',
};

async function mockDetails(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([STOCK]))),
  );
  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([FUND]))),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [{ security_code: STOCK.security_code, dividend_per_share: 1350 }] })),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([RECEIPT_STOCK]))),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([RECEIPT_FUND]))),
  );
}

async function expectDetailRowsNotToOverlap(dialog: Locator) {
  const rows = dialog.locator('.detail-dialog-row');
  await expect.poll(() => rows.count()).toBeGreaterThan(0);
  const overlaps = await rows.evaluateAll((elements) => elements.flatMap((row) => {
    const label = row.querySelector('.detail-dialog-label')?.getBoundingClientRect();
    const value = row.querySelector('.detail-dialog-value')?.getBoundingClientRect();
    if (!label || !value) return ['項目名または値がありません'];
    const intersects = label.left < value.right && value.left < label.right
      && label.top < value.bottom && value.top < label.bottom;
    return intersects ? [row.textContent?.trim() ?? '文字矩形が重なっています'] : [];
  }));
  expect(overlaps).toEqual([]);
}

for (const width of [390, 360]) {
  test(`幅${width}px: 保有株の詳細で全項目が重ならない`, async ({ page }) => {
    await page.setViewportSize({ width, height: 844 });
    await mockDetails(page);
    await page.goto('/assetbalance?tab=stock');
    await page.getByTestId('asset-compact-row')
      .filter({ hasText: STOCK_NAME })
      .getByTestId('asset-row-button').click();
    const dialog = page.getByTestId('asset-holding-dialog');
    await expect(dialog).toBeVisible();
    await expect(dialog).toContainText('¥19,706.78');
    await expectDetailRowsNotToOverlap(dialog);
  });

  test(`幅${width}px: 保有投信の詳細で全項目が重ならない`, async ({ page }) => {
    await page.setViewportSize({ width, height: 844 });
    await mockDetails(page);
    await page.goto('/assetbalance?tab=fund');
    await page.getByTestId('asset-compact-row')
      .filter({ hasText: FUND_NAME })
      .getByTestId('asset-row-button').click();
    const dialog = page.getByTestId('asset-fund-dialog');
    await expect(dialog).toBeVisible();
    await expect(dialog).toContainText('¥19,706.78');
    await expectDetailRowsNotToOverlap(dialog);
  });

  test(`幅${width}px: 取引明細の詳細で全項目が重ならない`, async ({ page }) => {
    await page.setViewportSize({ width, height: 844 });
    await mockDetails(page);
    await page.goto('/receipts');
    await page.getByRole('tab', { name: '国内株式' }).click();
    await page.getByTestId('receipt-row-button').first().click();
    const dialog = page.getByTestId('receipt-detail-dialog');
    await expect(dialog).toBeVisible();
    await expectDetailRowsNotToOverlap(dialog);
  });
}
