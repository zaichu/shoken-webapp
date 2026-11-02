import type { Page } from '@playwright/test';
import { expect, test, json, paginated, mockSession } from '../support/test';

// 桁の違う月を混ぜる。2桁の月(10/20 など)の行を含める
const DIVIDENDS = [
  { date: '2026-12-31', code: '9901', name: '架空石油化学工業株式会社' },
  { date: '2026-10-20', code: '9902', name: '架空精密機器工業株式会社' },
  { date: '2026-11-05', code: '9903', name: '架空都市開発ホールディングス株式会社' },
  { date: '2026-09-02', code: '9904', name: '架空食品薬品研究所株式会社' },
  { date: '2026-12-29', code: '9905', name: '架空総合重工株式会社' },
].map((row, index) => ({
  id: `dividend-${index + 1}`,
  settlement_date: row.date,
  product: '国内株式',
  account: '一般',
  security_code: row.code,
  security_name: row.name,
  unit_price: 1,
  shares: 1,
  dividends_before_tax: 1000,
  taxes: 200,
  net_amount_received: 800,
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
}));

const STOCKS = [
  {
    id: 'asset-balance-1',
    security_code: '9901',
    security_name: '架空石油化学工業株式会社',
    shares: 100,
    average_purchase_price: 2500,
    total_purchase_amount: 250000,
    current_price: 2600,
    daily_change: 50,
    price_as_of: '2026-12-31',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-12-31T00:00:00Z',
  },
];

async function mockApi(page: Page) {
  await page.route(/\/api\/v1\//, (route) => route.fulfill(json(paginated([]))));
  await mockSession(page);
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DIVIDENDS))),
  );
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(STOCKS))),
  );
}

test('最近の配当入金は2行で名前と金額が重ならず全文が出る', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await mockApi(page);
  await page.goto('/');

  const list = page.getByTestId('home-recent-list');
  await expect(list).toBeVisible();

  const rows = list.getByTestId('home-dividend-row');
  await expect(rows).toHaveCount(5);
  await expect(rows.first().locator('.list-row-name')).toContainText('架空');
  await expect(rows.first().locator('.list-row-primary')).toContainText('¥');
  await expect(rows.first().locator('.list-row-meta')).toContainText('9901');
  await expect(rows.first().locator('.list-row-secondary')).toHaveCount(0);

  const overflow = await list.evaluate((element) => {
    return [...element.querySelectorAll<HTMLElement>('.list-row-item')].map((row) => {
      const name = row.querySelector<HTMLElement>('.list-row-name')!;
      const primary = row.querySelector<HTMLElement>('.list-row-primary')!;
      return {
        nameRight: name.getBoundingClientRect().right,
        primaryLeft: primary.getBoundingClientRect().left,
      };
    });
  });

  for (const row of overflow) {
    expect(row.nameRight).toBeLessThanOrEqual(row.primaryLeft);
  }
});

test('評価損益の大きい銘柄は2行で名前と金額が重ならない', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await mockApi(page);
  await page.goto('/');

  const list = page.getByTestId('home-pnl-list');
  await expect(list).toBeVisible();

  const rows = await list.evaluate((element) => {
    return [...element.querySelectorAll<HTMLElement>('.list-row-item')].map((row) => {
      const name = row.querySelector<HTMLElement>('.list-row-name')!;
      const amount = row.querySelector<HTMLElement>('.list-row-primary')!;
      const rate = row.querySelector<HTMLElement>('.list-row-secondary')!;
      const amountBox = amount.getBoundingClientRect();
      const rateBox = rate.getBoundingClientRect();
      return {
        name: name.textContent?.trim() ?? '',
        nameRight: name.getBoundingClientRect().right,
        amountLeft: amountBox.left,
        amountRight: amountBox.right,
        amountBottom: amountBox.bottom,
        rateTop: rateBox.top,
        rateRight: rateBox.right,
      };
    });
  });

  expect(rows.length).toBeGreaterThan(0);
  for (const row of rows) {
    // 1行目: 名前と金額が重ならない
    expect(row.nameRight, `銘柄名(${row.name})と金額が重なっている`).toBeLessThanOrEqual(
      row.amountLeft,
    );
    // 2行目: 損益率は金額の下にあり、右端がそろう
    expect(row.rateTop, `損益率が金額と同じ行にある`).toBeGreaterThanOrEqual(
      row.amountBottom - 1,
    );
    expect(
      Math.abs(row.rateRight - row.amountRight),
      `金額と損益率の右端がそろっていない`,
    ).toBeLessThanOrEqual(2);
  }
});
