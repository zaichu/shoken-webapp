import type { Page } from '@playwright/test';
import { expect, test, json, paginated, mockSession } from '../support/test';

/**
 * #2032: スマホの検索欄と並べ替えの入口の高さをそろえる。
 * 390px の資産管理・取引明細で、2つの高さの差が 1px 以内であること。
 * 押せる高さ 44px 以上と、検索欄の文字 16px 以上(iOS の入力時拡大防止)も保つ。
 */

const STOCKS = [
  {
    id: 'asset-balance-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: '9904',
    security_name: '架空自動車',
    shares: 100,
    average_purchase_price: 2500,
    total_purchase_amount: 250000,
    current_price: 2600,
    daily_change: 50,
  },
];

const DIVIDENDS = [
  {
    id: '00000000-0000-0000-0000-000000000001',
    settlement_date: '2024-03-01',
    product: '特定口座',
    account: 'SBI証券',
    security_code: '9904',
    security_name: '架空自動車',
    unit_price: 30.0,
    shares: 100,
    dividends_before_tax: 3000,
    taxes: 609,
    net_amount_received: 2391,
    created_at: '2024-03-01T00:00:00Z',
    updated_at: '2024-03-01T00:00:00Z',
  },
];

async function mockAsset(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(STOCKS))),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

async function mockReceipts(page: Page) {
  await page.route(/\/api\/v1\//, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated([])),
    }),
  );
  await mockSession(page);
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated(DIVIDENDS)),
    }),
  );
}

/** 検索欄の箱と並べ替えの入口の高さ・文字の大きさを読む */
async function readHeights(page: Page, sortTestId: string, inputTestId: string) {
  return page.evaluate(
    ([sortId, inputId]) => {
      const sort = document.querySelector(`[data-testid="${sortId}"]`);
      const input = document.querySelector(`[data-testid="${inputId}"]`);
      if (!sort || !input) {
        return null;
      }
      const box = (input.closest('.mobile-search-box') ?? input) as HTMLElement;
      const sortRect = (sort as HTMLElement).getBoundingClientRect();
      const boxRect = box.getBoundingClientRect();
      return {
        sortHeight: sortRect.height,
        boxHeight: boxRect.height,
        inputFontSize: parseFloat(getComputedStyle(input as Element).fontSize),
        sortFontSize: parseFloat(getComputedStyle(sort as Element).fontSize),
      };
    },
    [sortTestId, inputTestId] as const,
  );
}

async function expectSearchSortAligned(
  page: Page,
  label: string,
  sortTestId: string,
  inputTestId: string,
) {
  const sort = page.getByTestId(sortTestId);
  await expect(sort).toBeVisible();
  await expect(page.getByTestId(inputTestId)).toBeVisible();
  const sizes = await readHeights(page, sortTestId, inputTestId);
  expect(sizes, `${label}: 検索欄と並べ替えがある`).not.toBeNull();
  expect(
    Math.abs(sizes!.sortHeight - sizes!.boxHeight),
    `${label}: 検索欄と並べ替えの高さの差が 1px 以内`,
  ).toBeLessThanOrEqual(1);
  expect(sizes!.sortHeight, `${label}: 並べ替えは押せる高さ 44px 以上`).toBeGreaterThanOrEqual(44);
  expect(sizes!.boxHeight, `${label}: 検索欄は押せる高さ 44px 以上`).toBeGreaterThanOrEqual(44);
  expect(
    sizes!.inputFontSize,
    `${label}: 検索欄の文字は 16px 以上(iOS の入力時拡大を防ぐ)`,
  ).toBeGreaterThanOrEqual(16);
  expect(sizes!.sortFontSize, `${label}: 並べ替えの文字は検索欄と同じ大きさ`).toBe(
    sizes!.inputFontSize,
  );
}

test.beforeEach(async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
});

test('資産管理: 390px で検索欄と並べ替えの高さの差が 1px 以内', async ({ page }) => {
  await mockAsset(page);
  await page.goto('/assetbalance');
  await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
  await expectSearchSortAligned(page, '資産管理', 'asset-search-sheet-entry', 'asset-search-input-inline');
});

test('取引明細: 390px で検索欄と並べ替えの高さの差が 1px 以内', async ({ page }) => {
  await mockReceipts(page);
  await page.goto('/receipts');
  await expect(page.getByTestId('receipt-card-list')).toBeVisible();
  await expectSearchSortAligned(
    page,
    '取引明細',
    'receipts-search-sheet-entry',
    'receipts-search-input-inline',
  );
});
