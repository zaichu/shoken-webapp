import type { Page } from '@playwright/test';
import { expect, test, json } from '../support/test';

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
};

const FACETS = {
  securities: [
    { value: '7203', label: 'トヨタ自動車', count: 1 },
    { value: '6758', label: 'ソニーグループ', count: 1 },
    { value: '9432', label: '日本電信電話', count: 1 },
  ],
};


async function setupAssetBalanceMocks(page: Page) {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill(json(MOCK_USER)),
  );

  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => {
    if (route.request().method() !== 'GET') {
      return route.abort();
    }
    const data = [TOYOTA, SONY, NTT];
    return route.fulfill(
      json({
        data,
        total: data.length,
        page: 1,
        per_page: 1000,
        facets: FACETS,
      }),
    );
  });

  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

async function gotoAssetBalance(page: Page) {
  await page.goto('/assetbalance');
  await expect(page.getByTestId('portfolio-pie-chart')).toBeVisible();
}

test.beforeEach(async ({ page }) => {
  await setupAssetBalanceMocks(page);
});

test('390px ではパネルが畳んで始まり、ドロワーでCSV・検索を操作できる', async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await gotoAssetBalance(page);

  // 狭い帯ではパネルが畳んで始まり、横バーのハンドルが一覧より上に出る
  const toggle = page.getByTestId('asset-utility-toggle');
  await expect(toggle).toBeVisible();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  const rail = page.getByTestId('assetbalance-utility-rail');
  const main = page.getByTestId('assetbalance-main-stage');
  const railBox = await rail.boundingBox();
  const mainBox = await main.boundingBox();
  expect(railBox).not.toBeNull();
  expect(mainBox).not.toBeNull();
  expect(railBox!.y).toBeLessThan(mainBox!.y);
  await expect(page.locator('#securities-search')).toBeHidden();

  // ハンドルで開くとドロワーが被さり、CSV → 検索 → プロンプトの順に並ぶ
  const mainBoxBefore = mainBox!;
  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');

  const railInner = rail.locator('.ws-panel-body > div').first();
  const railSections = railInner.locator('> *');
  expect(await railSections.count()).toBeGreaterThanOrEqual(3);
  await expect(
    railSections.first().getByTestId('assetbalance-csv-toggle'),
  ).toBeVisible();
  const railOrder = await railInner.evaluate((el) =>
    Array.from(el.children)
      .map((child) => (child as HTMLElement).dataset?.testid ?? '')
      .filter((id) => id.length > 0),
  );
  expect(railOrder).toEqual(['search-card', 'asset-review-prompt-card']);

  // ドロワーは被さるだけで一覧を押し下げない(開くとバー分だけ上に詰まる)
  const mainBoxAfter = (await main.boundingBox())!;
  expect(mainBoxAfter.y).toBeLessThanOrEqual(mainBoxBefore.y + 1);
  expect(mainBoxAfter.x).toBeCloseTo(mainBoxBefore.x, 0);

  // main 内は集計情報 → 保有内訳
  const summary = page.getByTestId('asset-portfolio-summary');
  const firstCard = page.getByTestId('portfolio-holding-card').first();
  const summaryBox = await summary.boundingBox();
  const cardBox = await firstCard.boundingBox();
  expect(summaryBox).not.toBeNull();
  expect(cardBox).not.toBeNull();
  expect(summaryBox!.y).toBeLessThan(cardBox!.y);

  await expect(summary).toBeVisible();
  await expect(page.getByTestId('portfolio-kpi-grid')).toBeVisible();
  await expect(firstCard).toBeVisible();
  await expect(page.getByTestId('portfolio-card-identity').first()).toBeHidden();
  await expect(page.locator('#securities-search')).toBeVisible();

  const csvToggle = page.getByTestId('assetbalance-csv-toggle');
  const region = page.getByRole('region', { name: 'CSV取り込み・削除' });
  await expect(csvToggle).toBeVisible();
  await expect(csvToggle).toHaveAttribute('aria-expanded', 'false');
  await expect(csvToggle).toHaveAttribute('aria-controls', 'assetbalance-csv-body');
  const toggleBox = await csvToggle.boundingBox();
  expect(toggleBox).not.toBeNull();
  expect(toggleBox!.height).toBeGreaterThanOrEqual(44);
  await expect(region).toBeHidden();

  await csvToggle.click();
  await expect(csvToggle).toHaveAttribute('aria-expanded', 'true');
  await expect(region).toBeVisible();
  await expect(page.getByTestId('csv-file-input')).toBeAttached();

  await csvToggle.click();
  await expect(csvToggle).toHaveAttribute('aria-expanded', 'false');
  await expect(region).toBeHidden();

  // Esc でドロワーを閉じるとバーが戻る
  await page.keyboard.press('Escape');
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(page.locator('#securities-search')).toBeHidden();
  const mainBoxClosed = (await main.boundingBox())!;
  expect(mainBoxClosed.y).toBeCloseTo(mainBoxBefore.y, 0);
});

