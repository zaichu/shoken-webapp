import { expect, test, type Page } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

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

// 年ピッカーがレール下端をはみ出す高さになるよう、異なる年の明細を多めに用意する
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
    shares: String(100 * ((i % 5) + 1)),
    dividends_before_tax: String(3000 + i * 1234),
    taxes: String(609 + i * 250),
    net_amount_received: String(2391 + i * 984),
    created_at: '2024-03-01T00:00:00Z',
    updated_at: '2024-03-01T00:00:00Z',
  };
});

const DOMESTIC_STOCKS = [
  {
    id: 'domestic-print',
    user_id: MOCK_USER.id,
    trade_date: '2024-03-01',
    settlement_date: '2024-03-04',
    security_code: '8306',
    security_name: '三菱UFJフィナンシャル・グループ',
    account: '特定口座',
    shares: 123456,
    asked_price: 12345,
    proceeds: 123456789,
    purchase_price: 22345,
    realized_profit_and_loss: -123456789,
    taxes: 0,
    realized_profit_and_loss_after_tax: -123456789,
    created_at: '2024-03-01T00:00:00Z',
    updated_at: '2024-03-01T00:00:00Z',
  },
];

const MUTUAL_FUNDS = [
  {
    id: 'mutual-fund-print',
    user_id: MOCK_USER.id,
    trade_date: '2024-03-01',
    settlement_date: '2024-03-04',
    fund_name: 'eMAXIS Slim 全世界株式（オール・カントリー）',
    account: '特定口座',
    shares: '123456',
    exchange_rate: '1',
    cancellation_unit_price_yen: '12345',
    cancellation_amount_yen: '123456789',
    average_acquisition_price_yen: '22345',
    dividends: '0',
    realized_profit_and_loss: '-123456789',
    taxes: '0',
    realized_profit_and_loss_after_tax: '-123456789',
    created_at: '2024-03-01T00:00:00Z',
    updated_at: '2024-03-01T00:00:00Z',
  },
];

const PRINTABLE_A4_VIEWPORTS = [
  { name: 'portrait', width: 718, height: 1047 },
  { name: 'landscape', width: 1047, height: 718 },
] as const;

function paginated(data: unknown[]) {
  return { data, total: data.length, page: 1, per_page: Math.max(data.length, 1) };
}

function json(body: unknown) {
  return { status: 200, contentType: 'application/json', body: JSON.stringify(body) };
}

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

async function shoot(page: Page, name: string) {
  const dir = path.resolve(test.info().project.testDir, '../../.playwright-mcp');
  await fs.promises.mkdir(dir, { recursive: true });
  await page.waitForTimeout(500);
  await page.screenshot({ path: path.join(dir, `${name}.png`), fullPage: true });
}

async function shootPrintPreview(page: Page, orientation: string) {
  const dir = path.resolve(test.info().project.testDir, '../../.playwright-mcp/pr1088');
  await fs.promises.mkdir(dir, { recursive: true });
  await page.screenshot({
    path: path.join(dir, `receipts-a4-${orientation}.png`),
    fullPage: true,
  });
}

test.beforeEach(async ({ page }) => {
  await mockApi(page);
});

test('390px ではモバイル表示を維持する(カード表示・CSV折り畳み・レール上段)', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  await expect(cardList).toBeVisible();
  await expect(cardList.getByTestId('receipt-card')).toHaveCount(DIVIDENDS.length);
  await expect(page.getByRole('table')).toBeHidden();

  // スマホでは rail が main より上(CSV帯→検索→集計→カード)
  const rail = page.getByTestId('receipt-utility-rail');
  const main = page.getByTestId('receipt-main-stage');
  const railBox = await rail.boundingBox();
  const mainBox = await main.boundingBox();
  expect(railBox).not.toBeNull();
  expect(mainBox).not.toBeNull();
  expect(railBox!.y).toBeLessThan(mainBox!.y);

  const toggle = page.getByTestId('receipt-csv-toggle');
  await expect(toggle).toBeVisible();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(page.getByRole('region', { name: 'CSV取り込み・削除' })).toBeHidden();

  await shoot(page, 'leptos-962-390');

  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await expect(page.getByRole('region', { name: 'CSV取り込み・削除' })).toBeVisible();
});

