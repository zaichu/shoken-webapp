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

test('1279px は1カラム、1280px で右レール2カラムに切り替わる', async ({ page }) => {
  await page.setViewportSize({ width: 1279, height: 900 });
  await gotoReceipts(page);

  const rail = page.getByTestId('receipt-utility-rail');
  const main = page.getByTestId('receipt-main-stage');
  let railBox = await rail.boundingBox();
  let mainBox = await main.boundingBox();
  expect(railBox).not.toBeNull();
  expect(mainBox).not.toBeNull();
  expect(railBox!.y).toBeLessThan(mainBox!.y);
  await shoot(page, 'leptos-962-1279');

  await page.setViewportSize({ width: 1280, height: 900 });
  railBox = await rail.boundingBox();
  mainBox = await main.boundingBox();
  expect(railBox).not.toBeNull();
  expect(mainBox).not.toBeNull();
  expect(railBox!.x).toBeGreaterThan(mainBox!.x);
  expect(railBox!.width).toBeGreaterThanOrEqual(310);
  expect(railBox!.width).toBeLessThanOrEqual(330);
  await shoot(page, 'leptos-962-1280');
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

  // 表はカード内で横スクロールし、ページ幅を広げない
  const tableScroll = await page
    .getByRole('table')
    .evaluate((table) => {
      const wrapper = table.parentElement;
      if (!wrapper) return { overflowX: '', scrollable: false };
      return {
        overflowX: getComputedStyle(wrapper).overflowX,
        scrollable: wrapper.scrollWidth > wrapper.clientWidth,
      };
    });
  expect(tableScroll.overflowX).toBe('auto');

  // 全件削除はレール内のボタン(全幅の赤枠ではない)
  const deleteButton = rail.getByRole('button', { name: /全件削除/ });
  await expect(deleteButton).toBeVisible();
  const deleteBox = await deleteButton.boundingBox();
  expect(deleteBox).not.toBeNull();
  expect(deleteBox!.width).toBeLessThanOrEqual(railBox!.width);
  await expect(page.getByRole('button', { name: /全件削除/ })).toHaveCount(1);

  await shoot(page, 'leptos-962-1920');
});

// React との見た目比較用。Vite dev server (port 8080) を別途起動し、
// REACT_BASE_URL を指定したときだけ実行する
// 例: REACT_BASE_URL=http://127.0.0.1:8080 npx playwright test --config ../frontend-leptos/playwright.leptos.config.ts -g "react比較"
test.describe('react比較スクリーンショット', () => {
  const reactBase = process.env.REACT_BASE_URL;
  test.skip(!reactBase, 'REACT_BASE_URL 未指定時はスキップ');
  test.use({ baseURL: reactBase ?? 'http://127.0.0.1:8080' });

  for (const width of [1920, 390]) {
    test(`react receipts ${width}px`, async ({ page }) => {
      await page.setViewportSize({ width, height: width === 1920 ? 1080 : 844 });
      await page.goto('/receipts');
      await expect(page.getByRole('tab').first()).toBeVisible();
      if (width >= 640) {
        await expect(page.getByRole('table')).toBeVisible();
      } else {
        await expect(page.getByTestId('receipt-card').first()).toBeVisible();
      }
      await shoot(page, `react-962-${width}`);
    });
  }
});