test('保有カードの詳細は見出しを繰り返さず、銘柄情報への文全体がリンクになる', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await gotoAssetBalance(page);
  const card = page.getByTestId('portfolio-holding-card').first();
  await card.getByRole('button').click();
  await expect(card.locator('dt').filter({ hasText: /^銘柄名$/ })).toHaveCount(0);
  await expect(card.locator('dt').filter({ hasText: /^取得総額$/ })).toHaveCount(0);
  await expect(card.getByText('トヨタ自動車', { exact: true })).toHaveCount(1);
  await expect(card.getByText('¥250,000', { exact: true })).toHaveCount(1);
  const link = card.getByRole('link', { name: /7203.*の銘柄情報を見る/ });
  await expect(link).toBeVisible();
  await expect(link).toHaveAttribute('href', '/search?code=7203');
  await expect(card.getByText('評価額', { exact: true })).toHaveCount(0);
  await expect(card.getByText('評価損益', { exact: true })).toHaveCount(0);
});

test('一覧に無い銘柄を選ぶとフィルタ済み空状態と解除ボタンが出る', async ({
  page,
}) => {
  await page.unroute(/\/api\/v1\/asset-balances(?:\?.*)?$/);
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => {
    if (route.request().method() !== 'GET') {
      return route.abort();
    }
    return route.fulfill(
      json({
        data: [TOYOTA, SONY, NTT],
        total: 3,
        page: 1,
        per_page: 1000,
        facets: {
          securities: [...FACETS.securities, { value: '9999', label: '架空銘柄', count: 1 }],
        },
      }),
    );
  });
  await page.setViewportSize({ width: 390, height: 844 });
  await gotoAssetBalance(page);

  // 狭い帯ではパネルが畳んで始まるので、先にドロワーを開ける
  const emptyPanelToggle = page.getByTestId('asset-utility-toggle');
  await emptyPanelToggle.click();
  await expect(emptyPanelToggle).toHaveAttribute('aria-expanded', 'true');

  await page.locator('#securities-search').selectOption('9999');
  const heading = page.getByRole('heading', { name: '該当する銘柄がありません' });
  await expect(heading).toBeVisible();
  await expect(
    page.getByRole('button', { name: '絞り込みを解除' }),
  ).toBeVisible();
});

test('639px ではモバイル表示、640px でPC表示に切り替わる', async ({ page }) => {
  await page.setViewportSize({ width: 639, height: 844 });
  await gotoAssetBalance(page);

  // 狭い帯ではパネルが畳んで始まるので、先にドロワーを開ける
  const panelToggle = page.getByTestId('asset-utility-toggle');
  await panelToggle.click();
  await expect(panelToggle).toHaveAttribute('aria-expanded', 'true');

  await expect(page.getByTestId('assetbalance-csv-toggle')).toBeVisible();
  await expect(
    page.getByRole('region', { name: 'CSV取り込み・削除' }),
  ).toBeHidden();
  await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
  await expect(page.getByTestId('portfolio-kpi-grid')).toBeVisible();
  await expect(page.getByTestId('portfolio-holding-card').first()).toBeVisible();
  await expect(page.getByTestId('portfolio-card-identity').first()).toBeHidden();

  await page.setViewportSize({ width: 640, height: 844 });

  await expect(page.getByTestId('assetbalance-csv-toggle')).toBeHidden();
  await expect(
    page.getByRole('region', { name: 'CSV取り込み・削除' }),
  ).toBeVisible();
  await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
  await expect(page.getByTestId('portfolio-kpi-grid')).toBeVisible();
  await expect(page.getByTestId('portfolio-holding-card').first()).toBeHidden();
  await expect(page.getByTestId('portfolio-card-identity').first()).toBeVisible();
});

test('1920px では左パネルと一覧の2カラムになる', async ({ page }) => {
  await page.setViewportSize({ width: 1920, height: 1080 });
  await gotoAssetBalance(page);

  // DOM 順は rail 先(キーボード・読み上げ順)。見た目も左パネルが先
  const domOrder = await page
    .getByTestId('assetbalance-workspace')
    .evaluate((el) =>
      Array.from(el.children)
        .map((child) => (child as HTMLElement).dataset?.testid ?? '')
        .filter((id) => id.length > 0),
    );
  expect(domOrder).toEqual(['assetbalance-utility-rail', 'assetbalance-main-stage']);

  const rail = page.getByTestId('assetbalance-utility-rail');
  const main = page.getByTestId('assetbalance-main-stage');
  const railBox = await rail.boundingBox();
  const mainBox = await main.boundingBox();
  expect(railBox).not.toBeNull();
  expect(mainBox).not.toBeNull();
  // 左パネルはビューポート左端に密着し、一覧より左にある
  expect(railBox!.x).toBeCloseTo(0, 0);
  expect(railBox!.x + railBox!.width).toBeLessThanOrEqual(mainBox!.x + 1);

  await expect(rail.getByTestId('search-card')).toBeVisible();
  await expect(page.getByTestId('assetbalance-csv-toggle')).toBeHidden();
  await expect(
    page.getByRole('region', { name: 'CSV取り込み・削除' }),
  ).toBeVisible();
});
