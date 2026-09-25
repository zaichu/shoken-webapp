import { expect, test, type Page } from '@playwright/test';

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
};

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

// dev サーバの再ビルドや wasm ロードと遷移が衝突するとアプリが一度も描画されないことがある。
// タブが出る = マウントと認証確認が終わった印なので、出なければ1回だけ遷移し直す
async function gotoReceipts(page: Page) {
  const tabs = page.getByRole('tab');
  for (let attempt = 0; ; attempt++) {
    await page.goto('/receipts');
    try {
      await expect(tabs).toHaveCount(3);
      return;
    } catch (error) {
      if (attempt === 1) {
        throw error;
      }
    }
  }
}

// 認証確認と一覧取得が終わるまで workspace 内にローディング(role=status)が残る
function receiptWorkspace(page: Page) {
  return page.getByTestId('receipts-workspace');
}

test('取引明細ページの初期表示でタブが3つある', async ({ page }) => {
  await setupAuthMocks(page);

  await gotoReceipts(page);

  await expect(page.getByRole('tab')).toHaveCount(3);
});

test('配当金タブにデータがないとき EmptyState が表示される', async ({ page }) => {
  await setupAuthMocks(page, { dividends: [] });

  await gotoReceipts(page);
  const workspace = receiptWorkspace(page);
  await expect(workspace.getByRole('status')).toBeHidden();

  await expect(workspace.getByText('データがありません')).toBeVisible();
});

test('配当金データが1件あるとき行が表示される', async ({ page }) => {
  await setupAuthMocks(page, { dividends: DIVIDEND_RECORD });

  await gotoReceipts(page);
  const workspace = receiptWorkspace(page);
  await expect(workspace.getByRole('status')).toBeHidden();

  await expect(
    workspace.getByRole('cell', { name: 'トヨタ自動車' }),
  ).toBeVisible();
});
