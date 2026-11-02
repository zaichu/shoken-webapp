import type { Page } from '@playwright/test';
import { expect, test, json, mockSession } from '../support/test';

const FUND_ROWS = [
  {
    id: 'fund-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    fund_name: '架空バランスファンドA号',
    account: '特定',
    dividends_course: '受取型',
    shares: 10000,
    average_purchase_price: 10000,
    total_purchase_amount: 100000,
    base_price: 11000,
    base_price_day_change: 50,
    base_price_month_change: null,
    market_value: 110000,
    profit_loss: 10000,
    profit_loss_rate: 10.0,
    total_return: 10000,
    price_as_of: '2026-01-05',
    price_source: 'api',
    nav_is_latest: true,
    expected_nav_date: null,
  },
];

const ROUTES = {
  assetBalances: /\/api\/v1\/asset-balances(?:\?.*)?$/,
  fundBalances: /\/api\/v1\/fund-balances(?:\?.*)?$/,
  dividendPerShareBatch: /\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/,
};

async function setupMocks(
  page: Page,
  navStatus: Record<string, unknown>,
  rows = FUND_ROWS,
) {
  await mockSession(page);
  await page.route(ROUTES.dividendPerShareBatch, (route) =>
    route.fulfill(json({ items: [] })),
  );
  await page.route(ROUTES.assetBalances, (route) =>
    route.fulfill(json({ data: [], total: 0, page: 1, per_page: 0 })),
  );
  await page.route(ROUTES.fundBalances, (route) => {
    const data = rows.map((row) => ({ ...row }));
    return route.fulfill(
      json({
        data,
        total: data.length,
        page: 1,
        per_page: data.length,
        summary: {
          total_market_value: 110000,
          total_purchase_amount: 100000,
          total_daily_change: 50,
          latest_nav_count: 1,
          pending_nav_count: 0,
          nav_status: navStatus,
        },
      }),
    );
  });
}

async function openFundTab(page: Page) {
  await page.goto('/assetbalance');
  await page.getByTestId('asset-kind-tab-fund').click();
  await expect(page.getByTestId('asset-fund-table')).toBeVisible();
}

test('帯の下の注記は出さず、基準日の時点だけを帯のラベル横に出す', async ({
  page,
}) => {
  await setupMocks(page, {
    updated_at: '2026-10-20 23:02',
    fetched_count: 2,
    failed_count: 0,
    fund_count: 2,
    auto_update_target_count: 2,
    failure_reason: null,
  });
  await openFundTab(page);

  // 注記行は出さない(詳細は CSV 管理に任せる)
  await expect(page.locator('.page-skeleton-note')).toHaveCount(0);
  // 『計算基準を見る』は全ページから削除
  await expect(page.getByText('計算基準を見る')).toHaveCount(0);
  // 時点だけは帯のラベル横に小さく残す(数字を嘘にしない)
  const band = page.getByTestId('asset-portfolio-summary');
  await expect(band).toContainText('1/5 時点');
});

test('更新待ちのファンドがあるときは帯に一部更新待ちを出す', async ({
  page,
}) => {
  const stale = FUND_ROWS.map((row) => ({
    ...row,
    nav_is_latest: false,
    expected_nav_date: '2026-01-06',
  }));
  await setupMocks(
    page,
    {
      updated_at: '2026-10-20 23:02',
      fetched_count: 0,
      failed_count: 2,
      fund_count: 2,
      auto_update_target_count: 2,
      failure_reason: 'HTTP 403',
    },
    stale,
  );
  await openFundTab(page);

  // 失敗件数や理由は帯の下に出さない
  await expect(page.locator('.page-skeleton-note')).toHaveCount(0);
  await expect(page.getByText('計算基準を見る')).toHaveCount(0);
  // 時点と更新待ちだけ出す
  const band = page.getByTestId('asset-portfolio-summary');
  await expect(band).toContainText('1/5 時点');
  await expect(band).toContainText('一部更新待ち');
});
