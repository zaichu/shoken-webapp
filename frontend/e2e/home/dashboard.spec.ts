import type { Page } from '@playwright/test';
import { expect, test, json, paginated, mockSession } from '../support/test';

const THIS_YEAR = new Date().getFullYear();
const LAST_YEAR = THIS_YEAR - 1;

const STOCKS = [
  {
    id: 'asset-balance-1',
    security_code: '9904',
    security_name: '架空自動車',
    shares: 100,
    average_purchase_price: 2500,
    total_purchase_amount: 250000,
    current_price: 2600,
    daily_change: 50,
    price_as_of: `${THIS_YEAR}-01-10`,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  },
  {
    id: 'asset-balance-2',
    security_code: '9914',
    security_name: '架空エレクトロニクス',
    shares: 50,
    average_purchase_price: 3000,
    total_purchase_amount: 150000,
    current_price: 2000,
    daily_change: -20,
    price_as_of: `${THIS_YEAR}-01-10`,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  },
];

const STOCK_SUMMARY = {
  total_purchase_amount: 400000,
  total_market_value: 360000,
  total_daily_change: 30,
  priced_purchase_amount: 400000,
  priced_market_value: 360000,
};

const FUNDS = [
  {
    id: 'fund-balance-1',
    fund_name: '架空投資信託グローバル株式ファンド',
    dividends_course: null,
    account: '特定',
    shares: 1000000,
    average_purchase_price: 10000,
    total_purchase_amount: 250000,
    base_price: 12000,
    base_price_day_change: null,
    base_price_month_change: null,
    market_value: 300000,
    profit_loss: 50000,
    profit_loss_rate: 20,
    total_return: null,
    price_as_of: `${THIS_YEAR}-01-10`,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  },
];

const FUND_SUMMARY = {
  total_purchase_amount: 250000,
  total_market_value: 300000,
  total_daily_change: 0,
};

function dividend(id: string, settlementDate: string, net: number, name: string) {
  return {
    id,
    settlement_date: settlementDate,
    product: '国内株式',
    account: '特定',
    security_code: '9904',
    security_name: name,
    unit_price: 1,
    shares: 1,
    dividends_before_tax: net * 1.25,
    taxes: net * 0.25,
    net_amount_received: net,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  };
}

const CURRENT_DIVIDENDS = [
  dividend('div-c1', `${THIS_YEAR}-03-10`, 12000, '架空自動車'),
  dividend('div-c2', `${THIS_YEAR}-06-15`, 8000, '架空エレクトロニクス'),
];
const PREVIOUS_DIVIDENDS = [
  dividend('div-p1', `${LAST_YEAR}-03-10`, 5000, '架空自動車'),
  dividend('div-p2', `${LAST_YEAR}-11-20`, 9000, '架空自動車'),
];
const RECENT_DIVIDENDS = Array.from({ length: 6 }, (_, i) =>
  dividend(`div-r${i}`, `${THIS_YEAR}-06-${String(20 - i).padStart(2, '0')}`, 1000 * (i + 1), `架空銘柄${i}`),
);

const ESTIMATES = {
  items: [
    { security_code: '9904', dividend_per_share: 50, status: 'ok' },
    { security_code: '9914', dividend_per_share: 0, status: 'zero' },
  ],
};

async function setupHomeMocks(page: Page) {
  await mockSession(page);

  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(STOCKS, { summary: STOCK_SUMMARY }))),
  );

  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(FUNDS, { summary: FUND_SUMMARY }))),
  );

  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) => {
    const url = new URL(route.request().url());
    const year = url.searchParams.get('year');
    const perPage = Number(url.searchParams.get('per_page') ?? '0');
    const rows =
      year === String(THIS_YEAR)
        ? CURRENT_DIVIDENDS
        : year === String(LAST_YEAR)
          ? PREVIOUS_DIVIDENDS
          : RECENT_DIVIDENDS;
    // 直近入金の取得は件数指定だけで year を付けない。per_page だけ効かせる
    return route.fulfill(json(paginated(perPage > 0 ? rows.slice(0, perPage) : rows)));
  });

  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json(ESTIMATES)),
  );
}

test('帯は3項目で株式と投信を合算し、カード群が埋まる', async ({ page }) => {
  await setupHomeMocks(page);

  await page.goto('/');

  const band = page.getByTestId('home-overview');
  // 株式 360,000 + 投信 300,000 の合算
  await expect(band).toContainText('¥660,000');
  // 損益 = (360,000-400,000) + (300,000-250,000)
  await expect(band).toContainText('+¥10,000');
  // 年間予想配当 = 9904 の 100株 × ¥50
  await expect(band).toContainText('¥5,000');
  // 帯に置くのは3項目だけ。「今年の税引後配当金」は置かない
  await expect(band.locator('dt')).toHaveCount(3);
  await expect(band).not.toContainText('税引後');

  // 今年の配当金カード: 税引後の受取と予想への進捗
  const dividend = page.getByTestId('home-dividend-card');
  await expect(dividend).toContainText('¥20,000');
  await expect(dividend).toContainText('前年同期より');

  // 資産の内訳: 商品別は株式・投信の2区分
  const composition = page.getByTestId('home-composition-card');
  await expect(composition).toContainText('国内株式');
  await expect(composition).toContainText('投資信託');

  // グラフの棒・進捗・ドーナツ・凡例・口座別の帯は本番 CSP 下でも描画される
  const monthBars = dividend.locator('.home-month-bar');
  expect(
    await monthBars.evaluateAll((els) =>
      els.some((el) => el.getBoundingClientRect().height > 20),
    ),
  ).toBe(true);
  const progress = dividend.locator('.home-progress > div');
  expect((await progress.boundingBox())?.width).toBeGreaterThan(0);
  await expect(composition.locator('.home-swatch').first()).toHaveCSS(
    'background-color',
    /^rgb\(/,
  );
  await expect(composition.locator('.home-donut')).toHaveCSS(
    'background-image',
    /conic-gradient/,
  );
  const accountBar = composition.locator('.home-stacked-bar > span').first();
  expect((await accountBar.boundingBox())?.width).toBeGreaterThan(0);

  // 最近の配当入金: per_page=5 の要求なので 6 件用意しても 5 行
  await expect(page.getByTestId('home-recent-list').locator('li')).toHaveCount(5);

  // 評価損益の大きい銘柄: 含み益と含み損の両方が出る
  const pnl = page.getByTestId('home-pnl-card');
  await expect(pnl).toContainText('架空投資信託グローバル株式ファンド');
  await expect(pnl).toContainText('含み損');
});

test('一部の取得が失敗しても、取れた集計で帯とカードを出す', async ({ page }) => {
  await setupHomeMocks(page);
  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json({ message: 'error' }, 500)),
  );

  await page.goto('/');

  await expect(page.getByText('一部の集計を取得できませんでした')).toBeVisible();

  const band = page.getByTestId('home-overview');
  // 投信の取得に失敗した分は株式だけの集計に倒れる
  await expect(band).toContainText('¥360,000');

  // 内訳カードも株式側は出る
  const composition = page.getByTestId('home-composition-card');
  await expect(composition).toContainText('国内株式');
});
