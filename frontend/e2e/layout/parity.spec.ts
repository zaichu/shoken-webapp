import type { BrowserContext, Page } from '@playwright/test';
import { expectTableFit } from '../support/layout';
import { expect, test, json, paginated, MOCK_USER, mockSession } from '../support/test';
import { csvRow } from '../support/csv-page';

const STOCKS: Array<[string, string]> = [
  ['9904', '架空自動車'],
  ['9910', '架空電信電話'],
  ['9914', '架空エレクトロニクス'],
  ['9907', '架空フィナンシャル・グループ'],
  ['9915', '架空通信ホールディングス'],
  ['9905', '架空医薬品'],
  ['9911', '架空精密機器'],
  ['9918', '架空商事'],
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
        ? '架空インデックス 全世界株式（オール・ワールド）'
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

// context 単位でモックするため、比較用に同じ context へ追加したページにも効く
async function mockApi(context: BrowserContext) {
  // 後から登録した route が優先されるため catch-all を先に登録する
  await context.route(/\/api\/v1\//, (route) => route.fulfill(json(paginated([]))));
  await mockSession(context);
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
      .getByText('架空自動車')
      .filter({ visible: true })
      .first(),
  ).toBeVisible();
}

// スマホ下部の常設検索欄。PC の検索バーは同名の入力を持つので scoped で取る
function mobileSearch(page: Page) {
  return page
    .getByRole('search', { name: '検索', exact: true })
    .getByPlaceholder('銘柄名・コード');
}

async function selectReceiptTab(page: Page, tab: ReceiptTab) {
  const button = page.getByRole('tab', { name: tab.name });
  await button.click();
  await expect(button).toHaveAttribute('aria-selected', 'true');
  // 件数バッジはタブ別フェッチの完了を意味する
  await expect(page.getByTestId(`tab-count-${tab.slug}`)).toHaveText(String(tab.count));
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
  await page.goto('/assetbalance');
  await expectAssetDataLoaded(page);
  // どの銘柄にも一致しない文字列で 0 件にする
  await page.getByTestId('asset-search-input-pc').fill('9999');
  await expect(
    page.getByRole('button', { name: '検索条件を解除' }),
  ).toBeVisible();
}

async function expectReceiptsErrorLayout(page: Page, width: number) {
  const main = page.getByTestId('receipts-workspace');
  const alert = main.getByRole('alert');
  await expect(alert).toBeVisible();
  await expect(alert).not.toBeEmpty();
  await expect(page.getByRole('alert')).toHaveCount(1);
  await expect(main.getByTestId('receipt-card')).toHaveCount(0);
  // 検索は常に画面下手前にあるのでどの幅でも残る
  if (width < 1024) {
    await expect(mobileSearch(page)).toBeVisible();
  }
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
  await page.getByRole('button', { name: '＋ 口座' }).click();
  await page
    .getByRole('dialog', { name: '口座を選ぶ' })
    .getByRole('button', { name: '特定', exact: true })
    .click();
  const table = page.getByRole('table');
  await expect(table).toContainText('特定');
  await expect(table).not.toContainText('NISA口座');
  await expectTableFit(page);
  await expectNoPageOverflow(page, 1440);
});

test('資産管理に評価額と評価損益を表示する', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/assetbalance');
  await expectAssetDataLoaded(page);
  const main = page.getByTestId('assetbalance-main-stage');
  const summary = main.getByTestId('asset-portfolio-summary');
  await expect(summary).toBeVisible();
  await expect(summary).toContainText('評価額合計');
  await expect(summary).toContainText('評価損益');
  const row = page.getByTestId('asset-compact-row').first();
  await expect(row).toBeVisible();
  // 行には評価額と符号付きの評価損益が出る
  await expect(row).toContainText('¥');
  await expect(row).toContainText(/[+-]¥/);
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
    await expect(summary).toContainText('¥14,568,000');
  });

  test(`取引明細の取得失敗を知らせ検索を残す(${width}px)`, async ({ page }) => {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await mockReceiptFetchErrors(page);
    await page.goto('/receipts');
    await expectReceiptsErrorLayout(page, width);
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
    await expect(main).not.toContainText('資産管理データがありません');
    await expectNoPageOverflow(page, width);
  });
}

