import type { Page, Route } from '@playwright/test';
import { expect, test, paginated, mockSession } from '../support/test';
import { csvRow } from '../support/csv-page';
import * as path from 'path';

const ROUTES = {
  dividends: /\/api\/v1\/dividends(?:\?.*)?$/,
  domesticStocks: /\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/,
  mutualfunds: /\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/,
  dividendPreview: /\/api\/v1\/dividend-import-validations$/,
};

const CSV_FIXTURES = path.resolve(process.cwd(), 'e2e/fixtures/csv');

const DIVIDEND_RECORD = [
  {
    id: '00000000-0000-0000-0000-000000000001',
    user_id: '00000000-0000-0000-0000-000000000002',
    settlement_date: '2024-03-01',
    product: '特定口座',
    account: 'SBI証券',
    security_code: '9904',
    security_name: '架空自動車',
    unit_price: '30.0',
    shares: '100',
    dividends_before_tax: '3000',
    taxes: '609',
    net_amount_received: '2391',
    created_at: '2024-03-01T00:00:00Z',
    updated_at: '2024-03-01T00:00:00Z',
  },
];

async function setupAuthMocks(
  page: Page,
  {
    dividends = [],
    domesticStocks = [],
    mutualfunds = [],
  }: {
    dividends?: unknown[];
    domesticStocks?: unknown[];
    mutualfunds?: unknown[];
  } = {},
) {
  await mockSession(page);
  await page.route(ROUTES.dividends, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated(dividends)),
    }),
  );
  await page.route(ROUTES.domesticStocks, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated(domesticStocks)),
    }),
  );
  await page.route(ROUTES.mutualfunds, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated(mutualfunds)),
    }),
  );
}

// 認証確認と一覧取得が終わるまで workspace 内にローディングの section(role=status)が残る。
// Spinner の svg も role=status を持つため、文言で絞って strict 違反を避ける
function receiptLoadingSection(page: Page) {
  return page
    .getByTestId('receipts-workspace')
    .getByRole('status')
    .filter({ hasText: /読み込んでいます|確認しています/ });
}

test('取引明細ページの初期表示でタブが3つある', async ({ page }) => {
  await setupAuthMocks(page);

  await page.goto('/receipts');

  await expect(page.getByRole('tab')).toHaveCount(3);
});







test('配当金データが1件あるとき行が表示される', async ({ page }) => {
  await setupAuthMocks(page, { dividends: DIVIDEND_RECORD });

  await page.goto('/receipts');
  await expect(receiptLoadingSection(page)).toBeHidden();

  await expect(
    page
      .getByTestId('receipts-workspace')
      .getByRole('cell', { name: '架空自動車' }),
  ).toBeVisible();
});
