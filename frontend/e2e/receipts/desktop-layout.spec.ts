import type { Page } from '@playwright/test';
import { expect, test, json, paginated } from '../support/test';
import {
  domesticStocksFixture,
  LONG_FUND_NAME,
  mutualFundsFixture,
} from '../fixtures/receipts-print';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const STOCKS: Array<[string, string]> = [
  ['7203', 'トヨタ自動車'],
  ['9432', '日本電信電話'],
  ['6758', 'ソニーグループ'],
  ['8306', '三菱UFJフィナンシャル・グループ'],
  ['9984', 'ソフトバンクグループ'],
  ['4502', '武田薬品工業'],
];

const DIVIDENDS = Array.from({ length: 39 }, (_, i) => {
  const [code, name] = STOCKS[i % STOCKS.length];
  const month = String((i % 12) + 1).padStart(2, '0');
  const day = String((i % 27) + 1).padStart(2, '0');
  return {
    id: `dividend-${i}`,
    user_id: MOCK_USER.id,
    settlement_date: `${1995 + i}-${month}-${day}`,
    product: i % 2 === 0 ? '特定口座' : 'NISA口座',
    account: i % 3 === 0 ? 'SBI証券' : '楽天証券',
    security_code: code,
    security_name: name,
    unit_price: String(10 + i * 3),
    shares: String(1000000 + i * 111111),
    dividends_before_tax: String(12345678 + i * 12345),
    taxes: String(2345678 + i * 1234),
    net_amount_received: String(9876543 + i * 5678),
    created_at: '2024-03-01T00:00:00Z',
    updated_at: '2024-03-01T00:00:00Z',
  };
});

const DOMESTIC_STOCKS = domesticStocksFixture(MOCK_USER.id);
const MUTUAL_FUNDS = mutualFundsFixture(MOCK_USER.id);

const RECEIPT_TABS = [
  { label: '配当金', slug: 'dividend', count: DIVIDENDS.length },
  { label: '国内株式', slug: 'domesticstock', count: DOMESTIC_STOCKS.length },
  { label: '投資信託', slug: 'mutualfund', count: MUTUAL_FUNDS.length },
] as const;

async function mockApi(page: Page) {
  // 後から登録した route が優先されるため catch-all を先に登録する
  await page.route(/\/api\/v1\//, (route) => route.fulfill(json(paginated([]))));
  await page.route(/\/api\/v1\/session$/, (route) => route.fulfill(json(MOCK_USER)));
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DIVIDENDS))),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DOMESTIC_STOCKS))),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(MUTUAL_FUNDS))),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ data: [] })),
  );
}

async function gotoReceipts(page: Page) {
  await page.goto('/receipts');
  await expect(page.getByRole('table')).toBeVisible();
}

async function expectNoPageOverflow(page: Page, width: number) {
  const scrollWidth = await page.evaluate(() => document.documentElement.scrollWidth);
  expect(scrollWidth, 'ページ自体は横にはみ出さない').toBeLessThanOrEqual(width + 1);
}

test.beforeEach(async ({ page }) => {
  await mockApi(page);
});

for (const width of [1024, 1280, 1440]) {
  test(`取引明細は表と横1行の検索バーで、どのタブでも幅${width}pxに収まる`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    await gotoReceipts(page);

    await expect(page.getByRole('search', { name: '取引明細の検索' })).toBeVisible();
    await expect(page.getByTestId('receipt-summary-strip')).toBeVisible();
    // 下部ナビと下部の常設操作帯は PC では出さない
    await expect(page.locator('.receipts-mobile-bottom-section')).toBeHidden();
    await expect(page.locator('.app-bottom-nav')).toBeHidden();

    for (const tab of RECEIPT_TABS) {
      await page.getByRole('tab', { name: tab.label }).click();
      await expect(page.getByTestId(`tab-count-${tab.slug}`)).toHaveText(String(tab.count));
      const table = page.getByRole('table');
      await expect(table).toBeVisible();
      const fits = await table.evaluate((element) => {
        const wrapper = element.parentElement!;
        return wrapper.scrollWidth <= wrapper.clientWidth + 1;
      });
      expect(fits, `${tab.label} の表が枠に収まる`).toBe(true);
      await expectNoPageOverflow(page, width);
    }
  });
}

test('PC の検索バーで口座を絞ると一覧・件数・集計が同じ対象になる', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await gotoReceipts(page);

  const account = page.getByRole('button', { name: 'SBI証券', exact: true });
  await expect(account).toHaveAttribute('aria-pressed', 'false');
  await account.click();
  await expect(account).toHaveAttribute('aria-pressed', 'true');

  const expected = DIVIDENDS.filter((row) => row.account === 'SBI証券').length;
  await expect(page.getByTestId('tab-count-dividend')).toHaveText(String(expected));
  const table = page.getByRole('table');
  await expect(table).toContainText('SBI証券');
  await expect(table).not.toContainText('楽天証券');

  // 条件を解除すると件数と一覧が元に戻る
  await page.getByRole('button', { name: '条件をクリア' }).click();
  await expect(page.getByTestId('tab-count-dividend')).toHaveText(
    String(DIVIDENDS.length),
  );
  await expect(table).toContainText('楽天証券');
});

