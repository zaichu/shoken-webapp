import type { Page } from '@playwright/test';
import { expect, test } from './support/test';
import { domesticStocksFixture, mutualFundsFixture } from './__fixtures__/receipts-print';

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

  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await expect(page.getByRole('region', { name: 'CSV取り込み・削除' })).toBeVisible();
});

test('画面幅を変えてもレールと明細が重ならず表が収まる', async ({ page }) => {
  await page.setViewportSize({ width: 1023, height: 900 });
  await gotoReceipts(page);
  const rail = page.getByTestId('receipt-utility-rail');
  const main = page.getByTestId('receipt-main-stage');
  for (const width of [1023, 1024, 1280]) {
    await page.setViewportSize({ width, height: 900 });
    await expect(rail).toBeVisible();
    await expect(main).toBeVisible();
    const railBox = (await rail.boundingBox())!;
    const mainBox = (await main.boundingBox())!;
    const separated = railBox.y + railBox.height <= mainBox.y + 1 ||
      mainBox.x + mainBox.width <= railBox.x + 1;
    expect(separated, 'レールと明細が重ならない').toBe(true);
    const tableFits = await page.getByRole('table').evaluate((table) => {
      const wrapper = table.parentElement!;
      return wrapper.scrollWidth <= wrapper.clientWidth + 1;
    });
    expect(tableFits).toBe(true);
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(width + 1);
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
  expect(railBox!.x).toBeGreaterThanOrEqual(mainBox!.x + mainBox!.width - 1);

  await expect(rail.getByTestId('search-card')).toBeVisible();
  await expect(rail.getByRole('region', { name: 'CSV取り込み・削除' })).toBeVisible();
  await expect(page.getByTestId('receipt-csv-toggle')).toBeHidden();
  await expect(main.getByTestId('receipt-summary-strip')).toBeVisible();
  await expect(main.getByRole('table')).toBeVisible();

  // 表は内部スクロールせずページごとスクロールし、ページ幅を広げない
  const tableScroll = await page.getByRole('table').evaluate((table) => {
    const wrapper = table.parentElement!;
    return {
      width: wrapper.clientWidth,
      scrollWidth: wrapper.scrollWidth,
      height: wrapper.clientHeight,
      scrollHeight: wrapper.scrollHeight,
    };
  });
  expect(tableScroll.scrollWidth).toBeLessThanOrEqual(tableScroll.width + 1);
  expect(tableScroll.scrollHeight).toBeLessThanOrEqual(tableScroll.height + 1);

  // ページ自体は横にはみ出さない
  const documentWidth = await page.evaluate(
    () => document.documentElement.scrollWidth,
  );
  expect(documentWidth).toBeLessThanOrEqual(1920);

  // 印刷時も高さ制限とスクロールはなく全行を出力する
  await page.emulateMedia({ media: 'print' });
  const printWrap = await page.getByRole('table').evaluate((table) => {
    const wrapper = table.parentElement!;
    return { height: wrapper.clientHeight, scrollHeight: wrapper.scrollHeight };
  });
  expect(printWrap.scrollHeight).toBeLessThanOrEqual(printWrap.height + 1);
  await page.emulateMedia({ media: 'screen' });

  // 全件削除はレール内のボタン(全幅の赤枠ではない)
  const deleteButton = rail.getByRole('button', { name: /全件削除/ });
  await expect(deleteButton).toBeVisible();
  const deleteBox = await deleteButton.boundingBox();
  expect(deleteBox).not.toBeNull();
  expect(deleteBox!.width).toBeLessThanOrEqual(railBox!.width);
  await expect(page.getByRole('button', { name: /全件削除/ })).toHaveCount(1);
});

test('A4の印字可能領域に全タブの右端の列を収めて印刷できる', async ({ page }) => {
  await page.goto('/receipts');
  await expect(page.getByRole('table')).toBeVisible();

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
        const cardRect = element.closest('[data-testid="receipt-card"]')?.getBoundingClientRect();
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
        await expect(table.locator('tbody tr:last-child td[data-negative="true"]').first())
          .toHaveText('-¥876,543,210,987');
      }

      // 行数を固定せず、金額の全桁がセル内で読めることを確かめる
      const amounts = await table.locator('td').evaluateAll((cells) =>
        cells.filter((cell) => /^-?¥/.test(cell.textContent?.trim() ?? '')).map((cell) => {
          const range = document.createRange();
          range.selectNodeContents(cell);
          const bounds = cell.getBoundingClientRect();
          const rects = [...range.getClientRects()];
          return {
            text: cell.textContent?.trim(),
            fits: rects.length > 0 && rects.every((rect) =>
              rect.left >= bounds.left - 1 && rect.right <= bounds.right + 1 &&
              rect.top >= bounds.top - 1 && rect.bottom <= bounds.bottom + 1),
          };
        }),
      );
      expect(amounts.length).toBeGreaterThan(0);
      for (const amount of amounts) {
        expect(amount.fits, amount.text).toBe(true);
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
