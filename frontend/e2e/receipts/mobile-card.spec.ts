import type { Page } from '@playwright/test';
import { expect, test, paginated } from '../support/test';
import * as path from 'path';

test.use({ viewport: { width: 390, height: 844 } });

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

async function mockApi(page: Page) {
  await page.route(/\/api\/v1\//, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated([])),
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
      body: JSON.stringify(paginated([DIVIDEND])),
    }),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated([DOMESTIC])),
    }),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated([FUND])),
    }),
  );
}

test('CSVプレビューで内容が同じ行も明細行ごとに1件ずつ出す', async ({ page }) => {
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
  await page.goto('/receipts/csv?kind=dividend');

  const fileInput = page.getByTestId('csv-file-input');
  await expect(fileInput).toBeEnabled();
  await fileInput.setInputFiles(
    path.resolve(
      test.info().project.testDir,
      'fixtures/csv/dividend-base.csv',
    ),
  );

  const cardList = page.getByTestId('receipt-card-list');
  const cards = cardList.getByTestId('receipt-card');
  await expect(cards).toHaveCount(2);

  for (const index of [0, 1]) {
    const card = cards.nth(index);
    await expect(card.locator('[aria-expanded]')).toHaveCount(0);
    await expect(
      card.getByRole('button', { name: 'トヨタ自動車(7203) をコピー' }),
    ).toBeVisible();
    await expect(card.locator('.receipt-compact-net')).toHaveText('¥2,391');
  }
});

test('行一覧はスマホ幅に収まり、長いファンド名は省略せず表示する', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  await expect(cardList).toBeVisible();
  const listBox = await cardList.boundingBox();
  expect(listBox, '一覧の幅').not.toBeNull();
  expect(listBox!.x).toBeGreaterThanOrEqual(0);
  expect(listBox!.x + listBox!.width).toBeLessThanOrEqual(390 + 1);

  await page.getByRole('tab', { name: /投資信託/ }).click();
  const name = cardList.getByRole('button', {
    name: 'eMAXIS Slim 全世界株式(オール・カントリー) をコピー',
  });
  await expect(name).toBeVisible();
  const nameBox = await name.boundingBox();
  expect(nameBox, 'ファンド名ボタンの幅').not.toBeNull();
  expect(nameBox!.x).toBeGreaterThanOrEqual(listBox!.x - 1);
  expect(nameBox!.x + nameBox!.width).toBeLessThanOrEqual(listBox!.x + listBox!.width + 1);

  const wrap = await name.locator('span').first().evaluate((el) => {
    const range = document.createRange();
    range.selectNodeContents(el);
    const lines = new Set(
      [...range.getClientRects()].map((rect) => Math.round(rect.top)),
    ).size;
    return {
      truncated: el.scrollWidth > el.clientWidth + 1 || el.scrollHeight > el.clientHeight + 1,
      lines,
    };
  });
  expect(wrap.truncated, 'ファンド名は省略しない').toBe(false);
  expect(wrap.lines, 'ファンド名が描画される').toBeGreaterThan(0);
});

test('マイナスの損益は符号と色を保ち、税引後だけが負になる', async ({ page }) => {
  await mockApi(page);
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(
        paginated([
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
  await expect(card.locator('.receipt-compact-net')).toHaveText('-¥3,985');
  await expect(
    card.locator('.receipt-compact-net span[data-negative="true"]'),
  ).toHaveCount(1);
  // 売却額は損益ではないので負の装飾を付けない
  await expect(card.locator('.receipt-compact-primary')).toContainText('¥50,000');
  await expect(
    card.locator('.receipt-compact-primary span[data-negative]'),
  ).toHaveCount(0);
});

test('行の日付は常に年付きで出す', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const card = page.getByTestId('receipt-card-list').getByTestId('receipt-card').first();
  await expect(card).toBeVisible();
  await expect(card.locator('.receipt-compact-facts')).toContainText('2024/03/01');
  await expect(card.getByText('03/01', { exact: true })).toHaveCount(0);
});

test('国内株式は1件日の小計を省き、複数件日は小計を残す', async ({ page }) => {
  await mockApi(page);
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated([
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
  await expect(single).toContainText('1件');
  const multiple = groups.filter({ hasText: '2024年2月2日' });
  await expect(multiple).toContainText('2件');

  await page.setViewportSize({ width: 1280, height: 900 });
  const table = page.getByRole('table');
  const singleHeading = table.locator('tbody tr').filter({ hasText: '2024年2月1日' });
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
        paginated([{ ...DOMESTIC, account: longAccount }]),
      ),
    }),
  );
  await page.goto('/receipts');
  await page.getByRole('tab', { name: '国内株式' }).click();

  const facts = page
    .getByTestId('receipt-card-list')
    .getByTestId('receipt-card')
    .first()
    .locator('.receipt-compact-facts');
  await expect(facts).toContainText(longAccount);

  // 1行目から溢れた分は折り返して出す
  const metrics = await facts.evaluate((el) => {
    const range = document.createRange();
    range.selectNodeContents(el);
    return {
      clipped: el.scrollHeight > el.clientHeight + 1,
      lines: new Set(
        [...range.getClientRects()].map((rect) => Math.round(rect.top)),
      ).size,
    };
  });
  expect(metrics.clipped, '口座名は省略しない').toBe(false);
  expect(metrics.lines, '口座名は折り返して全文を出す').toBeGreaterThanOrEqual(2);
});