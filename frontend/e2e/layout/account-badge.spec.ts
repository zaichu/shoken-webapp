import type { Locator, Page } from '@playwright/test';
import { expect, test, json, paginated } from '../support/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000011',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const THIS_YEAR = new Date().getFullYear();

// 証券会社の CSV は複数口座を連結した値を1つの口座区分として持つ
const JOINED_ACCOUNT = '特定・一般';

function dividend(id: string, account: string) {
  return {
    id,
    user_id: MOCK_USER.id,
    settlement_date: `${THIS_YEAR}-05-10`,
    product: '国内株式',
    account,
    security_code: '9904',
    security_name: '架空自動車',
    unit_price: 100,
    shares: 100,
    dividends_before_tax: 5000,
    taxes: 1000,
    net_amount_received: 4000,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  };
}

const DIVIDENDS = [
  dividend('dividend-joined', JOINED_ACCOUNT),
  dividend('dividend-specific', '特定'),
];

async function mockApi(page: Page) {
  // 後から登録した route が優先されるため catch-all を先に登録する
  await page.route(/\/api\/v1\//, (route) => route.fulfill(json(paginated([]))));
  await page.route(/\/api\/v1\/session$/, (route) => route.fulfill(json(MOCK_USER)));
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) => {
    const url = new URL(route.request().url());
    const perPage = Number(url.searchParams.get('per_page') ?? '0');
    const rows = perPage > 0 ? DIVIDENDS.slice(0, perPage) : DIVIDENDS;
    return route.fulfill(json(paginated(rows)));
  });
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([]))),
  );
  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([]))),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

// 札の配色だけを見る。余白や折り返しは画面ごとに違うので見ない
async function badgeColors(badge: Locator) {
  return badge.evaluate((element) => {
    const style = getComputedStyle(element);
    return [style.backgroundColor, style.color, style.borderColor].join('|');
  });
}

test('同じ口座値は取引明細でもホームでも1つの札になり、配色も同じ', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await mockApi(page);

  await page.goto('/receipts');
  const table = page.getByRole('table');
  await expect(table).toBeVisible();
  // 連結された口座は1つの札。分割した「特定」「一般」の札は出さない
  await expect(table.locator('.account-badge')).toHaveCount(DIVIDENDS.length);
  const receiptBadge = table.locator('.account-badge', { hasText: JOINED_ACCOUNT });
  await expect(receiptBadge).toHaveCount(1);
  await expect(receiptBadge).toHaveText(JOINED_ACCOUNT);

  await page.goto('/');
  const recentList = page.getByTestId('home-recent-list');
  await expect(recentList).toBeVisible();
  await expect(recentList.locator('.account-badge')).toHaveCount(DIVIDENDS.length);
  const homeBadge = recentList.locator('.account-badge', { hasText: JOINED_ACCOUNT });
  await expect(homeBadge).toHaveCount(1);
  await expect(homeBadge).toHaveText(JOINED_ACCOUNT);

  // どの画面でも同じ口座値は同じ配色で出る
  expect(await badgeColors(homeBadge)).toBe(await badgeColors(receiptBadge));
});

test('取引明細のカード表示でも口座は1つの札になる', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await mockApi(page);

  await page.goto('/receipts');
  const cards = page.getByTestId('receipt-card-list');
  await expect(cards).toBeVisible();
  const badges = cards.locator('.account-badge');
  await expect(badges).toHaveCount(DIVIDENDS.length);
  await expect(badges.filter({ hasText: JOINED_ACCOUNT })).toHaveCount(1);
});
