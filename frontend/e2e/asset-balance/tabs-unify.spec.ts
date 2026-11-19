import type { Page } from '@playwright/test';
import { expect, test, json, mockSession } from '../support/test';

const THIS_YEAR = new Date().getFullYear();

// 小数桁がそろっていない価格を入れて、列の中で桁がそろうことを確かめる
const STOCK_ROWS = [
  {
    id: 'asset-1',
    created_at: `${THIS_YEAR}-01-01T00:00:00Z`,
    updated_at: `${THIS_YEAR}-01-01T00:00:00Z`,
    security_code: '9901',
    security_name: '架空自動車',
    account: '特定',
    shares: 100,
    average_purchase_price: 5997.6,
    total_purchase_amount: 599760,
    current_price: 6187.25,
    daily_change: 12,
    price_as_of: `${THIS_YEAR}-10-20`,
    prev_close: 6175.25,
  },
  {
    id: 'asset-2',
    created_at: `${THIS_YEAR}-01-01T00:00:00Z`,
    updated_at: `${THIS_YEAR}-01-01T00:00:00Z`,
    security_code: '9902',
    security_name: '架空精密機器',
    account: '一般',
    shares: 200,
    average_purchase_price: 187.36,
    total_purchase_amount: 37472,
    current_price: 162,
    daily_change: -3,
    price_as_of: `${THIS_YEAR}-10-20`,
    prev_close: 165,
  },
];

const FUND_ROWS = [
  {
    id: 'fund-1',
    created_at: `${THIS_YEAR}-01-01T00:00:00Z`,
    updated_at: `${THIS_YEAR}-01-01T00:00:00Z`,
    fund_name: '架空バランスファンドA号',
    account: '特定',
    dividends_course: '受取型',
    shares: 10000,
    average_purchase_price: 9970.5,
    total_purchase_amount: 9970,
    base_price: 10012.25,
    base_price_day_change: 5,
    base_price_month_change: 120,
    market_value: 10012,
    profit_loss: 42,
    profit_loss_rate: 0.42,
    total_return: 42,
    price_as_of: `${THIS_YEAR}-10-20`,
    price_source: 'api',
    nav_is_latest: true,
    expected_nav_date: null,
  },
  {
    id: 'fund-2',
    created_at: `${THIS_YEAR}-01-01T00:00:00Z`,
    updated_at: `${THIS_YEAR}-01-01T00:00:00Z`,
    fund_name: '架空インデックスファンドB号',
    account: '一般',
    dividends_course: '再投資型',
    shares: 20000,
    average_purchase_price: 9500,
    total_purchase_amount: 19000,
    base_price: 9800,
    base_price_day_change: -30,
    base_price_month_change: 120,
    market_value: 19600,
    profit_loss: 600,
    profit_loss_rate: 3.16,
    total_return: 600,
    price_as_of: `${THIS_YEAR}-10-20`,
    price_source: 'csv',
    nav_is_latest: true,
    expected_nav_date: null,
  },
];

async function setupMocks(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(
      json({
        data: STOCK_ROWS,
        total: STOCK_ROWS.length,
        page: 1,
        per_page: 1000,
      }),
    ),
  );
  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
    route.fulfill(
      json({
        data: FUND_ROWS,
        total: FUND_ROWS.length,
        page: 1,
        per_page: 1000,
        summary: {
          total_market_value: 29612,
          total_purchase_amount: 28970,
          total_daily_change: 0,
          latest_nav_count: FUND_ROWS.length,
          has_missing_daily_change: false,
          nav_status: {
            updated_at: '2026-10-20 18:05',
            fetched_count: 1,
            failed_count: 0,
            fund_count: FUND_ROWS.length,
            failure_reason: null,
          },
        },
      }),
    ),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

// 同じ列の値が同じ小数桁を持つこと
function decimalsOf(text: string): number | null {
  const matched = /(\d+)\.(\d+)/.exec(text);
  return matched ? matched[2].length : null;
}

async function columnDecimals(page: Page, columnName: string): Promise<number[]> {
  const table = page.locator('.asset-table').first();
  const header = table.getByRole('columnheader', { name: columnName });
  const index = await header.evaluate((th) =>
    Array.from(th.parentElement?.children ?? []).indexOf(th),
  );
  return table.evaluate(
    (element, columnIndex) =>
      Array.from(element.querySelectorAll('tbody tr')).map((row) => {
        const cell = row.children[columnIndex];
        return cell?.textContent?.trim() ?? '';
      }),
    index,
  );
}

