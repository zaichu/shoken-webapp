import type { Page } from '@playwright/test';
import { expect, test, json, paginated } from '../support/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000073',
  email: 'test@example.com',
  name: 'テストユーザー',
};

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
  await page.route(/\/api\/v1\/session$/, (route) => route.fulfill(json(MOCK_USER)));
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DIVIDENDS))),
  );
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(STOCKS))),
  );
}

test('最近の配当入金は日付が列からはみ出さず、コードと重ならない', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await mockApi(page);
  await page.goto('/');

  const list = page.getByTestId('home-recent-list');
  await expect(list).toBeVisible();

  const rows = await list.evaluate((element) => {
    return [...element.querySelectorAll<HTMLElement>('.home-dividend-row')].map((row) => {
      const date = row.querySelector<HTMLElement>('.home-dividend-date')!;
      const code = row.querySelector<HTMLElement>('.home-dividend-code')!;
      // はみ出したぶんも測るために、要素の枠ではなく文字の範囲で測る
      const range = document.createRange();
      range.selectNodeContents(date);
      return {
        date: date.textContent?.trim() ?? '',
        textRight: range.getBoundingClientRect().right,
        textWidth: range.getBoundingClientRect().width,
        cellWidth: date.clientWidth,
        codeLeft: code.getBoundingClientRect().left,
      };
    });
  });

  expect(rows.length).toBeGreaterThan(0);
  expect(
    rows.some((row) => /^\d{2}\//.test(row.date)),
    '2桁の月の行が含まれていない',
  ).toBe(true);

  for (const row of rows) {
    expect(row.textWidth, `日付(${row.date})が列からはみ出している`).toBeLessThanOrEqual(
      row.cellWidth,
    );
    expect(row.textRight, `日付(${row.date})とコードが重なっている`).toBeLessThan(
      row.codeLeft,
    );
  }
});

test('評価損益の大きい銘柄も列が重ならない', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await mockApi(page);
  await page.goto('/');

  const list = page.getByTestId('home-pnl-list');
  await expect(list).toBeVisible();

  const rows = await list.evaluate((element) => {
    return [...element.querySelectorAll<HTMLElement>('.home-pnl-row')].map((row) => {
      const name = row.querySelector<HTMLElement>('.home-pnl-name')!;
      const amount = row.querySelector<HTMLElement>('.home-pnl-amount')!;
      const rate = row.querySelector<HTMLElement>('.home-pnl-rate')!;
      const range = document.createRange();
      range.selectNodeContents(amount);
      return {
        name: name.textContent?.trim() ?? '',
        nameRight: name.getBoundingClientRect().right,
        amountTextRight: range.getBoundingClientRect().right,
        rateLeft: rate.getBoundingClientRect().left,
      };
    });
  });

  expect(rows.length).toBeGreaterThan(0);
  for (const row of rows) {
    expect(row.nameRight, `銘柄名(${row.name})と金額が重なっている`).toBeLessThanOrEqual(
      row.amountTextRight,
    );
    expect(row.amountTextRight, `金額と損益率が重なっている`).toBeLessThan(row.rateLeft);
  }
});
