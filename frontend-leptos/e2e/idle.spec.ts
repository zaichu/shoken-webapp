import { expect, test } from '@playwright/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000003',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const MINUTE = 60 * 1000;

test('30分無操作でログアウトし、操作で延長され、ログアウト後はタイマーが止まる', async ({
  page,
}) => {
  await page.clock.install();

  let sessionDeletes = 0;
  await page.route(/\/api\/v1\/session$/, async (route) => {
    if (route.request().method() === 'DELETE') {
      sessionDeletes += 1;
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: '{"message":"ok"}',
      });
    } else {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(MOCK_USER),
      });
    }
  });
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'メニュー' })).toBeVisible();

  await page.clock.fastForward(29 * MINUTE);
  await page.mouse.move(200, 200);
  await page.clock.fastForward(29 * MINUTE);
  expect(sessionDeletes).toBe(0);
  await expect(page.getByRole('button', { name: 'メニュー' })).toBeVisible();

  await page.clock.fastForward(2 * MINUTE);
  await expect.poll(() => sessionDeletes).toBe(1);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();

  await page.mouse.move(300, 300);
  await page.clock.fastForward(30 * MINUTE);
  expect(sessionDeletes).toBe(1);
});
