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
    created_at: `${THIS_YEAR}-01-01T00:00:00Z`,
    updated_at: `${THIS_YEAR}-01-01T00:00:00Z`,
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
    created_at: `${THIS_YEAR}-01-01T00:00:00Z`,
    updated_at: `${THIS_YEAR}-01-01T00:00:00Z`,
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
    created_at: `${THIS_YEAR}-01-01T00:00:00Z`,
    updated_at: `${THIS_YEAR}-01-01T00:00:00Z`,
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
    created_at: `${THIS_YEAR}-01-01T00:00:00Z`,
    updated_at: `${THIS_YEAR}-01-01T00:00:00Z`,
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

test('帯は5項目で株式と投信を合算し、カード群が埋まる', async ({ page }) => {
  await setupHomeMocks(page);

  await page.goto('/');

  const band = page.getByTestId('home-overview');
  // 株式 360,000 + 投信 300,000 の合算
  await expect(band).toContainText('¥660,000');
  // 損益 = (360,000-400,000) + (300,000-250,000)
  await expect(band).toContainText('+¥10,000');
  // 年間予想配当 = 9904 の 100株 × ¥50
  await expect(band).toContainText('¥5,000');
  // 帯に置くのは5項目だけ(評価額・評価損益・前日比・年間予想配当・取得総額)。「今年の税引後配当金」は置かない
  // スマホ用の内訳は閉じたまま DOM にあるため、見えている項目だけ数える
  await expect(band.locator('dt:visible')).toHaveCount(5);
  await expect(band).not.toContainText('税引後');

  // 今年の配当金カード: 税引後の配当金と予想への進捗
  const dividend = page.getByTestId('home-dividend-card');
  await expect(dividend).not.toContainText('残り');
  await expect(dividend).not.toContainText('前年の12月は');
  await expect(dividend).not.toContainText('前年同期より');
  await expect(dividend).toContainText('¥20,000');

  // 資産の内訳: 商品別は株式・投信の2区分
  const composition = page.getByTestId('home-composition-card');
  await expect(composition).toContainText('国内株式');
  await expect(composition).toContainText('投資信託');

  // グラフの累計の折れ線は本番 CSP 下でも描画される
  const ticks = dividend.locator('.home-cum-tick');
  await expect(ticks.filter({ hasText: /^0$/ })).toHaveCount(1);
  expect(await ticks.count()).toBeGreaterThanOrEqual(3);
  await expect(dividend.locator('.home-cum-line-current')).toBeVisible();
  await expect(dividend.locator('.home-cum-line-previous')).toBeVisible();
  const progress = dividend.locator('.home-progress > div');
  expect((await progress.boundingBox())?.width).toBeGreaterThan(0);
  const compositionRows = composition.locator('.home-composition-rows').first().locator('li');
  await expect(compositionRows).toHaveCount(2);
  const compositionAmounts = await compositionRows.locator('> span').allTextContents();
  expect(compositionAmounts).toEqual(['¥360,000', '¥300,000']);
  const compositionTotal = composition.locator('.home-composition-total > span').last();
  await expect(compositionTotal).toHaveText('¥660,000');
  expect(
    compositionAmounts.reduce((sum, amount) => sum + Number(amount.replace(/[^\d-]/g, '')), 0),
  ).toBe(Number((await compositionTotal.textContent())?.replace(/[^\d-]/g, '')));
  await expect(compositionRows.nth(0)).toContainText('2件');
  await expect(compositionRows.nth(1)).toContainText('1件');
  const productBar = composition.locator('.home-stacked-bar').first().locator('> span');
  expect(await productBar.count()).toBeGreaterThan(0);
  expect((await productBar.first().boundingBox())?.width).toBeGreaterThan(0);
  const accountBar = composition.locator('.home-stacked-bar > span').first();
  expect((await accountBar.boundingBox())?.width).toBeGreaterThan(0);

  // 最近の配当入金: per_page=5 の要求なので 6 件用意しても 5 行
  await expect(page.getByTestId('home-recent-list').locator('li')).toHaveCount(5);

  // 評価損益の大きい銘柄: 含み益と含み損の両方が出る
  const pnl = page.getByTestId('home-pnl-card');
  await expect(pnl).toContainText('架空投資信託グローバル株式ファンド');
  await expect(pnl).toContainText('含み損');
});

