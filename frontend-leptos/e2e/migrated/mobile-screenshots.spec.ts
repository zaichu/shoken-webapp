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
};

const STOCKS = [
  ['7203', 'トヨタ自動車'],
  ['9432', '日本電信電話'],
  ['6758', 'ソニーグループ'],
  ['8306', '三菱UFJフィナンシャル・グループ'],
  ['9984', 'ソフトバンクグループ'],
  ['4502', '武田薬品工業'],
  ['6861', 'キーエンス'],
  ['8058', '三菱商事'],
];

// 実使用に近い件数で描画を確認するため、複数日付にまたがる明細を生成する
const DIVIDENDS = Array.from({ length: 18 }, (_, i) => {
  const [code, name] = STOCKS[i % STOCKS.length];
  const month = String((i % 12) + 1).padStart(2, '0');
  const day = String((i % 27) + 1).padStart(2, '0');
  return {
    id: `dividend-${i}`,
    user_id: MOCK_USER.id,
    settlement_date: `2024-${month}-${day}`,
    product: i % 2 === 0 ? '特定口座' : 'NISA口座',
    account: i % 3 === 0 ? 'SBI証券' : '楽天証券',
    security_code: code,
    security_name: name,
    unit_price: String(10 + i * 3),
    shares: String(100 * ((i % 5) + 1)),
    dividends_before_tax: String(3000 + i * 1234),
    taxes: String(609 + i * 250),
    net_amount_received: String(2391 + i * 984),
    created_at: '2024-03-01T00:00:00Z',
    updated_at: '2024-03-01T00:00:00Z',
  };
});

const DOMESTIC_STOCKS = Array.from({ length: 15 }, (_, i) => {
  const [code, name] = STOCKS[i % STOCKS.length];
  const month = String((i % 12) + 1).padStart(2, '0');
  const day = String((i % 27) + 1).padStart(2, '0');
  return {
    id: `domestic-${i}`,
    trade_date: `2024-${month}-${day}`,
    settlement_date: `2024-${month}-${day}`,
    security_code: code,
    security_name: name,
    account: i % 2 === 0 ? '特定口座' : 'NISA口座',
    shares: String(100 * ((i % 4) + 1)),
    asked_price: String(2800 + i * 150),
    proceeds: String(280000 + i * 15000),
    purchase_price: String(2500 + i * 120),
    realized_profit_and_loss: String(30000 - i * 2500),
    taxes: String(6090 - i * 400),
    realized_profit_and_loss_after_tax: String(23910 - i * 2100),
    created_at: '2024-05-10T00:00:00Z',
    updated_at: '2024-05-10T00:00:00Z',
  };
});

const MUTUALFUNDS = Array.from({ length: 12 }, (_, i) => {
  const month = String((i % 12) + 1).padStart(2, '0');
  const day = String((i % 27) + 1).padStart(2, '0');
  return {
    id: `mutualfund-${i}`,
    trade_date: `2024-${month}-${day}`,
    settlement_date: `2024-${month}-${day}`,
    fund_name:
      i % 2 === 0
        ? 'eMAXIS Slim 全世界株式（オール・カントリー）'
        : 'SBI・V・S&P500インデックス・ファンド',
    account: i % 2 === 0 ? '特定口座' : 'NISA口座',
    shares: String(10000 + i * 1500),
    exchange_rate: '1',
    cancellation_unit_price_yen: String(21000 + i * 800),
    cancellation_amount_yen: String(210000 + i * 12000),
    average_acquisition_price_yen: String(18000 + i * 600),
    dividends: '0',
    realized_profit_and_loss: String(30000 - i * 1800),
    taxes: String(6090 - i * 350),
    realized_profit_and_loss_after_tax: String(23910 - i * 1450),
    created_at: '2024-04-02T00:00:00Z',
    updated_at: '2024-04-02T00:00:00Z',
  };
});

const ASSET_BALANCES = Array.from({ length: 8 }, (_, i) => {
  const [code, name] = STOCKS[i % STOCKS.length];
  return {
    id: `asset-balance-${i}`,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: code,
    security_name: name,
    shares: 100 * (i + 1),
    executing_shares: 0,
    average_purchase_price: 2500 + i * 300,
    total_purchase_amount: 250000 + i * 120000,
    current_price: 2600 + i * 310,
    daily_change: 50 - i * 12,
    market_value: 260000 + i * 125000,
    profit_loss_rate: 4.0 - i * 0.7,
  };
});

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
        data: STOCKS.map(([code]) => ({
          security_code: code,
          dividend_per_share: 30,
          status: 'ok',
          is_stale: false,
        })),
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

for (const width of [390, 639, 640, 1920]) {
  test(`資産評価レイアウト ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: width === 1920 ? 1080 : 844 });
    await mockApi(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('portfolio-pie-chart').last()).toBeVisible();
    await page.waitForTimeout(700);
    await page.screenshot({ path: `${SHOT_DIR}/asset-valuation-${width}.png`, fullPage: true, scale: 'css' });
  });
}
