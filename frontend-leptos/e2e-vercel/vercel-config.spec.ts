import { expect, test } from '@playwright/test';

const BACKEND_ORIGIN = 'https://shoken-backend.fly.dev';

test('index と SPA フォールバックが CSP 付きの HTML を返す', async ({ request }) => {
  const root = await request.get('/');
  expect(root.status()).toBe(200);
  expect(root.headers()['content-type']).toContain('text/html');

  const csp = root.headers()['content-security-policy'] ?? '';
  expect(csp).toContain("default-src 'self'");
  expect(csp).toContain("script-src 'self' 'wasm-unsafe-eval'");
  expect(csp).toContain(`connect-src 'self' ${BACKEND_ORIGIN}`);
  expect(csp).toContain('img-src');
  expect(csp).toContain('lh3.googleusercontent.com');
  expect(root.headers()['x-content-type-options']).toBe('nosniff');
  expect(root.headers()['x-frame-options']).toBe('DENY');

  // index.html / SPA fallback は常に再検証させる(ハッシュ無しのため)
  expect(root.headers()['cache-control']).toContain('max-age=0');

  const deep = await request.get('/receipts');
  expect(deep.status()).toBe(200);
  expect(deep.headers()['content-type']).toContain('text/html');
  expect(await deep.text()).toContain('<script type="module" src="/init-');
});

test('ハッシュ付き資産は immutable、wasm は application/wasm で返る', async ({ request }) => {
  const root = await request.get('/');
  const html = await root.text();

  const assetNames = [...html.matchAll(/\/(frontend-leptos-[a-f0-9]+(?:_bg)?\.(?:js|wasm)|output-[a-f0-9]+\.css|init-[a-f0-9]+\.js)/g)].map(
    (m) => m[1],
  );
  expect(assetNames.length).toBeGreaterThanOrEqual(4);

  for (const name of assetNames) {
    const res = await request.get(`/${name}`);
    expect(res.status(), name).toBe(200);
    expect(res.headers()['cache-control'], name).toContain('immutable');
    if (name.endsWith('.wasm')) {
      expect(res.headers()['content-type'], name).toBe('application/wasm');
    }
  }
});

test('CSP 下で wasm が起動して画面が描画され、API が埋め込み先へ向く', async ({ page }) => {
  const cspViolations: string[] = [];
  page.on('console', (msg) => {
    if (/Content Security Policy|Refused to/i.test(msg.text())) cspViolations.push(msg.text());
  });
  page.on('pageerror', (err) => cspViolations.push(String(err)));

  // API 接続先がビルド時の SHOKEN_WEBAPI_URL に埋め込まれていることを、
  // 実ネットワークへ出さずに確認する
  let apiRequestUrl = '';
  await page.route(`${BACKEND_ORIGIN}/**`, (route) => {
    apiRequestUrl = route.request().url();
    return route.fulfill({ status: 401, contentType: 'application/json', body: '{}' });
  });

  await page.goto('/');
  await expect(page.getByRole('heading', { name: '証券Web' })).toBeVisible();

  await expect
    .poll(() => apiRequestUrl, { message: 'session API が埋め込み先へ向かない' })
    .toContain(`${BACKEND_ORIGIN}/api/v1/session`);

  expect(cspViolations).toEqual([]);
});

test('未認証で保護パスを直接開くと /login へリダイレクトする', async ({ page }) => {
  await page.route(`${BACKEND_ORIGIN}/**`, (route) =>
    route.fulfill({ status: 401, contentType: 'application/json', body: '{}' }),
  );

  await page.goto('/receipts');
  await expect(page).toHaveURL(/\/login$/);
  await expect(page.getByRole('button', { name: 'Googleでログイン' })).toBeVisible();
});
