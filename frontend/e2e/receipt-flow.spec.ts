import { expect, test, type Page } from '@playwright/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const ROUTES = {
  authMe: /\/api\/v1\/session$/,
  dividends: /\/dividends$/,
  domesticStocks: /\/domestic-stocks$/,
  mutualfunds: /\/mutualfunds$/,
};

const DIVIDEND_RECORD = [
  {
    id: '00000000-0000-0000-0000-000000000001',
    user_id: '00000000-0000-0000-0000-000000000002',
    settlement_date: '2024-03-01',
    product: '特定口座',
    account: 'SBI証券',
    security_code: '7203',
    security_name: 'トヨタ自動車',
    unit_price: '30.0',
    shares: '100',
    dividends_before_tax: '3000',
    taxes: '609',
    net_amount_received: '2391',
    created_at: '2024-03-01T00:00:00Z',
    updated_at: '2024-03-01T00:00:00Z',
  },
];

async function setupAuthMocks(
  page: Page,
  {
    dividends = [],
    domesticStocks = [],
    mutualfunds = [],
  }: {
    dividends?: unknown[];
    domesticStocks?: unknown[];
    mutualfunds?: unknown[];
  } = {},
) {
  await page.route(ROUTES.authMe, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_USER),
    }),
  );
  await page.route(ROUTES.dividends, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(dividends),
    }),
  );
  await page.route(ROUTES.domesticStocks, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(domesticStocks),
    }),
  );
  await page.route(ROUTES.mutualfunds, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(mutualfunds),
    }),
  );
}

test('取引明細ページの初期表示でタブが3つある', async ({ page }) => {
  await setupAuthMocks(page);

  await page.goto('/receipts');

  await expect(page.getByRole('tab')).toHaveCount(3);
});

test('配当金タブにデータがないとき EmptyState が表示される', async ({ page }) => {
  await setupAuthMocks(page, { dividends: [] });

  await page.goto('/receipts');

  await expect(page.getByText('データがありません')).toBeVisible();
});

test('配当金データが1件あるとき行が表示される', async ({ page }) => {
  await setupAuthMocks(page, { dividends: DIVIDEND_RECORD });

  await page.goto('/receipts');

  await expect(page.getByRole('cell', { name: 'トヨタ自動車' })).toBeVisible();
});
