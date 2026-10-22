import type { BrowserContext, Page } from '@playwright/test';
import { expect, test, json, paginated } from '../support/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000010',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const STOCKS = Array.from({ length: 30 }, (_, i) => ({
  id: `asset-balance-${i}`,
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: `99${String(10 + i).padStart(2, '0')}`,
  security_name: `三菱商事株式会社${i + 1}`,
  shares: 100,
  average_purchase_price: 96166,
  total_purchase_amount: 9616666,
  current_price: 125743,
  daily_change: -41152,
  market_value: 12574300,
  profit_loss_rate: 30.8,
  account: '特定',
  price_as_of: '2026-10-21',
}));

const FUNDS = Array.from({ length: 30 }, (_, i) => ({
  id: `fund-balance-${i}`,
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  fund_name: '架空インデックス全世界株式ファンド（オール・ワールド）',
  dividends_course: null,
  account: '特定',
  shares: 10000,
  average_purchase_price: 18000,
  total_purchase_amount: 18000000,
  base_price: 21000,
  base_price_day_change: null,
  base_price_month_change: null,
  market_value: 21000000,
  profit_loss: 3000000,
  profit_loss_rate: 16.6,
  total_return: null,
  price_as_of: '2026-10-21',
  price_source: 'api',
  nav_is_latest: true,
  expected_nav_date: null,
}));

const DOMESTIC = Array.from({ length: 30 }, (_, i) => ({
  id: `domestic-${i}`,
  trade_date: `2026-09-${String((i % 28) + 1).padStart(2, '0')}`,
  settlement_date: `2026-09-${String((i % 28) + 1).padStart(2, '0')}`,
  security_code: `99${String(10 + i).padStart(2, '0')}`,
  security_name: `三菱商事株式会社${i + 1}`,
  account: '特定',
  shares: 1000,
  asked_price: 125743,
  proceeds: 125743000,
  purchase_price: 96166,
  realized_profit_and_loss: 29577000,
  taxes: 7432000,
  realized_profit_and_loss_after_tax: 22145000,
  created_at: '2026-05-10T00:00:00Z',
  updated_at: '2026-05-10T00:00:00Z',
}));

const MUTUALFUNDS = Array.from({ length: 30 }, (_, i) => ({
  id: `mutualfund-${i}`,
  trade_date: `2026-09-${String((i % 28) + 1).padStart(2, '0')}`,
  settlement_date: `2026-09-${String((i % 28) + 1).padStart(2, '0')}`,
  fund_name: '架空インデックス全世界株式ファンド（オール・ワールド）',
  account: '特定',
  shares: 100000,
  exchange_rate: 1,
  cancellation_unit_price_yen: 21000,
  cancellation_amount_yen: 210000000,
  average_acquisition_price_yen: 18000,
  dividends: '0',
  realized_profit_and_loss: 29577000,
  taxes: 7432000,
  realized_profit_and_loss_after_tax: 22145000,
  created_at: '2026-04-02T00:00:00Z',
  updated_at: '2026-04-02T00:00:00Z',
}));

const DIVIDENDS = Array.from({ length: 30 }, (_, i) => ({
  id: `dividend-${i}`,
  user_id: MOCK_USER.id,
  settlement_date: `2026-09-${String((i % 28) + 1).padStart(2, '0')}`,
  product: '国内株式',
  account: '特定',
  security_code: `99${String(10 + i).padStart(2, '0')}`,
  security_name: `三菱商事株式会社${i + 1}`,
  unit_price: 1257,
  shares: 1000,
  dividends_before_tax: 12574300,
  taxes: 3143575,
  net_amount_received: 9430725,
  created_at: '2026-03-01T00:00:00Z',
  updated_at: '2026-03-01T00:00:00Z',
}));

async function mockApi(context: BrowserContext) {
  await context.route(/\/api\/v1\//, (route) => route.fulfill(json(paginated([]))));
  await context.route(/\/api\/v1\/session$/, (route) => route.fulfill(json(MOCK_USER)));
  await context.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(STOCKS))),
  );
  await context.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(FUNDS))),
  );
  await context.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DIVIDENDS))),
  );
  await context.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DOMESTIC))),
  );
  await context.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(MUTUALFUNDS))),
  );
  await context.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(
      json({
        items: STOCKS.map((row) => ({
          security_code: row.security_code,
          dividend_per_share: 5000,
          status: 'ok',
          is_stale: false,
        })),
      }),
    ),
  );
}

type BandItem = {
  label: string;
  labelTop: number;
  valueBottom: number;
  labelColor: string;
  labelFontSize: string;
  valueFontSizes: string[];
};

