import type { Page } from '@playwright/test';
import { expect, test, mockSession } from '../support/test';

const ROUTES = {
  authMe: /\/api\/v1\/session$/,
};

async function mockUnauthorizedAuth(page: Page) {
  await page.route(ROUTES.authMe, (route) =>
    route.fulfill({ status: 401, contentType: 'application/json', body: '{}' }),
  );
}

async function mockAuthorizedAuth(page: Page) {
  await mockSession(page);
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
