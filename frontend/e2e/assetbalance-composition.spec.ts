import type { Page } from '@playwright/test';
import { expect, test } from './support/test';

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

// 22銘柄: 先頭は8桁の取得総額、末尾はマイナス取得額(その他集約に入る)
const HOLDINGS = [
  holding(1, 70_000_000),
  ...Array.from({ length: 20 }, (_, i) => holding(i + 2, (20 - i) * 10_000)),
  holding(22, -10_000),
];

function jsonResponse(body: unknown, status = 200) {
  return {
    status,
    contentType: 'application/json',
    body: JSON.stringify(body),
  };
}

async function setupAssetBalanceMocks(
  page: Page,
  holdings: ReturnType<typeof holding>[] = HOLDINGS,
) {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill(jsonResponse(MOCK_USER)),
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
      jsonResponse({
        data: holdings,
        total: holdings.length,
        page: 1,
        per_page: 1000,
        summary: {
          total_purchase_amount: totalPurchase,
          total_market_value: totalPurchase,
          total_daily_change: 0,
        },
        facets: {
          securities: holdings.map((h) => ({
            value: h.security_code,
            label: h.security_name,
            count: 1,
          })),
        },
      }),
    );
  });

  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(jsonResponse({ items: [] })),
  );
}

test('構成比の帯グラフと凡例は上位20銘柄+その他にまとまり銘柄ごとの色と対応する', async ({
  page,
}) => {
  await setupAssetBalanceMocks(page);
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('/assetbalance');

  const composition = page.getByTestId('portfolio-composition');
  await expect(composition).toBeVisible();

  const segments = composition.locator('div[aria-hidden="true"] > div');
  await expect(segments).toHaveCount(21);

  const legend = composition.locator('ul > li');
  await expect(legend).toHaveCount(21);
  await expect(legend.nth(0)).toContainText('銘柄01');
  await expect(legend.nth(0)).toContainText('97.1%');
  await expect(legend.nth(19)).toContainText('銘柄20');
  await expect(legend.nth(20)).toContainText('その他 2銘柄');
  const segmentColors = await segments.evaluateAll((elements) =>
    elements.map((element) => getComputedStyle(element).backgroundColor),
  );
  segmentColors.forEach((color, index) => {
    expect(color, `帯${index}は透明でない`).not.toBe('rgba(0, 0, 0, 0)');
  });
  for (let index = 1; index < segmentColors.length; index += 1) {
    expect(segmentColors[index], `帯${index}は前の帯と色が異なる`).not.toBe(
      segmentColors[index - 1],
    );
  }
  for (const index of [0, 19, 20]) {
    const color = await legend.nth(index).locator('span').first().evaluate(
      (element) => getComputedStyle(element).backgroundColor,
    );
    expect(color).toBe(segmentColors[index]);
  }

  const grid = page.getByTestId('portfolio-items-grid');
  const firstCard = grid.locator(':scope > div').filter({
    has: page.getByTestId('portfolio-card-identity').filter({ hasText: '銘柄01' }),
  });
  await expect(firstCard).toHaveCount(1);
  await expect(firstCard).toContainText('銘柄01');
  const twentiethCard = grid.locator(':scope > div').filter({
    has: page.getByTestId('portfolio-card-identity').filter({ hasText: '銘柄20' }),
  });
  await expect(twentiethCard).toHaveCount(1);
  await expect(twentiethCard).toContainText('銘柄20');

  const firstBar = firstCard.getByTestId('portfolio-card-composition-bar');
  expect(await firstBar.evaluate((element) => getComputedStyle(element).backgroundColor)).toBe(segmentColors[0]);
  const twentiethBar = twentiethCard.getByTestId('portfolio-card-composition-bar');
  expect(await twentiethBar.evaluate((element) => getComputedStyle(element).backgroundColor)).toBe(segmentColors[19]);

  const code = firstCard.getByTestId('portfolio-card-code');
  await expect(code).toBeVisible();
  const codeBox = await code.boundingBox();
  const nameBox = await firstCard.getByTestId('portfolio-card-identity').getByText('銘柄01').boundingBox();
  expect(codeBox).not.toBeNull();
  expect(nameBox).not.toBeNull();
  expect(codeBox!.x + codeBox!.width).toBeLessThanOrEqual(nameBox!.x + 1);
});

