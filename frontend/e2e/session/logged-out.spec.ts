import type { Page } from '@playwright/test';
import { expect, test, json, paginated, mockSession } from '../support/test';

const ROUTES = {
  authMe: /\/api\/v1\/session$/,
};

const ASSET_BALANCES = [
  {
    id: 'asset-balance-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: '9904',
    security_name: '架空自動車',
    shares: 100,
    average_purchase_price: 2500,
    total_purchase_amount: 250000,
    current_price: 2600,
    daily_change: 50,
    market_value: 260000,
    profit_loss_rate: 4.0,
  },
];

async function mockAnonymous(page: Page) {
  await page.route(ROUTES.authMe, (route) => route.fulfill(json({}, 401)));
}

async function mockAuthenticated(page: Page) {
  // 後から登録した route が優先されるため、基本モックの後に呼ぶ
  await page.route(/\/api\/v1\//, (route) => route.fulfill(json(paginated([]))));
  await mockSession(page);
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(ASSET_BALANCES))),
  );
}

// ログインが要る画面で出す案内。ログイン画面・ログアウト中のホームと同じ部品
function loginGuidance(page: Page) {
  return page.getByTestId('empty-state');
}

test('未ログインのホームはログインの案内だけになり、CSV管理の操作は出ない', async ({ page }) => {
  await mockAnonymous(page);

  await page.goto('/');

  const guidance = loginGuidance(page);
  await expect(guidance).toBeVisible();
  await expect(guidance.getByRole('button', { name: 'Googleでログイン' })).toBeVisible();
  // ログイン前に使えない操作は出さない
  await expect(page.getByRole('link', { name: 'CSV管理', exact: true })).toHaveCount(0);
});

for (const path of [
  '/assetbalance',
  '/receipts',
  '/settings',
  '/csv',
  // 旧ルートも保護対象のまま
  '/assetbalance/csv',
  '/receipts/csv',
]) {
  test(`未ログインで${path}を開くと他の画面と同じログインの案内が出る`, async ({ page }) => {
    await mockAnonymous(page);

    await page.goto(path);

    const guidance = loginGuidance(page);
    await expect(guidance).toBeVisible();
    await expect(guidance.getByRole('button', { name: 'Googleでログイン' })).toBeVisible();
  });
}

test('銘柄検索の空状態はログイン案内と同じ空状態の部品で出す', async ({ page }) => {
  await mockAnonymous(page);

  await page.goto('/search');

  const empty = page.getByTestId('empty-state');
  await expect(empty).toBeVisible();
  // 検索の案内なのでログインの操作は出さない
  await expect(empty.getByRole('button', { name: 'Googleでログイン' })).toHaveCount(0);
});

test('検索欄の文言は「〇〇で検索」の型でそろっている', async ({ page }) => {
  await mockAnonymous(page);

  await page.goto('/search');

  await expect(
    page.getByRole('searchbox', { name: '銘柄コードまたは銘柄名で検索' }),
  ).toBeVisible();
});

test('資産管理と取引明細の検索欄も「〇〇で検索」の型', async ({ page }) => {
  await mockAuthenticated(page);

  await page.goto('/assetbalance');
  await expect(
    page.getByRole('searchbox', { name: '銘柄・ファンド名・コード・口座で検索' }),
  ).toBeVisible();

  await page.goto('/receipts');
  await expect(
    page.getByRole('searchbox', { name: '銘柄名・コードで検索' }),
  ).toBeVisible();
});

test('検索ボタンは入力が空のあいだ押せず、文字が入ると押せる', async ({ page }) => {
  await mockAnonymous(page);

  await page.goto('/search');

  const button = page.getByRole('button', { name: '銘柄を検索' });
  await expect(button).toBeDisabled();

  await page
    .getByRole('searchbox', { name: '銘柄コードまたは銘柄名で検索' })
    .fill('9905');

  await expect(button).toBeEnabled();
});
