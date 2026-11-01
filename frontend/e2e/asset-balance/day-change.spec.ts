import type { Page } from '@playwright/test';
import { expect, test, json, mockSession } from '../support/test';

const HAS_PREV_CLOSE = {
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
  price_as_of: '2026-01-01',
  prev_close: 2550,
};

const NO_PREV_CLOSE = {
  id: 'asset-balance-2',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '9914',
  security_name: '架空エレクトロニクス',
  shares: 50,
  average_purchase_price: 3000,
  total_purchase_amount: 150000,
  current_price: 3200,
  daily_change: 0,
  price_as_of: '2026-01-01',
  prev_close: null,
};

async function setupMocks(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => {
    if (route.request().method() !== 'GET') {
      return route.abort();
    }
    return route.fulfill(
      json({
        data: [HAS_PREV_CLOSE, NO_PREV_CLOSE],
        total: 2,
        page: 1,
        per_page: 1000,
      }),
    );
  });
  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json({ data: [], total: 0, page: 1, per_page: 0 })),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

test('国内株式テーブルに1株あたりの前日比が出て、前日終値なし銘柄は「—」で合計から除かれる', async ({
  page,
}) => {
  await setupMocks(page);
  await page.goto('/assetbalance');
  await page.getByTestId('asset-kind-tab-stock').click();

  const table = page.getByTestId('asset-holdings-table');
  await expect(table).toBeVisible();

  // 前日比は独立した列
  await expect(
    table.getByRole('columnheader', { name: '前日比' }),
  ).toBeVisible();

  // 前日終値がある銘柄は円マーク付きの1株あたり前日比を表示する
  const priced = table.getByRole('row', { name: /架空自動車/ });
  await expect(priced).toContainText('+¥50');

  // 前日終値がない銘柄は「—」を表示する
  const unpriced = table.getByRole('row', { name: /架空エレクトロニクス/ });
  await expect(unpriced).toContainText('—');

  // 集計の前日比は前日終値あり銘柄だけの合計(50×100株=5,000)
  const kpi = page.getByTestId('portfolio-kpi-grid');
  await expect(kpi).toContainText('前日比');
  await expect(kpi).toContainText('+¥5,000');

  // 件数はタブと件数行に出す(タイトル横には出さない)。基準日は帯のラベル横に小さく出す
  await expect(page.getByTestId('page-title-count')).toHaveCount(0);
  await expect(page.getByTestId('asset-kind-tab-stock')).toContainText('2');
  await expect(page.getByTestId('asset-search-count')).toContainText(
    '2件 / 2件',
  );
  await expect(page.locator('.page-skeleton-note')).toHaveCount(0);
  await expect(page.getByText('計算基準を見る')).toHaveCount(0);
  const bandText = page.getByTestId('asset-portfolio-summary');
  await expect(bandText).toContainText('01/01 時点');

  // 「すべて」タブもタブと件数行に件数だけ。時点は帯のラベル横に残る
  await page.getByTestId('asset-kind-tab-all').click();
  await expect(page.getByTestId('asset-kind-tab-all')).toContainText('2');
  await expect(page.getByTestId('asset-search-count')).toContainText(
    '2件 / 2件',
  );
  await expect(page.getByTestId('asset-portfolio-summary')).toContainText(
    '01/01 時点',
  );
});

test('基準日が他の行と違う行だけ、前日比の横に日付を出す', async ({ page }) => {
  const DIFFERENT = {
    ...HAS_PREV_CLOSE,
    id: 'asset-balance-3',
    security_code: '9911',
    security_name: '架空精密機器',
    price_as_of: '2025-12-30',
  };
  await setupMocks(page);
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(
      json({
        data: [HAS_PREV_CLOSE, NO_PREV_CLOSE, DIFFERENT],
        total: 3,
        page: 1,
        per_page: 1000,
      }),
    ),
  );
  await page.goto('/assetbalance');
  await page.getByTestId('asset-kind-tab-stock').click();

  const table = page.getByTestId('asset-holdings-table');
  const different = table.getByRole('row', { name: /架空精密機器/ });
  await expect(different).toContainText('12/30');

  // 多数派と同じ日付の行は行内に日付を出さない
  const same = table.getByRole('row', { name: /架空自動車/ });
  await expect(same).not.toContainText('01/01');

  // タブと件数行は件数だけ。時点は帯のラベル横に最も古い日だけ出す
  await expect(page.getByTestId('asset-kind-tab-stock')).toContainText('3');
  await expect(page.getByTestId('asset-search-count')).not.toContainText('時点');
  await expect(page.getByTestId('asset-portfolio-summary')).toContainText(
    '12/30 時点',
  );
  await expect(page.getByTestId('asset-portfolio-summary')).not.toContainText(
    '01/01 時点',
  );
});
