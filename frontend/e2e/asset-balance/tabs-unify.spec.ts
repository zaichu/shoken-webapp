import type { Page } from '@playwright/test';
import { expect, test, json } from '../support/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000010',
  email: 'test@example.com',
  name: 'テストユーザー',
};

// 小数桁がそろっていない価格を入れて、列の中で桁がそろうことを確かめる
const STOCK_ROWS = [
  {
    id: 'asset-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: '9901',
    security_name: '架空自動車',
    account: '特定',
    shares: 100,
    average_purchase_price: 5997.6,
    total_purchase_amount: 599760,
    current_price: 6187.25,
    daily_change: 12,
    price_as_of: '2026-10-20',
    prev_close: 6175.25,
  },
  {
    id: 'asset-2',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: '9902',
    security_name: '架空精密機器',
    account: '一般',
    shares: 200,
    average_purchase_price: 187.36,
    total_purchase_amount: 37472,
    current_price: 162,
    daily_change: -3,
    price_as_of: '2026-10-20',
    prev_close: 165,
  },
];

const FUND_ROWS = [
  {
    id: 'fund-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    fund_name: '架空バランスファンドA号',
    account: '特定',
    dividends_course: '受取型',
    shares: 10000,
    average_purchase_price: 9970.5,
    total_purchase_amount: 9970,
    base_price: 10012.25,
    base_price_day_change: 5,
    base_price_month_change: 120,
    market_value: 10012,
    profit_loss: 42,
    profit_loss_rate: 0.42,
    total_return: 42,
    price_as_of: '2026-10-20',
    price_source: 'api',
    nav_is_latest: true,
    expected_nav_date: null,
  },
  {
    id: 'fund-2',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    fund_name: '架空インデックスファンドB号',
    account: '一般',
    dividends_course: '再投資型',
    shares: 20000,
    average_purchase_price: 9500,
    total_purchase_amount: 19000,
    base_price: 9800,
    base_price_day_change: -30,
    base_price_month_change: 120,
    market_value: 19600,
    profit_loss: 600,
    profit_loss_rate: 3.16,
    total_return: 600,
    price_as_of: '2026-10-20',
    price_source: 'csv',
    nav_is_latest: true,
    expected_nav_date: null,
  },
];

async function setupMocks(page: Page) {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill(json(MOCK_USER)),
  );
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(
      json({
        data: STOCK_ROWS,
        total: STOCK_ROWS.length,
        page: 1,
        per_page: 1000,
      }),
    ),
  );
  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
    route.fulfill(
      json({
        data: FUND_ROWS,
        total: FUND_ROWS.length,
        page: 1,
        per_page: 1000,
        summary: {
          total_market_value: 29612,
          total_purchase_amount: 28970,
          total_daily_change: 0,
          latest_nav_count: FUND_ROWS.length,
          has_missing_daily_change: false,
          nav_status: {
            updated_at: '2026-10-20 18:05',
            fetched_count: FUND_ROWS.length,
            failed_count: 0,
            failure_reason: null,
          },
        },
      }),
    ),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

// 同じ列の値が同じ小数桁を持つこと
function decimalsOf(text: string): number | null {
  const matched = /(\d+)\.(\d+)/.exec(text);
  return matched ? matched[2].length : null;
}

async function columnDecimals(page: Page, columnName: string): Promise<number[]> {
  const table = page.locator('.asset-table').first();
  const header = table.getByRole('columnheader', { name: columnName });
  const index = await header.evaluate((th) =>
    Array.from(th.parentElement?.children ?? []).indexOf(th),
  );
  return table.evaluate(
    (element, columnIndex) =>
      Array.from(element.querySelectorAll('tbody tr')).map((row) => {
        const cell = row.children[columnIndex];
        return cell?.textContent?.trim() ?? '';
      }),
    index,
  );
}

test.describe('資産管理のタブ', () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  test('国内株式と投資信託は同じ用語・同じ検索欄・同じ注記の位置で出す', async ({
    page,
  }) => {
    await setupMocks(page);
    await page.goto('/assetbalance');

    const placeholders: string[] = [];
    const bandLabels: string[][] = [];
    const noteOffsets: number[] = [];

    for (const tab of ['stock', 'fund'] as const) {
      await page.getByTestId(`asset-kind-tab-${tab}`).click();
      const band = page.getByTestId('asset-portfolio-summary');
      await expect(band).toBeVisible();

      // 帯の主要3項目(評価額・損益・前日比)が2タブで同じ
      bandLabels.push(
        await band
          .locator('.summary-band-group-main dt, .summary-band-item-daily dt')
          .allTextContents(),
      );

      // 検索欄の文言は同じ
      placeholders.push(
        await page.getByTestId('asset-search-input-pc').getAttribute('placeholder'),
      );

      // 注記は帯の直下にあり、「計算基準を見る」は注記と同じ行の右端
      const basis = page.getByText('計算基準を見る');
      await expect(basis).toBeVisible();
      const bandBox = await band.boundingBox();
      const basisBox = await basis.boundingBox();
      expect(bandBox).not.toBeNull();
      expect(basisBox).not.toBeNull();
      if (!bandBox || !basisBox) {
        return;
      }
      noteOffsets.push(Math.round(basisBox.y - (bandBox.y + bandBox.height)));
    }

    // タブを跨いで帯の主要項目・検索欄の文言・注記の位置がそろう
    expect(bandLabels[0]).toEqual(bandLabels[1]);
    expect(placeholders[0]).toBe(placeholders[1]);
    expect(noteOffsets[0]).toBe(noteOffsets[1]);
  });

  test('同じ列の金額は同じ小数桁でそろえる', async ({ page }) => {
    await setupMocks(page);
    await page.goto('/assetbalance');

    await page.getByTestId('asset-kind-tab-stock').click();
    for (const column of ['取得単価', '現在値']) {
      const values = await columnDecimals(page, column);
      expect(values.length).toBeGreaterThan(1);
      const decimals = values.map(decimalsOf);
      expect(decimals.every((value) => value !== null)).toBe(true);
      expect(new Set(decimals).size).toBe(1);
    }

    await page.getByTestId('asset-kind-tab-fund').click();
    const values = await columnDecimals(page, '基準価額');
    expect(values.length).toBeGreaterThan(1);
    const decimals = values.map(decimalsOf);
    expect(decimals.every((value) => value !== null)).toBe(true);
    expect(new Set(decimals).size).toBe(1);
  });
});
