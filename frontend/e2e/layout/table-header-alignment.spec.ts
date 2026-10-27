import type { Locator, Page } from '@playwright/test';
import { expect, test, json, paginated, MOCK_USER, mockSession } from '../support/test';

/**
 * 表の列見出しの規則を確かめる。
 * 見出しは列のデータの位置と同じ(文字位置=left/right が一致)、
 * 中央そろえは使わない、並べ替えの前後で見出しの文字が動かない。
 */

const STOCK_NAMES = [
  '架空自動車',
  '架空精密機器',
  '架空通信ホールディングス',
  '架空フィナンシャル・グループ',
];

const STOCKS = Array.from({ length: 12 }, (_, i) => ({
  id: `asset-balance-${i}`,
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: String(9900 + i),
  security_name: `${STOCK_NAMES[i % STOCK_NAMES.length]}${i}号`,
  shares: 100 * (i + 1),
  average_purchase_price: 2500 + i * 130,
  total_purchase_amount: 250000 + i * 33000,
  current_price: 2600 + i * 170,
  daily_change: 50 - i * 7,
  market_value: 260000 + i * 41000,
  profit_loss_rate: 4.0 - i * 0.4,
  account: i % 3 === 0 ? '特定・一般' : i % 3 === 1 ? 'NISA' : '一般',
}));

const FUNDS = Array.from({ length: 8 }, (_, i) => ({
  id: `fund-${i}`,
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  fund_name: '架空インデックス 全世界株式（オール・ワールド）投信ファンド第3号',
  account: i % 2 === 0 ? '特定・一般' : 'NISA',
  dividends_course: '再投資型',
  shares: 10000 + i * 1500,
  average_purchase_price: 10000 + i * 120,
  total_purchase_amount: 100000 + i * 12000,
  base_price: 11000 + i * 90,
  base_price_day_change: 50 - i * 9,
  base_price_month_change: null,
  market_value: 110000 + i * 22000,
  profit_loss: 10000 + i * 800,
  profit_loss_rate: 10.0 - i * 0.4,
  total_return: 10000 + i * 800,
  price_as_of: '2026-01-05',
  price_source: 'api',
  nav_is_latest: true,
  expected_nav_date: null,
}));

const DIVIDENDS = Array.from({ length: 12 }, (_, i) => ({
  id: `dividend-${i}`,
  user_id: MOCK_USER.id,
  settlement_date: `2024-12-${String((i % 27) + 1).padStart(2, '0')}`,
  product: '国内株式',
  account: i % 3 === 0 ? '特定・一般' : '一般',
  security_code: String(9900 + i),
  security_name: `${STOCK_NAMES[i % STOCK_NAMES.length]}${i}号`,
  unit_price: 10 + i * 3,
  shares: 100 * ((i % 5) + 1),
  dividends_before_tax: 3000 + i * 1234,
  taxes: 609 + i * 250,
  net_amount_received: 2391 + i * 984,
  created_at: '2024-03-01T00:00:00Z',
  updated_at: '2024-03-01T00:00:00Z',
}));

const DOMESTIC_STOCKS = Array.from({ length: 12 }, (_, i) => ({
  id: `domestic-${i}`,
  trade_date: `2024-12-${String((i % 27) + 1).padStart(2, '0')}`,
  settlement_date: `2024-12-${String((i % 27) + 1).padStart(2, '0')}`,
  security_code: String(9900 + i),
  security_name: `${STOCK_NAMES[i % STOCK_NAMES.length]}${i}号`,
  account: i % 3 === 0 ? '特定・一般' : '一般',
  shares: 100 * ((i % 4) + 1),
  asked_price: 2800 + i * 150,
  proceeds: 280000 + i * 15000,
  purchase_price: 2500 + i * 120,
  realized_profit_and_loss: 30000 - i * 2500,
  taxes: 6090 - i * 400,
  realized_profit_and_loss_after_tax: 23910 - i * 2100,
  created_at: '2024-05-10T00:00:00Z',
  updated_at: '2024-05-10T00:00:00Z',
}));

const MUTUALFUNDS = Array.from({ length: 12 }, (_, i) => ({
  id: `mutualfund-${i}`,
  trade_date: `2024-12-${String((i % 27) + 1).padStart(2, '0')}`,
  settlement_date: `2024-12-${String((i % 27) + 1).padStart(2, '0')}`,
  fund_name: '架空インデックス 全世界株式（オール・ワールド）投信ファンド第3号',
  account: i % 3 === 0 ? '特定・一般' : '一般',
  shares: 10000 + i * 1500,
  exchange_rate: 1,
  cancellation_unit_price_yen: 21000 + i * 800,
  cancellation_amount_yen: 210000 + i * 12000,
  average_acquisition_price_yen: 18000 + i * 600,
  dividends: '0',
  realized_profit_and_loss: 30000 - i * 1800,
  taxes: 6090 - i * 350,
  realized_profit_and_loss_after_tax: 23910 - i * 1450,
  created_at: '2024-04-02T00:00:00Z',
  updated_at: '2024-04-02T00:00:00Z',
}));

