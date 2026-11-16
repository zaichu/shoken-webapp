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
  return page.getByRole('navigation', { name: 'メインナビゲーション' });
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

test('下部ナビは設定・保有ファンド詳細でも出て、404では出ない', async ({
  page,
}) => {
  await mockApi(page);
  for (const path of ['/settings', '/assetbalance/funds/fund-1', '/assetbalance/composition']) {
    await page.goto(path);
    await expect(bottomNav(page)).toBeVisible();
  }
  await page.goto('/404');
  await expect(bottomNav(page)).toHaveCount(0);
});

test('下部ナビはログインでは出ない', async ({ page }) => {
  await page.route('**/api/v1/**', (route) =>
    route.fulfill(json(paginated([], { per_page: 1000 }))),
  );
  await page.route(/\/api\/v1\/session$/, (route) => route.fulfill(json({}, 401)));
  await page.goto('/login');
  await expect(bottomNav(page)).toHaveCount(0);
});

test('上バーのアカウントボタンは丸いアイコンと「アカウント」の文字を持つ', async ({
  page,
}) => {
  await mockApi(page);
  await page.goto('/');
  const trigger = page.getByRole('button', { name: /アカウントメニュー/ });
  await expect(trigger).toBeVisible();
  await expect(trigger.getByText('アカウント', { exact: true })).toBeVisible();
  await trigger.click();
  await expect(page.getByRole('menu', { name: 'ユーザーメニュー' })).toBeVisible();
});

test('スマホではCSV管理は見出し帯に出ず資産管理の本文とアカウントメニューにある', async ({
  page,
}) => {
  await mockApi(page);
  // 見出し帯(ページ名の横)にCSV管理ボタンは全画面で出ない
  for (const path of ['/', '/assetbalance', '/receipts']) {
    await page.goto(path);
    const header = page.locator('.page-frame-head');
    await expect(
      header.getByRole('link', { name: 'CSV管理', exact: true }),
    ).toHaveCount(0);
  }
  // 資産管理の本文にCSV管理›の文字リンクがある
  await page.goto('/assetbalance');
  await expect(page.getByTestId('asset-csv-link')).toBeVisible();
  await expect(page.getByTestId('asset-csv-link')).toHaveAttribute('href', '/csv');
  // ホーム・取引明細の本文にはない
  for (const path of ['/', '/receipts', '/search', '/settings']) {
    await page.goto(path);
    await expect(page.getByTestId('asset-csv-link')).toHaveCount(0);
  }
});

test('CSV管理へは資産管理の本文リンクとアカウントメニューから2操作で行ける', async ({
  page,
}) => {
  await mockApi(page);
  await page.goto('/');
  await bottomNav(page).getByRole('link', { name: '資産管理' }).click();
  await expect(page).toHaveURL('/assetbalance');
  await page.getByTestId('asset-csv-link').click();
  await expect(page).toHaveURL('/csv');

  await page.goto('/');
  await page.getByRole('button', { name: /アカウントメニュー/ }).click();
  await page.getByRole('menuitem', { name: 'CSV管理' }).click();
  await expect(page).toHaveURL('/csv');
});
