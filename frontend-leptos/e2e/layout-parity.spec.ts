import {
  expect,
  test,
  type BrowserContext,
  type Locator,
  type Page,
  type TestInfo,
} from '@playwright/test';
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
  ['6861', 'キーエンス'],
  ['8058', '三菱商事'],
];

const DIVIDENDS = Array.from({ length: 18 }, (_, i) => {
  const [code, name] = STOCKS[i % STOCKS.length];
  const month = String((i % 12) + 1).padStart(2, '0');
  const day = String((i % 27) + 1).padStart(2, '0');
  return {
    id: `dividend-${i}`,
    user_id: MOCK_USER.id,
    settlement_date: `2024-${month}-${day}`,
    product: i % 2 === 0 ? '特定口座' : 'NISA口座',
    account: i % 3 === 0 ? 'SBI証券' : '楽天証券',
    security_code: code,
    security_name: name,
    unit_price: 10 + i * 3,
    shares: 100 * ((i % 5) + 1),
    dividends_before_tax: 3000 + i * 1234,
    taxes: 609 + i * 250,
    net_amount_received: 2391 + i * 984,
    created_at: '2024-03-01T00:00:00Z',
    updated_at: '2024-03-01T00:00:00Z',
  };
});

const DOMESTIC_STOCKS = Array.from({ length: 15 }, (_, i) => {
  const [code, name] = STOCKS[i % STOCKS.length];
  const month = String((i % 12) + 1).padStart(2, '0');
  const day = String((i % 27) + 1).padStart(2, '0');
  return {
    id: `domestic-${i}`,
    trade_date: `2024-${month}-${day}`,
    settlement_date: `2024-${month}-${day}`,
    security_code: code,
    security_name: name,
    account: i % 2 === 0 ? '特定口座' : 'NISA口座',
    shares: 100 * ((i % 4) + 1),
    asked_price: 2800 + i * 150,
    proceeds: 280000 + i * 15000,
    purchase_price: 2500 + i * 120,
    realized_profit_and_loss: 30000 - i * 2500,
    taxes: 6090 - i * 400,
    realized_profit_and_loss_after_tax: 23910 - i * 2100,
    created_at: '2024-05-10T00:00:00Z',
    updated_at: '2024-05-10T00:00:00Z',
  };
});

const MUTUALFUNDS = Array.from({ length: 12 }, (_, i) => {
  const month = String((i % 12) + 1).padStart(2, '0');
  const day = String((i % 27) + 1).padStart(2, '0');
  return {
    id: `mutualfund-${i}`,
    trade_date: `2024-${month}-${day}`,
    settlement_date: `2024-${month}-${day}`,
    fund_name:
      i % 2 === 0
        ? 'eMAXIS Slim 全世界株式（オール・カントリー）'
        : 'SBI・V・S&P500インデックス・ファンド',
    account: i % 2 === 0 ? '特定口座' : 'NISA口座',
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
  };
});

const ASSET_BALANCES = Array.from({ length: 8 }, (_, i) => {
  const [code, name] = STOCKS[i % STOCKS.length];
  return {
    id: `asset-balance-${i}`,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: code,
    security_name: name,
    shares: 100 * (i + 1),
    executing_shares: 0,
    average_purchase_price: 2500 + i * 300,
    total_purchase_amount: 250000 + i * 120000,
    current_price: 2600 + i * 310,
    daily_change: 50 - i * 12,
    market_value: 260000 + i * 125000,
    profit_loss_rate: 4.0 - i * 0.7,
  };
});

const TABLE_SPEC = {
  dividend: {
    tabName: '配当金',
    widths: [84, 64, 64, 72, 160, 72, 56, 84, 64, 84],
  },
  domesticstock: {
    tabName: '国内株式',
    widths: [84, 72, 156, 60, 56, 76, 82, 82, 82, 64, 84],
  },
  mutualfund: {
    tabName: '投資信託',
    widths: [112, 300, 60, 112, 98, 128, 116, 112, 106, 118],
  },
} as const;

type ReceiptTabSlug = keyof typeof TABLE_SPEC;
const TABS = Object.keys(TABLE_SPEC) as ReceiptTabSlug[];

const TAB_COUNTS: Record<ReceiptTabSlug, number> = {
  dividend: DIVIDENDS.length,
  domesticstock: DOMESTIC_STOCKS.length,
  mutualfund: MUTUALFUNDS.length,
};

function paginated(data: unknown[]) {
  return { data, total: data.length, page: 1, per_page: Math.max(data.length, 1) };
}

function json(body: unknown) {
  return { status: 200, contentType: 'application/json', body: JSON.stringify(body) };
}