test('配当グラフは配当金の累計の折れ線で、点からその月までの累計を出せる', async ({ page }) => {
  await setupHomeMocks(page);
  await page.goto('/');

  const dividend = page.getByTestId('home-dividend-card');
  const chart = dividend.locator('.home-chart').first();

  await expect(dividend.locator('.home-cum-legend')).toHaveCount(0);
  const endLabels = chart.locator('.home-cum-end-label');
  await expect(endLabels.filter({ hasText: '今年' })).toHaveCount(1);
  await expect(endLabels.filter({ hasText: '前年' })).toHaveCount(1);
  await expect(endLabels.filter({ hasText: '今年' })).toContainText(/¥[\d,]+/);
  await expect(endLabels.filter({ hasText: '前年' })).toContainText(/¥[\d,]+/);
  await expect(dividend.locator('.home-cum-area-current')).toBeVisible();

  // 点を押すとその月の配当金とその月までの今年と前年の累計が出る
  const currentMonth = new Date().getMonth() + 1;
  const monthOf = (date: string) => Number(date.slice(5, 7));
  const currentMonthly = CURRENT_DIVIDENDS.filter((row) => monthOf(row.settlement_date) === currentMonth).reduce(
    (sum, row) => sum + row.net_amount_received,
    0,
  );
  const currentCum = CURRENT_DIVIDENDS.filter((row) => monthOf(row.settlement_date) <= currentMonth).reduce(
    (sum, row) => sum + row.net_amount_received,
    0,
  );
  const previousCum = PREVIOUS_DIVIDENDS.filter((row) => monthOf(row.settlement_date) <= currentMonth).reduce(
    (sum, row) => sum + row.net_amount_received,
    0,
  );
  const plot = chart.locator('.home-chart-plot');
  const tipFor = (month: number) =>
    chart.locator('.home-cum-col').nth(month - 1).locator('.home-cum-tip');
  // 描画域は1つのスライダー操作子(12個の柱ボタンは置かない)
  await expect(plot).toHaveAttribute('role', 'slider');
  await expect(plot).toHaveAttribute('aria-valuemin', '1');
  await expect(plot).toHaveAttribute('aria-valuemax', String(currentMonth));
  expect(await chart.locator('.home-cum-col[role="button"]').count()).toBe(0);

  // 描画域のその月の位置を押すと、その月の配当金とその月までの今年と前年の累計が出る
  const box = await plot.boundingBox();
  const xFor = (month: number) => (box!.width * (month - 0.5)) / 12;
  await plot.click({ position: { x: xFor(currentMonth), y: box!.height / 2 } });
  const tip = tipFor(currentMonth);
  await expect(tip).toBeVisible();
  await expect(tip).toContainText(`${currentMonth}月の配当金`);
  await expect(tip).toContainText(`¥${currentMonthly.toLocaleString('en-US')}`);
  await expect(tip).toContainText(`¥${currentCum.toLocaleString('en-US')}`);
  await expect(tip).toContainText(`¥${previousCum.toLocaleString('en-US')}`);
  await expect(plot).toHaveAttribute('aria-valuenow', String(currentMonth));

  // キーボードで月を移動できる(Home で1月、矢印で1か月ずつ、End で今月)
  await plot.press('Home');
  await expect(tipFor(1)).toBeVisible();
  await expect(plot).toHaveAttribute('aria-valuenow', '1');
  await expect(plot).toHaveAttribute('aria-valuetext', /1月 配当金/);
  const afterRight = Math.min(2, currentMonth);
  await plot.press('ArrowRight');
  await expect(tipFor(afterRight)).toBeVisible();
  await expect(plot).toHaveAttribute('aria-valuenow', String(afterRight));
  await plot.press('ArrowLeft');
  await expect(tipFor(1)).toBeVisible();
  await plot.press('End');
  await expect(tipFor(currentMonth)).toBeVisible();

  // Escape で閉じる
  await plot.press('Escape');
  await expect(tipFor(currentMonth)).toBeHidden();

  // 外側タップで閉じる
  const probeMonth = Math.min(3, currentMonth);
  await plot.click({ position: { x: xFor(probeMonth), y: box!.height / 2 } });
  await expect(tipFor(probeMonth)).toBeVisible();
  await dividend.locator('h2').first().click();
  await expect(tipFor(probeMonth)).toBeHidden();

  // Tab だけで操作域に到達し、Enter/Space で吹き出しを開閉できる
  await dividend.getByRole('link', { name: '取引明細で見る' }).focus();
  for (let i = 0; i < 10; i++) {
    if (await plot.evaluate((el) => el === document.activeElement)) {
      break;
    }
    await page.keyboard.press('Tab');
  }
  await expect(plot).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(tipFor(currentMonth)).toBeVisible();
  await page.keyboard.press(' ');
  await expect(tipFor(currentMonth)).toBeHidden();

  // 資産の内訳の行には横棒と同じ色の印が出る
  const composition = page.getByTestId('home-composition-card');
  const rows = composition.locator('.home-composition-row');
  const swatches = rows.locator('.home-swatch');
  expect(await swatches.count()).toBe(await rows.count());
  const firstBarClass = (await composition
    .locator('.home-stacked-bar')
    .first()
    .locator('> span')
    .first()
    .getAttribute('class')) ?? '';
  const firstSwatchClass = (await rows.first().locator('.home-swatch').getAttribute('class')) ?? '';
  const barColor = firstBarClass.split(' ').find((name) => name.startsWith('bg-chart-'));
  expect(barColor).toBeTruthy();
  expect(firstSwatchClass.split(' ')).toContain(barColor);
});

