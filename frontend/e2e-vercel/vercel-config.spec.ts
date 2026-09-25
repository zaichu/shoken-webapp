import { expect, test } from '@playwright/test';
import { get } from 'node:http';
import { loadBackendOrigin } from '../scripts/backend-origin.cjs';

const BACKEND_ORIGIN: string = loadBackendOrigin();
const SERVE_PORT = Number(process.env.VERCEL_E2E_PORT ?? '8190');

// URL パーサがデコード前にドットセグメントを潰すため、生のパスで送る必要がある
function rawGet(path: string): Promise<{ status: number; type: string }> {
  return new Promise((resolvePromise, reject) => {
    get({ host: '127.0.0.1', port: SERVE_PORT, path }, (res) => {
      res.resume();
      res.on('end', () =>
        resolvePromise({ status: res.statusCode ?? 0, type: res.headers['content-type'] ?? '' }),
      );
    }).on('error', reject);
  });
}

test('index と SPA フォールバックが CSP 付きの HTML を返す', async ({ request }) => {
  const root = await request.get('/');
  expect(root.status()).toBe(200);
  expect(root.headers()['content-type']).toContain('text/html');

  const csp = root.headers()['content-security-policy'] ?? '';
  expect(csp).toContain("default-src 'self'");
  expect(csp).toContain("script-src 'self' 'wasm-unsafe-eval'");
  // connect-src は 'self' と backend の URL だけで閉じること(他の外部 origin を足さない)
  expect(csp).toContain(`connect-src 'self' ${BACKEND_ORIGIN}; font-src 'self'`);
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

  const assetPaths = [
    ...new Set(
      [...html.matchAll(/(?:src|href)="(\/[^"]+\.(?:js|wasm|css))"/g)].map((m) => m[1]),
    ),
  ];
  expect(assetPaths.length).toBeGreaterThanOrEqual(4);

  for (const path of assetPaths) {
    const res = await request.get(path);
    expect(res.status(), path).toBe(200);
    const cacheControl = res.headers()['cache-control'] ?? '';
    if (path.startsWith('/snippets/')) {
      // スニペットのディレクトリ名は crate 由来で内容ハッシュではないため immutable を付けない
      expect(cacheControl, path).not.toContain('immutable');
    } else {
      expect(cacheControl, path).toContain('immutable');
    }
    if (path.endsWith('.wasm')) {
      expect(res.headers()['content-type'], path).toBe('application/wasm');
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

test('エンコードされた親参照でも dist 外を返さず、不正なパスは 400 でサーバが落ちない', async () => {
  // dist の親にある frontend/package.json が拾えないことを確認する
  const traversal = await rawGet('/%2e%2e/package.json');
  expect(traversal.type).not.toContain('application/json');

  const malformed = await rawGet('/%');
  expect(malformed.status).toBe(400);

  const alive = await rawGet('/');
  expect(alive.status).toBe(200);
});
