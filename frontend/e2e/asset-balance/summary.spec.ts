import type { Page } from '@playwright/test';
import { expect, test, json } from '../support/test';

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
};

// 行の合計と一致しない値にし、絞り込み中に API の summary が誤用されると検出できるようにする
const API_SUMMARY = {
  total_purchase_amount: 123456789,
  total_market_value: 987654321,
  total_daily_change: 0,
  priced_purchase_amount: 123456789,
  priced_market_value: 987654321,
};

async function setupAssetBalanceMocks(page: Page) {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill(json(MOCK_USER)),
  );

  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => {
    if (route.request().method() !== 'GET') {
      return route.abort();
    }
    return route.fulfill(
      json({
        data: [TOYOTA, SONY],
        total: 2,
        page: 1,
        per_page: 1000,
        summary: API_SUMMARY,
      }),
    );
  });

  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

test('絞り込み中もKPIはAPIのsummaryではなく全保有分の合計のまま', async ({
  page,
}) => {
  await setupAssetBalanceMocks(page);

  await page.goto('/assetbalance');

  const kpi = page.getByTestId('portfolio-kpi-strip');

  // 取得済みの行から評価額合計・含み損益を計算する(API summary は使わない)
  await expect(kpi).toContainText('¥400,000');
  await expect(kpi).toContainText('+¥20,000');
  await expect(kpi).toContainText('+5.0%');
  await expect(kpi).not.toContainText('¥123,456,789');
  await expect(kpi).not.toContainText('¥987,654,321');

  await page.getByTestId('asset-search-input').fill('ソニー');

  await expect(page.getByTestId('asset-search-count')).toContainText('1 / 2');
  // 絞り込み中も合計は全保有分のまま
  await expect(kpi).toContainText('¥400,000');
  await expect(kpi).toContainText('+¥20,000');

  await page.getByTestId('asset-search-clear').click();

  await expect(kpi).toContainText('¥400,000');
});