// context 単位でモックするため、比較用に同じ context へ追加したページにも効く
async function mockApi(context: BrowserContext) {
  // 後から登録した route が優先されるため catch-all を先に登録する
  await context.route(/\/api\/v1\//, (route) => route.fulfill(json(paginated([]))));
  await context.route(/\/api\/v1\/session$/, (route) => route.fulfill(json(MOCK_USER)));
  await context.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DIVIDENDS))),
  );
  await context.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DOMESTIC_STOCKS))),
  );
  await context.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(MUTUALFUNDS))),
  );
  await context.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(ASSET_BALANCES))),
  );
  await context.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(
      json({
        items: STOCKS.map(([code]) => ({
          security_code: code,
          dividend_per_share: 30,
          status: 'ok',
          is_stale: false,
        })),
      }),
    ),
  );
}

async function shoot(page: Page, testInfo: TestInfo, name: string) {
  const dir = path.resolve(testInfo.project.testDir, '../../.playwright-mcp');
  await fs.promises.mkdir(dir, { recursive: true });
  await page.screenshot({ path: path.join(dir, `${name}.png`), fullPage: true });
}

// 同名テキストは別ブレークポイント用の hidden カードにも存在するため visible で絞る
async function expectAssetDataLoaded(page: Page) {
  await expect(
    page
      .getByTestId('assetbalance-main-stage')
      .getByText('トヨタ自動車')
      .filter({ visible: true })
      .first(),
  ).toBeVisible();
}

async function selectReceiptTab(page: Page, slug: ReceiptTabSlug) {
  const spec = TABLE_SPEC[slug];
  await page.getByRole('tab', { name: spec.tabName }).click();
  await expect(page.getByRole('tab', { name: spec.tabName })).toHaveAttribute(
    'aria-selected',
    'true',
  );
  // 件数バッジはタブ別フェッチの完了を意味する
  await expect(page.getByTestId(`tab-count-${slug}`)).toHaveText(String(TAB_COUNTS[slug]));
}

interface TableMetrics {
  layout: string;
  tableWidth: number;
  clientWidth: number;
  scrollWidth: number;
  overflowX: string;
}

async function tableMetrics(page: Page): Promise<TableMetrics> {
  return page.getByRole('table').evaluate((table) => {
    const wrapper = table.parentElement;
    if (!wrapper) {
      return { layout: '', tableWidth: 0, clientWidth: 0, scrollWidth: 0, overflowX: '' };
    }
    const style = getComputedStyle(wrapper);
    return {
      layout: getComputedStyle(table).tableLayout,
      tableWidth: table.offsetWidth,
      clientWidth: wrapper.clientWidth,
      scrollWidth: wrapper.scrollWidth,
      overflowX: style.overflowX,
    };
  });
}

// table-fixed の表の実幅は max(列幅合計, コンテナ幅)。
// 収まる幅では横スクロールなし、収まらない幅では列幅を保ったまま内部スクロールする
function expectReactTableFit(metrics: TableMetrics, widths: readonly number[]) {
  const sum = widths.reduce((a, b) => a + b, 0);
  expect(metrics.layout).toBe('fixed');
  expect(metrics.overflowX).toBe('auto');
  expect(Math.abs(metrics.tableWidth - Math.max(sum, metrics.clientWidth))).toBeLessThanOrEqual(2);
  expect(Math.abs(metrics.scrollWidth - metrics.tableWidth)).toBeLessThanOrEqual(2);
  if (sum <= metrics.clientWidth) {
    expect(metrics.scrollWidth).toBeLessThanOrEqual(metrics.clientWidth + 1);
  } else {
    expect(metrics.scrollWidth).toBeGreaterThan(metrics.clientWidth);
  }
}

