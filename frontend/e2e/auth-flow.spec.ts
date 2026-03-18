import { expect, test, type Page } from '@playwright/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000001',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const ROUTES = {
  authMe: /\/auth\/me$/,
};

async function mockUnauthorizedAuth(page: Page) {
  await page.route(ROUTES.authMe, (route) =>
    route.fulfill({ status: 401, contentType: 'application/json', body: '{}' }),
  );
}

async function mockAuthorizedAuth(page: Page) {
  await page.route(ROUTES.authMe, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_USER),
    }),
  );
}

test('未認証時に /receipts へアクセスするとログインページにリダイレクトされる', async ({ page }) => {
  await mockUnauthorizedAuth(page);

  await page.goto('/receipts');

  await page.waitForURL('**/login');
  await expect(page).toHaveURL(/\/login$/);
});

test('未認証時に /assetbalance へアクセスするとログインページにリダイレクトされる', async ({ page }) => {
  await mockUnauthorizedAuth(page);

  await page.goto('/assetbalance');

  await page.waitForURL('**/login');
  await expect(page).toHaveURL(/\/login$/);
});

test('認証済みユーザーのホームページにユーザー名が表示される', async ({ page }) => {
  await mockAuthorizedAuth(page);

  await page.goto('/');

  await expect(page.getByText('テストユーザー')).toBeVisible();
});