test('1023px は1カラム、1024px で右レール2カラム(19rem)、1280px で20remに広がる', async ({ page }) => {
  await page.setViewportSize({ width: 1023, height: 900 });
  await gotoReceipts(page);

  const rail = page.getByTestId('receipt-utility-rail');
  const main = page.getByTestId('receipt-main-stage');
  let railBox = await rail.boundingBox();
  let mainBox = await main.boundingBox();
  expect(railBox).not.toBeNull();
  expect(mainBox).not.toBeNull();
  expect(railBox!.y).toBeLessThan(mainBox!.y);
  await shoot(page, 'leptos-962-1023');

  await page.setViewportSize({ width: 1024, height: 900 });
  railBox = await rail.boundingBox();
  mainBox = await main.boundingBox();
  expect(railBox).not.toBeNull();
  expect(mainBox).not.toBeNull();
  expect(railBox!.x).toBeGreaterThan(mainBox!.x);
  expect(railBox!.width).toBeGreaterThanOrEqual(295);
  expect(railBox!.width).toBeLessThanOrEqual(315);

  // 狭い主列では優先度の低い列を隠し、表は横スクロールせずに収まる
  const tableScroll = await page.getByRole('table').evaluate((table) => {
    const wrapper = table.parentElement;
    if (!wrapper) return { overflowX: '', scrollable: true };
    return {
      overflowX: getComputedStyle(wrapper).overflowX,
      scrollable: wrapper.scrollWidth > wrapper.clientWidth,
    };
  });
  expect(tableScroll.overflowX).toBe('visible');
  expect(tableScroll.scrollable).toBe(false);
  await shoot(page, 'leptos-962-1024');

  await page.setViewportSize({ width: 1280, height: 900 });
  railBox = await rail.boundingBox();
  expect(railBox).not.toBeNull();
  expect(railBox!.width).toBeGreaterThanOrEqual(310);
  expect(railBox!.width).toBeLessThanOrEqual(330);
});

test('1920px では集計+表の左列と CSV+検索の右レールになる', async ({ page }) => {
  await page.setViewportSize({ width: 1920, height: 1080 });
  await gotoReceipts(page);

  // DOM 順は rail 先(キーボード・読み上げ順)、見た目は order で main 先に戻す
  const domOrder = await page
    .getByTestId('receipt-workspace')
    .evaluate((el) =>
      Array.from(el.children).map((child) => (child as HTMLElement).dataset.testid),
    );
  expect(domOrder).toEqual(['receipt-utility-rail', 'receipt-main-stage']);

  const rail = page.getByTestId('receipt-utility-rail');
  const main = page.getByTestId('receipt-main-stage');
  const railBox = await rail.boundingBox();
  const mainBox = await main.boundingBox();
  expect(railBox).not.toBeNull();
  expect(mainBox).not.toBeNull();
  expect(railBox!.x).toBeGreaterThan(mainBox!.x);
  expect(railBox!.width).toBeGreaterThanOrEqual(310);
  expect(railBox!.width).toBeLessThanOrEqual(330);

  await expect(rail.getByTestId('search-card')).toBeVisible();
  await expect(rail.getByRole('region', { name: 'CSV取り込み・削除' })).toBeVisible();
  await expect(page.getByTestId('receipt-csv-toggle')).toBeHidden();
  await expect(main.getByTestId('receipt-summary-strip')).toBeVisible();
  await expect(main.getByRole('table')).toBeVisible();

  // 表は内部スクロールせずページごとスクロールし、ページ幅を広げない
  const tableScroll = await page
    .getByRole('table')
    .evaluate((table) => {
      const wrapper = table.parentElement;
      if (!wrapper) return { overflowX: '', overflowY: '', maxHeight: '' };
      const style = getComputedStyle(wrapper);
      return {
        overflowX: style.overflowX,
        overflowY: style.overflowY,
        maxHeight: style.maxHeight,
      };
    });
  expect(tableScroll.overflowX).toBe('visible');
  expect(tableScroll.overflowY).toBe('visible');
  expect(tableScroll.maxHeight).toBe('none');

  // ページ自体は横にはみ出さない
  const documentWidth = await page.evaluate(
    () => document.documentElement.scrollWidth,
  );
  expect(documentWidth).toBeLessThanOrEqual(1920);

  // 印刷時も高さ制限とスクロールはなく全行を出力する
  await page.emulateMedia({ media: 'print' });
  const printWrap = await page.getByRole('table').evaluate((table) => {
    const wrapper = table.parentElement;
    if (!wrapper) return { overflowY: '', maxHeight: '' };
    const style = getComputedStyle(wrapper);
    return { overflowY: style.overflowY, maxHeight: style.maxHeight };
  });
  expect(printWrap.overflowY).toBe('visible');
  expect(printWrap.maxHeight).toBe('none');
  await page.emulateMedia({ media: 'screen' });

  // 全件削除はレール内のボタン(全幅の赤枠ではない)
  const deleteButton = rail.getByRole('button', { name: /全件削除/ });
  await expect(deleteButton).toBeVisible();
  const deleteBox = await deleteButton.boundingBox();
  expect(deleteBox).not.toBeNull();
  expect(deleteBox!.width).toBeLessThanOrEqual(railBox!.width);
  await expect(page.getByRole('button', { name: /全件削除/ })).toHaveCount(1);

  await shoot(page, 'leptos-962-1920');
});

