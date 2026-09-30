import { expect, test, type Page } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';
import {
  domesticStocksFixture,
  LONG_FUND_NAME,
  mutualFundsFixture,
} from './__fixtures__/receipts-print';

function csvFixture(name: string): string {
  return path.resolve(test.info().project.testDir, '__fixtures__/csv', name);
}

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

// 列は帯(画面幅×レール開閉)で段階表示する。core は常に表示、md は md 帯と lg 畳み・
// xl 以降、wide は lg 畳み・xl 以降、wider は xl 畳み・1650px 以降で表示する
const RECEIPT_TABS = [
  {
    label: '配当金',
    slug: 'dividend',
    count: DIVIDENDS.length,
    nameHeader: '銘柄名',
    core: ['入金日', '銘柄名', '配当金', '税額', '受取額'],
    md: ['数量'],
    wide: ['銘柄コード'],
    wider: ['商品', '口座', '単価'],
  },
  {
    label: '国内株式',
    slug: 'domesticstock',
    count: DOMESTIC_STOCKS.length,
    nameHeader: '銘柄名',
    core: ['約定日', '銘柄名', '損益', '税額', '税引後'],
    md: [] as string[],
    wide: ['銘柄コード', '数量'],
    wider: ['口座', '売却単価', '売却額', '取得価額'],
  },
  {
    label: '投資信託',
    slug: 'mutualfund',
    count: MUTUAL_FUNDS.length,
    nameHeader: 'ファンド名',
    core: ['約定日', 'ファンド名', '実現損益', '税額', '税引損益'],
    md: [] as string[],
    wide: ['数量', '解約額'],
    wider: ['口座', '解約単価', '取得価額'],
  },
];

type ReceiptTabSpec = (typeof RECEIPT_TABS)[number];

// width × レール開閉で見える見出し集合を返す
function expectedHeaders(tab: ReceiptTabSpec, width: number, collapsed: boolean) {
  let headers: readonly string[];
  if (width < 768) {
    headers = tab.core;
  } else if (width < 1024) {
    headers = [...tab.core, ...tab.md];
  } else if (width < 1280) {
    headers = collapsed ? [...tab.core, ...tab.md, ...tab.wide] : tab.core;
  } else if (width < 1650) {
    headers = collapsed
      ? [...tab.core, ...tab.md, ...tab.wide, ...tab.wider]
      : [...tab.core, ...tab.md, ...tab.wide];
  } else {
    headers = [...tab.core, ...tab.md, ...tab.wide, ...tab.wider];
  }
  return [...headers].sort();
}

// データがあるとレールは畳まれた状態で始まるので、レール内の UI に触れる前に開く
async function openUtilityRail(page: Page) {
  const toggle = page.getByTestId('receipt-utility-toggle');
  if (await toggle.isVisible()) {
    await toggle.click();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  }
}

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

test('データがあるとレールは畳んだ状態で始まり、開閉で表がページ幅いっぱいに広がる', async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await gotoReceipts(page);

  const toggle = page.getByTestId('receipt-utility-toggle');
  const rail = page.getByTestId('receipt-utility-rail');
  const table = page.getByRole('table');

  // 初期状態: 畳み。aria-expanded/aria-controls がレールと結びついている
  await expect(toggle).toBeVisible();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  const controls = await toggle.getAttribute('aria-controls');
  expect(controls).toBe('receipt-utility-rail-dividend');
  await expect(page.locator(`#${controls}`)).toBeHidden();
  await expect(rail).toBeHidden();

  const collapsedWidth = (await table.boundingBox())!.width;

  // キーボード(Enter)で開ける
  await toggle.focus();
  await page.keyboard.press('Enter');
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await expect(rail).toBeVisible();
  const openWidth = (await table.boundingBox())!.width;
  expect(collapsedWidth - openWidth, '畳むと表がレール分だけ広い').toBeGreaterThan(300);

  // キーボード(Space)でも畳める。開閉の状態は localStorage に保存しない
  const storedBefore = await page.evaluate(() => JSON.stringify(localStorage));
  await toggle.focus();
  await page.keyboard.press(' ');
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(rail).toBeHidden();
  const storedAfter = await page.evaluate(() => JSON.stringify(localStorage));
  expect(storedAfter).toBe(storedBefore);
});