async function mockApi(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(STOCKS))),
  );
  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(FUNDS))),
  );
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DIVIDENDS))),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DOMESTIC_STOCKS))),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(MUTUALFUNDS))),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(
      json({
        items: STOCKS.map((row) => ({
          security_code: row.security_code,
          dividend_per_share: 30,
          status: 'ok',
          is_stale: false,
        })),
      }),
    ),
  );
}

type Column = {
  header: string;
  headerAlign: string;
  cellAlign: string | null;
  /** 見出しの文字の左右位置 */
  headerText: { left: number; right: number } | null;
  /** データセルの内容の左右位置(余白を除いた部分) */
  cellContent: { left: number; right: number } | null;
};

/** 見出しの文字の位置と、同じ列のデータの位置を比べる */
async function readColumns(table: Locator): Promise<Column[]> {
  return table.evaluate((el) => {
    const ths = Array.from(el.querySelectorAll('thead th')) as HTMLTableCellElement[];
    const rows = Array.from(el.querySelectorAll('tbody tr')) as HTMLTableRowElement[];
    const dataRow = rows.find(
      (tr) =>
        tr.children.length === ths.length &&
        !Array.from(tr.children).some((cell) => (cell as HTMLTableCellElement).colSpan > 1),
    );
    const cells = dataRow
      ? (Array.from(dataRow.children) as HTMLTableCellElement[])
      : [];
    // 並べ替えの矢印は別の要素なので、文字は最後の本文ノードで取る
    const textBox = (node: Element) => {
      const walker = document.createTreeWalker(node, NodeFilter.SHOW_TEXT);
      let last: Node | null = null;
      let current: Node | null = null;
      while ((current = walker.nextNode())) {
        if ((current.textContent ?? '').trim().length > 0) {
          last = current;
        }
      }
      if (!last) {
        return null;
      }
      const range = document.createRange();
      range.selectNodeContents(last);
      const rect = range.getBoundingClientRect();
      return { left: rect.left, right: rect.right };
    };
    return ths.map((th, index) => {
      const cell = cells[index] ?? null;
      const cellStyle = cell ? getComputedStyle(cell) : null;
      const cellRect = cell?.getBoundingClientRect() ?? null;
      return {
        header: (th.textContent ?? '').trim(),
        headerAlign: getComputedStyle(th).textAlign,
        cellAlign: cell ? cellStyle!.textAlign : null,
        headerText: textBox(th),
        cellContent:
          cellRect && cellStyle
            ? {
                left: cellRect.left + parseFloat(cellStyle.paddingLeft),
                right: cellRect.right - parseFloat(cellStyle.paddingRight),
              }
            : null,
      };
    });
  });
}

async function expectHeaderFollowsCells(table: Locator, label: string) {
  const columns = await readColumns(table);
  expect(columns.length, `${label}: 見出しがある`).toBeGreaterThan(0);
  for (const column of columns) {
    expect(column.cellAlign, `${label}: ${column.header} にデータ行がある`).not.toBeNull();
    expect(
      column.headerAlign,
      `${label}: ${column.header} の見出しとセルの位置が一致する`,
    ).toBe(column.cellAlign);
    expect(
      column.headerAlign,
      `${label}: ${column.header} は中央に寄せない`,
    ).not.toBe('center');
    expect(column.headerText, `${label}: ${column.header} に文字がある`).not.toBeNull();
    expect(column.cellContent, `${label}: ${column.header} にデータがある`).not.toBeNull();
    // 右そろえの列は文字の右端が、数据的右端と揃う
    if (column.headerAlign === 'right') {
      expect(
        Math.abs(column.headerText!.right - column.cellContent!.right),
        `${label}: ${column.header} の見出しの右端がデータと揃う`,
      ).toBeLessThanOrEqual(2);
    } else {
      expect(
        Math.abs(column.headerText!.left - column.cellContent!.left),
        `${label}: ${column.header} の見出しの左端がデータと揃う`,
      ).toBeLessThanOrEqual(2);
    }
  }
}

/** 見出しの文字(並べ替えの矢印を除く)の左右位置 */
async function headerTextBox(header: Locator) {
  return header.evaluate((el) => {
    const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
    let last: Node | null = null;
    let current: Node | null = null;
    while ((current = walker.nextNode())) {
      if ((current.textContent ?? '').trim().length > 0) {
        last = current;
      }
    }
    if (!last) {
      return null;
    }
    const range = document.createRange();
    range.selectNodeContents(last);
    const rect = range.getBoundingClientRect();
    return { left: rect.left, right: rect.right };
  });
}

