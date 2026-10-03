import type { Page } from '@playwright/test';
import { expect, test, json, paginated } from './support/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const DIVIDENDS = Array.from({ length: 10 }, (_, i) => ({
  id: `dividend-${i}`,
  user_id: MOCK_USER.id,
  settlement_date: `${2020 + (i % 5)}-0${(i % 9) + 1}-15`,
  product: '特定口座',
  account: 'SBI証券',
  security_code: '7203',
  security_name: 'トヨタ自動車',
  unit_price: '10',
  shares: '100',
  dividends_before_tax: '3000',
  taxes: '609',
  net_amount_received: '2391',
  created_at: '2024-03-01T00:00:00Z',
  updated_at: '2024-03-01T00:00:00Z',
}));



async function mockApi(page: Page) {
  await page.route(/\/api\/v1\//, (route) => route.fulfill(json(paginated([]))));
  await page.route(/\/api\/v1\/session$/, (route) => route.fulfill(json(MOCK_USER)));
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DIVIDENDS))),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ data: [] })),
  );
}

test.beforeEach(async ({ page }) => {
  await mockApi(page);
});

test('開閉しても表のx座標と幅が変わらない(MS Learn と同じく列幅は固定)', async ({ page }) => {
  for (const width of [1280, 1440, 1920]) {
    await page.setViewportSize({ width, height: 900 });
    await page.goto('/receipts');
    await expect(page.getByRole('table')).toBeVisible();
    const toggle = page.getByTestId('receipt-utility-toggle');
    // デスクトップでは開いた状態で始まる
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    await expect(page.getByTestId('search-card')).toBeVisible();

    const table = page.getByRole('table');
    const before = (await table.boundingBox())!;

    await toggle.click();
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await expect(page.getByTestId('search-card')).toBeHidden();
    const closed = (await table.boundingBox())!;
    expect(closed.x).toBeCloseTo(before.x, 0);
    expect(closed.width).toBeCloseTo(before.width, 0);

    await toggle.click();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    await expect(page.getByTestId('search-card')).toBeVisible();
    const opened = (await table.boundingBox())!;
    expect(opened.x).toBeCloseTo(before.x, 0);
    expect(opened.width).toBeCloseTo(before.width, 0);

    // 開閉の状態は localStorage に保存しない
    const storedBefore = await page.evaluate(() => JSON.stringify(localStorage));
    await toggle.click();
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await toggle.click();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    const storedAfter = await page.evaluate(() => JSON.stringify(localStorage));
    expect(storedAfter).toBe(storedBefore);
  }
});

test('キーボードで開いた直後のTabがレール内へ進む', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('/receipts');
  await expect(page.getByRole('table')).toBeVisible();
  const toggle = page.getByTestId('receipt-utility-toggle');
  // 初期状態は開きなので、一度畳んでからキーボードで開き直す
  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await toggle.focus();
  await page.keyboard.press('Enter');
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await page.keyboard.press('Tab');
  const focusedRail = await page.evaluate(() => {
    const rail = document.querySelector('[data-testid="receipt-utility-rail"]');
    return rail?.contains(document.activeElement) ?? false;
  });
  expect(focusedRail).toBe(true);
});

test('集計帯は低く12桁金額でもはみ出さない', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(
      json(
        paginated(
          DIVIDENDS.map((row) => ({
            ...row,
            dividends_before_tax: '123456789012',
            taxes: '12345678901',
            net_amount_received: '-123456789012',
          })),
        ),
      ),
    ),
  );
  await page.goto('/receipts');
  await expect(page.getByRole('table')).toBeVisible();
  const strip = page.getByTestId('receipt-summary-strip');
  await expect(strip).toBeVisible();
  const height = (await strip.boundingBox())!.height;
  expect(height).toBeLessThanOrEqual(110);
  const overflows = await strip.evaluate((el) =>
    Array.from(el.querySelectorAll('*')).map((child) => ({
      scrollWidth: (child as HTMLElement).scrollWidth,
      clientWidth: (child as HTMLElement).clientWidth,
    })),
  );
  for (const box of overflows) {
    expect(box.scrollWidth).toBeLessThanOrEqual(box.clientWidth + 1);
  }
});

test('取引明細の見出しはsr-onlyのh1だけ', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('/receipts');
  await expect(page.getByRole('table')).toBeVisible();
  const h1 = page.getByRole('heading', { level: 1, name: '取引明細' });
  await expect(h1).toHaveCount(1);
  await expect(h1).toHaveClass(/sr-only/);
  const box = (await h1.boundingBox())!;
  expect(box.height).toBeLessThanOrEqual(2);
});
