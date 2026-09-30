import type { BrowserContext, Page } from '@playwright/test';
import { expect, test } from './support/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const STOCKS: Array<[string, string]> = [
  ['7203', 'トヨタ自動車'],
  ['9432', '日本電信電話'],
  ['6758', 'ソニーグループ'],
  ['8306', '三菱UFJフィナンシャル・グループ'],
  ['9984', 'ソフトバンクグループ'],
  ['4502', '武田薬品工業'],
  ['6861', 'キーエンス'],
  ['8058', '三菱商事'],
];

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
    unit_price: 10 + i * 3,
    shares: 100 * ((i % 5) + 1),
    dividends_before_tax: 3000 + i * 1234,
    taxes: 609 + i * 250,
    net_amount_received: 2391 + i * 984,
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
    shares: 100 * ((i % 4) + 1),
    asked_price: 2800 + i * 150,
    proceeds: 280000 + i * 15000,
    purchase_price: 2500 + i * 120,
    realized_profit_and_loss: 30000 - i * 2500,
    taxes: 6090 - i * 400,
    realized_profit_and_loss_after_tax: 23910 - i * 2100,
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
    shares: 10000 + i * 1500,
    exchange_rate: 1,
    cancellation_unit_price_yen: 21000 + i * 800,
    cancellation_amount_yen: 210000 + i * 12000,
    average_acquisition_price_yen: 18000 + i * 600,
    dividends: '0',
    realized_profit_and_loss: 30000 - i * 1800,
    taxes: 6090 - i * 350,
    realized_profit_and_loss_after_tax: 23910 - i * 1450,
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

const TABS = [
  { slug: 'dividend', name: '配当金', count: DIVIDENDS.length },
  { slug: 'domesticstock', name: '国内株式', count: DOMESTIC_STOCKS.length },
  { slug: 'mutualfund', name: '投資信託', count: MUTUALFUNDS.length },
] as const;

type ReceiptTab = (typeof TABS)[number];

function paginated(data: unknown[]) {
  return { data, total: data.length, page: 1, per_page: Math.max(data.length, 1) };
}

function json(body: unknown) {
  return { status: 200, contentType: 'application/json', body: JSON.stringify(body) };
}

// context 単位でモックするため、比較用に同じ context へ追加したページにも効く
async function mockApi(context: BrowserContext) {
  // 後から登録した route が優先されるため catch-all を先に登録する
  await context.route(/\/api\/v1\//, (route) => route.fulfill(json(paginated([]))));
  await context.route(/\/api\/v1\/session$/, (route) => route.fulfill(json(MOCK_USER)));
  await context.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DIVIDENDS))),
  );
  await context.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DOMESTIC_STOCKS))),
  );
  await context.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(MUTUALFUNDS))),
  );
  await context.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(ASSET_BALANCES))),
  );
  await context.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(
      json({
        items: STOCKS.map(([code]) => ({
          security_code: code,
          dividend_per_share: 30,
          status: 'ok',
          is_stale: false,
        })),
      }),
    ),
  );
}

// 同名テキストは別ブレークポイント用の hidden カードにも存在するため visible で絞る
async function expectAssetDataLoaded(page: Page) {
  await expect(
    page
      .getByTestId('assetbalance-main-stage')
      .getByText('トヨタ自動車')
      .filter({ visible: true })
      .first(),
  ).toBeVisible();
}

async function selectReceiptTab(page: Page, tab: ReceiptTab) {
  const button = page.getByRole('tab', { name: tab.name });
  await button.click();
  await expect(button).toHaveAttribute('aria-selected', 'true');
  // 件数バッジはタブ別フェッチの完了を意味する
  await expect(page.getByTestId(`tab-count-${tab.slug}`)).toHaveText(String(tab.count));
}

async function expectTableFit(page: Page) {
  const table = page.getByRole('table');
  await expect(table).toBeVisible();
  await expect(table.locator('tbody tr').last()).toBeVisible();
  const metrics = await table.evaluate((element) => {
    const wrapper = element.parentElement!;
    const rect = element.getBoundingClientRect();
    const wrapperRect = wrapper.getBoundingClientRect();
    return {
      left: rect.left,
      right: rect.right,
      wrapperLeft: wrapperRect.left,
      wrapperRight: wrapperRect.right,
      scrollWidth: wrapper.scrollWidth,
      clientWidth: wrapper.clientWidth,
    };
  });
  expect(metrics.left).toBeGreaterThanOrEqual(metrics.wrapperLeft - 1);
  expect(metrics.right).toBeLessThanOrEqual(metrics.wrapperRight + 1);
  expect(metrics.scrollWidth).toBeLessThanOrEqual(metrics.clientWidth + 1);
}

async function expectNoPageOverflow(page: Page, width: number) {
  const scrollWidth = await page.evaluate(() => document.documentElement.scrollWidth);
  expect(scrollWidth, 'ページ自体は横にはみ出さない').toBeLessThanOrEqual(width + 1);
}

function serverError() {
  return { status: 500, contentType: 'application/json', body: '{}' };
}

// 後から登録した route が優先されるため、基本モックの後に呼ぶ
async function mockReceiptFetchErrors(page: Page) {
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) => route.fulfill(serverError()));
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) => route.fulfill(serverError()));
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) => route.fulfill(serverError()));
}