/** 並べ替えの前後で見出しの文字が動かない */
async function expectSortKeepsHeaderText(
  table: Locator,
  headerName: string,
) {
  const header = table.getByRole('columnheader', { name: headerName });
  const button = header.getByRole('button');
  await expect(button).toBeVisible();
  const before = await headerTextBox(header);
  expect(before, `${headerName} に文字がある`).not.toBeNull();

  const state = async () =>
    `${await header.getAttribute('aria-sort')}|${await button.getAttribute('aria-pressed')}`;
  const beforeState = await state();
  await button.click();
  // 並べ替えの状態が変わるまで待つ(順序だけ反転しても状態は変わる)
  await expect.poll(state).not.toBe(beforeState);

  const after = await headerTextBox(header);
  expect(after, `${headerName} に文字がある`).not.toBeNull();
  expect(after!.left, `${headerName} の文字の左位置が動かない`).toBeCloseTo(
    before!.left,
    1,
  );
  expect(after!.right, `${headerName} の文字の右位置が動かない`).toBeCloseTo(
    before!.right,
    1,
  );
}

test.beforeEach(async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await mockApi(page);
});

test('資産管理・国内株式の表', async ({ page }) => {
  await page.goto('/assetbalance');
  const table = page.getByTestId('asset-holdings-table').locator('table');
  await expect(table).toBeVisible();
  await expectHeaderFollowsCells(table, '資産管理・国内株式');
  await expectSortKeepsHeaderText(table, '評価額');
});

test('資産管理・投資信託の表', async ({ page }) => {
  await page.goto('/assetbalance');
  await page.getByTestId('asset-kind-tab-fund').click();
  const table = page.getByTestId('asset-fund-table').locator('table');
  await expect(table).toBeVisible();
  await expectHeaderFollowsCells(table, '資産管理・投資信託');
  await expectSortKeepsHeaderText(table, '評価額');
});

test('資産管理・すべての表', async ({ page }) => {
  await page.goto('/assetbalance');
  await page.getByTestId('asset-kind-tab-all').click();
  const table = page.getByTestId('asset-merged-table').locator('table');
  await expect(table).toBeVisible();
  await expectHeaderFollowsCells(table, '資産管理・すべて');
  await expectSortKeepsHeaderText(table, '評価損益');
});

for (const [label, tab] of [
  ['取引明細・配当金', '配当金'],
  ['取引明細・国内株式', '国内株式'],
  ['取引明細・投資信託', '投資信託'],
] as const) {
  test(`${label}の表`, async ({ page }) => {
    await page.goto('/receipts');
    await page.getByRole('tab', { name: tab }).click();
    const table = page.getByRole('table');
    await expect(table).toBeVisible();
    await expectHeaderFollowsCells(table, label);
  });
}

test('取引明細のCSVプレビューの表', async ({ page }) => {
  await page.route(/\/api\/v1\/dividend-import-validations$/, (route) =>
    route.fulfill(
      json({
        total_rows: DIVIDENDS.length,
        valid_rows: DIVIDENDS.length,
        errors: [],
        rows: DIVIDENDS,
      }),
    ),
  );
  await page.goto('/csv?kind=dividend');
  const input = page.getByTestId('csv-file-input');
  await expect(input).toBeEnabled();
  await input.setInputFiles({
    name: 'dividend.csv',
    mimeType: 'text/csv',
    buffer: Buffer.from('入金日\n2024/12/24\n', 'utf-8'),
  });
  const table = page.getByTestId('csv-preview').getByRole('table');
  await expect(table).toBeVisible();
  await expectHeaderFollowsCells(table, '取引明細のCSVプレビュー');
});

test('資産管理のCSVプレビューの表', async ({ page }) => {
  await page.route(/\/api\/v1\/asset-balance-import-validations$/, (route) =>
    route.fulfill(
      json({
        total_rows: STOCKS.length,
        valid_rows: STOCKS.length,
        errors: [],
        rows: STOCKS,
      }),
    ),
  );
  await page.goto('/csv?kind=assetbalance');
  const input = page.getByTestId('csv-file-input');
  await expect(input).toBeEnabled();
  await input.setInputFiles({
    name: 'assetbalance.csv',
    mimeType: 'text/csv',
    buffer: Buffer.from('銘柄コード\n9900\n', 'utf-8'),
  });
  const table = page.getByTestId('csv-preview').getByRole('table');
  await expect(table).toBeVisible();
  await expectHeaderFollowsCells(table, '資産管理のCSVプレビュー');
});