test('各月の丸はその月の目盛りの文字の真上にある', async ({ page }) => {
  await setupHomeMocks(page);
  await page.goto('/');

  const chart = page.getByTestId('home-dividend-card').locator('.home-chart').first();
  await expect(chart.locator('.home-cum-line-current')).toBeVisible();
  await expect(chart.locator('.home-chart-month-label')).toHaveCount(12);

  const diffs = await chart.evaluate((root) => {
    const labels = Array.from(root.querySelectorAll('.home-chart-month-label'));
    return Array.from(root.querySelectorAll('.home-cum-col')).flatMap((col, index) => {
      const label = labels[index] as HTMLElement | undefined;
      if (!label) return [];
      const labelBox = label.getBoundingClientRect();
      const center = labelBox.left + labelBox.width / 2;
      return Array.from(col.querySelectorAll('.home-cum-dot')).map((dot) => {
        const box = (dot as HTMLElement).getBoundingClientRect();
        return Math.abs(box.left + box.width / 2 - center);
      });
    });
  });
  expect(diffs.length).toBeGreaterThan(0);
  for (const dx of diffs) {
    expect(dx).toBeLessThanOrEqual(2);
  }
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

test('評価額のそばは株価の日付だけを出し、取り込み日は出さない', async ({ page }) => {
  await setupHomeMocks(page);
  await page.goto('/');

  const band = page.getByTestId('home-overview');
  await expect(band).toContainText(`株価 ${THIS_YEAR}/01/10 時点`);
  await expect(band).not.toContainText('取り込み');
  await expect(band).not.toContainText('（');
  await expect(page.locator('.page-frame-head')).not.toContainText('取り込み');
});

test('取込の状態によらず取り込み日は出さず、株価の日付だけ出す', async ({ page }) => {
  await mockSession(page);

  const freshStocks = STOCKS.map((row) => ({
    ...row,
    updated_at: `${THIS_YEAR}-01-10T00:00:00Z`,
  }));
  const freshFunds = FUNDS.map((row) => ({
    ...row,
    updated_at: `${THIS_YEAR}-01-10T00:00:00Z`,
  }));
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(freshStocks, { summary: STOCK_SUMMARY }))),
  );
  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(freshFunds, { summary: FUND_SUMMARY }))),
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
    return route.fulfill(json(paginated(perPage > 0 ? rows.slice(0, perPage) : rows)));
  });
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json(ESTIMATES)),
  );

  await page.goto('/');

  const band = page.getByTestId('home-overview');
  await expect(band).toContainText(`株価 ${THIS_YEAR}/01/10 時点`);
  await expect(band).not.toContainText('取り込み');
  await expect(page.locator('.page-frame-head')).not.toContainText('取り込み');
});