test('0 件のタブではレールは開いた状態で始まり、データのあるタブでは畳む', async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  // 配当金だけ 0 件にする
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([]))),
  );
  await page.goto('/receipts');
  await expect(page.getByText('データがありません')).toBeVisible();

  const toggle = page.getByTestId('receipt-utility-toggle');
  const rail = page.getByTestId('receipt-utility-rail');
  // 0 件なので開いた状態で始まる(CSV 取り込みにすぐ触れる)
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await expect(rail).toBeVisible();
  await expect(rail.getByRole('button', { name: /ファイルを選択/ })).toBeVisible();

  // データのある国内株式へ切り替えると畳んだ状態になる
  await page.getByRole('tab', { name: '国内株式' }).click();
  await expect(page.getByTestId('tab-count-domesticstock')).toHaveText(
    String(DOMESTIC_STOCKS.length),
  );
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(rail).toBeHidden();
});

test('取得失敗のタブではレールが開き、CSV 取り込みと検索に届く', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({ status: 500, contentType: 'application/json', body: '{}' }),
  );
  await gotoReceipts(page);
  const rail = page.getByTestId('receipt-utility-rail');
  // データのある配当金タブでは畳まれた状態で始まる。一度開いて畳み直し、
  // ユーザーが開閉を決定済み(decided)の状態にしておく
  await expect(rail).toBeHidden();
  const toggle = page.getByTestId('receipt-utility-toggle');
  await toggle.click();
  await expect(rail).toBeVisible();
  await toggle.click();
  await expect(rail).toBeHidden();

  // 取得失敗のタブには開閉トグルが描画されないので、畳んだままにすると
  // CSV 取り込み・検索に届かない。ユーザーの開閉状態は変えず表示時だけ開く
  await page.getByRole('tab', { name: '国内株式' }).click();
  await expect(page.getByTestId('list-load-error')).toBeVisible();
  await expect(rail).toBeVisible();
  const fileInput = rail.getByTestId('csv-file-input');
  await expect(fileInput).toBeEnabled();
  await expect(rail.getByTestId('search-card')).toBeVisible();

  // 失敗タブで CSV プレビューが出ると main stage も描画されるが、
  // 表示で強制 open のレールと食い違う開閉トグルは出さない
  await page.route(/\/api\/v1\/domestic-stock-import-validations$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        total_rows: 1,
        valid_rows: 1,
        errors: [],
        rows: [
          {
            trade_date: '2026-01-20',
            settlement_date: '2026-01-23',
            security_code: '9104',
            security_name: '商船三井',
            account: '特定',
            shares: 50,
            asked_price: 5200,
            proceeds: 260000,
            purchase_price: 4800,
            realized_profit_and_loss: 20000,
            taxes: 4063,
            realized_profit_and_loss_after_tax: 15937,
          },
        ],
      }),
    }),
  );
  await fileInput.setInputFiles(csvFixture('domesticstock-base.csv'));
  await expect(page.getByText('1件 追加で保存されます')).toBeVisible();
  await expect(page.getByTestId('receipt-preview-banner')).toBeVisible();
  await expect(page.getByTestId('receipt-utility-toggle')).toHaveCount(0);

  // 戻るとユーザーの決定(畳み)が残り、トグルで開閉できる
  await page.getByRole('tab', { name: '配当金' }).click();
  await expect(page.getByRole('table')).toBeVisible();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(rail).toBeHidden();
  await toggle.click();
  await expect(rail).toBeVisible();
  await toggle.click();
  await expect(rail).toBeHidden();
});