async function expectColumnWidthsAndEllipsis(page: Page, widths: readonly number[]) {
  const headerStyles = await page
    .getByRole('table')
    .locator('thead th')
    .evaluateAll((cells) =>
      cells.map((cell) => {
        const el = cell as HTMLElement;
        const style = getComputedStyle(el);
        return {
          width: el.style.width,
          maxWidth: el.style.maxWidth,
          overflow: style.overflow,
          textOverflow: style.textOverflow,
          whiteSpace: style.whiteSpace,
        };
      }),
    );
  expect(headerStyles).toHaveLength(widths.length);
  headerStyles.forEach((style, i) => {
    expect(style.width, `th[${i}] の固定幅`).toBe(`${widths[i]}px`);
    expect(style.maxWidth, `th[${i}] の最大幅`).toBe(`${widths[i]}px`);
    expect(style.overflow).toBe('hidden');
    expect(style.textOverflow).toBe('ellipsis');
    expect(style.whiteSpace).toBe('nowrap');
  });

  const cellStylesOf = (row: Locator) =>
    row.locator('td').evaluateAll((cells) =>
      cells.map((cell) => {
        const el = cell as HTMLTableCellElement;
        const style = getComputedStyle(el);
        return {
          colSpan: el.colSpan,
          overflow: style.overflow,
          textOverflow: style.textOverflow,
          whiteSpace: style.whiteSpace,
        };
      }),
    );
  const expectEllipsis = (styles: { overflow: string; textOverflow: string; whiteSpace: string }[]) =>
    styles.forEach((style) => {
      expect(style.overflow).toBe('hidden');
      expect(style.textOverflow).toBe('ellipsis');
      expect(style.whiteSpace).toBe('nowrap');
    });

  // 先頭行はグループ集計行。結合セルと小計セルにも省略指定があることを確認する
  const summaryStyles = await cellStylesOf(
    page.getByRole('table').locator('tbody tr').first(),
  );
  expect(summaryStyles.length).toBeGreaterThan(1);
  expect(summaryStyles[0].colSpan, '集計行の先頭は結合セル').toBeGreaterThan(1);
  expectEllipsis(summaryStyles);

  // 末尾行は必ず明細行なので、本文セル側の省略指定をそこで確認する
  const cellStyles = await cellStylesOf(page.getByRole('table').locator('tbody tr').last());
  expect(cellStyles.length).toBe(widths.length);
  expectEllipsis(cellStyles);
}

async function expectNoPageOverflow(page: Page, width: number) {
  const scrollWidth = await page.evaluate(() => document.documentElement.scrollWidth);
  expect(scrollWidth, 'ページ自体は横にはみ出さない').toBeLessThanOrEqual(width + 1);
}

async function expectNoPageSurface(page: Page) {
  await expect(page.locator('.page-surface')).toHaveCount(0);
  // ページ見出しが main の直接の子(=カードに包まれていない)ことを確認する
  await expect(page.locator('#main-content > *').first().locator('h1')).toBeVisible();
}

// スクロールラッパーの max-height は計測後に入るので、数値が入るまでを描画完了の合図にする
async function expectTableSettled(page: Page) {
  await expect(page.getByRole('table').locator('xpath=..')).toHaveAttribute(
    'style',
    /max-height:\s*[\d.]+px/,
  );
}

test.beforeEach(async ({ context }) => {
  await mockApi(context);
});

for (const width of [1440, 1024]) {
  for (const slug of TABS) {
    test(`取引明細テーブルはカード内に収まるかReact同様に内部スクロールする(${slug} ${width}px)`, async ({
      page,
    }, testInfo) => {
      await page.setViewportSize({ width, height: 900 });
      await page.goto('/receipts');
      await selectReceiptTab(page, slug);

      const table = page.getByRole('table');
      await expect(table).toBeVisible();
      await expect(table.locator('tbody tr').first()).toBeVisible();
      await expectNoPageSurface(page);
      await expectNoPageOverflow(page, width);

      const spec = TABLE_SPEC[slug];
      await expectColumnWidthsAndEllipsis(page, spec.widths);
      expectReactTableFit(await tableMetrics(page), spec.widths);
      await expectTableSettled(page);

      await shoot(page, testInfo, `receipts-${slug}-data-${width}-leptos`);
    });
  }
}

test('口座検索で列が前に出ても列幅は列に追随する(国内株式 1440px)', async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto('/receipts');
  await selectReceiptTab(page, 'domesticstock');
  await page
    .getByTestId('search-card')
    .getByRole('button', { name: '特定口座', exact: true })
    .click();
  const reordered = [84, 72, 60, 156, 56, 76, 82, 82, 82, 64, 84];
  const ths = page.getByRole('table').locator('thead th');
  await expect(ths.nth(2)).toHaveText('口座');
  await expect(ths.nth(3)).toHaveText('銘柄名');
  await expectColumnWidthsAndEllipsis(page, reordered);
  expectReactTableFit(await tableMetrics(page), reordered);
  await expectTableSettled(page);
  await shoot(page, testInfo, 'receipts-domesticstock-search-1440-leptos');
});

