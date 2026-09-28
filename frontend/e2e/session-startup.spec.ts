import { expect, test, type Page } from '@playwright/test';

// 起動時のセッション確認(index.html のプローブ + 前回表示のスナップショット)の E2E。

const ALICE = {
  id: '00000000-0000-0000-0000-0000000000a1',
  email: 'alice@example.com',
  name: 'アリス',
};
const BOB = {
  id: '00000000-0000-0000-0000-0000000000b2',
  email: 'bob@example.com',
  // アバターの頭文字と表示名が一致しないよう、長い表示名にしておく
  name: 'ボブ山 太郎',
};

const SNAPSHOT_KEY = 'session_snapshot';
const ALICE_SNAPSHOT = JSON.stringify({ id: ALICE.id, name: ALICE.name });

function seedSnapshot(page: Page, snapshot = ALICE_SNAPSHOT) {
  return page.addInitScript(
    ([key, value]) => localStorage.setItem(key, value),
    [SNAPSHOT_KEY, snapshot],
  );
}

function mockSession(page: Page, user: object | null) {
  return page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill({
      status: user ? 200 : 401,
      contentType: 'application/json',
      body: user ? JSON.stringify(user) : '{}',
    }),
  );
}

test('セッション確認が遅くても前回の表示名が先に出る', async ({ page }) => {
  await seedSnapshot(page);
  let release!: () => void;
  const gate = new Promise<void>((resolve) => {
    release = resolve;
  });
  await page.route(/\/api\/v1\/session$/, async (route) => {
    await gate;
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(ALICE),
    });
  });
  await page.goto('/');

  // セッション応答を止めたままでも、ヘッダーに前回の表示名が出る
  await expect(page.getByText(ALICE.name)).toBeVisible();
  await expect(page.getByRole('button', { name: 'メニュー' })).toBeVisible();

  // 応答を解放しても同じユーザーなら表示はそのまま
  release();
  await expect(page.getByText(ALICE.name)).toBeVisible();
});

test('スナップショットが無ければ確認中は読み込み表示になる', async ({ page }) => {
  let release!: () => void;
  const gate = new Promise<void>((resolve) => {
    release = resolve;
  });
  await page.route(/\/api\/v1\/session$/, async (route) => {
    await gate;
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(ALICE),
    });
  });
  await page.goto('/');

  await expect(
    page.getByRole('banner').getByText('読み込み中...'),
  ).toBeVisible();
  release();
  await expect(page.getByText(ALICE.name)).toBeVisible();
  // 確認できたユーザーは次回起動用に保存される(個人情報のメールアドレスは入らない)
  await expect
    .poll(() =>
      page.evaluate(
        (key) => JSON.parse(localStorage.getItem(key) ?? 'null'),
        SNAPSHOT_KEY,
      ),
    )
    .toMatchObject({ id: ALICE.id, name: ALICE.name });
  const stored = await page.evaluate(
    (key) => JSON.parse(localStorage.getItem(key) ?? 'null'),
    SNAPSHOT_KEY,
  );
  expect(stored).not.toHaveProperty('email');
});

test('セッションが切れていれば表示名を消してログインを出す', async ({ page }) => {
  await seedSnapshot(page);
  await mockSession(page, null);
  await page.goto('/');

  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect(page.getByText(ALICE.name)).toHaveCount(0);
  // 期限切れのスナップショットは残さない
  await expect
    .poll(() => page.evaluate((key) => localStorage.getItem(key), SNAPSHOT_KEY))
    .toBeNull();
});

test('別のユーザーに変わっていれば表示名とスナップショットを入れ替える', async ({
  page,
}) => {
  await seedSnapshot(page);
  await mockSession(page, BOB);
  await page.goto('/');

  await expect(page.getByText(BOB.name)).toBeVisible();
  await expect(page.getByText(ALICE.name)).toHaveCount(0);
  await expect
    .poll(() =>
      page.evaluate(
        (key) => JSON.parse(localStorage.getItem(key) ?? 'null'),
        SNAPSHOT_KEY,
      ),
    )
    .toMatchObject({ id: BOB.id, name: BOB.name });
});

test('ログアウトするとスナップショットも消える', async ({ page }) => {
  await seedSnapshot(page);
  await page.route(/\/api\/v1\/session$/, async (route) => {
    if (route.request().method() === 'DELETE') {
      return route.fulfill({ status: 200, body: '' });
    }
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(ALICE),
    });
  });
  await page.goto('/');
  await expect(page.getByText(ALICE.name)).toBeVisible();

  await page.getByRole('button', { name: 'メニュー' }).click();
  await page.getByRole('menuitem', { name: 'ログアウト' }).click();

  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect
    .poll(() => page.evaluate((key) => localStorage.getItem(key), SNAPSHOT_KEY))
    .toBeNull();
});
