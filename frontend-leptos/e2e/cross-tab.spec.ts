import { expect, test, type BrowserContext, type Page } from '@playwright/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000003',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const ACTIVITY_KEY = 'last_activity_at';
const MINUTE = 60_000;

async function logoutViaUserMenu(page: Page) {
  await page.getByRole('button', { name: 'メニュー' }).click();
  await page.getByTestId('logout').click();
}

async function stubSession(context: BrowserContext, onDelete: () => void) {
  await context.route(/\/api\/v1\/session$/, async (route) => {
    if (route.request().method() === 'DELETE') {
      onDelete();
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: '{"message":"ok"}',
      });
    }
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_USER),
    });
  });
}

async function openLoggedInPages(context: BrowserContext) {
  const pageA = await context.newPage();
  const pageB = await context.newPage();
  await pageA.goto('/');
  await pageB.goto('/');
  await expect(pageA.getByRole('button', { name: 'メニュー' })).toBeVisible();
  await expect(pageB.getByRole('button', { name: 'メニュー' })).toBeVisible();
  return { pageA, pageB };
}

test('片方のタブでログアウトするともう片方も未認証になり、APIは実行したタブだけが呼ぶ', async ({
  context,
}) => {
  let deleteCount = 0;
  await stubSession(context, () => {
    deleteCount += 1;
  });
  const { pageA, pageB } = await openLoggedInPages(context);

  await logoutViaUserMenu(pageA);

  await expect(pageA.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect(pageB.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect.poll(() => deleteCount).toBe(1);
  await pageA.waitForTimeout(500);
  expect(deleteCount).toBe(1);
});

test('片方のタブでアカウント削除するともう片方も未認証になる', async ({ context }) => {
  await stubSession(context, () => {});
  await context.route(/\/api\/v1\/account-deletion-confirmations$/, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '{"message":"ok"}' }),
  );
  await context.route(/\/api\/v1\/account$/, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '{"message":"ok"}' }),
  );
  const { pageA, pageB } = await openLoggedInPages(context);

  await pageA.getByRole('button', { name: 'メニュー' }).click();
  await pageA.getByRole('menuitem', { name: 'アカウント削除' }).click();
  await pageA
    .getByRole('dialog', { name: 'アカウント削除の確認' })
    .getByRole('button', { name: '削除する' })
    .click();

  await expect(pageA.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect(pageB.getByRole('button', { name: 'ログイン' })).toBeVisible();
});

test('BroadcastChannelが無い環境ではstorageイベントでログアウトを共有する', async ({ context }) => {
  await context.addInitScript(() => {
    Object.defineProperty(window, 'BroadcastChannel', { value: undefined });
  });
  let deleteCount = 0;
  await stubSession(context, () => {
    deleteCount += 1;
  });
  const { pageA, pageB } = await openLoggedInPages(context);

  await logoutViaUserMenu(pageA);

  await expect(pageA.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect(pageB.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect.poll(() => deleteCount).toBe(1);
});

test('片方のタブで操作を続けるともう片方もログアウトせず、全タブが無操作になるとログアウトする', async ({
  context,
}) => {
  // page.clock はコンテキスト全体で共有されるので、両ページで1本の時計として扱う
  const pageA = await context.newPage();
  const pageB = await context.newPage();
  await pageA.clock.install();

  let deleteCount = 0;
  await stubSession(context, () => {
    deleteCount += 1;
  });

  await pageA.goto('/');
  await pageB.goto('/');
  await expect(pageA.getByRole('button', { name: 'メニュー' })).toBeVisible();
  await expect(pageB.getByRole('button', { name: 'メニュー' })).toBeVisible();

  await pageA.clock.fastForward(29 * MINUTE);

  // B の操作が最終操作時刻として共有される
  const activityBefore = await pageA.evaluate(
    (key) => window.localStorage.getItem(key),
    ACTIVITY_KEY,
  );
  await pageB.mouse.move(200, 200);
  await expect
    .poll(() => pageA.evaluate((key) => window.localStorage.getItem(key), ACTIVITY_KEY))
    .not.toBe(activityBefore);

  // A のタイマーは共有で延びているので、30 分を過ぎても未認証にならない
  await pageA.clock.fastForward(2 * MINUTE);
  await expect(pageA.getByRole('button', { name: 'メニュー' })).toBeVisible();
  expect(deleteCount).toBe(0);

  // どのタブでも操作が止まって 30 分経つとログアウトし、両方が未認証になる
  await pageA.clock.fastForward(29 * MINUTE);
  await expect.poll(() => deleteCount).toBe(1);
  await expect(pageB.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect(pageA.getByRole('button', { name: 'ログイン' })).toBeVisible();
});

test('未認証のタブの操作は最終操作時刻に記録されず、認証済みタブのログアウトを延ばさない', async ({
  context,
}) => {
  const pageA = await context.newPage();
  const pageB = await context.newPage();
  await pageA.clock.install();
  await pageA.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_USER),
    }),
  );
  await pageB.route(/\/api\/v1\/session$/, (route) => route.fulfill({ status: 401, body: '{}' }));

  await pageA.goto('/');
  await pageB.goto('/');
  await expect(pageA.getByRole('button', { name: 'メニュー' })).toBeVisible();
  await expect(pageB.getByRole('button', { name: 'ログイン' })).toBeVisible();

  await pageA.clock.fastForward(29 * MINUTE);
  const activityBefore = await pageA.evaluate(
    (key) => window.localStorage.getItem(key),
    ACTIVITY_KEY,
  );
  await pageB.mouse.move(200, 200);
  await pageA.waitForTimeout(500);
  expect(
    await pageA.evaluate((key) => window.localStorage.getItem(key), ACTIVITY_KEY),
  ).toBe(activityBefore);

  await pageA.clock.fastForward(2 * MINUTE);
  await expect(pageA.getByRole('button', { name: 'ログイン' })).toBeVisible();
});
