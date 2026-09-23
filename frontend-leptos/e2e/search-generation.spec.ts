import { expect, test } from '@playwright/test';

const USER_A = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const STOCK_X = {
  date: '2024-03-01',
  code: '7974',
  name: '任天堂X',
  market_category: 'プライム',
};

test('検索の応答が届く前にログアウトすると古い応答は表示されない', async ({ page }) => {
  let deleteCount = 0;
  await page.route(/\/api\/v1\/session$/, (route) => {
    if (route.request().method() === 'DELETE') {
      deleteCount += 1;
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ message: 'ログアウトしました' }),
      });
    }
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(USER_A),
    });
  });
  await page.route(/\/api\/v1\/stocks(?:\?.*)?$/, async (route) => {
    await new Promise((resolve) => setTimeout(resolve, 800));
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(STOCK_X),
    });
  });

  await page.goto('/search');
  await page.getByRole('textbox', { name: '銘柄コードまたは銘柄名' }).fill('7974');
  await page.getByRole('button', { name: '銘柄を検索' }).click();

  await page.getByTestId('logout').click();
  await expect(page.getByRole('link', { name: 'ログイン' })).toBeVisible();
  expect(deleteCount).toBe(1);
  await expect(page).toHaveURL(/\/search$/);

  await page.waitForTimeout(1500);
  await expect(page).toHaveURL(/\/search$/);
  await expect(page.getByRole('heading', { name: '任天堂X' })).toBeHidden();
  await expect(page.getByRole('heading', { name: '銘柄を検索' })).toBeVisible();
});
