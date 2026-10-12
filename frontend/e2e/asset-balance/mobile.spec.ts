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

test('390px では検索入力とCSV管理へのリンクにそのまま届く', async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await gotoAssetBalance(page);

  // CSV 操作は専用ページに移ったため、一覧にはドロワーがない
  await expect(page.getByTestId('asset-utility-toggle')).toHaveCount(0);
  const main = page.getByTestId('assetbalance-main-stage');
  // 検索はメイン領域にある
  const searchInput = page.getByTestId('asset-search-input');
  await expect(searchInput).toBeVisible();
  await expect(searchInput).toBeEditable();

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
  await expect(searchInput).toBeVisible();

  // CSV 管理ページにはヘッダーのリンクから入り、AI プロンプトもそこにある
  await page.getByRole('link', { name: 'CSV管理' }).click();
  await expect(page).toHaveURL('/assetbalance/csv');
  await expect(page.getByTestId('csv-file-input')).toBeAttached();
  await expect(page.getByTestId('asset-review-prompt-card')).toBeVisible();
  await page.getByRole('link', { name: '一覧に戻る' }).click();
  await expect(page).toHaveURL('/assetbalance');
  await expect(main).toBeVisible();
});

test('保有カードの詳細は見出しを繰り返さず、銘柄情報への文全体がリンクになる', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await gotoAssetBalance(page);
  const card = page.getByTestId('portfolio-holding-card').first();
  // 閉じた面には評価額と含み損益が出る
  await expect(card.getByText('評価額', { exact: true })).toHaveCount(1);
  await expect(card.getByText('+¥10,000（+4.0%）')).toHaveCount(1);
  await card.getByRole('button').click();
  await expect(card.locator('dt').filter({ hasText: /^銘柄名$/ })).toHaveCount(0);
  await expect(card.locator('dt').filter({ hasText: /^評価額$/ })).toHaveCount(0);
  await expect(card.locator('dt').filter({ hasText: /^取得総額$/ })).toHaveCount(1);
  await expect(card.getByText('トヨタ自動車', { exact: true })).toHaveCount(1);
  await expect(card.getByText('¥250,000', { exact: true })).toHaveCount(1);
  const link = card.getByRole('link', { name: /7203.*の銘柄情報を見る/ });
  await expect(link).toBeVisible();
  await expect(link).toHaveAttribute('href', '/search?code=7203');
});

test('一覧に無い文字列を入れるとフィルタ済み空状態と解除ボタンが出る', async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await gotoAssetBalance(page);

  await page.getByTestId('asset-search-input').fill('9999');
  const heading = page.getByRole('heading', { name: '該当する銘柄がありません' });
  await expect(heading).toBeVisible();
  await expect(
    page.getByRole('button', { name: '絞り込みを解除' }),
  ).toBeVisible();
});

test('639px ではモバイル表示、640px でPC表示に切り替わる', async ({ page }) => {
  await page.setViewportSize({ width: 639, height: 844 });
  await gotoAssetBalance(page);

  await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
  await expect(page.getByTestId('portfolio-kpi-grid')).toBeVisible();
  await expect(page.getByTestId('portfolio-holding-card').first()).toBeVisible();
  await expect(page.getByTestId('portfolio-card-identity').first()).toBeHidden();

  await page.setViewportSize({ width: 640, height: 844 });

  await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
  await expect(page.getByTestId('portfolio-kpi-grid')).toBeVisible();
  await expect(page.getByTestId('portfolio-holding-card').first()).toBeHidden();
  await expect(page.getByTestId('portfolio-card-identity').first()).toBeVisible();
});

test('1920px でも一覧は1カラムでページをはみ出さない', async ({ page }) => {
  await page.setViewportSize({ width: 1920, height: 1080 });
  await gotoAssetBalance(page);

  // CSV 操作と AI プロンプトは専用ページに移ったためユーティリティレールは無い
  await expect(page.getByTestId('assetbalance-utility-rail')).toHaveCount(0);
  const main = page.getByTestId('assetbalance-main-stage');
  await expect(main).toBeVisible();
  await expect(page.getByTestId('asset-search-input')).toBeVisible();
  const width = await page.evaluate(() => document.documentElement.scrollWidth);
  expect(width).toBeLessThanOrEqual(1920);
});
