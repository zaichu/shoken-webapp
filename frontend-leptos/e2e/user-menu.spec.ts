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

test('アカウント削除は確認ダイアログ経由で実行され、成功すると未認証表示になる', async ({
  page,
}) => {
  await mockAuthorizedAuth(page);
  const calls: string[] = [];
  await page.route(/\/api\/v1\/account-deletion-confirmations$/, async (route) => {
    calls.push('confirm');
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  await page.route(/\/api\/v1\/account$/, async (route) => {
    calls.push('delete');
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  await page.goto('/');

  await page.getByRole('button', { name: 'メニュー' }).click();
  await page.getByRole('menuitem', { name: 'アカウント削除' }).click();

  const dialog = page.getByRole('dialog', { name: 'アカウント削除の確認' });
  await expect(dialog).toBeVisible();

  // キャンセルはAPIを呼ばずに閉じる
  await dialog.getByRole('button', { name: 'キャンセル' }).click();
  await expect(dialog).toHaveCount(0);
  expect(calls).toHaveLength(0);

  // 確定すると確認API→削除APIの順に呼ばれ、ヘッダーが未認証表示に戻る
  await page.getByRole('button', { name: 'メニュー' }).click();
  await page.getByRole('menuitem', { name: 'アカウント削除' }).click();
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect.poll(() => calls.join(',')).toBe('confirm,delete');
  await expect(dialog).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
});

test('アカウント削除の確認APIが失敗したとき削除APIは呼ばれずダイアログは開いたままになる', async ({
  page,
}) => {
  await mockAuthorizedAuth(page);
  const calls: string[] = [];
  await page.route(/\/api\/v1\/account-deletion-confirmations$/, async (route) => {
    calls.push('confirm');
    await route.fulfill({
      status: 500,
      contentType: 'application/json',
      body: '{"error":"server error"}',
    });
  });
  await page.route(/\/api\/v1\/account$/, async (route) => {
    calls.push('delete');
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  await page.goto('/');

  await page.getByRole('button', { name: 'メニュー' }).click();
  await page.getByRole('menuitem', { name: 'アカウント削除' }).click();

  const dialog = page.getByRole('dialog', { name: 'アカウント削除の確認' });
  await expect(dialog).toBeVisible();
  await dialog.getByRole('button', { name: '削除する' }).click();

  // 確認APIはリトライなしの1回のみで、削除APIには到達しない
  await expect.poll(() => calls.length).toBe(1);
  await page.waitForTimeout(500);
  expect(calls).toEqual(['confirm']);
  await expect(dialog).toBeVisible();
});
