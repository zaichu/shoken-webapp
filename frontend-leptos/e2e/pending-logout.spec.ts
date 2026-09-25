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

  // 偽時計はナビゲーションを跨いで持続し、アプリの再送タイマー(setTimeout系)ごと止める。
  // 起動時のセッション確認が一時的に失敗しても再送で回復できるよう、起動確認が済むまでは実時計のままにする
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'メニュー' })).toBeVisible();
  expect(getCount).toBeGreaterThanOrEqual(1);

  await page.clock.install();

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
  const deletesBeforeReload = deleteCount;
  const getsBeforeReload = getCount;
  // 偽時計のまま再読み込みすると再送の再試行が発火せず失敗から回復できないため、実時計に戻す
  await page.clock.resume();
  await page.reload();
  await expect.poll(() => deleteCount).toBeGreaterThan(deletesBeforeReload);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'メニュー' })).toBeHidden();
  await expect.poll(() => pendingFlag(page)).toBeNull();
  expect(getCount).toBe(getsBeforeReload);
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

test('保留中にログインすると一度だけ再送して記録を消し、戻ってきたセッションは消えない', async ({ page }) => {
  let getCount = 0;
  let deleteCount = 0;
  await page.route(/\/api\/v1\/session$/, async (route) => {
    if (route.request().method() === 'DELETE') {
      deleteCount += 1;
      return route.fulfill({ status: 400, body: '{}' });
    }
    getCount += 1;
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_USER),
    });
  });
  await page.route(/\/api\/v1\/oauth\/google\/authorize/, (route) =>
    route.fulfill({ status: 200, contentType: 'text/html', body: '<html></html>' }),
  );

  // 偽時計はナビゲーションを跨いで持続し、アプリの再送タイマー(setTimeout系)ごと止める。
  // 起動時のセッション確認が一時的に失敗しても再送で回復できるよう、起動確認が済むまでは実時計のままにする
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'メニュー' })).toBeVisible();

  await page.clock.install();

  await logoutViaUserMenu(page);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect.poll(() => pendingFlag(page)).toBe('1');
  await expect.poll(() => deleteCount).toBe(1);

  await Promise.all([
    page.waitForURL(/\/api\/v1\/oauth\/google\/authorize/),
    page.getByRole('button', { name: 'ログイン' }).click(),
  ]);
  await expect.poll(() => deleteCount).toBe(2);
  await expect.poll(() => pendingFlag(page)).toBeNull();

  const getsBeforeReload = getCount;
  // 偽時計のまま遷移するとセッション確認の再試行が発火せず失敗から回復できないため、実時計に戻す
  await page.clock.resume();
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'メニュー' })).toBeVisible();
  expect(deleteCount).toBe(2);
  expect(getCount).toBeGreaterThan(getsBeforeReload);
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
