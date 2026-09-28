import { expect, test } from '@playwright/test';

const ALICE = {
  id: '00000000-0000-0000-0000-0000000000a1',
  email: 'alice@example.com',
  name: 'アリス',
};

function fulfillSession(user: object | null) {
  return {
    status: user ? 200 : 401,
    contentType: 'application/json',
    body: user ? JSON.stringify(user) : '{}',
  };
}

test('プローブが出したセッション確認を Wasm 側は撃ち直さない', async ({
  page,
}) => {
  let sessionAt = -1;
  let wasmFinishedAt = -1;
  page.on('request', (request) => {
    if (request.url().endsWith('/api/v1/session')) {
      sessionAt = Date.now();
    }
  });
  page.on('requestfinished', (request) => {
    if (request.url().endsWith('.wasm')) {
      wasmFinishedAt = Date.now();
    }
  });
  let sessionCount = 0;
  await page.route(/\/api\/v1\/session$/, (route) => {
    sessionCount += 1;
    return route.fulfill(fulfillSession(ALICE));
  });

  await page.goto('/');
  await expect(page.getByText(ALICE.name)).toBeVisible();

  expect(sessionCount).toBe(1);
  // Wasm 側の発射は .wasm のダウンロード完了を待つため、プローブ由来なら
  // 要求発行が wasm 完了より前になる。index.html からプローブを外すとここで検知できる
  expect(sessionAt).toBeGreaterThan(0);
  expect(wasmFinishedAt).toBeGreaterThan(0);
  expect(sessionAt).toBeLessThan(wasmFinishedAt);
});

test('プローブが通信失敗したときだけ Wasm 側が撃ち直す', async ({ page }) => {
  let sessionCount = 0;
  await page.route(/\/api\/v1\/session$/, (route) => {
    sessionCount += 1;
    if (sessionCount === 1) {
      return route.abort('failed');
    }
    return route.fulfill(fulfillSession(ALICE));
  });

  await page.goto('/');
  await expect(page.getByText(ALICE.name)).toBeVisible();
  expect(sessionCount).toBe(2);
});

test('匿名セッション(401)ならログインボタンを出し、撃ち直さない', async ({
  page,
}) => {
  let sessionCount = 0;
  await page.route(/\/api\/v1\/session$/, (route) => {
    sessionCount += 1;
    return route.fulfill(fulfillSession(null));
  });
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  expect(sessionCount).toBe(1);
});

test('セッション確認の1回目が 5xx でも撃ち直して表示名が出る', async ({
  page,
}) => {
  let sessionCount = 0;
  await page.route(/\/api\/v1\/session$/, (route) => {
    sessionCount += 1;
    if (sessionCount === 1) {
      return route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: '{}',
      });
    }
    return route.fulfill(fulfillSession(ALICE));
  });
  await page.goto('/');
  await expect(page.getByText(ALICE.name)).toBeVisible();
  // プローブ + 撃ち直し
  expect(sessionCount).toBe(2);
});

test('セッション応答が遅くても待ち時間内なら未ログインにしない', async ({
  page,
}) => {
  let sessionCount = 0;
  await page.route(/\/api\/v1\/session$/, async (route) => {
    sessionCount += 1;
    // プローブの時間切れ(3s)より遅いが、撃ち直しの残り予算(7s)には間に合う
    await new Promise((resolve) => setTimeout(resolve, 6_000));
    return route.fulfill(fulfillSession(ALICE));
  });

  await page.goto('/');
  await expect(page.getByText(ALICE.name)).toBeVisible({ timeout: 15_000 });
  // プローブ + 撃ち直し
  expect(sessionCount).toBe(2);
});

test('無応答でも従来の合計時間内で未ログイン表示になる', async ({ page }) => {
  let sessionCount = 0;
  let firstSessionAt = -1;
  await page.route(/\/api\/v1\/session$/, async () => {
    sessionCount += 1;
    if (firstSessionAt < 0) firstSessionAt = Date.now();
    // 応答しないままぶら下げる
    await new Promise(() => {});
  });

  await page.goto('/');
  // プローブ 3s + 撃ち直し1回 7s で従来経路(約10.5s)を超えないことを固定
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible({
    timeout: 20_000,
  });
  expect(firstSessionAt).toBeGreaterThan(0);
  expect(Date.now() - firstSessionAt).toBeLessThan(11_500);
  // プローブ + 撃ち直し1回
  expect(sessionCount).toBe(2);
});

test('セッション確認が遅いときは読み込み表示のまま待つ', async ({ page }) => {
  let release!: () => void;
  const gate = new Promise<void>((resolve) => {
    release = resolve;
  });
  await page.route(/\/api\/v1\/session$/, async (route) => {
    await gate;
    return route.fulfill(fulfillSession(ALICE));
  });

  await page.goto('/');
  await expect(
    page.getByRole('banner').getByText('読み込み中...'),
  ).toBeVisible();
  release();
  await expect(page.getByText(ALICE.name)).toBeVisible();
});

test('ログアウト保留中はプローブを出さず DELETE が先に出る', async ({
  page,
}) => {
  await page.addInitScript(() =>
    window.localStorage.setItem('pending_logout', '1'),
  );
  const sessionMethods: string[] = [];
  await page.route(/\/api\/v1\/session$/, (route) => {
    sessionMethods.push(route.request().method());
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });

  await page.goto('/');
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect.poll(() => sessionMethods.length).toBe(1);
  expect(sessionMethods).toEqual(['DELETE']);
});