test('390px では2行の行表示になり表とPCの検索バーは隠れる', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  await expect(cardList).toBeVisible();
  await expect(cardList.getByTestId('receipt-card')).toHaveCount(DIVIDENDS.length);
  await expect(page.getByRole('table')).toBeHidden();
  await expect(page.getByRole('search', { name: '取引明細の検索' })).toBeHidden();
  // 検索欄は一覧の下、下部ナビの上に常設する
  await expect(
    page.getByRole('search', { name: '検索', exact: true }).getByPlaceholder('銘柄名・コード'),
  ).toBeVisible();
  await expect(page.getByRole('link', { name: '検索条件' })).toBeVisible();
  await expect(page.locator('.app-bottom-nav')).toBeVisible();
  await expectNoPageOverflow(page, 390);
});

test('A4の印字可能領域に全タブが収まり、操作バーは印刷に出ない', async ({ page }) => {
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(
      json(
        paginated(
          MUTUAL_FUNDS.map((row, index) =>
            index === 1 ? { ...row, fund_name: LONG_FUND_NAME } : row,
          ),
        ),
      ),
    ),
  );
  await page.setViewportSize({ width: 1047, height: 718 });
  await page.goto('/receipts');

  for (const tab of RECEIPT_TABS) {
    await page.emulateMedia({ media: 'screen' });
    await page.getByRole('tab', { name: tab.label }).click();
    await expect(page.getByTestId(`tab-count-${tab.slug}`)).toHaveText(String(tab.count));
    await page.emulateMedia({ media: 'print' });

    const table = page.getByRole('table');
    await expect(table).toBeVisible();
    await expect(page.locator('.pc-header')).toBeHidden();
    await expect(page.locator('.page-frame-head')).toBeHidden();
    await expect(page.getByRole('search', { name: '取引明細の検索' })).toBeHidden();

    const metrics = await table.evaluate((element) => {
      const main = document.querySelector('main');
      const mainRect = main?.getBoundingClientRect();
      const mainStyle = main ? getComputedStyle(main) : undefined;
      const tableRect = element.getBoundingClientRect();
      const lastCellRect = element
        .querySelector('tbody tr:last-child td:last-child')
        ?.getBoundingClientRect();
      return {
        tableLeft: tableRect.left,
        tableRight: tableRect.right,
        lastCellRight: lastCellRect?.right ?? 0,
        mainContentLeft:
          (mainRect?.left ?? 0) + Number.parseFloat(mainStyle?.paddingLeft ?? '0'),
        mainContentRight:
          (mainRect?.right ?? 0) - Number.parseFloat(mainStyle?.paddingRight ?? '0'),
      };
    });
    expect(metrics.tableLeft).toBeGreaterThanOrEqual(metrics.mainContentLeft - 1);
    expect(metrics.tableRight).toBeLessThanOrEqual(metrics.mainContentRight + 1);
    expect(metrics.lastCellRight).toBeLessThanOrEqual(metrics.mainContentRight + 1);
  }
  await page.emulateMedia({ media: 'screen' });
});

test('PCの数値列は折り返さず1行で表示され、表は枠に収まる', async ({ page }) => {
  for (const width of [1024, 1280]) {
    await page.setViewportSize({ width, height: 900 });
    await gotoReceipts(page);

    for (const tab of RECEIPT_TABS) {
      await page.getByRole('tab', { name: tab.label }).click();
      await expect(page.getByTestId(`tab-count-${tab.slug}`)).toHaveText(String(tab.count));
      const table = page.getByRole('table');
      await expect(table).toBeVisible();

      const fits = await table.evaluate((element) => {
        const wrapper = element.parentElement!;
        return wrapper.scrollWidth <= wrapper.clientWidth + 1;
      });
      expect(fits, `${tab.label} の表が枠に収まる`).toBe(true);

      const hasWrappedNumberCell = await page.evaluate(() => {
        // td は box なので自身の getClientRects() は行数に関わらず 1 つ。
        // 中身を Range で囲んで行数を数え、さらに computed style で
        // white-space: nowrap が効いていることを確かめる (CSS の退行検出)。
        const visible = (element: Element) => {
          const rect = element.getBoundingClientRect();
          return rect.width > 0 && rect.height > 0 && (element.textContent ?? '').trim().length > 0;
        };
        const numericCells = Array.from(document.querySelectorAll('td.text-right')).filter(visible);
        return numericCells.some((cell) => {
          const range = document.createRange();
          range.selectNodeContents(cell);
          const lineCount = Array.from(range.getClientRects()).filter((rect) => rect.height > 0).length;
          if (lineCount > 1) return true;
          return getComputedStyle(cell).whiteSpace !== 'nowrap';
        });
      });
      expect(hasWrappedNumberCell, `${tab.label} の数値列が折り返していない`).toBe(false);
      await expectNoPageOverflow(page, width);
    }
  }
});