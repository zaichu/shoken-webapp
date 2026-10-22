import type { Page } from '@playwright/test';
import { expect, test, json, paginated } from '../support/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000010',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const STOCKS = [
  {
    id: 'asset-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: '9901',
    security_name: '架空自動車',
    account: '特定',
    shares: 100,
    average_purchase_price: 5997.6,
    total_purchase_amount: 599760,
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
    security_name: '架空精密機器',
    account: '一般',
    shares: 200,
    average_purchase_price: 187.36,
    total_purchase_amount: 37472,
    current_price: 162,
    daily_change: -3,
    price_as_of: '2026-10-20',
    prev_close: 165,
  },
];

const FUNDS = [
  {
    id: 'fund-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    fund_name: '架空バランスファンドA号',
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
    fund_name: '架空インデックスファンドB号',
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
    price_source: 'csv',
    nav_is_latest: true,
    expected_nav_date: null,
  },
];

const DIVIDENDS = Array.from({ length: 30 }, (_, i) => ({
  id: `dividend-${i}`,
  user_id: MOCK_USER.id,
  settlement_date: `2026-09-${String((i % 28) + 1).padStart(2, '0')}`,
  product: '国内株式',
  account: i % 2 === 0 ? '特定口座' : 'NISA口座',
  security_code: '9901',
  security_name: '架空自動車',
  unit_price: '100',
  shares: '1000',
  dividends_before_tax: '12345',
  taxes: '2345',
  net_amount_received: '10000',
  created_at: '2026-03-01T00:00:00Z',
  updated_at: '2026-03-01T00:00:00Z',
}));

async function setupMocks(page: Page) {
  await page.route(/\/api\/v1\/session$/, (route) => route.fulfill(json(MOCK_USER)));
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
    route.fulfill(
      json({
        data: FUNDS,
        total: FUNDS.length,
        page: 1,
        per_page: 1000,
        summary: {
          total_market_value: 29612,
          total_purchase_amount: 28970,
          total_daily_change: 0,
          latest_nav_count: FUNDS.length,
          pending_nav_count: 0,
          nav_status: {
            updated_at: '2026-10-20 18:05',
            fetched_count: FUNDS.length,
            failed_count: 0,
            failure_reason: null,
          },
        },
      }),
    ),
  );
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DIVIDENDS))),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

// ページの骨格の各段。上下の順と、段どうしの間隔を測る
type Skeleton = {
  tops: number[];
  bottoms: number[];
};

async function measureSkeleton(page: Page, selectors: string[]): Promise<Skeleton> {
  const boxes = await page.evaluate((sels) => {
    return sels.map((sel) => {
      const element = document.querySelector(sel);
      if (!element) {
        throw new Error(`要素が見つかりません: ${sel}`);
      }
      const rect = element.getBoundingClientRect();
      return { top: rect.top, bottom: rect.bottom };
    });
  }, selectors);
  return {
    tops: boxes.map((box) => box.top),
    bottoms: boxes.map((box) => box.bottom),
  };
}

const ASSET_SELECTORS = [
  '.page-skeleton-tabs',
  '[data-testid="asset-portfolio-summary"]',
  '.page-skeleton-note',
  '.search-field',
  '.add-condition-row',
  '.asset-table',
];

const RECEIPT_SELECTORS = [
  '.page-skeleton-tabs',
  '[data-testid="receipt-summary-strip"]',
  '.page-skeleton-note',
  '.search-field',
  '.add-condition-row',
  '.receipts-table',
];

test.describe('資産管理と取引明細のページの骨格', () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  test('縦の順番が同じで、段どうしの間隔もそろう', async ({ page }) => {
    await setupMocks(page);

    await page.goto('/assetbalance');
    await page.getByTestId('asset-kind-tab-stock').click();
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    const asset = await measureSkeleton(page, ASSET_SELECTORS);

    await page.goto('/receipts');
    await expect(page.getByTestId('receipt-summary-strip')).toBeVisible();
    const receipt = await measureSkeleton(page, RECEIPT_SELECTORS);

    // 段の順番は上下に並ぶ(上の段が下の段より上にある)
    for (const skeleton of [asset, receipt]) {
      for (let index = 1; index < skeleton.tops.length; index += 1) {
        expect(
          skeleton.tops[index],
          `段${index}は段${index - 1}より下にある`,
        ).toBeGreaterThan(skeleton.tops[index - 1]);
      }
    }

    // 段どうしの間隔が両画面で同じ(差 2px 以内)
    const gaps = (skeleton: Skeleton) =>
      skeleton.tops.slice(1).map((top, index) => top - skeleton.bottoms[index]);
    const assetGaps = gaps(asset);
    const receiptGaps = gaps(receipt);
    expect(assetGaps.length).toBe(receiptGaps.length);
    for (let index = 0; index < assetGaps.length; index += 1) {
      expect(
        Math.abs(assetGaps[index] - receiptGaps[index]),
        `段${index + 1}と段${index + 2}の間隔`,
      ).toBeLessThanOrEqual(2);
    }
  });

  test('絞り込み中は注記に件数が出て、条件を消すと消える', async ({ page }) => {
    await setupMocks(page);

    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    await expect(page.getByTestId('page-filtering-count')).toHaveCount(0);

    await page.getByTestId('asset-search-input-pc').fill('精密');
    await expect(page.getByTestId('page-filtering-count')).toBeVisible();
    await expect(page.getByTestId('page-filtering-count')).toContainText('1件 / 2件');

    await page.getByTestId('asset-search-clear').click();
    await expect(page.getByTestId('page-filtering-count')).toHaveCount(0);

    await page.goto('/receipts');
    await expect(page.getByTestId('receipt-summary-strip')).toBeVisible();
    await expect(page.getByTestId('page-filtering-count')).toHaveCount(0);

    await page.getByRole('button', { name: '＋ 口座' }).click();
    const dialog = page.getByRole('dialog', { name: '口座を選ぶ' });
    await expect(dialog).toBeVisible();
    await dialog.getByRole('button', { name: '特定', exact: true }).click();
    await expect(dialog).toBeHidden();

    await expect(page.getByTestId('page-filtering-count')).toBeVisible();
    await expect(page.getByTestId('page-filtering-count')).toContainText('15件 / 30件');

    await page.getByRole('button', { name: 'すべてクリア' }).click();
    await expect(page.getByTestId('page-filtering-count')).toHaveCount(0);
  });

  test('検索欄は入力のたびに作り直されない(文字ごとに打っても入力が続けられる)', async ({
    page,
  }) => {
    await setupMocks(page);

    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    const assetInput = page.getByTestId('asset-search-input-pc');
    await assetInput.click();
    await page.keyboard.type('精密');
    await expect(assetInput).toHaveValue('精密');

    await page.goto('/receipts');
    await expect(page.getByTestId('receipt-summary-strip')).toBeVisible();
    const receiptInput = page
      .getByRole('search', { name: '取引明細の検索' })
      .getByPlaceholder('銘柄名・コード');
    await receiptInput.click();
    await page.keyboard.type('9901');
    await expect(receiptInput).toHaveValue('9901');
  });
});
