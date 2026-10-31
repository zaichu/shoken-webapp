import type { Page } from '@playwright/test';
import { promises as fs } from 'fs';
import { expect, test, json, paginated, mockSession } from '../support/test';

const DIVIDEND = {
  id: '00000000-0000-0000-0000-000000000001',
  settlement_date: '2024-03-01',
  product: '特定口座',
  account: '架空証券',
  security_code: '9904',
  security_name: '架空自動車',
  unit_price: 30.0,
  shares: 100,
  dividends_before_tax: 3000,
  taxes: 609,
  net_amount_received: 2391,
  created_at: '2024-03-01T00:00:00Z',
  updated_at: '2024-03-01T00:00:00Z',
};

async function mockAuthorizedAuth(page: Page) {
  await mockSession(page);
}

async function mockReceiptLists(page: Page) {
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([DIVIDEND]))),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([]))),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([]))),
  );
}

test('表示設定を変えると localStorage に保存されリロード後も維持される', async ({ page }) => {
  await mockAuthorizedAuth(page);
  await page.goto('/settings');

  await page
    .getByTestId('setting-amount-unit')
    .getByRole('button', { name: '千円' })
    .click();
  await expect
    .poll(() => page.evaluate(() => localStorage.getItem('setting_amount_unit')))
    .toBe('sen');

  await page.reload();
  const unit = page.getByTestId('setting-amount-unit');
  await expect(unit.getByRole('button', { name: '千円' })).toHaveAttribute('aria-pressed', 'true');
  await expect(unit.getByRole('button', { name: '円', exact: true })).toHaveAttribute(
    'aria-pressed',
    'false',
  );
});

test('テーマ設定を変えると localStorage に保存され html の data-theme に反映される', async ({
  page,
}) => {
  await mockAuthorizedAuth(page);
  await page.goto('/settings');

  await page.getByTestId('setting-theme').getByRole('button', { name: 'ダーク' }).click();
  await expect
    .poll(() => page.evaluate(() => localStorage.getItem('setting_theme')))
    .toBe('dark');
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');

  await page.reload();
  const theme = page.getByTestId('setting-theme');
  await expect(theme.getByRole('button', { name: 'ダーク' })).toHaveAttribute(
    'aria-pressed',
    'true',
  );
  await expect(theme.getByRole('button', { name: 'ライト' })).toHaveAttribute(
    'aria-pressed',
    'false',
  );
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
});

test('端末に合わせるテーマでは OS の配色設定に従う', async ({ page }) => {
  await mockAuthorizedAuth(page);
  await page.emulateMedia({ colorScheme: 'dark' });
  await page.goto('/settings');
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');

  // リロードせず OS 設定が変わったときも追従する
  await page.emulateMedia({ colorScheme: 'light' });
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
});

test('localStorage の値が壊れているとき設定は既定値に戻る', async ({ page }) => {
  await mockAuthorizedAuth(page);
  await page.addInitScript(() => {
    localStorage.setItem('setting_amount_unit', 'invalid');
    localStorage.setItem('setting_auto_logout_minutes', '7');
    localStorage.setItem('setting_theme', 'sepia');
  });
  await page.goto('/settings');

  const unit = page.getByTestId('setting-amount-unit');
  await expect(unit.getByRole('button', { name: '円', exact: true })).toHaveAttribute(
    'aria-pressed',
    'true',
  );
  await expect(
    page.getByTestId('setting-idle-minutes').getByRole('button', { name: '30分' }),
  ).toHaveAttribute('aria-pressed', 'true');
  await expect(
    page.getByTestId('setting-theme').getByRole('button', { name: '端末に合わせる' }),
  ).toHaveAttribute('aria-pressed', 'true');
});

test('取引明細の初期タブ設定が /receipts の最初のタブに反映される', async ({ page }) => {
  await mockAuthorizedAuth(page);
  await mockReceiptLists(page);
  await page.addInitScript(() => {
    localStorage.setItem('setting_receipts_initial_tab', 'stock');
  });
  await page.goto('/receipts');

  await expect(page.getByRole('tab', { name: /国内株式/ })).toHaveAttribute(
    'aria-selected',
    'true',
  );
  await expect(page.getByRole('tab', { name: /配当金/ })).toHaveAttribute(
    'aria-selected',
    'false',
  );
});