async function gotoFilteredEmptyAssetBalance(page: Page) {
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(
      json({
        ...paginated(ASSET_BALANCES),
        facets: {
          securities: [
            ...STOCKS.map(([value, label]) => ({ value, label, count: 1 })),
            { value: '9999', label: '架空銘柄', count: 1 },
          ],
        },
      }),
    ),
  );
  await page.goto('/assetbalance');
  await expectAssetDataLoaded(page);
  await page.locator('#securities-search').selectOption('9999');
  await expect(page.getByRole('button', { name: '絞り込みを解除' })).toBeVisible();
}

test.beforeEach(async ({ context }) => {
  await mockApi(context);
});

for (const width of [1440, 1024]) {
  test(`取引明細の3タブがカード内に収まりページがはみ出さない(${width}px)`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    await page.goto('/receipts');
    for (const tab of TABS) {
      await selectReceiptTab(page, tab);
      await expectTableFit(page);
      await expectNoPageOverflow(page, width);
    }
  });
}

test('口座検索で絞り込んでも表が収まり該当データが残る', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto('/receipts');
  await selectReceiptTab(page, TABS[1]);
  await page.getByTestId('search-card').getByRole('button', { name: '特定口座', exact: true }).click();
  const table = page.getByRole('table');
  await expect(table).toContainText('特定口座');
  await expect(table).not.toContainText('NISA口座');
  await expectTableFit(page);
  await expectNoPageOverflow(page, 1440);
});

test('資産管理に評価額・評価損益は表示しない', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/assetbalance');
  await expectAssetDataLoaded(page);
  const main = page.getByTestId('assetbalance-main-stage');
  const summary = main.getByTestId('asset-portfolio-summary');
  await expect(summary).toBeVisible();
  await expect(summary).not.toContainText('評価額');
  await expect(summary).not.toContainText('評価損益');
  await expect(page.getByTestId('portfolio-holding-card').first()).toBeVisible();
  await expect(main).not.toContainText('評価額');
  await expect(main).not.toContainText('評価損益');
  await expect(main.locator('[data-negative="true"]')).toHaveCount(0);
});

for (const width of [1440, 390]) {
  test(`資産管理のデータと合計が表示されページがはみ出さない(${width}px)`, async ({ page }) => {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await page.goto('/assetbalance');
    await expect(page.locator('#main-content h1').first()).toBeVisible();
    await expect(page.getByTestId('assetbalance-workspace')).toBeVisible();
    await expectAssetDataLoaded(page);
    await expectNoPageOverflow(page, width);
    const summary = page.getByTestId('asset-portfolio-summary');
    await expect(summary).toBeVisible();
    await expect(summary).toContainText('¥5,360,000');
  });

  test(`取引明細の取得失敗を知らせ検索を残す(${width}px)`, async ({ page }) => {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await mockReceiptFetchErrors(page);
    await page.goto('/receipts');
    const main = page.getByTestId('receipt-main-stage');
    const alert = main.getByRole('alert');
    await expect(alert).toBeVisible();
    await expect(alert).not.toBeEmpty();
    await expect(page.getByRole('alert')).toHaveCount(1);
    await expect(page.getByTestId('receipt-utility-rail').getByTestId('search-card-compact')).toBeVisible();
    await expect(main.getByTestId('receipt-card')).toHaveCount(0);
    await expectNoPageOverflow(page, width);
  });

  test(`資産管理の取得失敗を知らせ誤った空状態を出さない(${width}px)`, async ({ page }) => {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => route.fulfill(serverError()));
    await page.goto('/assetbalance');
    const main = page.getByTestId('assetbalance-main-stage');
    const alert = main.getByRole('alert');
    await expect(alert).toBeVisible();
    await expect(alert).not.toBeEmpty();
    await expect(page.getByRole('alert')).toHaveCount(1);
    await expect(page.getByTestId('asset-review-prompt-card')).toHaveCount(0);
    await expect(main).not.toContainText('資産管理データがありません');
    await expectNoPageOverflow(page, width);
  });
}

test('検索の取得失敗を通知する', async ({ page }) => {
  await page.route(/\/api\/v1\/stocks(?:\?.*)?$/, (route) => route.fulfill(serverError()));
  await page.goto('/search?code=7203');
  await expect(page.getByRole('alert')).toBeVisible();
  await expect(page.getByRole('alert')).not.toBeEmpty();
});

test('資産管理で該当銘柄がないとき絞り込みを解除できる', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await gotoFilteredEmptyAssetBalance(page);
  await expect(page.getByTestId('portfolio-card-identity')).toHaveCount(0);
  await page.getByRole('button', { name: '絞り込みを解除' }).click();
  await expectAssetDataLoaded(page);
});

test('資産管理の検索とCSV操作に390pxでもアクセスできる', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/assetbalance');
  await expectAssetDataLoaded(page);
  const rail = page.getByTestId('assetbalance-utility-rail');
  await expect(rail.getByTestId('assetbalance-csv-toggle')).toBeVisible();
  await expect(rail.getByTestId('search-card-compact')).toBeVisible();
  await expect(rail.getByTestId('asset-review-prompt-card')).toBeVisible();
  await expect(page.locator('#securities-search')).toBeVisible();
});

test('取引明細 390px はカード表示でページはみ出しなし', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/receipts');
  await expect(page.getByRole('tab')).toHaveCount(3);
  const cardList = page.getByTestId('receipt-card-list');
  for (const tab of TABS) {
    await selectReceiptTab(page, tab);
    await expect(cardList.getByTestId('receipt-card')).toHaveCount(tab.count);
    await expect(page.getByRole('table')).toBeHidden();
    await expectNoPageOverflow(page, 390);
  }
});
