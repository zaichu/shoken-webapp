import { expect, test, type Page } from '@playwright/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

async function mockAuthorizedAuth(page: Page) {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_USER),
    }),
  );
}

test('メニューボタンでユーザーメニューが開閉し、aria-expanded が連動する', async ({ page }) => {
  await mockAuthorizedAuth(page);
  await page.goto('/');

  const trigger = page.getByRole('button', { name: 'メニュー' });
  const menu = page.getByRole('menu', { name: 'ユーザーメニュー' });

  await expect(trigger).toHaveAttribute('aria-expanded', 'false');
  await expect(menu).toHaveCount(0);

  await trigger.click();
  await expect(trigger).toHaveAttribute('aria-expanded', 'true');
  await expect(menu).toBeVisible();
  await expect(menu.getByRole('menuitem', { name: 'ログアウト' })).toBeVisible();

  await trigger.click();
  await expect(trigger).toHaveAttribute('aria-expanded', 'false');
  await expect(menu).toHaveCount(0);
});

test('ユーザーメニューが外側クリックと Escape で閉じる', async ({ page }) => {
  await mockAuthorizedAuth(page);
  await page.goto('/');

  const trigger = page.getByRole('button', { name: 'メニュー' });
  const menu = page.getByRole('menu', { name: 'ユーザーメニュー' });

  await trigger.click();
  await expect(menu).toBeVisible();
  await page.getByText('© 2026 shoken-webapp').click();
  await expect(menu).toHaveCount(0);

  await trigger.click();
  await expect(menu).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(menu).toHaveCount(0);
});
