import type { Page } from '@playwright/test';
import { expectTableFit } from '../support/layout';
import { expect, test, json } from '../support/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000010',
  email: 'test@example.com',
  name: 'テストユーザー',
};

// 長い名前と小数の値段の行。表が枠から横にはみ出さないことを確かめる
const STOCK_ROWS = [
  {
    id: 'asset-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: '9901',
    security_name: '架空ニフティ indexed-abate Cable-technology-Leading',
    account: '特定・一般',
    shares: 1200,
    average_purchase_price: 5997.6,
    total_purchase_amount: 7197120,
    current_price: 6187.25,
    daily_change: 12,
    price_as_of: '2026-10-20',
    prev_close: 6175.25,
  },
  {
    id: 'asset-2',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: '9902',
    security_name: '架空半導體 International-High-Dividend-Blue-Chip',
    account: 'NISA成長投資枠',
    shares: 300,
    average_purchase_price: 187.36,
    total_purchase_amount: 56208,
    current_price: 162,
    daily_change: -3,
    price_as_of: '2026-10-20',
    prev_close: 165,
  },
];

const FUND_ROWS = [
  {
    id: 'fund-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    fund_name: '架空全世界株式インデックスファンド 長期成長型',
    account: '特定・一般',
    dividends_course: '再投資型',
    shares: 2000000,
    average_purchase_price: 9970.5,
    total_purchase_amount: 1994100,
    base_price: 10012.25,
    base_price_day_change: 5,
    base_price_month_change: 120,
    market_value: 2002450,
    profit_loss: 8350,
    profit_loss_rate: 0.42,
    total_return: 8350,
    price_as_of: '2026-10-20',
    price_source: 'api',
    nav_is_latest: true,
    expected_nav_date: null,
  },
];

async function setupMocks(page: Page) {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill(json(MOCK_USER)),
  );
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(
      json({
        data: STOCK_ROWS,
        total: STOCK_ROWS.length,
        page: 1,
        per_page: 1000,
      }),
    ),
  );
  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
    route.fulfill(
      json({
        data: FUND_ROWS,
        total: FUND_ROWS.length,
        page: 1,
        per_page: 1000,
        summary: {
          total_market_value: 2002450,
          total_purchase_amount: 1994100,
          total_daily_change: 0,
          latest_nav_count: FUND_ROWS.length,
          has_missing_daily_change: false,
        },
      }),
    ),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

for (const width of [1024, 1280, 1440, 1920]) {
  test(`資産管理の表は ${width}px で枠に収まる`, async ({ page }) => {
    await setupMocks(page);
    await page.setViewportSize({ width, height: 900 });
    await page.goto('/assetbalance');

    for (const testid of [
      'asset-kind-tab-stock',
      'asset-kind-tab-fund',
      'asset-kind-tab-all',
    ]) {
      await page.getByTestId(testid).click();
      await expect(page.getByRole('table')).toBeVisible();
      await expectTableFit(page);
    }
  });
}