// 帯の項目ごとのラベル位置・数字の下端・見た目を測る
async function bandItems(page: Page, selector: string): Promise<BandItem[]> {
  return page.evaluate((sel) => {
    const band = document.querySelector(sel);
    if (!band) {
      throw new Error(`帯が見つかりません: ${sel}`);
    }
    return [...band.querySelectorAll('.summary-item')].map((item) => {
      const label = item.querySelector('dt');
      const value = item.querySelector('dd');
      if (!label || !value) {
        throw new Error('ラベルか数字がない項目があります');
      }
      const labelRect = label.getBoundingClientRect();
      const valueRect = value.getBoundingClientRect();
      const labelStyle = getComputedStyle(label);
      const valueStyle = getComputedStyle(value);
      // 補足(率など)も同じ行の数字なので、まとめて大きさを見る
      const valueFontSizes = [
        valueStyle.fontSize,
        ...[...value.querySelectorAll('*')].map(
          (child) => getComputedStyle(child).fontSize,
        ),
      ];
      return {
        label: (label.textContent ?? '').trim(),
        labelTop: labelRect.top,
        valueBottom: valueRect.bottom,
        labelColor: labelStyle.color,
        labelFontSize: labelStyle.fontSize,
        valueFontSizes,
      };
    });
  }, selector);
}

// 帯は段ごとに並べる。段の中の項目でラベル行と数字の基準線をそろえる
function expectBandAligned(items: BandItem[]) {
  expect(items.length).toBeGreaterThan(1);
  const rows = new Map<number, BandItem[]>();
  for (const item of items) {
    const key = Math.round(item.labelTop);
    const group = rows.get(key);
    if (group) {
      group.push(item);
    } else {
      rows.set(key, [item]);
    }
  }
  for (const [labelTop, row] of rows) {
    for (const item of row) {
      // ラベルの上端が同じ段の中でそろっている
      expect(
        Math.abs(item.labelTop - labelTop),
        `ラベル「${item.label}」の上端`,
      ).toBeLessThanOrEqual(1);
    }
    const bottoms = row.map((item) => item.valueBottom);
    // 数字は大きさが違っても下端(基準線)がそろう
    expect(
      Math.max(...bottoms) - Math.min(...bottoms),
      `段「${row.map((item) => item.label).join('・')}」の数字の下端`,
    ).toBeLessThanOrEqual(2);
  }
  // ラベルの見た目(大きさ・色)は全項目で同じ
  const labelColors = new Set(items.map((item) => item.labelColor));
  expect([...labelColors]).toHaveLength(1);
  const labelFontSizes = new Set(items.map((item) => item.labelFontSize));
  expect([...labelFontSizes]).toHaveLength(1);
  // 数字の大きさは主と従の2種類だけ
  const valueFontSizes = new Set(items.flatMap((item) => item.valueFontSizes));
  expect([...valueFontSizes].sort()).toHaveLength(2);
}

const ASSET_BAND = '[data-testid="asset-portfolio-summary"]';
const HOME_BAND = '[data-testid="home-overview"]';
const RECEIPTS_BAND = '[data-testid="receipt-summary-strip"]';

async function selectReceiptTab(page: Page, name: string) {
  const tab = page.getByRole('tab', { name });
  await tab.click();
  await expect(tab).toHaveAttribute('aria-selected', 'true');
}

for (const width of [1280, 390]) {
  test.describe(`幅 ${width}`, () => {
    test.use({ viewport: { width, height: width === 390 ? 844 : 800 } });

    test('資産管理の3タブの帯はラベル行と数字の基準線がそろう', async ({
      page,
      context,
    }) => {
      await mockApi(context);
      await page.goto('/assetbalance');
      for (const tab of ['国内株式', '投資信託', 'すべて']) {
        await page.getByRole('tab', { name: tab }).click();
        const band = page.getByTestId('asset-portfolio-summary');
        await expect(band).toBeVisible();
        expectBandAligned(await bandItems(page, ASSET_BAND));
      }
    });

    test('ホームの帯はラベル行と数字の基準線がそろう', async ({
      page,
      context,
    }) => {
      await mockApi(context);
      await page.goto('/');
      await expect(page.getByTestId('home-overview')).toBeVisible();
      expectBandAligned(await bandItems(page, HOME_BAND));
    });

    test('取引明細の3タブの帯はラベル行と数字の基準線がそろう', async ({
      page,
      context,
    }) => {
      await mockApi(context);
      await page.goto('/receipts');
      for (const tab of ['配当金', '国内株式', '投資信託']) {
        await selectReceiptTab(page, tab);
        await expect(page.getByTestId('receipt-summary-strip')).toBeVisible();
        expectBandAligned(await bandItems(page, RECEIPTS_BAND));
      }
    });
  });
}
