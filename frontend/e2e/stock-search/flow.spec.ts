import type { Page } from '@playwright/test';
import { expect, test, mockSession } from '../support/test';

const MOCK_STOCK = {
  date: '2024-03-01',
  code: '9905',
  name: '架空ゲーム',
  market_category: 'プライム',
  industry_category_33: '情報・通信業',
  industry_category_17: '情報通信・サービスその他',
  size_category: 'TOPIX Large70',
};

const ROUTES = {
  // query string を含む URL でも一致するよう path 部分のみでマッチ
  stock: /\/api\/v1\/stocks(?:\?.*)?$/,
};

async function setupAuthMocks(page: Page) {
  await mockSession(page);
  await page.route(ROUTES.stock, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_STOCK),
    }),
  );
}

test('検索ページの初期表示で検索フォームが表示される', async ({ page }) => {
  await setupAuthMocks(page);

  await page.goto('/search');
  await page.waitForLoadState('networkidle');

  await expect(page.getByRole('searchbox', { name: '銘柄コードまたは銘柄名で検索' })).toBeVisible();
});

test('PC では検索フォームが本文の上部に1つだけ表示される', async ({ page }) => {
  await setupAuthMocks(page);
  await page.setViewportSize({ width: 1440, height: 900 });

  await page.goto('/search');
  await page.waitForLoadState('networkidle');

  // PC は下部操作帯を出さないため、見える検索領域は本文直上の1つだけ
  const search = page.getByRole('search', { name: '銘柄検索' });
  await expect(search).toHaveCount(1);
  await expect(
    search.getByRole('searchbox', { name: '銘柄コードまたは銘柄名で検索' }),
  ).toBeVisible();
});

test('モバイルでも検索フォームが見える', async ({ page }) => {
  await setupAuthMocks(page);
  await page.setViewportSize({ width: 390, height: 844 });

  await page.goto('/search');
  await page.waitForLoadState('networkidle');

  const search = page.getByRole('search', { name: '銘柄検索' });
  await expect(search).toHaveCount(1);
  await expect(
    search.getByRole('searchbox', { name: '銘柄コードまたは銘柄名で検索' }),
  ).toBeVisible();
});

test('検索フォームに文字を入力できる', async ({ page }) => {
  await setupAuthMocks(page);

  await page.goto('/search');
  await page.waitForLoadState('networkidle');

  const searchInput = page.getByRole('searchbox', { name: '銘柄コードまたは銘柄名で検索' });
  await searchInput.fill('架空ゲーム');

  await expect(searchInput).toHaveValue('架空ゲーム');
});

test('検索の取得が失敗したとき再読み込みで再取得できる', async ({ page }) => {
  await mockSession(page);
  // 500 はクライアント側で1回リトライされるので、初回 dispatch は2リクエスト分を失敗させる
  let stockCalls = 0;
  await page.route(ROUTES.stock, (route) => {
    stockCalls += 1;
    if (stockCalls <= 2) {
      return route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'server error' }),
      });
    }
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_STOCK),
    });
  });

  await page.goto('/search?code=9905');

  const error = page.getByTestId('list-load-error');
  await expect(error).toBeVisible();
  await expect(error.getByRole('alert')).toContainText('エラー:');

  await page.getByRole('button', { name: '再読み込み' }).click();
  await expect(page.getByText('架空ゲーム')).toBeVisible();
  expect(stockCalls).toBe(3);
});

test('検索ボタンの読み込み属性がリクエストの開始と完了に連動する', async ({ page }) => {
  await setupAuthMocks(page);
  let finish!: () => void;
  const pending = new Promise<void>((resolve) => { finish = resolve; });
  await page.route(ROUTES.stock, async (route) => {
    await pending;
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_STOCK),
    });
  });

  await page.goto('/search?code=9905');
  const loading = page.getByRole('button', { name: '検索中' });
  await expect(loading).toBeDisabled();
  await expect(loading).toHaveAttribute('data-loading', 'true');

  finish();
  const ready = page.getByRole('button', { name: '銘柄を検索' });
  await expect(ready).toBeEnabled();
  await expect(ready).not.toHaveAttribute('data-loading', 'true');
});
