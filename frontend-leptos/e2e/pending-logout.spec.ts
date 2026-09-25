import { expect, test, type Page } from '@playwright/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000003',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const PENDING_KEY = 'pending_logout';

async function logoutViaUserMenu(page: Page) {
  await page.getByRole('button', { name: 'メニュー' }).click();
  await page.getByTestId('logout').click();
}

async function pendingFlag(page: Page) {
  return page.evaluate((key) => window.localStorage.getItem(key), PENDING_KEY);
}

test('ログアウト失敗を保留し、間隔を伸ばして再送し、再読み込みではセッション確認より先にログアウトを再送する', async ({
  page,
}) => {
  await page.clock.install();

  let getCount = 0;
  let deleteCount = 0;
  let deleteFails = true;
  await page.route(/\/api\/v1\/session$/, async (route) => {
    if (route.request().method() === 'DELETE') {
      deleteCount += 1;
      if (deleteFails) {
        return route.abort('failed');
      }
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: '{"message":"ok"}',
      });
    }
    getCount += 1;
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_USER),
    });
  });

  await page.goto('/');
  await expect(page.getByRole('button', { name: 'メニュー' })).toBeVisible();
  expect(getCount).toBe(1);

  await logoutViaUserMenu(page);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect.poll(() => pendingFlag(page)).toBe('1');
  await expect.poll(() => deleteCount).toBe(1);

  // 通信断はクライアント内部でも3回再送する(1s/2s/4s後)ので計4回で失敗確定する
  await page.clock.fastForward(1_000);
  await expect.poll(() => deleteCount).toBe(2);
  await page.clock.fastForward(2_000);
  await expect.poll(() => deleteCount).toBe(3);
  await page.clock.fastForward(4_000);
  await expect.poll(() => deleteCount).toBe(4);

  // 保留中は間隔を伸ばして再送を続ける(10s→30s→60s、各回さらに内部再送あり)
  await page.clock.fastForward(10_000);
  await expect.poll(() => deleteCount).toBeGreaterThan(4);
  const resent = deleteCount;
  await page.clock.fastForward(60_000);
  await expect.poll(() => deleteCount).toBeGreaterThan(resent);

  // セッションAPIが有効を返し続けても、起動時は先にログアウトを再送して未認証のままにする
  deleteFails = false;
  await page.reload();
  await expect.poll(() => deleteCount).toBeGreaterThan(resent);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'メニュー' })).toBeHidden();
  await expect.poll(() => pendingFlag(page)).toBeNull();
  expect(getCount).toBe(1);
});

test('ログアウトが401を返したらセッション消失とみなして保留にしない', async ({ page }) => {
  let deleteCount = 0;
  await page.route(/\/api\/v1\/session$/, async (route) => {
    if (route.request().method() === 'DELETE') {
      deleteCount += 1;
      return route.fulfill({ status: 401, body: '{}' });
    }
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_USER),
    });
  });

  await page.goto('/');
  await expect(page.getByRole('button', { name: 'メニュー' })).toBeVisible();

  await logoutViaUserMenu(page);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect.poll(() => pendingFlag(page)).toBeNull();
  expect(deleteCount).toBe(1);
});

test('localStorageが使えなくてもログインとログアウトが動く', async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(window, 'localStorage', {
      get() {
        throw new DOMException('denied', 'SecurityError');
      },
    });
  });
  await page.route(/\/api\/v1\/session$/, async (route) => {
    if (route.request().method() === 'DELETE') {
      return route.fulfill({ status: 500, body: '{}' });
    }
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_USER),
    });
  });

  await page.goto('/');
  await expect(page.getByRole('button', { name: 'メニュー' })).toBeVisible();

  await logoutViaUserMenu(page);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
});