test('A4の印字可能領域に全タブの右端の列を収めて印刷できる', async ({ page }) => {
  await page.goto('/receipts');
  await expect(page.getByRole('table')).toBeVisible();

  const pageRule = await page.evaluate(() => {
    const findPageRule = (rules: CSSRuleList): string | undefined => {
      for (const rule of rules) {
        if (rule.cssText.startsWith('@page')) return rule.cssText;
        if ('cssRules' in rule) {
          const nested = findPageRule((rule as CSSGroupingRule).cssRules);
          if (nested) return nested;
        }
      }
    };
    for (const sheet of document.styleSheets) {
      const found = findPageRule(sheet.cssRules);
      if (found) return found;
    }
  });
  expect(pageRule).toContain('size: a4');
  expect(pageRule).toContain('margin: 10mm');

  for (const paper of PRINTABLE_A4_VIEWPORTS) {
    await page.setViewportSize(paper);
    for (const tab of [
      { label: '配当金', slug: 'dividend', count: String(DIVIDENDS.length) },
      { label: '国内株式', slug: 'domesticstock', count: '1' },
      { label: '投資信託', slug: 'mutualfund', count: '1' },
    ]) {
      await page.emulateMedia({ media: 'screen' });
      await page.getByRole('tab', { name: tab.label }).click();
      await expect(page.getByTestId(`tab-count-${tab.slug}`)).toHaveText(tab.count);
      await page.emulateMedia({ media: 'print' });

      const table = page.getByRole('table');
      await expect(table).toBeVisible();
      await expect(page.getByTestId('receipt-utility-rail')).toBeHidden();
      const metrics = await table.evaluate((element) => {
        const main = document.querySelector('main');
        const mainRect = main?.getBoundingClientRect();
        const mainStyle = main ? getComputedStyle(main) : undefined;
        const tableRect = element.getBoundingClientRect();
        const cardRect = element.closest('.table-card')?.getBoundingClientRect();
        const lastCellRect = element
          .querySelector('tbody tr:last-child td:last-child')
          ?.getBoundingClientRect();
        return {
          tableLeft: tableRect.left,
          tableRight: tableRect.right,
          cardRight: cardRect?.right ?? 0,
          lastCellRight: lastCellRect?.right ?? 0,
          mainContentLeft:
            (mainRect?.left ?? 0) + Number.parseFloat(mainStyle?.paddingLeft ?? '0'),
          mainContentRight:
            (mainRect?.right ?? 0) - Number.parseFloat(mainStyle?.paddingRight ?? '0'),
        };
      });

      expect(metrics.tableLeft).toBeGreaterThanOrEqual(metrics.mainContentLeft - 1);
      expect(metrics.tableRight).toBeLessThanOrEqual(metrics.cardRight + 1);
      expect(metrics.cardRight).toBeLessThanOrEqual(metrics.mainContentRight + 1);
      expect(metrics.lastCellRight).toBeLessThanOrEqual(metrics.mainContentRight + 1);

      if (tab.slug === 'domesticstock') {
        const negativeMetrics = await table
          .locator('tbody tr:last-child td[data-negative="true"]')
          .first()
          .evaluate((cell) => {
            const range = document.createRange();
            range.selectNodeContents(cell);
            const lineTops = [...range.getClientRects()].map((rect) => Math.round(rect.top));
            return {
              text: cell.textContent?.trim(),
              whiteSpace: getComputedStyle(cell).whiteSpace,
              lineCount: new Set(lineTops).size,
            };
          });
        expect(negativeMetrics.text).toBe('-¥123,456,789');
        expect(negativeMetrics.whiteSpace).toBe('nowrap');
        expect(negativeMetrics.lineCount).toBe(1);
        await shootPrintPreview(page, paper.name);
      }
    }
  }
});

test('640px 以上で年ピッカーの選択肢がレール下端を超えても末尾の年を選べる', async ({ page }) => {
  for (const width of [768, 1280]) {
    await page.setViewportSize({ width, height: 900 });
    await gotoReceipts(page);

    const trigger = page.getByRole('button', { name: '年を選択' });
    await trigger.click();
    const listbox = page.getByRole('listbox', { name: '年候補' });
    await expect(listbox).toBeVisible();

    // 末尾の年も実際にクリックできる(768px の1カラム幅ではドロップダウンが下の表と重なり得る)
    const lastOption = listbox.getByRole('option').last();
    const yearLabel = (await lastOption.textContent())!.trim();
    await lastOption.click({ timeout: 5000 });
    await expect(trigger).toContainText(yearLabel);
  }
});
