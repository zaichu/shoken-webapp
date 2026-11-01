import { expect, test, json, paginated } from '../support/test';
import { MOCK_USER } from '../support/test';
import type { Page } from '@playwright/test';

test.use({ viewport: { width: 390, height: 844 } });

async function mockApi(page: Page) {
  await page.route('**/api/v1/**', (route) =>
    route.fulfill(json(paginated([], { per_page: 1000 }))),
  );
  await page.route('**/api/v1/session', (route) => route.fulfill(json(MOCK_USER)));
}

function bottomNav(page: Page) {
  return page.getByRole('navigation', { name: '主要ナビゲーション' });
}

test('下タブバーは4つのリンクを持ちアイコンは装飾として隠す', async ({ page }) => {
  await mockApi(page);
  await page.goto('/');
  const nav = bottomNav(page);
  await expect(nav).toBeVisible();
  const links = nav.getByRole('link');
  await expect(links).toHaveCount(4);
  await expect(links.nth(0)).toHaveAttribute('href', '/');
  await expect(links.nth(1)).toHaveAttribute('href', '/assetbalance');
  await expect(links.nth(2)).toHaveAttribute('href', '/receipts');
  await expect(links.nth(3)).toHaveAttribute('href', '/search');
  await expect(nav.locator('svg[aria-hidden="true"]')).toHaveCount(4);
});

test('下タブバーは現在地にaria-currentを付けタップで遷移できる', async ({ page }) => {
  await mockApi(page);
  await page.goto('/');
  const nav = bottomNav(page);
  await expect(nav.getByRole('link', { name: 'ホーム' })).toHaveAttribute(
    'aria-current',
    'page',
  );
  await nav.getByRole('link', { name: '資産管理' }).click();
  await expect(page).toHaveURL('/assetbalance');
  await expect(
    bottomNav(page).getByRole('link', { name: '資産管理' }),
  ).toHaveAttribute('aria-current', 'page');
});

test('CSV管理はホーム・資産管理・取引明細に出て銘柄検索・設定には出ない', async ({
  page,
}) => {
  await mockApi(page);
  for (const path of ['/', '/assetbalance', '/receipts']) {
    await page.goto(path);
    await expect(
      page.getByRole('link', { name: 'CSV管理', exact: true }).first(),
    ).toBeVisible();
  }
  for (const path of ['/search', '/settings']) {
    await page.goto(path);
    await expect(
      page.getByRole('link', { name: 'CSV管理', exact: true }),
    ).toHaveCount(0);
  }
});
