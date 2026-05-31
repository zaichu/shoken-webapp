/**
 * アクセシビリティ監査 E2E テスト（API モック使用）
 *
 * @axe-core/playwright を使用して主要ページの WCAG 違反を検出する。
 * page.route() でバックエンド API をモックし、認証なしで実行可能。
 *
 * 使用方法:
 *   cd frontend && npx playwright test e2e/a11y.spec.ts --config playwright.ci.config.ts
 */
import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const MOCK_USER = { id: 1, email: 'test@example.com', name: 'テストユーザー' };

const ROUTES = {
  authMe: /\/api\/v1\/session$/,
  dividends: /\/dividends$/,
  domesticStocks: /\/domestic-stocks$/,
  mutualfunds: /\/mutualfunds$/,
  assetBalances: /\/asset-balances$/,
  stock: /\/stocks\/[^/]+$/,
};

async function setupAuthMocks(page: import('@playwright/test').Page) {
  await page.route(ROUTES.authMe, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(MOCK_USER) }),
  );
  await page.route(ROUTES.dividends, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
  await page.route(ROUTES.domesticStocks, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
  await page.route(ROUTES.mutualfunds, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
  await page.route(ROUTES.assetBalances, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
}

// WCAG2AA の critical/serious 違反のみチェックする
function buildAxe(page: import('@playwright/test').Page) {
  return new AxeBuilder({ page })
    .withTags(['wcag2a', 'wcag2aa'])
    .options({ runOnly: { type: 'tag', values: ['wcag2a', 'wcag2aa'] } });
}

test('ホームページに重大な WCAG 違反がない', async ({ page }) => {
  await setupAuthMocks(page);
  await page.goto('/');
  await page.waitForLoadState('networkidle');

  const results = await buildAxe(page).analyze();
  const critical = results.violations.filter((v) => v.impact === 'critical' || v.impact === 'serious');

  expect(
    critical,
    critical.map((v) => `[${v.impact}] ${v.id}: ${v.description}`).join('\n'),
  ).toHaveLength(0);
});

test('銘柄検索ページに重大な WCAG 違反がない', async ({ page }) => {
  await setupAuthMocks(page);
  await page.route(ROUTES.stock, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
  await page.goto('/search');
  await page.waitForLoadState('networkidle');

  const results = await buildAxe(page).analyze();
  const critical = results.violations.filter((v) => v.impact === 'critical' || v.impact === 'serious');

  expect(
    critical,
    critical.map((v) => `[${v.impact}] ${v.id}: ${v.description}`).join('\n'),
  ).toHaveLength(0);
});

test('資産管理ページに重大な WCAG 違反がない', async ({ page }) => {
  await setupAuthMocks(page);
  await page.goto('/assetbalance');
  await page.waitForLoadState('networkidle');

  const results = await buildAxe(page).analyze();
  const critical = results.violations.filter((v) => v.impact === 'critical' || v.impact === 'serious');

  expect(
    critical,
    critical.map((v) => `[${v.impact}] ${v.id}: ${v.description}`).join('\n'),
  ).toHaveLength(0);
});

test('取引明細ページに重大な WCAG 違反がない（全タブ）', async ({ page }) => {
  await setupAuthMocks(page);
  await page.goto('/receipts');
  await page.waitForLoadState('networkidle');

  const tabs = [
    { label: '配当金', selector: 'button[role="tab"][id="tab-dividend"]' },
    { label: '国内株式', selector: 'button[role="tab"][id="tab-domesticstock"]' },
    { label: '投資信託', selector: 'button[role="tab"][id="tab-mutualfund"]' },
  ];

  for (const tab of tabs) {
    await page.click(tab.selector);
    await page.waitForLoadState('networkidle');

    const results = await buildAxe(page).analyze();
    const critical = results.violations.filter((v) => v.impact === 'critical' || v.impact === 'serious');

    expect(
      critical,
      `[${tab.label}タブ] ` + critical.map((v) => `[${v.impact}] ${v.id}: ${v.description}`).join('\n'),
    ).toHaveLength(0);
  }
});
