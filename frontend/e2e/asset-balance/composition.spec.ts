import type { Page } from '@playwright/test';
import { expect, test, json } from '../support/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000010',
  email: 'test@example.com',
  name: 'テストユーザー',
};

function holding(index: number, purchase: number) {
  return {
    id: `asset-balance-${index}`,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: `70${String(index).padStart(2, '0')}`,
    security_name: `銘柄${String(index).padStart(2, '0')}`,
    shares: 100,
    executing_shares: 0,
    average_purchase_price: 100,
    total_purchase_amount: purchase,
    current_price: 100,
    daily_change: 0,
  };
}

// 22銘柄: 先頭は8桁の取得総額、末尾はマイナス取得額
const HOLDINGS = [
  holding(1, 70_000_000),
  ...Array.from({ length: 20 }, (_, i) => holding(i + 2, (20 - i) * 10_000)),
  holding(22, -10_000),
];


async function setupAssetBalanceMocks(
  page: Page,
  holdings: ReturnType<typeof holding>[] = HOLDINGS,
) {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill(json(MOCK_USER)),
  );

  const totalPurchase = holdings.reduce(
    (sum, h) => sum + h.total_purchase_amount,
    0,
  );

  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => {
    if (route.request().method() !== 'GET') {
      return route.abort();
    }
    return route.fulfill(
      json({
        data: holdings,
        total: holdings.length,
        page: 1,
        per_page: 1000,
        summary: {
          total_purchase_amount: totalPurchase,
          total_market_value: totalPurchase,
          total_daily_change: 0,
          priced_purchase_amount: totalPurchase,
          priced_market_value: totalPurchase,
        },
      }),
    );
  });

  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

test('構成比詳細は直接 URL で開け、基準と対象を示して取得額順に全保有を並べる', async ({
  page,
}) => {
  await setupAssetBalanceMocks(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/assetbalance/composition');

  await expect(page.getByText('対象: 保存済みの保有全体')).toBeVisible();
  await expect(page.getByText('基準(取得総額):')).toBeVisible();
  await expect(page.getByText('¥72,110,000')).toBeVisible();

  const items = page.getByTestId('asset-composition-list').getByRole('listitem');
  await expect(items).toHaveCount(22);
  await expect(items.nth(0)).toContainText('銘柄01');
  await expect(items.nth(0)).toContainText('97.1%');
  // 取得額が少ない順に末尾へ並ぶ。0・マイナスも一覧には残る
  await expect(items.nth(21)).toContainText('銘柄22');

  // トップからの導線
  await page.goto('/assetbalance');
  await page.getByRole('link', { name: '構成比を見る' }).click();
  await expect(page).toHaveURL('/assetbalance/composition');
});

test('取得額が0の銘柄は構成比を出さずに一覧へ残す', async ({ page }) => {
  const gifted = { ...holding(2, 0), security_name: '贈与銘柄' };
  await setupAssetBalanceMocks(page, [holding(1, 100_000), gifted]);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/assetbalance/composition');

  const items = page.getByTestId('asset-composition-list').getByRole('listitem');
  await expect(items).toHaveCount(2);
  await expect(items.nth(0)).toContainText('銘柄01');
  await expect(items.nth(0)).toContainText('100.0%');
  const giftedItem = items.filter({ hasText: '贈与銘柄' });
  await expect(giftedItem).toHaveCount(1);
  await expect(giftedItem).toContainText('—');
});

test('取得総額の合計が0のときは算出不可を表示する', async ({ page }) => {
  const holdings = [holding(1, 0), holding(2, 0)];
  await setupAssetBalanceMocks(page, holdings);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/assetbalance/composition');

  await expect(page.getByText('算出不可')).toBeVisible();
});
