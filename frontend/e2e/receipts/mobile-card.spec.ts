import type { Page } from '@playwright/test';
import { expect, test, paginated, mockSession } from '../support/test';
import * as path from 'path';

test.use({ viewport: { width: 390, height: 844 } });

const DIVIDEND = {
  id: '00000000-0000-0000-0000-000000000001',
  settlement_date: '2024-03-01',
  product: '特定口座',
  account: 'SBI証券',
  security_code: '9904',
  security_name: '架空自動車',
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
  fund_name: '架空インデックス 全世界株式(オール・ワールド)',
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
  account: '特定口座',
  security_code: '9905',
  security_name: '架空ゲーム',
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
  await mockSession(page);
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
  const name = cardList.locator('.list-row-name').first();
  await expect(name).toBeVisible();
  await expect(name).toContainText('架空インデックス');
  const nameBox = await name.boundingBox();
  expect(nameBox, 'ファンド名ボタンの幅').not.toBeNull();
  expect(nameBox!.x).toBeGreaterThanOrEqual(listBox!.x - 1);
  expect(nameBox!.x + nameBox!.width).toBeLessThanOrEqual(listBox!.x + listBox!.width + 1);

  const wrap = await name.evaluate((el) => {
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

test('マイナスの損益は符号と色を保つ', async ({ page }) => {
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
            taxes: 0,
            realized_profit_and_loss_after_tax: -5000,
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
  await expect(card.locator('.list-row-primary')).toHaveText('-¥5,000');
  await expect(
    card.locator('.list-row-primary [data-negative="true"]'),
  ).toHaveCount(1);
  // 補助の売却額は行に出さない
  await expect(card.locator('.list-row-secondary')).toHaveCount(0);
  // 詳細ダイアログには売却額を出す
  await card.getByTestId('receipt-row-button').click();
  const dialog = page.getByTestId('receipt-detail-dialog');
  await expect(dialog).toBeVisible();
  await expect(dialog).toContainText('売却額');
  await expect(dialog).toContainText('¥55,000');
  await dialog.getByTestId('detail-dialog-close').click();
});

test('行の日付は月/日で出す', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const card = page.getByTestId('receipt-card-list').getByTestId('receipt-card').first();
  await expect(card).toBeVisible();
  await expect(card.locator('.list-row-meta')).toContainText('3/1');
});

test('国内株式は月単位でまとめ、日ごとの見出しは出さない', async ({ page }) => {
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
  await expect(groups).toHaveCount(1);
  await expect(groups.first()).toContainText('2024年2月');
  await expect(groups.first()).toContainText('3件');
  await expect(groups.first()).not.toContainText('2024年2月1日');
  await expect(groups.first()).not.toContainText('2024年2月2日');

  await page.setViewportSize({ width: 1280, height: 900 });
  const table = page.getByRole('table');
  const monthHeading = table.locator('tbody tr').filter({ hasText: '2024年2月' });
  await expect(monthHeading.first()).toBeVisible();
  await expect(monthHeading.first().locator('td:not([colspan])')).toHaveText([
    '+¥15,000', '-¥3,045', '+¥11,955',
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

  const meta = page
    .getByTestId('receipt-card-list')
    .getByTestId('receipt-card')
    .first()
    .locator('.list-row-meta');
  await expect(meta).toContainText(longAccount);

  // 1行目から溢れた分は折り返して出す
  const metrics = await meta.evaluate((el) => {
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