test.describe('資産管理のタブ', () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  test('国内株式と投資信託は同じ用語・同じ検索欄・同じ時点の出し方で出す', async ({
    page,
  }) => {
    await setupMocks(page);
    await page.goto('/assetbalance');

    const placeholders: string[] = [];
    const bandLabels: string[][] = [];

    for (const tab of ['stock', 'fund'] as const) {
      await page.getByTestId(`asset-kind-tab-${tab}`).click();
      const band = page.getByTestId('asset-portfolio-summary');
      await expect(band).toBeVisible();

      // 帯の主要3項目(評価額・損益・前日比)が2タブで同じ
      bandLabels.push(
        (await band
          .locator('.summary-band-pc .summary-item dt')
          .allTextContents()).slice(0, 3),
      );

      // 検索欄の文言は同じ
      placeholders.push(
        await page.getByTestId('asset-search-input-pc').getAttribute('placeholder'),
      );

      // 注記行は出さず、時点だけを帯のラベル横に出す
      await expect(page.locator('.page-skeleton-note')).toHaveCount(0);
      await expect(page.getByText('計算基準を見る')).toHaveCount(0);
      await expect(band).toContainText(`株価 ${THIS_YEAR}/10/20 時点`);
    }

    // タブを跨いで帯の主要項目・検索欄の文言がそろう
    expect(bandLabels[0]).toEqual(bandLabels[1]);
    expect(placeholders[0]).toBe(placeholders[1]);
  });

  test('同じ列の金額は同じ小数桁でそろえる', async ({ page }) => {
    await setupMocks(page);
    await page.goto('/assetbalance');

    await page.getByTestId('asset-kind-tab-stock').click();
    for (const column of ['取得単価', '現在値']) {
      const values = await columnDecimals(page, column);
      expect(values.length).toBeGreaterThan(1);
      const decimals = values.map(decimalsOf);
      expect(decimals.every((value) => value !== null)).toBe(true);
      expect(new Set(decimals).size).toBe(1);
    }

    await page.getByTestId('asset-kind-tab-fund').click();
    const values = await columnDecimals(page, '基準価額(1万口あたり)');
    expect(values.length).toBeGreaterThan(1);
    const decimals = values.map(decimalsOf);
    expect(decimals.every((value) => value !== null)).toBe(true);
    expect(new Set(decimals).size).toBe(1);
  });
});

// PC のタブはスマホと同じ下線タブ。選ばれたタブの下に青い1本が出て、
// 切り替えでは1本が横に滑る(reduced-motion では瞬時)
async function setupReceiptMocks(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json({ data: [], total: 0, page: 1, per_page: 1 })),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json({ data: [], total: 0, page: 1, per_page: 1 })),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json({ data: [], total: 0, page: 1, per_page: 1 })),
  );
}

async function pcUnderlineSettled(page: Page, selector: string) {
  // 下線の滑り(180ms)が終わるまで待ってから測る。終わらなければ失敗にする
  await page.waitForFunction((listSelector: string) => {
    const lists = Array.from(document.querySelectorAll(listSelector));
    const list = lists.find(
      (candidate) => candidate instanceof HTMLElement && candidate.offsetParent !== null,
    );
    if (!(list instanceof HTMLElement)) {
      return false;
    }
    const selected = list.querySelector('[role="tab"][aria-selected="true"]');
    if (!(selected instanceof HTMLElement)) {
      return false;
    }
    const listAfter = getComputedStyle(list, '::after');
    if (listAfter.display === 'none') {
      return true;
    }
    const listBox = list.getBoundingClientRect();
    const selectedBox = selected.getBoundingClientRect();
    return (
      Math.abs(Number.parseFloat(listAfter.left) - (selectedBox.left - listBox.left)) <= 2 &&
      Math.abs(Number.parseFloat(listAfter.width) - selectedBox.width) <= 2
    );
  }, selector);
  return pcUnderline(page, selector);
}

async function pcUnderline(page: Page, selector: string) {
  return page.evaluate((listSelector: string) => {
    const lists = Array.from(document.querySelectorAll(listSelector));
    const list = lists.find(
      (candidate) => candidate instanceof HTMLElement && candidate.offsetParent !== null,
    );
    if (!(list instanceof HTMLElement)) {
      return null;
    }
    const selected = list.querySelector('[role="tab"][aria-selected="true"]');
    if (!(selected instanceof HTMLElement)) {
      return null;
    }
    const listAfter = getComputedStyle(list, '::after');
    const tabAfter = getComputedStyle(selected, '::after');
    const ink = list.querySelector('.tab-swipe-ink');
    const underlines: string[] = [];
    if (listAfter.display !== 'none') {
      underlines.push('list');
    }
    if (tabAfter.display !== 'none' && tabAfter.content !== 'none') {
      underlines.push('tab');
    }
    const listBox = list.getBoundingClientRect();
    const selectedBox = selected.getBoundingClientRect();
    return {
      underlines,
      inkDisplay: ink instanceof HTMLElement ? getComputedStyle(ink).display : 'none',
      left: Number.parseFloat(listAfter.left),
      width: Number.parseFloat(listAfter.width),
      selectedLeft: selectedBox.left - listBox.left,
      selectedWidth: selectedBox.width,
      selectedId: selected.id || selected.getAttribute('data-testid'),
      color: listAfter.borderBottomColor,
      transitionDuration: listAfter.transitionDuration,
    };
  }, selector);
}

