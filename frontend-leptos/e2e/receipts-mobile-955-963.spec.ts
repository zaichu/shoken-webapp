import { expect, test, type Locator, type Page } from '@playwright/test';
import * as path from 'path';

test.use({ viewport: { width: 390, height: 844 } });

const SHOT_DIR = '../.playwright-mcp';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const DIVIDEND = {
  id: '00000000-0000-0000-0000-000000000001',
  settlement_date: '2024-03-01',
  product: '特定口座',
  account: 'SBI証券',
  security_code: '7203',
  security_name: 'トヨタ自動車',
  unit_price: 30.0,
  shares: 100,
  dividends_before_tax: 3000,
  taxes: 609,
  net_amount_received: 2391,
  created_at: '2024-03-01T00:00:00Z',
  updated_at: '2024-03-01T00:00:00Z',
};

const FUND = {
  id: '00000000-0000-0000-0000-000000000005',
  trade_date: '2024-02-01',
  settlement_date: '2024-02-03',
  fund_name: 'eMAXIS Slim 全世界株式(オール・カントリー)',
  account: 'NISA',
  shares: 1000,
  exchange_rate: 1,
  cancellation_unit_price_yen: 20000,
  cancellation_amount_yen: 20000,
  average_acquisition_price_yen: 18000,
  realized_profit_and_loss: 2000,
  taxes: 406,
  realized_profit_and_loss_after_tax: 1594,
  created_at: '2024-02-01T00:00:00Z',
  updated_at: '2024-02-01T00:00:00Z',
};

function paginatedResponse(data: unknown[]) {
  return { data, total: data.length, page: 1, per_page: data.length };
}

async function mockApi(page: Page) {
  await page.route(/\/api\/v1\//, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginatedResponse([])),
    }),
  );
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_USER),
    }),
  );
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginatedResponse([DIVIDEND])),
    }),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginatedResponse([FUND])),
    }),
  );
}

test('CSVプレビューで表示内容が同じ行でもカードは個別に開閉できる', async ({ page }) => {
  await mockApi(page);
  await page.route(/\/api\/v1\/dividend-import-validations$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        total_rows: 2,
        valid_rows: 2,
        errors: [],
        rows: [{ ...DIVIDEND }, { ...DIVIDEND }],
      }),
    }),
  );
  await page.goto('/receipts');

  await page.getByTestId('receipt-csv-toggle').click();
  await page
    .getByTestId('csv-file-input')
    .setInputFiles(
      path.resolve(
        test.info().project.testDir,
        '../../frontend/e2e/__fixtures__/csv/dividend-base.csv',
      ),
    );

  const cardList = page.getByTestId('receipt-card-list');
  const cards = cardList.getByTestId('receipt-card');
  await expect(cards).toHaveCount(2);

  const first = cards
    .nth(0)
    .getByRole('button', { name: 'トヨタ自動車 ¥ 2,391' });
  const second = cards
    .nth(1)
    .getByRole('button', { name: 'トヨタ自動車 ¥ 2,391' });
  await first.click();
  await expect(first).toHaveAttribute('aria-expanded', 'true');
  await expect(second).toHaveAttribute('aria-expanded', 'false');
  await expect(cards.nth(0).getByRole('region')).toBeVisible();
  await expect(cards.nth(1).getByRole('region')).toBeHidden();

  await page.screenshot({ path: `${SHOT_DIR}/leptos-955-identical-cards.png` });

  await second.click();
  await expect(second).toHaveAttribute('aria-expanded', 'true');
  await expect(first).toHaveAttribute('aria-expanded', 'true');

  await first.click();
  await expect(first).toHaveAttribute('aria-expanded', 'false');
  await expect(second).toHaveAttribute('aria-expanded', 'true');
});

async function expectTabInsideViewport(tab: Locator, viewportWidth: number) {
  // scrollIntoView は端でサブピクセル単位の見切れを残すことがあるため 1px 許容する
  await expect
    .poll(async () => (await tab.boundingBox())?.x ?? Number.NEGATIVE_INFINITY)
    .toBeGreaterThanOrEqual(-1);
  await expect
    .poll(async () => {
      const box = await tab.boundingBox();
      return box ? box.x + box.width : Number.POSITIVE_INFINITY;
    })
    .toBeLessThanOrEqual(viewportWidth + 1);
}

test('選択したタブは390pxでも320pxでも見切れない', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');
  await expect(page.getByRole('tab')).toHaveCount(3);

  const dividend = page.getByRole('tab', { name: /配当金/ });
  const domestic = page.getByRole('tab', { name: /国内株式/ });
  const fund = page.getByRole('tab', { name: /投資信託/ });

  await fund.click();
  await expect(fund).toHaveAttribute('aria-selected', 'true');
  await expectTabInsideViewport(fund, 390);
  await expectTabInsideViewport(domestic, 390);
  await expectTabInsideViewport(dividend, 390);

  await page.screenshot({ path: `${SHOT_DIR}/leptos-955-tabs-390.png` });

  await page.setViewportSize({ width: 320, height: 844 });
  const tablist = page.getByRole('tablist');
  const overflows = await tablist.evaluate(
    (element) => element.scrollWidth > element.clientWidth,
  );
  expect(overflows, '320px ではタブが横スクロールする').toBe(true);

  // locator.click() は要素を自動スクロールするため、自動スクロールしない
  // element.click() で選択し、選択追従のスクロール実装が効いているかだけを見る
  await tablist.evaluate((element) => {
    element.scrollLeft = element.scrollWidth;
  });
  await page.evaluate(() => document.getElementById('tab-dividend')?.click());
  await expect(dividend).toHaveAttribute('aria-selected', 'true');
  await expectTabInsideViewport(dividend, 320);

  await page.evaluate(() => document.getElementById('tab-mutualfund')?.click());
  await expect(fund).toHaveAttribute('aria-selected', 'true');
  await expectTabInsideViewport(fund, 320);
});

test('カード一覧はスマホ幅でページ全幅を使いファンド名の省略位置がReactに近い', async ({
  page,
}) => {
  await mockApi(page);
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  await expect(cardList).toBeVisible();
  const listBox = await cardList.boundingBox();
  expect(listBox, 'カード一覧の幅').not.toBeNull();
  // main の px-4 を除いた全幅(390-32=358)。page-surface の p-6 が残ると 310 まで狭まる
  expect(listBox!.width).toBeGreaterThanOrEqual(356);

  await page.getByRole('tab', { name: /投資信託/ }).click();
  const name = cardList.getByRole('button', {
    name: /eMAXIS Slim 全世界株式\(オール・カントリー\)/,
  });
  await expect(name).toBeVisible();
  const nameBox = await name.boundingBox();
  expect(nameBox, 'ファンドカードの幅').not.toBeNull();
  expect(nameBox!.width).toBeGreaterThanOrEqual(356);

  await page.screenshot({ path: `${SHOT_DIR}/leptos-955-fund-name-390.png` });
});
