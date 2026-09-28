import { expect, test, type Page } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';
import {
  domesticStocksFixture,
  LONG_FUND_NAME,
  mutualFundsFixture,
} from './__fixtures__/receipts-print';

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

const DOMESTIC_STOCKS = domesticStocksFixture(MOCK_USER.id);

const MUTUAL_FUNDS = mutualFundsFixture(MOCK_USER.id);

const PRINTABLE_A4_VIEWPORTS = [
  { name: 'portrait', width: 718, height: 1047 },
  { name: 'landscape', width: 1047, height: 718 },
] as const;

// narrow は lg(1024px)で出す列、wide は xl(1280px)で追加される列。
// 配当金の数量は lg から出す。国内株式・投資信託は列が多く銘柄名も長いため、
// xl で数量まで出すと名前が2行に収まらず、2xl からにとどめる
const RECEIPT_TABS = [
  {
    label: '配当金',
    slug: 'dividend',
    count: DIVIDENDS.length,
    nameHeader: '銘柄名',
    narrow: ['入金日', '銘柄コード', '銘柄名', '数量', '配当金', '税額', '受取額'],
    wide: ['口座', '単価'],
  },
  {
    label: '国内株式',
    slug: 'domesticstock',
    count: DOMESTIC_STOCKS.length,
    nameHeader: '銘柄名',
    narrow: ['約定日', '銘柄コード', '銘柄名', '損益', '税額', '税引後'],
    wide: ['売却単価', '売却額'],
  },
  {
    label: '投資信託',
    slug: 'mutualfund',
    count: MUTUAL_FUNDS.length,
    nameHeader: 'ファンド名',
    narrow: ['約定日', 'ファンド名', '解約額', '実現損益', '税額', '税引損益'],
    wide: ['解約単価', '取得価額'],
  },
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

test('1024px・1280px・1440pxで表示セルがはみ出さず、銘柄名は2行まで表示する', async ({ page }) => {
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(
      json(
        paginated(
          DIVIDENDS.map((row) => ({
            ...row,
            account: '特定・一般',
            unit_price: '27400',
            shares: '1950',
            dividends_before_tax: '294460',
            taxes: '28590',
            net_amount_received: '265870',
          })),
        ),
      ),
    ),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(
      json(
        paginated(
          DOMESTIC_STOCKS.map((row) => ({
            ...row,
            account: '特定・一般',
            shares: 1950,
            asked_price: 27400,
            proceeds: 294460,
            purchase_price: 22800,
            realized_profit_and_loss: -180000,
            taxes: 18000,
            realized_profit_and_loss_after_tax: -198000,
          })),
        ),
      ),
    ),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(
      json(
        paginated(
          MUTUAL_FUNDS.map((row) => ({
            ...row,
            account: '特定・一般',
            shares: '1950',
            cancellation_unit_price_yen: '27400',
            cancellation_amount_yen: '306000',
            average_acquisition_price_yen: '22800',
            realized_profit_and_loss: '-10000',
            taxes: '0',
            realized_profit_and_loss_after_tax: '-10000',
          })),
        ),
      ),
    ),
  );

  for (const width of [1024, 1280, 1440]) {
    await page.setViewportSize({ width, height: 900 });
    await page.goto('/receipts');

    for (const tab of RECEIPT_TABS) {
      await page.getByRole('tab', { name: tab.label }).click();
      await expect(page.getByTestId(`tab-count-${tab.slug}`)).toHaveText(String(tab.count));

      const metrics = await page.getByRole('table').evaluate((table, { nameHeader, width }) => {
        const headers = Array.from(table.tHead?.rows[0]?.cells ?? []);
        const rows = Array.from(table.tBodies[0]?.rows ?? []).filter(
          (candidate) =>
            candidate.cells.length === headers.length && candidate.querySelector('.copyable-name'),
        );
        if (rows.length === 0) throw new Error('取引明細のデータ行がありません');

        const nameIndex = headers.findIndex((header) => header.textContent?.trim() === nameHeader);
        const overflows = rows.flatMap((row) =>
          headers.flatMap((header, index) => {
            const cell = row.cells[index];
            if (
              index === nameIndex ||
              getComputedStyle(header).display === 'none' ||
              getComputedStyle(cell).display === 'none' ||
              cell.scrollWidth <= cell.clientWidth
            ) {
              return [];
            }
            return [
              {
                header: header.textContent?.trim(),
                text: cell.textContent?.trim(),
                title: cell.getAttribute('title'),
                clientWidth: cell.clientWidth,
                scrollWidth: cell.scrollWidth,
              },
            ];
          }),
        );
        const headerClips = headers.flatMap((header) => {
          if (
            getComputedStyle(header).display === 'none' ||
            header.scrollWidth <= header.clientWidth
          ) {
            return [];
          }
          return [header.textContent?.trim()];
        });
        const nameCells = rows.map((row) => row.cells[nameIndex]);
        // 1行に収まる短い名前では、コピーボタンは文字幅だけを占める(セル全幅ではない)
        const copyWidths = nameCells.flatMap((cell) => {
          const button = cell.querySelector('.copyable-name');
          const span = button?.querySelector('span');
          if (!button || !span) return [];
          const range = document.createRange();
          range.selectNodeContents(span);
          const lines = new Set(
            [...range.getClientRects()].map((rect) => Math.round(rect.top)),
          ).size;
          if (lines > 1) return [];
          return [
            {
              cellWidth: cell.clientWidth,
              buttonWidth: (button as HTMLElement).offsetWidth,
            },
          ];
        });
        const firstNameCell = nameCells[0];
        const nameText = firstNameCell
          .querySelector('.copyable-name > span')
          ?.textContent?.trim();
        const nameStyle = getComputedStyle(
          firstNameCell.querySelector('.copyable-name > span') as HTMLElement,
        );
        // 2行クランプで切り捨てられた行(2行に収まらない極端に長い名前)を拾う。
        // 幅の狭い 1024px では長いファンド名が 2 行に収まらず省略されることがあるが、
        // その場合でも td の title に全文が残る必要がある
        const clampedNames = nameCells.flatMap((cell) => {
          const span = cell.querySelector('.copyable-name > span');
          if (!span || span.scrollHeight <= span.clientHeight) return [];
          return [{ text: span.textContent?.trim(), title: cell.getAttribute('title') }];
        });
        const namelessClips = clampedNames.filter((cell) => cell.title !== cell.text);
        const wrapper = table.parentElement;

        return {
          overflows,
          headerClips,
          visibleHeaders: headers
            .filter((header) => getComputedStyle(header).display !== 'none')
            .map((header) => header.textContent?.trim()),
          copyDeltas: copyWidths.map(({ cellWidth, buttonWidth }) =>
            Math.round(cellWidth - buttonWidth),
          ),
          nameText,
          nameTitle: firstNameCell.getAttribute('title'),
          nameClientWidth: firstNameCell.clientWidth,
          nameOverflow: nameStyle.overflow,
          nameLineClamp: nameStyle.webkitLineClamp,
          nameWhiteSpace: nameStyle.whiteSpace,
          clampedNames,
          namelessClips,
          tableClientWidth: wrapper?.clientWidth ?? 0,
          tableScrollWidth: wrapper?.scrollWidth ?? Number.POSITIVE_INFINITY,
          pageOverflows: document.documentElement.scrollWidth > width,
        };
      }, { nameHeader: tab.nameHeader, width });

      expect(metrics.overflows, `${width}px ${tab.label}`).toEqual([]);
      expect(metrics.headerClips, `${width}px ${tab.label}`).toEqual([]);
      const expectedHeaders = [
        ...tab.narrow,
        ...(width >= 1280 ? tab.wide : []),
      ].sort();
      expect(
        [...metrics.visibleHeaders].sort(),
        `${width}px ${tab.label} の表示列`,
      ).toEqual(expectedHeaders);
      if (metrics.copyDeltas.length > 0) {
        expect(
          Math.min(...metrics.copyDeltas),
          `${width}px ${tab.label} のコピー範囲は文字幅のみ`,
        ).toBeGreaterThan(4);
      }
      expect(metrics.nameTitle).toBe(metrics.nameText);
      expect(metrics.nameClientWidth, `${width}px ${tab.label}`).toBeGreaterThanOrEqual(80);
      expect(metrics.nameOverflow).toBe('hidden');
      expect(metrics.nameLineClamp).toBe('2');
      expect(metrics.nameWhiteSpace).toBe('normal');
      expect(metrics.namelessClips, `${width}px ${tab.label}`).toEqual([]);
      if (width >= 1280) {
        expect(metrics.clampedNames, `${width}px ${tab.label}`).toEqual([]);
      }
      expect(metrics.tableScrollWidth, `${width}px ${tab.label}`).toBeLessThanOrEqual(
        metrics.tableClientWidth,
      );
      expect(metrics.pageOverflows).toBe(false);
      await shoot(page, `receipts-after-${tab.slug}-${width}`);
    }
  }
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
  // 長いファンド名は印刷で行数制限を外して全文を出すことを確かめるため、投資信託にだけ差し込む
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
  expect(pageRule).toContain('margin: 10mm');
  expect(pageRule).not.toContain('size');

  for (const paper of PRINTABLE_A4_VIEWPORTS) {
    await page.setViewportSize(paper);
    for (const tab of [
      { label: '配当金', slug: 'dividend', count: String(DIVIDENDS.length) },
      { label: '国内株式', slug: 'domesticstock', count: String(DOMESTIC_STOCKS.length) },
      { label: '投資信託', slug: 'mutualfund', count: String(MUTUAL_FUNDS.length) },
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
        expect(negativeMetrics.text).toBe('-¥876,543,210,987');
        expect(negativeMetrics.whiteSpace).toBe('nowrap');
        expect(negativeMetrics.lineCount).toBe(1);
        await shootPrintPreview(page, paper.name);
      }

      // 長い金額が複数並んでも金額セルは1行を保つ
      const amountLines = await table.locator('td.text-right').evaluateAll((cells) =>
        cells.map((cell) => {
          const range = document.createRange();
          range.selectNodeContents(cell);
          const lineTops = [...range.getClientRects()].map((rect) => Math.round(rect.top));
          return { text: cell.textContent?.trim(), lineCount: new Set(lineTops).size };
        }),
      );
      expect(amountLines.length).toBeGreaterThan(0);
      for (const amount of amountLines) {
        expect(amount.lineCount).toBe(1);
      }

      // 印刷では銘柄名の2行クランプを外し、長い名前も全文を折り返して出す
      const nameMetrics = await table.evaluate((element) =>
        Array.from(
          element.querySelectorAll('.receipt-instrument-cell .copyable-name > span'),
        ).map((span) => {
          const style = getComputedStyle(span);
          const range = document.createRange();
          range.selectNodeContents(span);
          const lineTops = [
            ...range.getClientRects(),
          ].map((rect) => Math.round(rect.top));
          return {
            text: span.textContent?.trim(),
            clamp: style.webkitLineClamp,
            display: style.display,
            overflow: style.overflow,
            clipped: span.scrollHeight > span.clientHeight + 1,
            lineCount: new Set(lineTops).size,
          };
        }),
      );
      for (const name of nameMetrics) {
        expect(name.clamp, `${tab.label} ${name.text}`).toBe('none');
        expect(name.display, `${tab.label} ${name.text}`).not.toBe('-webkit-box');
        expect(name.overflow, `${tab.label} ${name.text}`).toBe('visible');
        expect(name.clipped, `${tab.label} ${name.text}`).toBe(false);
      }
      if (tab.slug === 'mutualfund') {
        // クランプ解除が効いていれば、2行を超える名前も省略されず全行が描画される
        const longFund = nameMetrics.find((name) => name.text === LONG_FUND_NAME);
        expect(longFund, '長いファンド名が印刷 DOM にある').toBeTruthy();
        expect(longFund?.lineCount).toBeGreaterThanOrEqual(2);
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