test('取引明細・資産管理は page-surface で包まれない', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  for (const [path, title] of [
    ['/receipts', '取引明細'],
    ['/assetbalance', '資産管理'],
  ] as const) {
    await page.goto(path);
    await expect(page.locator('#main-content h1').first()).toHaveText(title);
    await expectNoPageSurface(page);
    await expectNoPageOverflow(page, 1440);
  }
  // 同じ .page-surface を使う /search は影響を受けていないことの防衛チェック
  await page.goto('/search');
  await expect(page.locator('.page-surface')).toHaveCount(1);
  await shoot(page, testInfo, 'search-empty-1440-leptos');
});

for (const width of [1440, 1024, 390]) {
  test(`資産管理のページ構成がReactと同じ(${width}px)`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await page.goto('/assetbalance');
    await expect(page.locator('#main-content h1').first()).toHaveText('資産管理');
    await expect(page.getByTestId('assetbalance-workspace')).toBeVisible();
    await expectAssetDataLoaded(page);
    await expectNoPageSurface(page);
    await expectNoPageOverflow(page, width);
    await shoot(page, testInfo, `assetbalance-data-${width}-leptos`);
  });
}

test('取引明細 390px はカード表示で page-surface もページはみ出しもない', async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/receipts');
  await expect(page.getByRole('tab')).toHaveCount(3);

  const cardList = page.getByTestId('receipt-card-list');
  for (const slug of TABS) {
    await selectReceiptTab(page, slug);
    await expect(cardList.getByTestId('receipt-card')).toHaveCount(TAB_COUNTS[slug]);
    await expect(page.getByRole('table')).toBeHidden();
    await expectNoPageSurface(page);
    await expectNoPageOverflow(page, 390);
    await shoot(page, testInfo, `receipts-${slug}-data-390-leptos`);
  }
});

test.describe('react比較', () => {
  const reactBase = process.env.REACT_BASE_URL;
  const leptosBase = `http://127.0.0.1:${process.env.LEPTOS_E2E_PORT ?? '8081'}`;
  test.skip(!reactBase, 'REACT_BASE_URL 未指定時はスキップ');
  test.use({ baseURL: reactBase ?? 'http://127.0.0.1:5187' });

  for (const width of [1440, 1024]) {
    for (const slug of TABS) {
      test(`react receipts ${slug} ${width}px`, async ({ page }, testInfo) => {
        await page.setViewportSize({ width, height: 900 });
        await page.goto('/receipts');
        await selectReceiptTab(page, slug);
        const table = page.getByRole('table');
        await expect(table).toBeVisible();
        await expect(table.locator('tbody tr').first()).toBeVisible();
        expectReactTableFit(await tableMetrics(page), TABLE_SPEC[slug].widths);
        await shoot(page, testInfo, `receipts-${slug}-data-${width}-react`);
      });
    }
    test(`react assetbalance ${width}px`, async ({ page }, testInfo) => {
      await page.setViewportSize({ width, height: 900 });
      await page.goto('/assetbalance');
      await expect(page.locator('#main-content h1').first()).toHaveText('資産管理');
      await expectAssetDataLoaded(page);
      await shoot(page, testInfo, `assetbalance-data-${width}-react`);
    });
  }

  test(`react receipts/assetbalance 390px`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto('/receipts');
    await expect(page.getByRole('tab')).toHaveCount(3);
    for (const slug of TABS) {
      await selectReceiptTab(page, slug);
      await shoot(page, testInfo, `receipts-${slug}-data-390-react`);
    }
    await page.goto('/assetbalance');
    await expect(page.locator('#main-content h1').first()).toHaveText('資産管理');
    await expectAssetDataLoaded(page);
    await shoot(page, testInfo, 'assetbalance-data-390-react');
  });

  test('表の実幅と横スクロール量は React 版と一致する', async ({ page, context }) => {
    const leptos = await context.newPage();
    for (const width of [1440, 1024]) {
      for (const slug of TABS) {
        for (const [target, base] of [
          [page, ''],
          [leptos, leptosBase],
        ] as const) {
          await target.setViewportSize({ width, height: 900 });
          await target.goto(`${base}/receipts`);
          await selectReceiptTab(target, slug);
          const table = target.getByRole('table');
          await expect(table).toBeVisible();
          await expect(table.locator('tbody tr').first()).toBeVisible();
        }
        const react = await tableMetrics(page);
        const leptosMetrics = await tableMetrics(leptos);
        expect(
          Math.abs(react.tableWidth - leptosMetrics.tableWidth),
          `${slug} ${width}px: 表の実幅が React とずれている`,
        ).toBeLessThanOrEqual(2);
        expect(
          Math.abs(react.scrollWidth - leptosMetrics.scrollWidth),
          `${slug} ${width}px: 横スクロール量が React とずれている`,
        ).toBeLessThanOrEqual(2);
      }
    }
    await leptos.close();
  });
});
