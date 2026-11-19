import type { Page } from '@playwright/test';
import { expect, test, json, mockSession } from '../support/test';

const THIS_YEAR = new Date().getFullYear();

const SYN_AUTO = {
  id: 'asset-balance-1',
  created_at: `${THIS_YEAR}-01-01T00:00:00Z`,
  updated_at: `${THIS_YEAR}-01-01T00:00:00Z`,
  security_code: '9904',
  security_name: '架空自動車',
  shares: 100,
  average_purchase_price: 2500,
  total_purchase_amount: 250000,
  current_price: 2600,
  daily_change: 50,
};

const SYN_ELECTRONICS = {
  id: 'asset-balance-2',
  created_at: `${THIS_YEAR}-01-01T00:00:00Z`,
  updated_at: `${THIS_YEAR}-01-01T00:00:00Z`,
  security_code: '9914',
  security_name: '架空エレクトロニクス',
  shares: 50,
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
  await mockSession(page);

  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => {
    if (route.request().method() !== 'GET') {
      return route.abort();
    }
    return route.fulfill(
      json({
        data: [SYN_AUTO, SYN_ELECTRONICS],
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

  const kpi = page.getByTestId('portfolio-kpi-grid');

  // 取得済みの行から評価額合計・含み損益を計算する(API summary は使わない)
  await expect(kpi).toContainText('¥420,000');
  await expect(kpi).toContainText('+¥20,000');
  await expect(kpi).toContainText('+5.0%');
  await expect(kpi).not.toContainText('¥123,456,789');
  await expect(kpi).not.toContainText('¥987,654,321');

  await page.getByTestId('asset-search-input-pc').fill('エレクトロ');

  await expect(page.getByTestId('asset-search-count')).toContainText('1件 / 2件');
  // 絞り込み中も合計は全保有分のまま
  await expect(kpi).toContainText('¥420,000');
  await expect(kpi).toContainText('+¥20,000');

  await page.getByTestId('asset-search-clear').click();

  await expect(kpi).toContainText('¥420,000');
});

test('取込が株価より古いときは帯に印と読み上げ文を出し、かっこ書きは出さない', async ({
  page,
}) => {
  await mockSession(page);
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => {
    if (route.request().method() !== 'GET') {
      return route.abort();
    }
    return route.fulfill(
      json({
        // 価格 1/10 に対して取込 1/1 のため、取込が古い
        data: [{ ...SYN_AUTO, price_as_of: `${THIS_YEAR}-01-10` }],
        total: 1,
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

  await page.goto('/assetbalance');
  await page.getByTestId('asset-kind-tab-stock').click();

  // 取込が古いときは PC の右上に出す。集計帯の数字はほかの列とそろったままにする
  const header = page.locator('.page-frame-head');
  await expect(header.getByText('取り込み 1/1').first()).toBeVisible();
  await expect(
    header
      .getByText('取り込み 1/1')
      .first()
      .locator('svg[aria-hidden="true"]'),
  ).toBeVisible();
  // 理由は画面外の文字で読み上げに残す
  await expect(
    header.getByText('株価の日付より古い', { exact: true }),
  ).toBeAttached();

  const band = page.getByTestId('asset-portfolio-summary');
  // 画面にはかっこ書きを出さない
  await expect(band).not.toContainText('（株価');
  // PC の集計帯には出さない
  await expect(band.locator('.summary-band-pc')).not.toContainText('取り込み');
  // スマホは帯の2行目に出す
  await expect(band.locator('.summary-band-rows')).toContainText('取り込み 1/1');
});

test('取込が古くないときは帯に取込の行を出さない', async ({ page }) => {
  await mockSession(page);
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => {
    if (route.request().method() !== 'GET') {
      return route.abort();
    }
    return route.fulfill(
      json({
        // 価格と取込が同じ日のため、取込は古くない
        data: [
          {
            ...SYN_AUTO,
            price_as_of: `${THIS_YEAR}-01-10`,
            updated_at: `${THIS_YEAR}-01-10T00:00:00Z`,
          },
        ],
        total: 1,
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

  await page.goto('/assetbalance');
  await page.getByTestId('asset-kind-tab-stock').click();

  const band = page.getByTestId('asset-portfolio-summary');
  await expect(band).toContainText('1/10 時点');
  await expect(band).not.toContainText('取り込み');
  await expect(page.locator('.page-frame-head')).not.toContainText('取り込み');
});
