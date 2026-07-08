import { expect, test } from '@playwright/test';

const BACKEND_URL = process.env.REAL_BACKEND_URL || process.env.VITE_SHOKEN_WEBAPI_API_URL || 'http://127.0.0.1:3001';
const FRONTEND_URL = process.env.REAL_FRONTEND_URL || process.env.BASE_URL || 'http://127.0.0.1:8080';
const SESSION_TOKEN = process.env.REAL_BACKEND_SESSION_TOKEN || '00000000-0000-0000-0000-000000000102';
const EXPECTED_SECURITY_NAME = process.env.REAL_BACKEND_EXPECTED_SECURITY_NAME || 'ＫＤＤＩ';
const REQUEST_TIMEOUT_MS = 5_000;

function cookieDomain(url: string) {
  return new URL(url).hostname;
}

test('実 backend 接続で配当金一覧の初期表示と集計付きAPIが成功する', async ({ page, request }) => {
  await page.context().addCookies([
    {
      name: 'session_token',
      value: SESSION_TOKEN,
      domain: cookieDomain(FRONTEND_URL),
      path: '/',
      httpOnly: true,
      sameSite: 'Lax',
    },
  ]);

  const cookieHeader = `session_token=${SESSION_TOKEN}`;
  const listResponse = await request.get(`${BACKEND_URL}/api/v1/dividends?per_page=1000&page=1`, {
    headers: { Cookie: cookieHeader },
    timeout: REQUEST_TIMEOUT_MS,
  });
  expect(listResponse.status()).toBe(200);

  const summaryResponse = await request.get(
    `${BACKEND_URL}/api/v1/dividends?per_page=1000&page=1&include_summary=true&include_facets=true`,
    { headers: { Cookie: cookieHeader }, timeout: REQUEST_TIMEOUT_MS },
  );
  expect(summaryResponse.status()).toBe(200);

  await page.goto('/receipts');

  await expect(page.getByRole('heading', { name: '取引明細' })).toBeVisible();
  await expect(page.getByText(EXPECTED_SECURITY_NAME).first()).toBeVisible();
});
