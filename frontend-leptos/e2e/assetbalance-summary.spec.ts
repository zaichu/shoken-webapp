import { expect, test, type Page } from '@playwright/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000010',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const TOYOTA = {
  id: 'asset-balance-1',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '7203',
  security_name: 'トヨタ自動車',
  shares: 100,
  executing_shares: 0,
  average_purchase_price: 2500,
  total_purchase_amount: 250000,
  current_price: 2600,
  daily_change: 50,
  market_value: 260000,
  profit_loss_rate: 4.0,
};

const SONY = {
  id: 'asset-balance-2',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '6758',
  security_name: 'ソニーグループ',
  shares: 50,
  executing_shares: 0,
  average_purchase_price: 3000,
  total_purchase_amount: 150000,
  current_price: 3200,
  daily_change: -20,
  market_value: 160000,
  profit_loss_rate: 6.67,
};

// 行の合計と一致しない値にし、絞り込み中に API の summary が誤用されると検出できるようにする
const API_SUMMARY = {
  total_purchase_amount: 123456789,
  total_market_value: 987654321,
  total_daily_change: 0,
};

const FACETS = {
  securities: [
    { value: '7203', label: 'トヨタ自動車', count: 1 },
    { value: '6758', label: 'ソニーグループ', count: 1 },
  ],
};

function jsonResponse(body: unknown, status = 200) {
  return {
    status,
    contentType: 'application/json',
    body: JSON.stringify(body),
  };
}

async function setupAssetBalanceMocks(page: Page) {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill(jsonResponse(MOCK_USER)),
  );

  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => {
    if (route.request().method() !== 'GET') {
      return route.fallback();
    }
    return route.fulfill(
      jsonResponse({
        data: [TOYOTA, SONY],
        total: 2,
        page: 1,
        per_page: 1000,
        summary: API_SUMMARY,
        facets: FACETS,
      }),
    );
  });

  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(jsonResponse({ items: [] })),
  );
}

test('絞り込み中のKPIはAPIのsummaryではなく表示行の合計になる', async ({ page }) => {
  await setupAssetBalanceMocks(page);

  await page.goto('/assetbalance');

  const kpi = page.getByTestId('portfolio-kpi-strip');

  await expect(kpi.getByText('¥ 123,456,789')).toBeVisible();
  await expect(kpi.getByText('¥ 987,654,321')).toBeVisible();

  await page.getByLabel('銘柄').selectOption('6758');

  await expect(page.getByText('絞り込み中: 1/2件')).toBeVisible();
  await expect(kpi.getByText('¥ 150,000')).toBeVisible();
  await expect(kpi.getByText('¥ 160,000')).toBeVisible();
  await expect(kpi.getByText('¥ 123,456,789')).not.toBeVisible();
  await expect(kpi.getByText('¥ 987,654,321')).not.toBeVisible();

  await page.getByRole('button', { name: '検索条件をクリア' }).click();

  await expect(kpi.getByText('¥ 123,456,789')).toBeVisible();
  await expect(kpi.getByText('¥ 987,654,321')).toBeVisible();
});
