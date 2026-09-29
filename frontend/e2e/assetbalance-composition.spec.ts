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

async function setupAssetBalanceMocks(page: Page) {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill(jsonResponse(MOCK_USER)),
  );

  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => {
    if (route.request().method() !== 'GET') {
      return route.abort();
    }
    return route.fulfill(
      jsonResponse({
        data: HOLDINGS,
        total: HOLDINGS.length,
        page: 1,
        per_page: 1000,
        summary: {
          total_purchase_amount: 72110000,
          total_market_value: 72120000,
          total_daily_change: 0,
        },
        facets: {
          securities: HOLDINGS.map((h) => ({
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

  // 帯は上位20色+その他(グレー)の21セグメント
  const segments = composition.locator('div[aria-hidden="true"] > div');
  await expect(segments).toHaveCount(21);
  await expect(segments.nth(0)).toHaveClass(/bg-holding-1\b/);
  await expect(segments.nth(19)).toHaveClass(/bg-holding-20\b/);
  await expect(segments.nth(20)).toHaveClass(/bg-fill-strong/);
  await expect(segments.nth(0)).toHaveAttribute('style', /^width: 97\.0/);

  // 凡例も同じ並び・同じ色対応で、帯の割合と一致する
  const legend = composition.locator('ul > li');
  await expect(legend).toHaveCount(21);
  await expect(legend.nth(0)).toContainText('銘柄01');
  await expect(legend.nth(0)).toContainText('97.1%');
  await expect(legend.nth(19)).toContainText('銘柄20');
  await expect(legend.nth(20)).toContainText('その他 2銘柄');
  await expect(
    legend.nth(0).locator('span').first(),
  ).toHaveClass(/bg-holding-1\b/);
  await expect(
    legend.nth(19).locator('span').first(),
  ).toHaveClass(/bg-holding-20\b/);
  await expect(
    legend.nth(20).locator('span').first(),
  ).toHaveClass(/bg-fill-strong/);

  // 保有カードの左端の帯色は並び位置のトークンと一致する
  const firstCard = page.locator('div.border-l-holding-1').filter({
    has: page.getByTestId('portfolio-card-identity'),
  });
  await expect(firstCard).toHaveCount(1);
  await expect(firstCard).toContainText('銘柄01');
  const twentiethCard = page.locator('div.border-l-holding-20').filter({
    has: page.getByTestId('portfolio-card-identity'),
  });
  await expect(twentiethCard).toHaveCount(1);
  await expect(twentiethCard).toContainText('銘柄20');

  // カード内の構成比バーも同じ位置トークンを使う
  const firstBar = firstCard.getByTestId('portfolio-card-composition-bar');
  await expect(firstBar).toHaveClass(/bg-holding-1\b/);
  const twentiethBar = twentiethCard.getByTestId('portfolio-card-composition-bar');
  await expect(twentiethBar).toHaveClass(/bg-holding-20\b/);

  // 銘柄コードは flush バッジ(左余白なし)でカード左端に揃う
  const code = firstCard.getByTestId('portfolio-card-code');
  await expect(code).toHaveClass(/(^|\s)code-badge-flush(\s|$)/);
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
  await expect(card).toHaveClass(/border-l-holding-1\b/);

  const code = card.getByTestId('portfolio-holding-card-code');
  await expect(code).toBeVisible();
  await expect(code).toHaveClass(/(^|\s)code-badge-flush(\s|$)/);
  const firstChildTestId = await code
    .locator('xpath=..')
    .evaluate((el) => el.firstElementChild?.getAttribute('data-testid'));
  expect(firstChildTestId).toBe('portfolio-holding-card-code');

  const codeBox = await code.boundingBox();
  const nameBox = await card.getByText('銘柄01').boundingBox();
  expect(codeBox).not.toBeNull();
  expect(nameBox).not.toBeNull();
  expect(codeBox!.x).toBeLessThan(nameBox!.x);
});
