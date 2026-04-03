import { expect, test, type Page } from '@playwright/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000001',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const MOCK_STOCK = {
  date: '2024-03-01',
  code: '7974',
  name: '任天堂',
  market_category: 'プライム',
  industry_code_33: '37',
  industry_category_33: '情報・通信業',
  industry_code_17: '10',
  industry_category_17: '情報通信・サービスその他',
  size_code: '7',
  size_category: 'TOPIX Large70',
};

const ROUTES = {
  authMe: /\/auth\/me$/,
  // API エンドポイントのみ一致させる（Vite のソースファイルパスと衝突しないよう末尾を限定）
  stock: /\/stocks\/\d+$/,
};

async function setupAuthMocks(page: Page) {
  await page.route(ROUTES.authMe, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_USER),
    }),
  );
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

  await expect(page.getByRole('textbox', { name: '銘柄コードまたは銘柄名' })).toBeVisible();
});

test('検索フォームに文字を入力できる', async ({ page }) => {
  await setupAuthMocks(page);

  await page.goto('/search');
  await page.waitForLoadState('networkidle');

  const searchInput = page.getByRole('textbox', { name: '銘柄コードまたは銘柄名' });
  await searchInput.fill('任天堂');

  await expect(searchInput).toHaveValue('任天堂');
});