test('保有カードの取得総額は8桁の金額でも省略されない', async ({ page }) => {
  await setupAssetBalanceMocks(page);
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('/assetbalance');

  const stats = page.getByTestId('portfolio-card-acquisition-stats').first();
  await expect(stats).toBeVisible();

  const value = stats.locator('p[title="¥70,000,000"]');
  await expect(value).toHaveText('¥70,000,000');
  const clipped = await value.evaluate(
    (node) => node.scrollWidth > node.clientWidth,
  );
  expect(clipped).toBe(false);
});

test('390px の保有カードは銘柄コードが銘柄名の左に並ぶ', async ({ page }) => {
  await setupAssetBalanceMocks(page);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/assetbalance');

  const card = page.getByTestId('portfolio-holding-card').first();
  await expect(card).toBeVisible();

  const code = card.getByTestId('portfolio-holding-card-code');
  await expect(code).toBeVisible();

  const codeBox = await code.boundingBox();
  const nameBox = await card.getByText('銘柄01').boundingBox();
  expect(codeBox).not.toBeNull();
  expect(nameBox).not.toBeNull();
  expect(codeBox!.x + codeBox!.width).toBeLessThanOrEqual(nameBox!.x + 1);
});

test('取得額が0の銘柄も保有カードに残り、帯と凡例からは外れる', async ({
  page,
}) => {
  const gifted = { ...holding(2, 0), security_name: '贈与銘柄' };
  await setupAssetBalanceMocks(page, [holding(1, 100_000), gifted]);
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('/assetbalance');

  await expect(page.getByTestId('portfolio-kpi-grid')).toContainText('2');
  await expect(page.getByTestId('portfolio-card-identity')).toHaveCount(2);
  const giftedCard = page
    .getByTestId('portfolio-card-identity')
    .filter({ hasText: '贈与銘柄' });
  await expect(giftedCard).toHaveCount(1);

  await expect(page.getByText('構成比 —')).toHaveCount(1);

  const composition = page.getByTestId('portfolio-composition');
  await expect(
    composition.locator('div[aria-hidden="true"] > div'),
  ).toHaveCount(1);
  const legend = composition.locator('ul > li');
  await expect(legend).toHaveCount(1);
  await expect(legend.first()).toContainText('銘柄01');
  await expect(legend.first()).toContainText('100.0%');
});

test('0円銘柄を含む21+1件ではカード一覧と帯・凡例の「その他」件数が一致する', async ({
  page,
}) => {
  const holdings = [
    ...Array.from({ length: 21 }, (_, i) => holding(i + 1, 100_000)),
    { ...holding(22, 0), security_name: '贈与銘柄' },
  ];
  await setupAssetBalanceMocks(page, holdings);
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('/assetbalance');

  const composition = page.getByTestId('portfolio-composition');
  const legend = composition.locator('ul > li');
  await expect(legend).toHaveCount(21);
  await expect(legend.nth(20)).toContainText('その他 1銘柄');

  const grid = page.getByTestId('portfolio-items-grid');
  await expect(grid.getByText('その他 1銘柄')).toHaveCount(1);
  await expect(page.getByTestId('portfolio-card-identity')).toHaveCount(21);
  await expect(
    page.getByTestId('portfolio-card-identity').filter({ hasText: '贈与銘柄' }),
  ).toHaveCount(1);
  await expect(
    page.getByRole('button', { name: '残り1銘柄を表示（全22）' }),
  ).toBeVisible();
});
