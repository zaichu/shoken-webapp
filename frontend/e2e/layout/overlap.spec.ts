import type { Page } from '@playwright/test';
import { expect, test, json, paginated, mockSession } from '../support/test';
import { expectFooterBelowContent } from '../support/layout';

test.use({ viewport: { width: 390, height: 844 } });

const SYN_GAME = {
  id: 'asset-balance-1',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '9905',
  security_name: '架空ゲーム株式会社',
  shares: 1000,
  average_purchase_price: 5000,
  total_purchase_amount: 5000000,
  current_price: 8458,
  daily_change: 30,
};

// 画面より高い内容にするため30件超の保有・複数月の配当を用意する
const BALANCES = Array.from({ length: 32 }, (_, i) => ({
  id: `asset-balance-${i + 1}`,
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: String(1300 + i),
  security_name: `テストホールディングス株式会社${i + 1}号長期運用ファンド`,
  shares: 100 + i * 37,
  average_purchase_price: 1000 + i * 123.45,
  total_purchase_amount: (100 + i * 37) * (1000 + i * 123.45),
  current_price: 1100 + i * 111.11,
  daily_change: i % 3 === 0 ? -i : i,
}));

const DIVIDENDS = Array.from({ length: 30 }, (_, i) => ({
  id: `00000000-0000-0000-0000-${String(i + 1).padStart(12, '0')}`,
  settlement_date: i < 10 ? '2026-09-15' : i < 20 ? '2026-06-15' : '2026-03-15',
  product: '特定口座',
  account: i % 2 === 0 ? 'SBI証券' : '楽天証券',
  security_code: String(1300 + i),
  security_name: `テスト銘柄${i + 1}`,
  unit_price: 30.0,
  shares: 100 + i,
  dividends_before_tax: 3000 + i * 100,
  taxes: 609 + i * 20,
  net_amount_received: 2391 + i * 80,
  created_at: '2026-03-01T00:00:00Z',
  updated_at: '2026-03-01T00:00:00Z',
}));

async function mockBase(page: Page) {
  await page.route(/\/api\/v1\//, (route) => route.fulfill(json(paginated([]))));
  await mockSession(page);
}

async function mockAssetBalances(page: Page, rows: unknown[]) {
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(rows, { per_page: 1000 }))),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

async function mockReceipts(page: Page) {
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DIVIDENDS))),
  );
}

test('保有銘柄詳細: 最下端でも本文がフッターに重ならない', async ({ page }) => {
  await mockBase(page);
  await mockAssetBalances(page, [SYN_GAME]);
  await page.goto('/assetbalance/holdings/asset-balance-1');
  await expect(page.getByRole('heading', { name: '架空ゲーム株式会社' })).toBeVisible();

  // 前提: 本文が画面より高いこと(低いと検査が空振りする)
  const tall = await page.evaluate(() => {
    const frame = document.querySelector<HTMLElement>('.page-frame')!;
    const scroller = document.querySelector<HTMLElement>('.app-scroll')!;
    return frame.scrollHeight > scroller.clientHeight;
  });
  expect(tall, 'モックの本文が画面より高くない').toBe(true);

  await expectFooterBelowContent(page);
});

test('構成比: 最下端でも本文がフッターに重ならない', async ({ page }) => {
  await mockBase(page);
  await mockAssetBalances(page, BALANCES);
  await page.goto('/assetbalance/composition');
  await expect(page.locator('.page-frame')).toBeVisible();
  await expectFooterBelowContent(page);
});

test('資産管理: 最下端でも本文がフッターに重ならない', async ({ page }) => {
  await mockBase(page);
  await mockAssetBalances(page, BALANCES);
  await page.goto('/assetbalance');
  await expect(page.getByTestId('asset-compact-row').first()).toBeVisible();
  await expectFooterBelowContent(page);
});

test('取引明細: 最下端でも本文がフッターに重ならない', async ({ page }) => {
  await mockBase(page);
  await mockReceipts(page);
  await page.goto('/receipts');
  await expect(page.getByTestId('receipt-card').first()).toBeVisible();
  await expectFooterBelowContent(page);
});

test('CSV管理: 最下端でも本文がフッターに重ならない', async ({ page }) => {
  await mockBase(page);
  await mockReceipts(page);
  await page.goto('/csv?kind=dividend');
  await expect(page.getByTestId('csv-import-dialog')).toBeHidden();
  await expect(page.getByRole('row', { name: /取引明細/ }).first()).toBeVisible();
  await expectFooterBelowContent(page);
});

test('銘柄検索: 最下端でも本文がフッターに重ならない', async ({ page }) => {
  await mockBase(page);
  await page.route(/\/api\/v1\/stocks(?:\?.*)?$/, (route) =>
    route.fulfill(
      json({
        code: '9905',
        name: '架空ゲーム株式会社',
        date: '2026-10-16',
        market_category: 'プライム',
        industry_category_33: '情報・通信業',
      }),
    ),
  );
  await page.goto('/search?code=9905');
  await expect(page.getByText('架空ゲーム株式会社')).toBeVisible();
  await expectFooterBelowContent(page);
});

test('集計帯はスクロール中も明細行より手前に描かれる', async ({ page }) => {
  await mockBase(page);
  await mockReceipts(page);
  await page.goto('/receipts');
  const band = page.locator('.receipts-sticky-top');
  await expect(band).toBeVisible();

  const max = await page.evaluate(() => {
    const ws = document.querySelector<HTMLElement>('[data-testid="receipts-workspace"]')!;
    return ws.scrollHeight - ws.clientHeight;
  });
  expect(max, '内側スクロールが発生していない').toBeGreaterThan(0);

  for (const y of [0, Math.floor(max / 3), Math.floor(max / 2), max]) {
    const res = await page.evaluate((scrollY) => {
      const ws = document.querySelector<HTMLElement>('[data-testid="receipts-workspace"]')!;
      ws.scrollTop = scrollY;
      const band = document.querySelector<HTMLElement>('.receipts-sticky-top')!;
      const r = band.getBoundingClientRect();
      const cx = r.left + r.width / 2;
      const cy = r.top + Math.min(r.height / 2, r.height - 2);
      const top = document.elementsFromPoint(cx, cy)[0];
      return {
        top: top ? top.tagName + '.' + String(top.className).slice(0, 80) : 'none',
        inside: top ? band.contains(top) : false,
      };
    }, y);
    expect(
      res.inside,
      `スクロール位置 ${y} で帯の最前面が帯の外にある: ${res.top}`,
    ).toBe(true);
  }
});
