import { expect, test, type Page } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000010',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const TOYOTA = {
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
};

const SONY = {
  id: 'asset-balance-2',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '6758',
  security_name: 'ソニーグループ',
  shares: 50,
  executing_shares: 0,
  average_purchase_price: 3000,
  total_purchase_amount: 150000,
  current_price: 3200,
  daily_change: -20,
  market_value: 160000,
  profit_loss_rate: 6.67,
};

const NTT = {
  id: 'asset-balance-3',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '9432',
  security_name: '日本電信電話',
  shares: 200,
  executing_shares: 0,
  average_purchase_price: 150,
  total_purchase_amount: 30000,
  current_price: 160,
  daily_change: 1,
  market_value: 32000,
  profit_loss_rate: 6.67,
};

const FACETS = {
  securities: [
    { value: '7203', label: 'トヨタ自動車', count: 1 },
    { value: '6758', label: 'ソニーグループ', count: 1 },
    { value: '9432', label: '日本電信電話', count: 1 },
  ],
};

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
      return route.fallback();
    }
    const data = [TOYOTA, SONY, NTT];
    return route.fulfill(
      jsonResponse({
        data,
        total: data.length,
        page: 1,
        per_page: 1000,
        facets: FACETS,
      }),
    );
  });

  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(jsonResponse({ items: [] })),
  );
}

async function gotoAssetBalance(page: Page) {
  await page.goto('/assetbalance');
  await expect(page.getByTestId('portfolio-pie-chart')).toBeVisible();
}

async function shoot(page: Page, name: string) {
  const dir = path.resolve(test.info().project.testDir, '../../.playwright-mcp');
  await fs.promises.mkdir(dir, { recursive: true });
  await page.screenshot({
    path: path.join(dir, `leptos-assetbalance-${name}.png`),
    fullPage: true,
  });
}

test.beforeEach(async ({ page }) => {
  await setupAssetBalanceMocks(page);
});

test('390px では検索レールが一覧より上に並びCSV操作は折り畳まれる', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await gotoAssetBalance(page);

  const rail = page.getByTestId('assetbalance-utility-rail');
  const main = page.getByTestId('assetbalance-main-stage');
  const railBox = await rail.boundingBox();
  const mainBox = await main.boundingBox();
  expect(railBox).not.toBeNull();
  expect(mainBox).not.toBeNull();
  expect(railBox!.y).toBeLessThan(mainBox!.y);

  await expect(page.getByTestId('portfolio-valuation-summary')).toBeVisible();
  await expect(page.getByTestId('portfolio-kpi-grid')).toBeHidden();
  await expect(page.getByTestId('portfolio-valuation-card').first()).toBeVisible();
  await expect(page.getByTestId('portfolio-card-identity').first()).toBeHidden();

  const searchCard = page.getByTestId('search-card');
  const railInner = rail.locator('> div').first();
  await expect(railInner).toHaveCSS('border-top-width', '1px');
  const railSections = railInner.locator('> *');
  expect(await railSections.count()).toBeGreaterThanOrEqual(3);
  await expect(railSections.first()).toHaveCSS('border-bottom-width', '1px');
  await expect(railSections.last()).toHaveCSS('border-bottom-width', '0px');
  const railRadius = await railInner.evaluate(
    (el) => getComputedStyle(el).borderTopLeftRadius,
  );
  expect(Number.parseFloat(railRadius)).toBeGreaterThan(0);
  await expect(railInner.locator('[data-testid="search-card"]')).toHaveCount(1);
  const searchStyle = await searchCard.evaluate((el) => {
    const s = getComputedStyle(el);
    return { borderTopWidth: s.borderTopWidth, borderRadius: s.borderTopLeftRadius };
  });
  expect(searchStyle.borderTopWidth).toBe('0px');
  expect(Number.parseFloat(searchStyle.borderRadius)).toBe(0);
  await expect(page.locator('#securities-search')).toBeVisible();

  const toggle = page.getByTestId('assetbalance-csv-toggle');
  const region = page.getByRole('region', { name: 'CSV取り込み・削除' });
  await expect(toggle).toBeVisible();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(toggle).toHaveAttribute('aria-controls', 'assetbalance-csv-body');
  const toggleBox = await toggle.boundingBox();
  expect(toggleBox).not.toBeNull();
  expect(toggleBox!.height).toBeGreaterThanOrEqual(44);
  await expect(region).toBeHidden();

  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await expect(region).toBeVisible();
  await expect(page.getByTestId('csv-file-input')).toBeAttached();

  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(region).toBeHidden();

  await shoot(page, '390');
});

test('639px ではモバイル表示、640px でPC表示に切り替わる', async ({ page }) => {
  await page.setViewportSize({ width: 639, height: 844 });
  await gotoAssetBalance(page);

  await expect(page.getByTestId('assetbalance-csv-toggle')).toBeVisible();
  await expect(
    page.getByRole('region', { name: 'CSV取り込み・削除' }),
  ).toBeHidden();
  await expect(page.getByTestId('portfolio-valuation-summary')).toBeVisible();
  await expect(page.getByTestId('portfolio-kpi-grid')).toBeHidden();
  await expect(page.getByTestId('portfolio-valuation-card').first()).toBeVisible();
  await expect(page.getByTestId('portfolio-card-identity').first()).toBeHidden();
  await shoot(page, '639');

  await page.setViewportSize({ width: 640, height: 844 });

  const railInner = page.getByTestId('assetbalance-utility-rail').locator('> div').first();
  await expect(railInner).toHaveCSS('border-top-width', '1px');
  await expect(page.getByTestId('search-card')).toHaveCSS('border-top-width', '0px');

  await expect(page.getByTestId('assetbalance-csv-toggle')).toBeHidden();
  await expect(
    page.getByRole('region', { name: 'CSV取り込み・削除' }),
  ).toBeVisible();
  await expect(page.getByTestId('portfolio-valuation-summary')).toBeHidden();
  await expect(page.getByTestId('portfolio-kpi-grid')).toBeVisible();
  await expect(page.getByTestId('portfolio-valuation-card').first()).toBeHidden();
  await expect(page.getByTestId('portfolio-card-identity').first()).toBeVisible();
  await shoot(page, '640');
});

test('1920px では一覧とユーティリティレールの2カラムになる', async ({ page }) => {
  await page.setViewportSize({ width: 1920, height: 1080 });
  await gotoAssetBalance(page);

  // DOM 順は rail 先(キーボード・読み上げ順)、見た目は order で main 先に戻す
  const domOrder = await page
    .getByTestId('assetbalance-workspace')
    .evaluate((el) =>
      Array.from(el.children).map(
        (child) => (child as HTMLElement).dataset.testid,
      ),
    );
  expect(domOrder).toEqual(['assetbalance-utility-rail', 'assetbalance-main-stage']);

  const rail = page.getByTestId('assetbalance-utility-rail');
  const main = page.getByTestId('assetbalance-main-stage');
  const railBox = await rail.boundingBox();
  const mainBox = await main.boundingBox();
  expect(railBox).not.toBeNull();
  expect(mainBox).not.toBeNull();
  expect(railBox!.x).toBeGreaterThan(mainBox!.x);
  expect(railBox!.width).toBeGreaterThanOrEqual(300);
  expect(railBox!.width).toBeLessThanOrEqual(340);

  await expect(rail.getByTestId('search-card')).toBeVisible();
  await expect(page.getByTestId('assetbalance-csv-toggle')).toBeHidden();
  await expect(
    page.getByRole('region', { name: 'CSV取り込み・削除' }),
  ).toBeVisible();

  await shoot(page, '1920');
});