test('検索の取得失敗を通知する', async ({ page }) => {
  await page.route(/\/api\/v1\/stocks(?:\?.*)?$/, (route) => route.fulfill(serverError()));
  await page.goto('/search?code=9904');
  await expect(page.getByRole('alert')).toBeVisible();
  await expect(page.getByRole('alert')).not.toBeEmpty();
});

test('資産管理で該当銘柄がないとき絞り込みを解除できる', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await gotoFilteredEmptyAssetBalance(page);
  await expect(page.getByTestId('asset-compact-row')).toHaveCount(0);
  await page.getByRole('button', { name: '検索条件を解除' }).click();
  await expectAssetDataLoaded(page);
});

test('資産管理の検索とCSV管理ページに390pxでもアクセスできる', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/assetbalance');
  await expectAssetDataLoaded(page);
  // 検索欄は一覧の上に常時出すのでドロワーを開かなくても届く
  await expect(page.getByTestId('asset-search-input')).toBeEditable();
  // CSV 操作は専用ページに置く
  await page.getByRole('link', { name: 'CSV管理' }).click();
  await expect(page).toHaveURL('/csv');
  await expect(csvRow(page, 'assetbalance')).toBeVisible();
  await page
    .getByRole('navigation', { name: 'パンくず' })
    .getByRole('link', { name: '資産管理' })
    .click();
  await expect(page).toHaveURL('/assetbalance');
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

const PARITY_FUNDS = [
  {
    id: 'fund-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    fund_name: '架空インデックス全世界株式オールワールド長期保有テストファンド',
    account: '特定',
    dividends_course: '受取型',
    shares: 10000,
    average_purchase_price: 9970.5,
    total_purchase_amount: 9970,
    base_price: 10012.25,
    base_price_day_change: 5,
    base_price_month_change: 120,
    market_value: 10012,
    profit_loss: 42,
    profit_loss_rate: 0.42,
    total_return: 42,
    price_as_of: '2026-10-20',
    price_source: 'api',
    nav_is_latest: true,
    expected_nav_date: null,
  },
  {
    id: 'fund-2',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    fund_name: '架空バランスファンドA号',
    account: '一般',
    dividends_course: '再投資型',
    shares: 20000,
    average_purchase_price: 9500,
    total_purchase_amount: 19000,
    base_price: 9800,
    base_price_day_change: -30,
    base_price_month_change: 120,
    market_value: 19600,
    profit_loss: 600,
    profit_loss_rate: 3.16,
    total_return: 600,
    price_as_of: '2026-10-20',
    price_source: 'api',
    nav_is_latest: true,
    expected_nav_date: null,
  },
];

async function setupParityMocks(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(ASSET_BALANCES))),
  );
  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
    route.fulfill(
      json({
        data: PARITY_FUNDS,
        total: PARITY_FUNDS.length,
        page: 1,
        per_page: 1000,
        summary: {
          total_market_value: 29612,
          total_purchase_amount: 28970,
          total_daily_change: 0,
          latest_nav_count: PARITY_FUNDS.length,
          pending_nav_count: 0,
          nav_status: {
            updated_at: '2026-10-20 18:05',
            fetched_count: 1,
            failed_count: 0,
            fund_count: PARITY_FUNDS.length,
            auto_update_target_count: PARITY_FUNDS.length,
            failure_reason: null,
          },
        },
      }),
    ),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

type Box = { y: number; height: number };

async function skeletonBox(page: Page, selector: string): Promise<Box> {
  await page.evaluate(() => window.scrollTo(0, 0));
  const locator = page.locator(selector).first();
  await expect(locator).toBeVisible();
  const box = await locator.boundingBox();
  expect(box).not.toBeNull();
  return { y: box!.y, height: box!.height };
}

async function expectTopsEqual(page: Page, other: Page, selector: string, label: string) {
  const a = await skeletonBox(page, selector);
  const b = await skeletonBox(other, selector);
  expect(Math.abs(a.y - b.y), `${label}の上端`).toBeLessThanOrEqual(1);
  expect(Math.abs(a.height - b.height), `${label}の高さ`).toBeLessThanOrEqual(1);
}

