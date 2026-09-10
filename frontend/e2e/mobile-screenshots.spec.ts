import { expect, test, type Page } from '@playwright/test';

/**
 * スマホ表示レビュー用のスクリーンショット取得。
 * API は page.route() でモックするためバックエンド・ログイン不要。
 * 実行: npx playwright test --config playwright.mobile.config.ts
 */

const SHOT_DIR = '../.playwright-mcp';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const ROUTES = {
  session: /\/api\/v1\/session$/,
  dividends: /\/api\/v1\/dividends(?:\?.*)?$/,
  domesticStocks: /\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/,
  mutualfunds: /\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/,
  assetBalances: /\/api\/v1\/asset-balances(?:\?.*)?$/,
  dividendPerShare: /\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/,
  stock: /\/api\/v1\/stocks(?:\?.*)?$/,
  facets: /\/api\/v1\/.*facets(?:\?.*)?$/,
};

const DIVIDENDS = [
  {
    id: '00000000-0000-0000-0000-000000000001',
    user_id: MOCK_USER.id,
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
  {
    id: '00000000-0000-0000-0000-000000000002',
    user_id: MOCK_USER.id,
    settlement_date: '2024-06-28',
    product: 'NISA口座',
    account: '楽天証券',
    security_code: '9432',
    security_name: '日本電信電話',
    unit_price: '2.5',
    shares: '1000',
    dividends_before_tax: '2500',
    taxes: '507',
    net_amount_received: '1993',
    created_at: '2024-06-28T00:00:00Z',
    updated_at: '2024-06-28T00:00:00Z',
  },
];

const DOMESTIC_STOCKS = [
  {
    id: '00000000-0000-0000-0000-000000000011',
    trade_date: '2024-05-10',
    settlement_date: '2024-05-14',
    security_code: '7203',
    security_name: 'トヨタ自動車',
    account: '特定口座',
    shares: '100',
    asked_price: '2800',
    proceeds: '280000',
    purchase_price: '2500',
    realized_profit_and_loss: '30000',
    taxes: '6090',
    realized_profit_and_loss_after_tax: '23910',
    created_at: '2024-05-10T00:00:00Z',
    updated_at: '2024-05-10T00:00:00Z',
  },
];

const MUTUALFUNDS = [
  {
    id: '00000000-0000-0000-0000-000000000021',
    trade_date: '2024-04-02',
    settlement_date: '2024-04-05',
    fund_name: 'eMAXIS Slim 全世界株式（オール・カントリー）',
    account: '特定口座',
    shares: '10000',
    exchange_rate: '1',
    cancellation_unit_price_yen: '21000',
    cancellation_amount_yen: '210000',
    average_acquisition_price_yen: '18000',
    dividends: '0',
    realized_profit_and_loss: '30000',
    taxes: '6090',
    realized_profit_and_loss_after_tax: '23910',
    created_at: '2024-04-02T00:00:00Z',
    updated_at: '2024-04-02T00:00:00Z',
  },
];

const ASSET_BALANCES = [
  {
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
  },
];

function paginated(data: unknown[]) {
  return { data, total: data.length, page: 1, per_page: Math.max(data.length, 1) };
}

function json(body: unknown) {
  return { status: 200, contentType: 'application/json', body: JSON.stringify(body) };
}

async function mockApi(page: Page) {
  // Playwright は後から登録した route が優先されるため、catch-all を最初に登録する
  await page.route(/\/api\/v1\//, (route) => route.fulfill(json(paginated([]))));

  await page.route(ROUTES.session, (route) => route.fulfill(json(MOCK_USER)));
  await page.route(ROUTES.dividends, (route) => route.fulfill(json(paginated(DIVIDENDS))));
  await page.route(ROUTES.domesticStocks, (route) =>
    route.fulfill(json(paginated(DOMESTIC_STOCKS))),
  );
  await page.route(ROUTES.mutualfunds, (route) => route.fulfill(json(paginated(MUTUALFUNDS))));
  await page.route(ROUTES.assetBalances, (route) => route.fulfill(json(paginated(ASSET_BALANCES))));
  await page.route(ROUTES.dividendPerShare, (route) =>
    route.fulfill(
      json({
        data: [{ security_code: '7203', dividend_per_share: 30, status: 'ok', is_stale: false }],
      }),
    ),
  );
  await page.route(ROUTES.stock, (route) =>
    route.fulfill(
      json({
        date: '2024-01-01',
        code: '7203',
        name: 'トヨタ自動車',
        market_category: 'プライム',
        industry_code_33: '3050',
        industry_category_33: '輸送用機器',
        industry_code_17: '6',
        industry_category_17: '自動車・輸送機',
        size_code: '1',
        size_category: 'TOPIX Core30',
      }),
    ),
  );
}

async function shoot(page: Page, name: string) {
  await page.waitForTimeout(700);
  await page.screenshot({ path: `${SHOT_DIR}/mobile-${name}.png`, fullPage: true });
}

test('スマホ幅の主要画面スクリーンショット', async ({ page }) => {
  await mockApi(page);

  await page.goto('/');
  await shoot(page, 'home');

  await page.goto('/search?code=7203');
  await shoot(page, 'search-result');

  await page.goto('/assetbalance');
  await shoot(page, 'assetbalance');

  await page.goto('/receipts');
  await expect(page.getByRole('tab').first()).toBeVisible();
  await shoot(page, 'receipts-dividend');

  const tabs = page.getByRole('tab');
  await tabs.nth(1).click();
  await shoot(page, 'receipts-domestic-stock');

  await tabs.nth(2).click();
  await shoot(page, 'receipts-mutualfund');

  await page.goto('/404');
  await shoot(page, 'notfound');
});

test('スマホ幅の未ログイン画面スクリーンショット', async ({ page }) => {
  await page.route(ROUTES.session, (route) => route.fulfill({ status: 401, body: '' }));
  await page.route(/\/api\/v1\//, (route) => route.fulfill({ status: 401, body: '' }));

  await page.goto('/login');
  await shoot(page, 'login');
});
