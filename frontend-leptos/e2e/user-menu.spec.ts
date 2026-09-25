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

  await dialog.getByRole('button', { name: 'キャンセル' }).click();
  await expect(dialog).toHaveCount(0);
  expect(calls).toHaveLength(0);

  await page.getByRole('button', { name: 'メニュー' }).click();
  await page.getByRole('menuitem', { name: 'アカウント削除' }).click();
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect.poll(() => calls.join(',')).toBe('confirm,delete');
  await expect(dialog).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
});

test('アカウント削除の確認APIが失敗したときダイアログにエラーが出て再試行で削除を完了できる', async ({
  page,
}) => {
  await mockAuthorizedAuth(page);
  const calls: string[] = [];
  let confirmFails = true;
  await page.route(/\/api\/v1\/account-deletion-confirmations$/, async (route) => {
    calls.push('confirm');
    await route.fulfill(
      confirmFails
        ? {
            status: 500,
            contentType: 'application/json',
            body: '{"error":"server error"}',
          }
        : {
            status: 200,
            contentType: 'application/json',
            body: '{"message":"ok"}',
          },
    );
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

  await expect.poll(() => calls.length).toBe(1);
  await page.waitForTimeout(1500);
  expect(calls).toEqual(['confirm']);
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole('alert')).toContainText(
    'サーバーエラーが発生しました。しばらくしてから再度お試しください',
  );
  await expect(page.getByRole('button', { name: 'メニュー' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'ログイン' })).toHaveCount(0);

  confirmFails = false;
  await dialog.getByRole('button', { name: '削除する' }).click();
  await expect.poll(() => calls.join(',')).toBe('confirm,confirm,delete');
  await expect(dialog).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
});

test('削除APIが500を返したときはダイアログにエラーが出て再試行で削除を完了できる', async ({
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
  let deleteFails = true;
  await page.route(/\/api\/v1\/account$/, async (route) => {
    calls.push('delete');
    await route.fulfill(
      deleteFails
        ? {
            status: 500,
            contentType: 'application/json',
            body: '{"error":{"code":"INTERNAL","message":"db error"}}',
          }
        : {
            status: 200,
            contentType: 'application/json',
            body: '{"message":"ok"}',
          },
    );
  });
  await page.goto('/');

  await page.getByRole('button', { name: 'メニュー' }).click();
  await page.getByRole('menuitem', { name: 'アカウント削除' }).click();

  const dialog = page.getByRole('dialog', { name: 'アカウント削除の確認' });
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect.poll(() => calls.join(',')).toBe('confirm,delete');
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole('alert')).toContainText(
    'サーバーエラーが発生しました。しばらくしてから再度お試しください',
  );
  await expect(page.getByRole('button', { name: 'メニュー' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'ログイン' })).toHaveCount(0);

  deleteFails = false;
  await dialog.getByRole('button', { name: '削除する' }).click();
  await expect.poll(() => calls.join(',')).toBe(
    'confirm,delete,confirm,delete',
  );
  await expect(dialog).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
});

test('削除処理中はキャンセル・Escape・背景クリックでダイアログを閉じられない', async ({
  page,
}) => {
  await mockAuthorizedAuth(page);
  await page.route(/\/api\/v1\/account-deletion-confirmations$/, async (route) => {
    await new Promise((resolve) => setTimeout(resolve, 1000));
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  await page.route(/\/api\/v1\/account$/, async (route) => {
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
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect(dialog.getByRole('button', { name: 'キャンセル' })).toBeDisabled();
  await expect(dialog.getByRole('button', { name: '閉じる' })).toBeDisabled();
  await page.keyboard.press('Escape');
  await page.mouse.click(10, 10);
  await expect(dialog).toBeVisible();

  await expect(dialog).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
});

test('削除APIの応答が失われてもセッション無効化を確認してダイアログを閉じる', async ({
  page,
}) => {
  let sessionDeleted = false;
  await page.route(/\/api\/v1\/session$/, async (route) => {
    await route.fulfill({
      status: sessionDeleted ? 401 : 200,
      contentType: 'application/json',
      body: sessionDeleted ? '' : JSON.stringify(MOCK_USER),
    });
  });
  await page.route(/\/api\/v1\/account-deletion-confirmations$/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  let deleteAttempts = 0;
  await page.route(/\/api\/v1\/account$/, async (route) => {
    deleteAttempts += 1;
    sessionDeleted = true;
    await route.abort();
  });
  await page.goto('/');

  await page.getByRole('button', { name: 'メニュー' }).click();
  await page.getByRole('menuitem', { name: 'アカウント削除' }).click();

  const dialog = page.getByRole('dialog', { name: 'アカウント削除の確認' });
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect(dialog).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  expect(deleteAttempts).toBe(1);
});

test('削除要求がサーバーに届かなかった一時的な失敗では再送して削除を完了する', async ({
  page,
}) => {
  await mockAuthorizedAuth(page);
  await page.route(/\/api\/v1\/account-deletion-confirmations$/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  let deleteAttempts = 0;
  await page.route(/\/api\/v1\/account$/, async (route) => {
    deleteAttempts += 1;
    if (deleteAttempts === 1) {
      await route.abort();
      return;
    }
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
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect(dialog).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  expect(deleteAttempts).toBe(2);
});

test('セッション失効で確認APIが401を返したとき削除APIは呼ばれずダイアログは開いたままになる', async ({
  page,
}) => {
  let sessionAlive = true;
  await page.route(/\/api\/v1\/session$/, async (route) => {
    await route.fulfill({
      status: sessionAlive ? 200 : 401,
      contentType: 'application/json',
      body: sessionAlive ? JSON.stringify(MOCK_USER) : '',
    });
  });
  let deleteAttempts = 0;
  await page.route(/\/api\/v1\/account-deletion-confirmations$/, async (route) => {
    sessionAlive = false;
    await route.fulfill({
      status: 401,
      contentType: 'application/json',
      body: '',
    });
  });
  await page.route(/\/api\/v1\/account$/, async (route) => {
    deleteAttempts += 1;
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
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole('alert')).toContainText('ログインが必要です');
  expect(deleteAttempts).toBe(0);
});

test('削除APIが401を返したときはダイアログが開いたままになる', async ({ page }) => {
  let sessionAlive = true;
  await page.route(/\/api\/v1\/session$/, async (route) => {
    await route.fulfill({
      status: sessionAlive ? 200 : 401,
      contentType: 'application/json',
      body: sessionAlive ? JSON.stringify(MOCK_USER) : '',
    });
  });
  await page.route(/\/api\/v1\/account-deletion-confirmations$/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  await page.route(/\/api\/v1\/account$/, async (route) => {
    sessionAlive = false;
    await route.fulfill({
      status: 401,
      contentType: 'application/json',
      body: '',
    });
  });
  await page.goto('/');

  await page.getByRole('button', { name: 'メニュー' }).click();
  await page.getByRole('menuitem', { name: 'アカウント削除' }).click();

  const dialog = page.getByRole('dialog', { name: 'アカウント削除の確認' });
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole('alert')).toContainText('ログインが必要です');
});
