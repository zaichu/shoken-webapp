import type { Locator, Page } from '@playwright/test';
import { expect, test, json, paginated, mockSession } from '../support/test';

// 証券会社の CSV は複数口座を連結した値を1つの口座区分として持つ
const JOINED_ACCOUNT = '特定・一般';

const ROWS = [
  { account: JOINED_ACCOUNT, security_name: '架空連結口座銘柄' },
  { account: '特定', security_name: '架空特定口座銘柄' },
  { account: '一般', security_name: '架空一般口座銘柄' },
  { account: 'NISA成長投資枠', security_name: '架空NISA成長銘柄' },
];

const DIVIDENDS = ROWS.map((row, index) => ({
  id: `dividend-${index}`,
  settlement_date: `2026-0${index + 1}-10`,
  product: '国内株式',
  account: row.account,
  security_code: `99${index}4`,
  security_name: row.security_name,
  unit_price: 100,
  shares: 100,
  dividends_before_tax: 5000,
  taxes: 1000,
  net_amount_received: 4000,
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
}));

async function mockApi(page: Page) {
  // 後から登録した route が優先されるため catch-all を先に登録する
  await page.route(/\/api\/v1\//, (route) => route.fulfill(json(paginated([]))));
  await mockSession(page);
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DIVIDENDS))),
  );
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

// 1行の口座の札が1つだけ出ることと、その配色を見る
async function badgeInRow(scope: Locator, securityName: string) {
  const row = scope.locator('tr, li').filter({ hasText: securityName }).first();
  const badge = row.locator('.account-badge');
  await expect(badge).toHaveCount(1);
  return badge;
}

test('連結した口座は1つの札になり、全画面で同じ配色で出る', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await mockApi(page);

  await page.goto('/receipts');
  const table = page.getByRole('table');
  await expect(table).toBeVisible();

  // 連結した口座は分割されず1つの札
  const receiptJoined = await badgeInRow(table, '架空連結口座銘柄');
  await expect(receiptJoined).toHaveText(JOINED_ACCOUNT);
  const receiptSpecific = await badgeInRow(table, '架空特定口座銘柄');
  const receiptGeneral = await badgeInRow(table, '架空一般口座銘柄');
  // 画面を移動すると札が消えるので、移動前に配色だけ控えておく
  const joinedColors = await badgeColors(receiptJoined);
  const generalColors = await badgeColors(receiptGeneral);
  const specificColors = await badgeColors(receiptSpecific);

  await page.goto('/');
  const recentList = page.getByTestId('home-recent-list');
  await expect(recentList).toBeVisible();
  const homeJoined = await badgeInRow(recentList, '架空連結口座銘柄');
  await expect(homeJoined).toHaveText(JOINED_ACCOUNT);
  const homeGeneral = await badgeInRow(recentList, '架空一般口座銘柄');

  // 同じ口座値は取引明細でもホームでも同じ配色
  expect(await badgeColors(homeJoined)).toBe(joinedColors);
  expect(await badgeColors(homeGeneral)).toBe(generalColors);

  // 連結した口座は一般と同じ中立色、単独の「特定」とは区別される
  expect(joinedColors).toBe(generalColors);
  expect(specificColors).not.toBe(joinedColors);
});

test('取引明細のカード表示でも連結した口座は1つの札になる', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await mockApi(page);

  await page.goto('/receipts');
  const cards = page.getByTestId('receipt-card-list');
  await expect(cards).toBeVisible();

  const joined = await badgeInRow(cards, '架空連結口座銘柄');
  await expect(joined).toHaveText(JOINED_ACCOUNT);
});
