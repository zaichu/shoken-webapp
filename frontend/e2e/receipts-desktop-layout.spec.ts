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

  // スマホではレールに枠を付けず、検索カードは従来どおり独立したカード
  const railInner = rail.locator('> div').first();
  await expect(railInner).toHaveCSS('border-top-width', '0px');
  const searchCard = page.getByTestId('search-card');
  await expect(searchCard).toHaveCSS('border-top-width', '1px');

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

  // 狭い主列では表が内部で横スクロールする
  const tableScroll = await page.getByRole('table').evaluate((table) => {
    const wrapper = table.parentElement;
    if (!wrapper) return { overflowX: '', scrollable: false };
    return {
      overflowX: getComputedStyle(wrapper).overflowX,
      scrollable: wrapper.scrollWidth > wrapper.clientWidth,
    };
  });
  expect(tableScroll.overflowX).toBe('auto');
  expect(tableScroll.scrollable).toBe(true);
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

  // 表ヘッダーは薄い灰色の背景に濃い文字(React 準拠)
  await expect(
    page.getByRole('table').locator('thead tr'),
  ).toHaveCSS('background-color', 'oklch(0.984 0.003 247.858)');
  await expect(
    page.getByRole('table').locator('thead th').first(),
  ).toHaveCSS('color', 'oklch(0.279 0.041 260.031)');

  // 表はカード内でスクロールし、ページ幅を広げない
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
  expect(tableScroll.overflowX).toBe('auto');
  expect(tableScroll.overflowY).toBe('auto');
  expect(tableScroll.maxHeight).toMatch(/^\d+(\.\d+)?px$/);

  // ページ自体は横にはみ出さない
  const documentWidth = await page.evaluate(
    () => document.documentElement.scrollWidth,
  );
  expect(documentWidth).toBeLessThanOrEqual(1920);

  // 表が内部スクロールするのでページが伸びず、レールはビューポート内に収まる
  expect(railBox!.y + railBox!.height).toBeLessThanOrEqual(1080);

  // 縦が短いビューポートでは表の内部縦スクロールが発生する
  await page.setViewportSize({ width: 1920, height: 480 });
  await expect
    .poll(async () =>
      page.getByRole('table').evaluate((table) => {
        const wrapper = table.parentElement;
        return wrapper ? wrapper.scrollHeight > wrapper.clientHeight : false;
      }),
    )
    .toBe(true);
  await page.setViewportSize({ width: 1920, height: 1080 });

  // 印刷時は高さ制限とスクロールを外して全行を出力する
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

test('640px 以上で年ピッカーの選択肢がレール下端を超えても末尾の年を選べる', async ({ page }) => {
  for (const width of [768, 1280]) {
    await page.setViewportSize({ width, height: 900 });
    await gotoReceipts(page);

    const railInner = page
      .getByTestId('receipt-utility-rail')
      .locator('> div')
      .first();
    // レールの枠がドロップダウンをクリップしない
    await expect(railInner).toHaveCSS('overflow', 'visible');

    const trigger = page.getByRole('button', { name: '年を選択' });
    await trigger.click();
    const listbox = page.getByRole('listbox', { name: '年候補' });
    await expect(listbox).toBeVisible();

    // 長い候補はリスト内でスクロールして全件に到達できる
    await expect(listbox).toHaveCSS('overflow-y', 'auto');
    await expect(listbox).toHaveCSS('max-height', '288px');

    // 末尾の年も実際にクリックできる(768px の1カラム幅ではドロップダウンが下の表と重なり得る)
    const lastOption = listbox.getByRole('option').last();
    const yearLabel = (await lastOption.textContent())!.trim();
    await lastOption.click({ timeout: 5000 });
    await expect(trigger).toContainText(yearLabel);
  }
});
