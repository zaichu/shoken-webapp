import type { Locator, Page } from '@playwright/test';
import { expect, test } from './support/test';
import * as fs from 'fs';
import * as path from 'path';

test.use({ viewport: { width: 390, height: 844 } });

async function shoot(page: Page, name: string) {
  const dir = path.resolve(test.info().project.testDir, '../../.playwright-mcp');
  await fs.promises.mkdir(dir, { recursive: true });
  await page.screenshot({ path: path.join(dir, `receipts-mobile-${name}.png`) });
}

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

const DOMESTIC = {
  id: '00000000-0000-0000-0000-000000000004',
  trade_date: '2024-02-01',
  settlement_date: '2024-02-03',
  account: 'SBI証券',
  security_code: '7974',
  security_name: '任天堂',
  shares: 10,
  asked_price: 5000,
  proceeds: 55000,
  purchase_price: 5000,
  realized_profit_and_loss: 5000,
  taxes: 1015,
  realized_profit_and_loss_after_tax: 3985,
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
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginatedResponse([DOMESTIC])),
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

test('CSVプレビューで表示内容が同じ行でもカードは個別に全項目を表示する', async ({
  page,
}) => {
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
        '__fixtures__/csv/dividend-base.csv',
      ),
    );

  const cardList = page.getByTestId('receipt-card-list');
  const cards = cardList.getByTestId('receipt-card');
  await expect(cards).toHaveCount(2);

  // プレビュー行も開閉なしで見出しと全項目を出す
  for (const index of [0, 1]) {
    const card = cards.nth(index);
    await expect(card.locator('[aria-expanded]')).toHaveCount(0);
    await expect(
      card.getByRole('button', { name: 'トヨタ自動車(7203) をコピー' }),
    ).toBeVisible();
    await expect(card.locator('dl').locator('dt')).toHaveCount(7);
    await expect(card.locator('dl').getByText('¥2,391')).toBeVisible();
  }

  await shoot(page, 'identical-cards');
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

  await shoot(page, 'tabs-390');

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

test('カード一覧はスマホ幅でページ全幅を使い、長いファンド名は省略せず折り返す', async ({
  page,
}) => {
  await mockApi(page);
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  await expect(cardList).toBeVisible();
  const listBox = await cardList.boundingBox();
  expect(listBox, 'カード一覧の幅').not.toBeNull();
  // main の px-4 と外側カードの枠線を除いた全幅(390-32-2=356)。page-surface の p-6 が残ると 308 まで狭まる
  expect(listBox!.width).toBeGreaterThanOrEqual(354);

  await page.getByRole('tab', { name: /投資信託/ }).click();
  const name = cardList.getByRole('button', {
    name: 'eMAXIS Slim 全世界株式(オール・カントリー) をコピー',
  });
  await expect(name).toBeVisible();
  const nameBox = await name.boundingBox();
  expect(nameBox, 'ファンド名ボタンの幅').not.toBeNull();
  // カード内側の余白(px-3)を引いた全幅=332
  expect(nameBox!.width).toBeGreaterThanOrEqual(328);

  // 全文が省略されず複数行に折り返して出る
  const wrap = await name.locator('span').first().evaluate((el) => {
    const range = document.createRange();
    range.selectNodeContents(el);
    const lines = new Set(
      [...range.getClientRects()].map((rect) => Math.round(rect.top)),
    ).size;
    return {
      truncated: el.scrollWidth > el.clientWidth + 1,
      lines,
    };
  });
  expect(wrap.truncated, 'ファンド名は省略しない').toBe(false);
  expect(wrap.lines, 'ファンド名は折り返して全文を出す').toBeGreaterThanOrEqual(2);

  await shoot(page, 'fund-name-390');
});

test('マイナスの損益は data-negative で赤字、太さは正の値と同じ font-semibold', async ({
  page,
}) => {
  await mockApi(page);
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(
        paginatedResponse([
          {
            ...DOMESTIC,
            realized_profit_and_loss: -5000,
            realized_profit_and_loss_after_tax: -3985,
          },
        ]),
      ),
    }),
  );
  await page.goto('/receipts');
  await page.getByRole('tab', { name: '国内株式' }).click();

  const card = page
    .getByTestId('receipt-card-list')
    .getByTestId('receipt-card')
    .first();
  await expect(card).toBeVisible();
  const ddFor = (label: string) =>
    card
      .locator('dl > div')
      .filter({ has: page.getByText(label, { exact: true }) })
      .locator('dd');

  for (const [label, value] of [
    ['損益', '-¥5,000'],
    ['税引後', '-¥3,985'],
  ] as const) {
    const amount = ddFor(label).locator('span[data-negative="true"]');
    await expect(amount).toHaveCount(1);
    await expect(amount).toHaveText(value);
    await expect(amount).toHaveCSS('color', 'rgb(185, 28, 28)');
    await expect(amount).toHaveCSS('font-weight', '600');
  }

  // 損益系以外の金額は data-negative を付けず、太さも揃う
  const proceeds = ddFor('売却額').locator('span:not([data-negative])');
  await expect(proceeds).toHaveText('¥55,000');
  await expect(proceeds).toHaveCSS('font-weight', '600');
});

