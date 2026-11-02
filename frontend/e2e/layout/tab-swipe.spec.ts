import type { Page } from '@playwright/test';
import { expect, test, json, mockSession } from '../support/test';

const STOCK_ROWS = [
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
];

const FUND_ROWS = [
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
];

async function setupAssetMocks(page: Page) {
  await mockSession(page);
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
      }),
    ),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

async function setupReceiptMocks(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json({ data: [], total: 0, page: 1, per_page: 1 })),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json({ data: [], total: 0, page: 1, per_page: 1 })),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json({ data: [], total: 0, page: 1, per_page: 1 })),
  );
}

async function swipe(
  page: Page,
  testid: string,
  fromX: number,
  toX: number,
  y = 400,
  toY?: number,
) {
  await page.getByTestId(testid).evaluate(
    (
      element,
      { fromX: startX, toX: endX, y: startY, toY: endY }: {
        fromX: number;
        toX: number;
        y: number;
        toY?: number;
      },
    ) => {
      const start = new Touch({
        identifier: 1,
        target: element,
        clientX: startX,
        clientY: startY,
      });
      element.dispatchEvent(
        new TouchEvent('touchstart', {
          touches: [start],
          bubbles: true,
          cancelable: true,
        }),
      );
      const end = new Touch({
        identifier: 1,
        target: element,
        clientX: endX,
        clientY: endY ?? startY,
      });
      element.dispatchEvent(
        new TouchEvent('touchend', {
          changedTouches: [end],
          bubbles: true,
          cancelable: true,
        }),
      );
    },
    { fromX, toX, y, toY },
  );
}

test.describe('下線タブの横スワイプ', () => {
  test.use({ viewport: { width: 390, height: 844 } });

  test('資産管理で左スワイプすると次のタブに切り替わり右スワイプで戻る', async ({
    page,
  }) => {
    await setupAssetMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    await expect(page.getByTestId('asset-kind-tab-stock')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await swipe(page, 'assetbalance-main-stage', 300, 200);
    await expect(page.getByTestId('asset-kind-tab-fund')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await swipe(page, 'assetbalance-main-stage', 100, 200);
    await expect(page.getByTestId('asset-kind-tab-stock')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    // 端から始めた接触・短い移動・縦方向の移動では切り替わらない
    await swipe(page, 'assetbalance-main-stage', 5, 200);
    await expect(page.getByTestId('asset-kind-tab-stock')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await swipe(page, 'assetbalance-main-stage', 300, 270);
    await expect(page.getByTestId('asset-kind-tab-stock')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await swipe(page, 'assetbalance-main-stage', 300, 280, 400, 200);
    await expect(page.getByTestId('asset-kind-tab-stock')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    // 端のタブ(投資信託)では外向きのスワイプで動かない
    await swipe(page, 'assetbalance-main-stage', 300, 200);
    await expect(page.getByTestId('asset-kind-tab-fund')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await swipe(page, 'assetbalance-main-stage', 300, 200);
    await expect(page.getByTestId('asset-kind-tab-fund')).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });

  test('取引明細で左スワイプすると次のタブに切り替わり検索ダイアログ中は動かない', async ({
    page,
  }) => {
    await setupReceiptMocks(page);
    await page.goto('/receipts');
    await expect(page.getByRole('tab')).toHaveCount(3);

    await swipe(page, 'receipts-workspace', 300, 200);
    await expect(page.locator('#tab-dividend')).toHaveAttribute(
      'aria-selected',
      'false',
    );

    await page.getByTestId('receipts-search-sheet-entry').click();
    await expect(page.getByTestId('receipts-search-sheet')).toBeVisible();
    await swipe(page, 'receipts-workspace', 300, 200);
    await expect(page.locator('#tab-dividend')).toHaveAttribute(
      'aria-selected',
      'false',
    );
  });
});