test.describe('資産管理と取引明細の上部 parity', () => {
  for (const { width, height } of [
    { width: 1280, height: 800 },
    { width: 1920, height: 1080 },
  ]) {
    test(`タブ・帯・検索の位置と高さが一致する(${width}px)`, async ({
      page,
      context,
    }) => {
      await page.setViewportSize({ width, height });
      await setupParityMocks(page);
      const other = await context.newPage();
      await other.setViewportSize({ width, height });
      await setupParityMocks(other);
      try {
        await page.goto('/assetbalance');
        await expectAssetDataLoaded(page);
        await other.goto('/receipts');
        await other.getByTestId('receipt-summary-strip').first().waitFor();
        for (const selector of [
          '.page-skeleton-tabs',
          '.page-skeleton-summary',
          '.page-skeleton-note',
          '.page-skeleton-search',
        ]) {
          await expectTopsEqual(page, other, selector, selector);
        }
        const assetTab = await skeletonBox(page, '.page-tab');
        const receiptTab = await skeletonBox(other, '.page-tab');
        expect(Math.abs(assetTab.height - receiptTab.height), 'タブの高さ').toBeLessThanOrEqual(1);
        // タブ切替でも上端が動かない
        const before = await skeletonBox(page, '.page-skeleton-tabs');
        await page.getByTestId('asset-kind-tab-fund').click();
        await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
        const afterFund = await skeletonBox(page, '.page-skeleton-tabs');
        expect(Math.abs(afterFund.y - before.y), '資産管理のタブ切替').toBeLessThanOrEqual(1);
        await page.getByTestId('asset-kind-tab-all').click();
        await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
        const afterAll = await skeletonBox(page, '.page-skeleton-tabs');
        expect(Math.abs(afterAll.y - before.y), '資産管理のタブ切替(すべて)').toBeLessThanOrEqual(1);
        const receiptBefore = await skeletonBox(other, '.page-skeleton-tabs');
        await selectReceiptTab(other, TABS[1]);
        const receiptAfter = await skeletonBox(other, '.page-skeleton-tabs');
        expect(Math.abs(receiptAfter.y - receiptBefore.y), '取引明細のタブ切替').toBeLessThanOrEqual(1);
      } finally {
        await other.close();
      }
    });
  }

  test('スマホではタブと帯の上端が一致する(390px)', async ({ page, context }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await setupParityMocks(page);
    const other = await context.newPage();
    await other.setViewportSize({ width: 390, height: 844 });
    await setupParityMocks(other);
    try {
      await page.goto('/assetbalance');
      await expectAssetDataLoaded(page);
      await other.goto('/receipts');
      await other.getByTestId('receipt-summary-strip').first().waitFor();
      // 検索欄は配置が違う(資産管理は本文内・取引明細は下部固定)のため、
      // スマホではタブと帯の上端だけを比べる。帯の高さは項目数が違うため比べない
      for (const selector of ['.page-skeleton-tabs', '.page-skeleton-summary']) {
        const a = await skeletonBox(page, selector);
        const b = await skeletonBox(other, selector);
        expect(Math.abs(a.y - b.y), `${selector}の上端`).toBeLessThanOrEqual(1);
      }
      await expectNoPageOverflow(page, 390);
      await expectNoPageOverflow(other, 390);
    } finally {
      await other.close();
    }
  });

  test('長い名前・30件でも上部がずれない', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await mockSession(page);
    const longStocks = Array.from({ length: 30 }, (_, i) => ({
      id: `asset-long-${i}`,
      created_at: '2026-01-01T00:00:00Z',
      updated_at: '2026-01-01T00:00:00Z',
      security_code: '9904',
      security_name: '架空自動車ホールディングスインターナショナルグループ株式会社',
      shares: 100 * (i + 1),
      average_purchase_price: 2500,
      total_purchase_amount: 250000,
      current_price: 2600,
      daily_change: 10,
      market_value: 260000,
      profit_loss_rate: 4.0,
    }));
    await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
      route.fulfill(json({ data: longStocks, total: 30, page: 1, per_page: 1000 })),
    );
    await page.goto('/assetbalance');
    await expectAssetDataLoaded(page);
    await expectNoPageOverflow(page, 1280);
    const tabs = await skeletonBox(page, '.page-skeleton-tabs');
    expect(tabs.y).toBeGreaterThan(0);
  });
});