test.describe('PCの下線タブ', () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  test('クリックで下線が選んだタブの下へ動き下線は1本(資産管理)', async ({
    page,
  }) => {
    await setupMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-kind-tab-stock')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    const before = await pcUnderlineSettled(page, '[data-testid="asset-kind-tabs"]');
    expect(before).not.toBeNull();
    expect(before?.underlines).toEqual(['list']);
    expect(before?.inkDisplay).toBe('none');
    expect(Math.abs((before?.left ?? 0) - (before?.selectedLeft ?? 0))).toBeLessThanOrEqual(2);
    expect(Math.abs((before?.width ?? 0) - (before?.selectedWidth ?? 0))).toBeLessThanOrEqual(2);

    await page.getByTestId('asset-kind-tab-fund').click();
    await expect(page.getByTestId('asset-kind-tab-fund')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    const after = await pcUnderlineSettled(page, '[data-testid="asset-kind-tabs"]');
    expect(after).not.toBeNull();
    expect(after?.underlines).toEqual(['list']);
    expect(after?.inkDisplay).toBe('none');
    expect(after?.selectedId).toContain('fund');
    expect(Math.abs((after?.left ?? 0) - (after?.selectedLeft ?? 0))).toBeLessThanOrEqual(2);
    expect(Math.abs((after?.width ?? 0) - (after?.selectedWidth ?? 0))).toBeLessThanOrEqual(2);
    // 下線は左から右へ動く。色は同じ青のまま
    expect(after?.left ?? 0).toBeGreaterThan(before?.left ?? 0);
    expect(after?.color).toBe(before?.color);
    // 滑って切り替わる。遷移時間のms値は固定しない(変えても挙動は同じ)
    expect(after?.transitionDuration).not.toBe('0s');
  });

  test('クリックで下線が選んだタブの下へ動き下線は1本(取引明細)', async ({
    page,
  }) => {
    await setupReceiptMocks(page);
    await page.goto('/receipts');
    await expect(page.locator('#tab-dividend')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await page.locator('#tab-domesticstock').click();
    await expect(page.locator('#tab-domesticstock')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    const state = await pcUnderlineSettled(page, '[role="tablist"]');
    expect(state).not.toBeNull();
    expect(state?.underlines).toEqual(['list']);
    expect(state?.inkDisplay).toBe('none');
    expect(state?.selectedId).toBe('tab-domesticstock');
    expect(Math.abs((state?.left ?? 0) - (state?.selectedLeft ?? 0))).toBeLessThanOrEqual(2);
    expect(Math.abs((state?.width ?? 0) - (state?.selectedWidth ?? 0))).toBeLessThanOrEqual(2);
  });

  test('左右キーとTabで切り替わりフォーカスが見える', async ({ page }) => {
    await setupMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-kind-tab-stock')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await page.getByTestId('asset-kind-tab-stock').click();
    await page.keyboard.press('ArrowRight');
    await expect(page.getByTestId('asset-kind-tab-fund')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await page.waitForFunction(
      () => document.activeElement?.getAttribute('data-testid') === 'asset-kind-tab-fund',
    );
    const outline = await page.evaluate(() =>
      getComputedStyle(document.activeElement as Element).outlineWidth,
    );
    expect(outline).not.toBe('0px');

    // 選ばれたタブだけが Tab 移動の対象(ロービング)
    const tabIndexes = await page.evaluate(() =>
      Array.from(document.querySelectorAll('[data-testid="asset-kind-tabs"] [role="tab"]')).map(
        (tab) => tab.getAttribute('tabindex'),
      ),
    );
    expect(tabIndexes).toEqual(['-1', '-1', '0']);

    await page.keyboard.press('ArrowLeft');
    await expect(page.getByTestId('asset-kind-tab-stock')).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });

  test('タブの押せる高さは44px以上', async ({ page }) => {
    await setupMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-kind-tabs')).toBeVisible();
    const heights = await page.evaluate(() =>
      Array.from(document.querySelectorAll('[data-testid="asset-kind-tabs"] [role="tab"]')).map(
        (tab) => (tab as HTMLElement).getBoundingClientRect().height,
      ),
    );
    expect(heights.length).toBe(3);
    for (const height of heights) {
      expect(height).toBeGreaterThanOrEqual(44);
    }
  });

  test('動きを減らす設定では瞬時に切り替わる', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await setupMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-kind-tab-stock')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await page.getByTestId('asset-kind-tab-fund').click();
    await expect(page.getByTestId('asset-kind-tab-fund')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    const state = await pcUnderlineSettled(page, '[data-testid="asset-kind-tabs"]');
    expect(state).not.toBeNull();
    expect(state?.underlines).toEqual(['list']);
    expect(state?.transitionDuration).toBe('0s');
  });
});