test('レールを畳んでいても絞り込み中は件数が表の上に出る', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await gotoReceipts(page);

  const toggle = page.getByTestId('receipt-utility-toggle');
  // 初期状態は畳み。絞り込んでいないので件数バッジは出ない
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(page.getByText(/絞り込み中/)).toHaveCount(0);

  // レールを開いて口座で絞り込み、畳み直しても件数が残る
  await openUtilityRail(page);
  await page
    .getByTestId('search-card')
    .getByRole('button', { name: 'SBI証券', exact: true })
    .click();
  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(
    page.getByTestId('receipt-utility-toggle-bar').getByText(/絞り込み中 \d+ \/ 39 件/),
  ).toBeVisible();
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

  // データがあるので lg 以上では畳まれた状態で始まる。レールを開いて2カラムを確かめる
  await page.setViewportSize({ width: 1024, height: 900 });
  await openUtilityRail(page);
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

test('768px〜1440px・レール開閉で表示セルがはみ出さず、銘柄名は2行まで表示する', async ({
  page,
}) => {
  // 金額は 7 桁(¥1,234,567)・負の 7 桁(-¥1,234,567)と 9 桁(¥111,111,102)を交互に混ぜ、
  // コードは 10 文字まで切れないことを固定する。集計行は合算で桁数に上限がないため、
  // 国内株式は同日の損失合算で -¥12,345,678,901(負 11 桁、損失なので税引後も同額)、
  // 投資信託は月次合算で ¥1,999,999,998(10桁)を出す。金額セルは省略せず折り返すことを固定する。
  // 加えて別日のペアで集計 -¥123,456,789(負 9 桁)も残し、9 桁以下の金額・集計は
  // 1行に収まること(列幅の方針が効いていること)を行数で固定する
  const domesticRows = [
    ...DOMESTIC_STOCKS.map((row, i) => ({
      ...row,
      account: '特定・一般',
      security_code: '1234567890',
      shares: 12345,
      asked_price: i % 2 === 0 ? 111111102 : 1234567,
      proceeds: i % 2 === 0 ? 111111102 : 1234567,
      purchase_price: i % 2 === 0 ? 111111102 : 1234567,
      realized_profit_and_loss: i % 2 === 0 ? -9999999999 : -2345678902,
      taxes: i % 2 === 0 ? 111111102 : 1234567,
      realized_profit_and_loss_after_tax:
        i % 2 === 0 ? -9999999999 : -2345678902,
    })),
    ...[0, 1].map((i) => ({
      ...DOMESTIC_STOCKS[i],
      id: `domestic-small-${i}`,
      trade_date: '2024-03-02',
      settlement_date: '2024-03-05',
      account: '特定・一般',
      security_code: '1234567890',
      shares: 12345,
      asked_price: 111111102,
      proceeds: 111111102,
      purchase_price: 111111102,
      realized_profit_and_loss: i === 0 ? -111111102 : -12345687,
      taxes: 111111102,
      realized_profit_and_loss_after_tax: i === 0 ? -111111102 : -12345687,
    })),
  ];
  const mutualRows = [
    ...MUTUAL_FUNDS.map((row, i) => ({
      ...row,
      account: '特定・一般',
      shares: '12345',
      cancellation_unit_price_yen: i % 2 === 0 ? '111111102' : '1234567',
      cancellation_amount_yen: i % 2 === 0 ? '111111102' : '1234567',
      average_acquisition_price_yen: i % 2 === 0 ? '111111102' : '1234567',
      realized_profit_and_loss: '999999999',
      taxes: i % 2 === 0 ? '111111102' : '1234567',
      realized_profit_and_loss_after_tax:
        i % 2 === 0 ? '-9999999999' : '-2345678902',
    })),
    ...[0, 1].map((i) => ({
      ...MUTUAL_FUNDS[i],
      id: `mutual-fund-small-${i}`,
      trade_date: '2024-04-01',
      settlement_date: '2024-04-04',
      account: '特定・一般',
      shares: '12345',
      cancellation_unit_price_yen: '111111102',
      cancellation_amount_yen: '111111102',
      average_acquisition_price_yen: '111111102',
      realized_profit_and_loss: i === 0 ? '-111111102' : '-12345687',
      taxes: '111111102',
      realized_profit_and_loss_after_tax: i === 0 ? '-111111102' : '-12345687',
    })),
  ];
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(
      json(
        paginated(
          DIVIDENDS.map((row, i) => ({
            ...row,
            account: '特定・一般',
            security_code: '1234567890',
            unit_price: i % 2 === 0 ? '111111102' : '1234567',
            shares: '12345',
            dividends_before_tax: i % 2 === 0 ? '111111102' : '123456789',
            taxes: i % 2 === 0 ? '111111102' : '123456789',
            net_amount_received: i % 2 === 0 ? '111111102' : '123456789',
          })),
        ),
      ),
    ),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(domesticRows))),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(mutualRows))),
  );

  for (const width of [768, 1024, 1280, 1440]) {
    await page.setViewportSize({ width, height: 900 });
    await page.goto('/receipts');

    for (const collapsed of [true, false]) {
      // lg 未満ではレールの開閉は出ない(上段に積まれる)ので開いた状態の検査は lg 以上だけ
      if (!collapsed && width < 1024) continue;
      if (!collapsed) await openUtilityRail(page);

      const counts: Record<string, number> = {
        dividend: DIVIDENDS.length,
        domesticstock: domesticRows.length,
        mutualfund: mutualRows.length,
      };
      for (const tab of RECEIPT_TABS) {
      await page.getByRole('tab', { name: tab.label }).click();
      await expect(page.getByTestId(`tab-count-${tab.slug}`)).toHaveText(String(counts[tab.slug]));

      const metrics = await page.getByRole('table').evaluate((table, { nameHeader, width }) => {
        const headers = Array.from(table.tHead?.rows[0]?.cells ?? []);
        const bodyRows = Array.from(table.tBodies[0]?.rows ?? []);
        const rows = bodyRows.filter(
          (candidate) =>
            candidate.cells.length === headers.length && candidate.querySelector('.copyable-name'),
        );
        if (rows.length === 0) throw new Error('取引明細のデータ行がありません');

        const nameIndex = headers.findIndex((header) => header.textContent?.trim() === nameHeader);
        // 集計行(見出しの colspan + 集計セル)も含めて、表示中の全セルをはみ出し検査する
        const overflows = bodyRows.flatMap((row) => {
          const isDataRow = row.cells.length === headers.length;
          return Array.from(row.cells).flatMap((cell, index) => {
            if (
              cell.querySelector('.copyable-name') ||
              getComputedStyle(cell).display === 'none' ||
              cell.scrollWidth <= cell.clientWidth
            ) {
              return [];
            }
            return [
              {
                header: isDataRow ? headers[index]?.textContent?.trim() : `集計行の${index}列目`,
                text: cell.textContent?.trim(),
                title: cell.getAttribute('title'),
                clientWidth: cell.clientWidth,
                scrollWidth: cell.scrollWidth,
              },
            ];
          });
        });
        // 金額・数量(text-right)は折り返せるので scrollWidth では桁溢れを検出できない。
        // 9桁以下の値(明細・集計とも)が1行に収まることを行数で固定し、
        // 10桁以上だけが折り返す境界とする
        const wrappedSmall = bodyRows.flatMap((row) => {
          const isDataRow = row.cells.length === headers.length;
          return Array.from(row.cells).flatMap((cell, index) => {
            const style = getComputedStyle(cell);
            if (
              style.display === 'none' ||
              !cell.classList.contains('text-right')
            ) {
              return [];
            }
            const digits = (cell.textContent?.match(/\d/g) ?? []).length;
            if (digits > 9) return [];
            const range = document.createRange();
            range.selectNodeContents(cell);
            const lines = new Set(
              [...range.getClientRects()].map((rect) => Math.round(rect.top)),
            ).size;
            if (lines <= 1) return [];
            return [
              {
                header: isDataRow
                  ? headers[index]?.textContent?.trim()
                  : `集計行の${index}列目`,
                text: cell.textContent?.trim(),
                lines,
              },
            ];
          });
        });
        // 隣接する表示セルの内容同士の見た目の間隔。クリップされた内容は
        // padding ボックスの端まで見えるので、はみ出しの最悪値もその端とする
        const contentGap = (row: HTMLTableRowElement) => {
          const cells = Array.from(row.cells).filter(
            (cell) => getComputedStyle(cell).display !== 'none',
          );
          const range = document.createRange();
          const gaps: number[] = [];
          for (let index = 0; index + 1 < cells.length; index += 1) {
            const a = cells[index];
            const b = cells[index + 1];
            range.selectNodeContents(a);
            const aText = range.getBoundingClientRect();
            range.selectNodeContents(b);
            const bText = range.getBoundingClientRect();
            const aBox = a.getBoundingClientRect();
            const bBox = b.getBoundingClientRect();
            const aBorder = Number.parseFloat(getComputedStyle(a).borderRightWidth) || 0;
            const bBorder = Number.parseFloat(getComputedStyle(b).borderLeftWidth) || 0;
            gaps.push(
              Math.max(bText.left, bBox.left + bBorder) -
                Math.min(aText.right, aBox.right - aBorder),
            );
          }
          return gaps;
        };
        const minContentGap = Math.min(
          ...Array.from(table.rows).flatMap((row) => contentGap(row)),
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
          wrappedSmall,
          minContentGap,
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

      const state = `${width}px ${tab.label} ${collapsed ? '畳み' : '開き'}`;
      // 同日分・月分を合算した集計行に桁溢れの境界値があること。はみ出し検査が
      // 空振りしないよう存在を固定し、省略されず全桁が表示されていることを確かめる
      const boundarySums: Record<string, string[]> = {
        dividend: ['¥123,456,789'],
        domesticstock: ['-¥12,345,678,901', '-¥123,456,789'],
        mutualfund: ['¥1,999,999,998', '-¥12,345,678,901', '-¥123,456,789'],
      };
      for (const sum of boundarySums[tab.slug]) {
        const cell = page.getByRole('cell', { name: sum }).first();
        await expect(cell, `${state} の集計 ${sum}`).toBeVisible();
        const clipped = await cell.evaluate(
          (el) =>
            el.scrollWidth > el.clientWidth + 1 ||
            el.scrollHeight > el.clientHeight + 1,
        );
        expect(clipped, `${state} の集計 ${sum} は全桁表示`).toBe(false);
      }
      expect(metrics.overflows, state).toEqual([]);
      // 金額セルは折り返せるため scrollWidth では桁溢れを検出できない。
      // 9桁以下の金額・集計は1行に収まることを固定し、10桁以上だけが折り返す境界にする
      expect(metrics.wrappedSmall, state).toEqual([]);
      // 隣の列の内容とくっつかないよう、セル間に 4px 以上の見た目の間隔があること
      expect(metrics.minContentGap, `${state} の隣接セル間隔`).toBeGreaterThanOrEqual(4);
      expect(metrics.headerClips, state).toEqual([]);
      expect(
        [...metrics.visibleHeaders].sort(),
        `${state} の表示列`,
      ).toEqual(expectedHeaders(tab, width, collapsed));
      if (metrics.copyDeltas.length > 0) {
        expect(
          Math.min(...metrics.copyDeltas),
          `${state} のコピー範囲は文字幅のみ`,
        ).toBeGreaterThan(4);
      }
      expect(metrics.nameTitle).toBe(metrics.nameText);
      // 1280px で全列を出す畳み状態だけ、集計列(-¥123,456,789 まで切れない幅)の
      // 帳尻で銘柄名は 70px 台まで譲る(title に全文・2行クランプは維持)
      expect(metrics.nameClientWidth, state).toBeGreaterThanOrEqual(
        collapsed && width === 1280 ? 70 : 80,
      );
      expect(metrics.nameOverflow).toBe('hidden');
      expect(metrics.nameLineClamp).toBe('2');
      expect(metrics.nameWhiteSpace).toBe('normal');
      expect(metrics.namelessClips, state).toEqual([]);
      // 畳み時は全列を出す要件上、1280px では銘柄名が2行に収まらず切り詰められる
      // (title に全文あり)。クランプを禁じるのは列を絞った開いた状態に限る
      if (width >= 1280 && !collapsed) {
        expect(metrics.clampedNames, state).toEqual([]);
      }
      expect(metrics.tableScrollWidth, state).toBeLessThanOrEqual(
        metrics.tableClientWidth,
      );
      expect(metrics.pageOverflows).toBe(false);
      await shoot(
        page,
        `receipts-after-${tab.slug}-${width}${collapsed ? '' : '-open'}`,
      );
    }
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

  // データがあるので畳まれた状態で始まる。開くと右レールが出る
  const rail = page.getByTestId('receipt-utility-rail');
  const main = page.getByTestId('receipt-main-stage');
  await openUtilityRail(page);
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
      // 開閉バーも印刷には出さない(操作できないボタンが残らないよう no-print)
      await expect(page.getByTestId('receipt-utility-toggle-bar')).toBeHidden();
      // 印刷は全列なので、集計見出しの結合セルは all だけが出る。
      // 畳んだレールの variant が残るとラベルセルが重複して金額がずれる
      const groupCheck = await table.evaluate((element) => {
        const headers = Array.from(element.tHead?.rows[0]?.cells ?? []).filter(
          (cell) => getComputedStyle(cell).display !== 'none',
        ).length;
        const groups = Array.from(element.tBodies[0]?.rows ?? [])
          .filter((row) => row.querySelector('td[colspan]'))
          .map((row) => {
            const visibleLabels = Array.from(
              row.querySelectorAll('td[colspan]'),
            ).filter((cell) => getComputedStyle(cell).display !== 'none');
            return {
              visibleCells: Array.from(row.cells).filter(
                (cell) => getComputedStyle(cell).display !== 'none',
              ).length,
              visibleLabels: visibleLabels.length,
              colspan: Number(visibleLabels[0]?.getAttribute('colspan')),
            };
          });
        return { headers, groups };
      });
      expect(
        groupCheck.groups.length,
        `${tab.label} の集計行が検査対象にある`,
      ).toBeGreaterThan(0);
      for (const group of groupCheck.groups) {
        expect(group.visibleLabels, `${tab.label} の集計見出しは1個だけ`).toBe(1);
        expect(group.visibleCells, `${tab.label} の集計行セル数`).toBe(4);
        expect(group.colspan, `${tab.label} の集計行 colspan`).toBe(
          groupCheck.headers - 3,
        );
      }
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
        expect(longFund?.lineCount).toBeGreaterThan(2);
      }
    }
  }
});

test('640px 以上で年ピッカーの選択肢がレール下端を超えても末尾の年を選べる', async ({ page }) => {
  for (const width of [768, 1280]) {
    await page.setViewportSize({ width, height: 900 });
    await gotoReceipts(page);
    await openUtilityRail(page);

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