test('金額の表示単位設定は合計欄だけに効き、明細の行は1円単位のまま出る', async ({ page }) => {
  await mockAuthorizedAuth(page);
  await mockReceiptLists(page);
  await page.addInitScript(() => {
    localStorage.setItem('setting_amount_unit', 'sen');
  });
  await page.goto('/receipts');

  // 合計欄(集計)は千円表示になり、明細の行は1円単位のまま。
  // 単価(1株配当 30 円)も円のまま出る(非表示列でも DOM に残るため存在だけ見る)
  await expect(page.getByText('2.4千円').first()).toBeVisible();
  await expect(page.getByRole('cell', { name: '¥2,391' }).first()).toBeVisible();
  await expect(page.getByText('¥30').first()).toBeAttached();
});

test('設定の目次は7項目あり各セクションへリンクする', async ({ page }) => {
  await mockAuthorizedAuth(page);
  await page.goto('/settings');

  const nav = page.getByRole('navigation', { name: '設定セクション' });
  await expect(nav).toBeVisible();
  const links = nav.getByRole('link');
  await expect(links).toHaveCount(7);
  const hrefs = await links.evaluateAll((els) =>
    els.map((el) => el.getAttribute('href')),
  );
  expect(hrefs).toEqual([
    '/settings#account',
    '/settings#display',
    '/settings#data',
    '/settings#security',
    '/settings#devices',
    '/settings#login-history',
    '/settings#danger',
  ]);
});

test.describe('設定の目次(PC)', () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  test('目次を押すと押した項目が強調され見出しが上部ナビに隠れない', async ({ page }) => {
    await mockAuthorizedAuth(page);
    await page.route(/\/api\/v1\/notification-settings$/, (route) =>
      route.fulfill(json({ login_notify_enabled: true })),
    );
    const history = Array.from({ length: 14 }, (_, index) => ({
      occurred_at: `2026-10-${String(31 - index).padStart(2, '0')}T00:12:00Z`,
      result: 'success',
      country: 'JP',
      region: 'Tokyo',
      browser: 'chrome',
      os: 'windows',
    }));
    await page.route(/\/api\/v1\/login-history(?:\?.*)?$/, (route) =>
      route.fulfill(json({ data: history })),
    );
    const devices = Array.from({ length: 6 }, (_, index) => ({
      id: `device-${index + 1}`,
      last_used_at: '2026-10-31T00:12:00Z',
      country: 'JP',
      region: 'Tokyo',
      browser: 'chrome',
      os: 'windows',
      is_current: index === 0,
    }));
    await page.route(/\/api\/v1\/devices$/, (route) =>
      route.fulfill(json({ data: devices })),
    );
    await page.goto('/settings');
    await page.waitForSelector('#login-history tbody tr');
    await page.waitForSelector('#devices li');

    const items = [
      ['settings-toc-account', 'account-heading'],
      ['settings-toc-display', 'display-heading'],
      ['settings-toc-data', 'data-heading'],
      ['settings-toc-security', 'security-heading'],
      ['settings-toc-login-history', 'login-history-heading'],
      ['settings-toc-devices', 'devices-heading'],
      ['settings-toc-danger', 'danger-heading'],
    ] as const;
    for (const [testid, heading] of items) {
      const link = page.getByTestId(testid);
      await link.click();
      await expect(link).toHaveAttribute('aria-current', 'true');
      for (const [other] of items) {
        if (other !== testid) {
          await expect(page.getByTestId(other)).not.toHaveAttribute('aria-current', 'true');
        }
      }
      const geometry = await page.evaluate((headingId: string) => {
        const headingElement = document.getElementById(headingId);
        const header = document.querySelector('.pc-header');
        if (!headingElement || !header) {
          return null;
        }
        const headingRect = headingElement.getBoundingClientRect();
        const headerRect = header.getBoundingClientRect();
        return {
          top: headingRect.top,
          headerBottom: headerRect.bottom,
          viewportHeight: window.innerHeight,
        };
      }, heading);
      expect(geometry).not.toBeNull();
      // 末尾の項目は上端まで届かなくてもよいが、見出しは上部ナビの下に出る
      expect(geometry!.top).toBeGreaterThanOrEqual(geometry!.headerBottom - 1);
      expect(geometry!.top).toBeLessThan(geometry!.viewportHeight);
    }
  });
});

test('配当金のCSVエクスポートがダウンロードされ内容に個人情報を含まない', async ({ page }) => {
  await mockAuthorizedAuth(page);
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([DIVIDEND]))),
  );
  await page.goto('/settings');

  const downloadPromise = page.waitForEvent('download');
  await page.getByTestId('export-dividends').click();
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toMatch(/^dividends_\d{4}-\d{2}-\d{2}\.csv$/);

  const path = await download.path();
  const content = await fs.readFile(path!, 'utf8');
  expect(content).toContain('入金日');
  expect(content).toContain('架空自動車');
  expect(content).not.toContain('test@example.com');
});
