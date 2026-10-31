import type { Page } from '@playwright/test';
import { expect, test, json, mockSession } from '../support/test';

const TOTAL = 25;

const entries = Array.from({ length: TOTAL }, (_, index) => ({
  occurred_at: new Date(Date.UTC(2026, 9, 31, 0, 12) - index * 3_600_000).toISOString(),
  result: index % 3 === 0 ? 'failure' : 'success',
  country: 'JP',
  region: index % 2 === 0 ? 'Tokyo' : 'Osaka',
  browser: 'chrome',
  os: 'windows',
}));

async function mockHistory(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/login-history(?:\?.*)?$/, (route) => {
    const params = new URL(route.request().url()).searchParams;
    const limit = Number(params.get('limit') ?? 10);
    const offset = Number(params.get('offset') ?? 0);
    return route.fulfill(json({ data: entries.slice(offset, offset + limit) }));
  });
  await page.route(/\/api\/v1\/devices$/, (route) => route.fulfill(json({ data: [] })));
  await page.route(/\/api\/v1\/notification-settings$/, (route) =>
    route.fulfill(json({ login_notify_enabled: false })),
  );
}

test('履歴は10件ずつ増え、残りが無くなると「もっと見る」が出ない', async ({ page }) => {
  await mockHistory(page);
  await page.goto('/settings#login-history');

  const table = page.locator('#login-history table');
  await expect(table.locator('tbody tr')).toHaveCount(10);
  const more = page.getByTestId('login-history-more');
  await expect(more).toBeVisible();

  await more.click();
  await expect(table.locator('tbody tr')).toHaveCount(20);
  await expect(more).toBeVisible();

  await more.click();
  await expect(table.locator('tbody tr')).toHaveCount(TOTAL);
  await expect(more).toHaveCount(0);
});

test('登録した端末はログイン履歴より上に並ぶ', async ({ page }) => {
  await mockHistory(page);
  await page.goto('/settings');

  const order = await page.evaluate(() => {
    const sections = Array.from(document.querySelectorAll('#devices, #login-history')).map(
      (el) => el.id,
    );
    return sections;
  });
  expect(order).toEqual(['devices', 'login-history']);
});
