import type { Page } from '@playwright/test';
import { expect, test, json, paginated } from '../support/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000011',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const DIVIDENDS = [
  {
    id: '00000000-0000-0000-0000-000000000001',
    settlement_date: '2024-03-01',
    product: '特定口座',
    account: 'SBI証券',
    security_code: '9904',
    security_name: '架空自動車',
    unit_price: 30.0,
    shares: 100,
    dividends_before_tax: 3000,
    taxes: 609,
    net_amount_received: 2391,
    created_at: '2024-03-01T00:00:00Z',
    updated_at: '2024-03-01T00:00:00Z',
  },
  {
    id: '00000000-0000-0000-0000-000000000002',
    settlement_date: '2023-06-01',
    product: 'NISA口座',
    account: '楽天証券',
    security_code: '9910',
    security_name: '架空電信電話',
    unit_price: 120.0,
    shares: 200,
    dividends_before_tax: 5000,
    taxes: 1000,
    net_amount_received: 4000,
    created_at: '2023-06-01T00:00:00Z',
    updated_at: '2023-06-01T00:00:00Z',
  },
];

async function mockSession(page: Page) {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill(json(MOCK_USER)),
  );
}

async function mockReceipts(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DIVIDENDS))),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([]))),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([]))),
  );
}

async function gotoConditions(page: Page) {
  await page.setViewportSize({ width: 390, height: 844 });
  await mockReceipts(page);
  await page.goto('/receipts');
  await expect(page.getByTestId('receipt-card').first()).toBeVisible();
  await page.getByRole('link', { name: '検索条件' }).click();
  await expect(page).toHaveURL('/receipts/conditions');
  // 選択肢は取得完了まで空なので、データが届いてから触れる
  await expect(page.getByRole('button', { name: 'SBI証券', exact: true })).toBeVisible();
}

test('検索条件ページから商品と口座を選んで一覧と件数に効く', async ({ page }) => {
  await gotoConditions(page);

  await page.getByRole('button', { name: 'SBI証券', exact: true }).click();
  await page.getByRole('link', { name: '適用して戻る' }).click();

  await expect(page).toHaveURL('/receipts');
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');
  await expect(
    page.getByTestId('receipt-card-list').getByText('架空自動車').filter({ visible: true }),
  ).toBeVisible();
  await expect(page.getByText('架空電信電話')).toHaveCount(0);
});

test('期間と条件は同時に併用でき、「すべて」で期間だけ解除できる', async ({ page }) => {
  await gotoConditions(page);

  await page.getByRole('button', { name: '年', exact: true }).click();
  await page.getByRole('button', { name: '2024年', exact: true }).click();
  await page.getByRole('button', { name: 'SBI証券', exact: true }).click();
  await page.getByRole('link', { name: '適用して戻る' }).click();

  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');
  await expect(
    page.getByTestId('receipt-card-list').getByText('架空自動車').filter({ visible: true }),
  ).toBeVisible();

  // 適用中の条件チップから個別に解除できる
  const conditions = page.getByRole('group', { name: '適用中の検索条件' });
  await expect(conditions).toContainText('2024');
  await expect(conditions).toContainText('SBI証券');

  await page.getByRole('button', { name: /期間 .* を解除/ }).click();
  // 口座の条件は残るので SBI証券の1件だけが絞り込まれたままになる
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');
});

test('変更せず戻るは下書きを捨てて一覧に戻れる', async ({ page }) => {
  await gotoConditions(page);

  await page.getByRole('button', { name: 'SBI証券', exact: true }).click();
  await page.getByRole('link', { name: '変更せず戻る' }).click();

  await expect(page).toHaveURL('/receipts');
  // SBI証券の選択は適用されないので2件のまま
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('2');
});

test('不明パスは /404 に置き換わる', async ({ page }) => {
  await mockSession(page);

  await page.goto('/no-such-page');

  await expect(page).toHaveURL(/\/404$/);
  await expect(page.getByText('404 - ページが見つかりません')).toBeVisible();
});