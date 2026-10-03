import type { Page, Route } from '@playwright/test';
import { expect, test } from '../support/test';
import * as path from 'path';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const ROUTES = {
  authMe: /\/api\/v1\/session$/,
  dividends: /\/api\/v1\/dividends(?:\?.*)?$/,
  domesticStocks: /\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/,
  mutualfunds: /\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/,
  dividendPreview: /\/api\/v1\/dividend-import-validations$/,
};

const CSV_FIXTURES = path.resolve(process.cwd(), 'e2e/__fixtures__/csv');

const DIVIDEND_RECORD = [
  {
    id: '00000000-0000-0000-0000-000000000001',
    user_id: '00000000-0000-0000-0000-000000000002',
    settlement_date: '2024-03-01',
    product: '特定口座',
    account: 'SBI証券',
    security_code: '7203',
    security_name: 'トヨタ自動車',
    unit_price: '30.0',
    shares: '100',
    dividends_before_tax: '3000',
    taxes: '609',
    net_amount_received: '2391',
    created_at: '2024-03-01T00:00:00Z',
    updated_at: '2024-03-01T00:00:00Z',
  },
];

function paginatedResponse(data: unknown[]) {
  return {
    data,
    total: data.length,
    page: 1,
    per_page: data.length,
  };
}

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
  await page.route(ROUTES.authMe, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_USER),
    }),
  );
  await page.route(ROUTES.dividends, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginatedResponse(dividends)),
    }),
  );
  await page.route(ROUTES.domesticStocks, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginatedResponse(domesticStocks)),
    }),
  );
  await page.route(ROUTES.mutualfunds, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginatedResponse(mutualfunds)),
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

  await expect(
    page.getByTestId('receipts-workspace').getByText('データがありません'),
  ).toBeVisible();
  // 絞り込み対象がないので検索オプションは出さない
  await expect(page.getByTestId('search-card')).toHaveCount(0);
  // EmptyState の CTA からファイル選択を開ける
  const cta = page.getByRole('button', { name: 'CSVを取り込む' });
  await expect(cta).toBeEnabled();
  const chooserPromise = page.waitForEvent('filechooser');
  await cta.click();
  await chooserPromise;
});

test('別タブの一覧取得中は空状態の CSV 取り込み CTA が無効になる', async ({ page }) => {
  await setupAuthMocks(page, { dividends: [] });
  // 国内株式タブの応答を保留して any_tab_fetching を維持する
  const pending: Route[] = [];
  await page.unroute(ROUTES.domesticStocks);
  await page.route(ROUTES.domesticStocks, (route) => {
    pending.push(route);
  });

  await page.goto('/receipts');

  const cta = page.getByRole('button', { name: 'CSVを取り込む' });
  await expect(cta).toBeVisible();
  await expect(cta).toBeDisabled();

  for (const route of pending) {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginatedResponse([])),
    });
  }
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
            security_code: '8591',
            security_name: 'オリックス',
            unit_price: 93.76,
            shares: 200,
            dividends_before_tax: 18752,
            taxes: 3808,
            net_amount_received: 14944,
          },
        ],
      }),
    }),
  );

  await page.goto('/receipts');
  await expect(receiptLoadingSection(page)).toBeHidden();

  const cta = page.getByRole('button', { name: 'CSVを取り込む' });
  await expect(cta).toBeEnabled();
  const chooserPromise = page.waitForEvent('filechooser');
  await cta.click();
  const chooser = await chooserPromise;
  await chooser.setFiles(path.join(CSV_FIXTURES, 'dividend-base.csv'));

  // 取り込み結果はパネルに出る。狭い帯ではドロワーを開けて確認する
  const panelToggle = page.getByTestId('receipt-utility-toggle');
  await panelToggle.click();
  await expect(panelToggle).toHaveAttribute('aria-expanded', 'true');
  await expect(page.getByTestId('csv-preview-notice')).toBeVisible();
  await expect(page.getByRole('button', { name: /追加で保存/ })).toBeVisible();
});

test('配当金データが1件あるとき行が表示される', async ({ page }) => {
  await setupAuthMocks(page, { dividends: DIVIDEND_RECORD });

  await page.goto('/receipts');
  await expect(receiptLoadingSection(page)).toBeHidden();

  await expect(
    page
      .getByTestId('receipts-workspace')
      .getByRole('cell', { name: 'トヨタ自動車' }),
  ).toBeVisible();
});