test('見出しが年月でないグループではカードの日付を年付きで出す', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  const card = cardList.getByTestId('receipt-card').first();
  await expect(card).toBeVisible();
  // 年月見出し(2024年3月)では年は見出し側にあり MM/DD で足りる
  await expect(card.getByText('03/01', { exact: true })).toBeVisible();
  await expect(cardList.getByText('1件', { exact: true })).toBeVisible();

  await page.getByTestId('receipt-search-toggle').click();
  await page.locator('#securities-search').selectOption('7203');
  await expect(
    cardList.getByRole('button', { name: /トヨタ自動車 1件 税引後 ¥2,391/ }),
  ).toBeVisible();

  // 見出しが銘柄名に変わると年が分からなくなるので YYYY/MM/DD で出す
  await expect(card.getByText('2024/03/01', { exact: true })).toBeVisible();
  await expect(card.getByText('03/01', { exact: true })).toHaveCount(0);
});

test('国内株式は1件日の小計を省き、複数件日は税額を含む集計を残す', async ({ page }) => {
  await mockApi(page);
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginatedResponse([
        DOMESTIC,
        { ...DOMESTIC, id: 'second', trade_date: '2024-02-02' },
        { ...DOMESTIC, id: 'third', trade_date: '2024-02-02' },
      ])),
    }),
  );
  await page.goto('/receipts');
  await page.getByRole('tab', { name: /国内株式/ }).click();
  const list = page.getByTestId('receipt-card-list');
  await expect(list.getByTestId('receipt-card')).toHaveCount(3);
  const groups = list.getByTestId('receipt-card-group');
  const single = groups.filter({ hasText: '2024年2月1日' });
  await expect(single.getByText('1件', { exact: true })).toBeVisible();
  await expect(single.locator('[aria-expanded]')).toHaveCount(0);
  const multiple = groups.filter({ hasText: '2024年2月2日' });
  const toggle = multiple.getByRole('button', { name: /2024年2月2日 2件 税引後/ });
  await toggle.click();
  await expect(multiple.getByRole('region').getByText('税額', { exact: true })).toBeVisible();
  const profit = multiple.getByRole('region').locator('dl > div')
    .filter({ has: page.getByText('損益', { exact: true }) }).locator('dd');
  await expect(profit).toHaveText('¥10,000');
  await page.setViewportSize({ width: 1280, height: 900 });
  const table = page.getByRole('table');
  const singleHeading = table.locator('tbody tr').filter({ hasText: '2024年2月1日' });
  await expect(singleHeading.locator('td:visible')).toHaveCount(1);
  await expect(singleHeading.getByText(/^¥/)).toHaveCount(0);
  const multipleHeading = table.locator('tbody tr').filter({ hasText: '2024年2月2日' });
  await expect(multipleHeading.locator('td:not([colspan])')).toHaveText([
    '¥10,000', '¥0', '¥10,000',
  ]);
});

test('長い口座名は省略せず折り返して全文を出す', async ({ page }) => {
  const longAccount =
    'SBI証券 東京本店第一営業部 特定口座(新NISA成長投資枠・つみたて投資枠・iDeCo・ジュニアNISA兼用) 管理番号1234567890';
  await mockApi(page);
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(
        paginatedResponse([{ ...DOMESTIC, account: longAccount }]),
      ),
    }),
  );
  await page.goto('/receipts');
  await page.getByRole('tab', { name: '国内株式' }).click();

  const card = page
    .getByTestId('receipt-card-list')
    .getByTestId('receipt-card')
    .first();
  const account = card.locator('span.flex-1');
  await expect(account).toHaveText(longAccount);

  const metrics = await account.evaluate((el) => {
    const range = document.createRange();
    range.selectNodeContents(el);
    const lines = new Set(
      [...range.getClientRects()].map((rect) => Math.round(rect.top)),
    ).size;
    return {
      clipped: el.scrollWidth > el.clientWidth + 1,
      lines,
    };
  });
  expect(metrics.clipped, '口座名は省略しない').toBe(false);
  expect(metrics.lines, '口座名は折り返して全文を出す').toBeGreaterThanOrEqual(2);
});
