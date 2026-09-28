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

test('匿名セッション(401)ならログインボタンを出す', async ({ page }) => {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill(fulfillSession(null)),
  );
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
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
