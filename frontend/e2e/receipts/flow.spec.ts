import type { Page, Route } from '@playwright/test';
import { expect, test, paginated, mockSession } from '../support/test';
import { openCsvImport } from '../support/csv-page';
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

test('配当金タブにデータがないとき EmptyState が表示される', async ({ page }) => {
  await setupAuthMocks(page, { dividends: [] });

  await page.goto('/receipts');
  await expect(receiptLoadingSection(page)).toBeHidden();

  await expect(page.getByTestId('receipts-workspace').getByText('データがありません')).toBeVisible();
  // EmptyState の CTA から CSV 管理ページに進める
  const cta = page.getByRole('link', { name: 'CSVを取り込む' });
  await expect(cta).toBeVisible();
  await cta.click();
  await expect(page).toHaveURL('/csv');
  // ダイアログは初期表示では開かず、⋯から開く
  await expect(page.getByTestId('csv-import-dialog')).toBeHidden();
  await openCsvImport(page, 'dividend');
  await expect(page.getByTestId('csv-file-input')).toBeAttached();
});

test('別タブの一覧取得中は CSV ページのファイル選択が無効になる', async ({ page }) => {
  await setupAuthMocks(page, { dividends: [] });
  // 国内株式タブの応答を保留して any_tab_fetching を維持する
  const pending: Route[] = [];
  await page.unroute(ROUTES.domesticStocks);
  await page.route(ROUTES.domesticStocks, (route) => {
    pending.push(route);
  });

  await page.goto('/csv?kind=dividend');
  await expect(page.getByTestId('csv-import-dialog')).toBeHidden();
  await openCsvImport(page, 'dividend');

  const fileInput = page.getByTestId('csv-file-input');
  await expect(fileInput).toBeAttached();
  await expect(fileInput).toBeDisabled();

  for (const route of pending) {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated([])),
    });
  }
  await expect(fileInput).toBeEnabled();
});

test('スマホ幅で空状態 CTA からプレビューして保存ボタンまで見える', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await setupAuthMocks(page, { dividends: [] });
  await page.route(ROUTES.dividendPreview, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        total_rows: 1,
        valid_rows: 1,
        errors: [],
        rows: [
          {
            settlement_date: '2025-12-09',
            product: '国内株式',
            account: '特定・一般',
            security_code: '9908',
            security_name: '架空リース',
            unit_price: 90.00,
            shares: 200,
            dividends_before_tax: 18000,
            taxes: 3656,
            net_amount_received: 14344,
          },
        ],
      }),
    }),
  );

  await page.goto('/receipts');
  await expect(receiptLoadingSection(page)).toBeHidden();

  const cta = page.getByRole('link', { name: 'CSVを取り込む' });
  await expect(cta).toBeVisible();
  await cta.click();
  await expect(page).toHaveURL('/csv');

  // 取り込み結果は CSV 管理ページに出る。ダイアログは⋯から開く
  await expect(page.getByTestId('csv-import-dialog')).toBeHidden();
  await openCsvImport(page, 'dividend');
  const fileInput = page.getByTestId('csv-file-input');
  await expect(fileInput).toBeEnabled();
  await fileInput.setInputFiles(path.join(CSV_FIXTURES, 'dividend-base.csv'));
  await expect(page.getByTestId('csv-action-bar')).toContainText(
    '1件を追加で保存できます',
  );
  await expect(page.getByRole('button', { name: /保存する/ })).toBeVisible();
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
