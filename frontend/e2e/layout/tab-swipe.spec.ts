import type { Page } from '@playwright/test';
import { expect, test, json, paginated, MOCK_USER, mockSession } from '../support/test';

const STOCK_ROWS = [
  {
    id: 'asset-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: '9901',
    security_name: '架空自動車',
    account: '特定',
    shares: 100,
    average_purchase_price: 5997.6,
    total_purchase_amount: 599760,
    current_price: 6187.25,
    daily_change: 12,
    price_as_of: '2026-10-20',
    prev_close: 6175.25,
  },
];

const FUND_ROWS = [
  {
    id: 'fund-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
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
    price_as_of: '2026-10-20',
    price_source: 'api',
    nav_is_latest: true,
    expected_nav_date: null,
  },
];

async function setupAssetMocks(page: Page) {
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
      }),
    ),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

const RICH_DIVIDENDS = Array.from({ length: 35 }, (_, i) => ({
  id: `00000000-0000-0000-0000-00000000${String(1000 + i).slice(1)}`,
  user_id: '00000000-0000-0000-0000-000000000002',
  settlement_date: `2024-03-${String((i % 28) + 1).padStart(2, '0')}`,
  product: '特定口座',
  account: ['SBI証券', '楽天証券', '松井証券'][i % 3],
  security_code: `${9900 + (i % 90)}`,
  security_name: `架空全世界半導体スーパーグロース高配当ブルーチップホールディングス${i}号`,
  unit_price: '30.0',
  shares: '100',
  dividends_before_tax: '3000',
  taxes: '609',
  net_amount_received: '2391',
  created_at: '2024-03-01T00:00:00Z',
  updated_at: '2024-03-01T00:00:00Z',
}));

async function setupReceiptRichMocks(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated(RICH_DIVIDENDS)),
    }),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(
        paginated(
          RICH_DIVIDENDS.map((row, i) => ({
            id: `domestic-rich-${i}`,
            user_id: MOCK_USER.id,
            trade_date: '2024-03-01',
            settlement_date: row.settlement_date,
            security_code: row.security_code,
            security_name: row.security_name,
            account: ['特定口座', 'NISA口座', '一般口座'][i % 3],
            shares: 100 + i,
            asked_price: 2500,
            proceeds: 260000,
            purchase_price: 2400,
            realized_profit_and_loss: 20000,
            taxes: 4058,
            realized_profit_and_loss_after_tax: 15942,
            created_at: row.created_at,
            updated_at: row.updated_at,
          })),
        ),
      ),
    }),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(
        paginated(
          Array.from({ length: 12 }, (_, i) => ({
            id: `mutual-rich-${i}`,
            user_id: MOCK_USER.id,
            trade_date: '2024-03-01',
            settlement_date: `2024-03-${String((i % 28) + 1).padStart(2, '0')}`,
            fund_name: `架空インデックス 全世界株式（オール・ワールド）長期積立専用${i}号ファンド`,
            account: ['特定口座', 'NISA口座', '一般口座'][i % 3],
            shares: '123456',
            exchange_rate: '1',
            cancellation_unit_price_yen: '12345',
            cancellation_amount_yen: '123456789',
            average_acquisition_price_yen: '22345',
            dividends: '0',
            realized_profit_and_loss: '123456',
            taxes: '0',
            realized_profit_and_loss_after_tax: '123456',
            created_at: '2024-03-01T00:00:00Z',
            updated_at: '2024-03-01T00:00:00Z',
          })),
        ),
      ),
    }),
  );
}

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

async function touchStart(page: Page, testid: string, x: number, y: number) {
  await page.getByTestId(testid).evaluate(
    (element, { x: clientX, y: clientY }: { x: number; y: number }) => {
      const start = new Touch({
        identifier: 1,
        target: element,
        clientX,
        clientY,
      });
      element.dispatchEvent(
        new TouchEvent('touchstart', {
          touches: [start],
          bubbles: true,
          cancelable: true,
        }),
      );
    },
    { x, y },
  );
}

async function touchMove(page: Page, testid: string, x: number, y: number) {
  await page.getByTestId(testid).evaluate(
    (element, { x: clientX, y: clientY }: { x: number; y: number }) => {
      const moved = new Touch({
        identifier: 1,
        target: element,
        clientX,
        clientY,
      });
      element.dispatchEvent(
        new TouchEvent('touchmove', {
          touches: [moved],
          bubbles: true,
          cancelable: true,
        }),
      );
    },
    { x, y },
  );
}

async function touchEnd(page: Page, testid: string, x: number, y: number) {
  await page.getByTestId(testid).evaluate(
    (element, { x: clientX, y: clientY }: { x: number; y: number }) => {
      const end = new Touch({
        identifier: 1,
        target: element,
        clientX,
        clientY,
      });
      element.dispatchEvent(
        new TouchEvent('touchend', {
          changedTouches: [end],
          bubbles: true,
          cancelable: true,
        }),
      );
    },
    { x, y },
  );
}

async function swipe(
  page: Page,
  testid: string,
  fromX: number,
  toX: number,
  y = 400,
  toY?: number,
) {
  await touchStart(page, testid, fromX, y);
  if (toY !== undefined) {
    await touchMove(page, testid, toX, toY);
  }
  await touchEnd(page, testid, toX, toY ?? y);
}

function parseTranslateXPercent(transform: string): number | null {
  const match = /translateX\((-?\d+(?:\.\d+)?)%\)/.exec(transform);
  return match ? Number(match[1]) : null;
}

async function activeSlideTransform(
  page: Page,
  testid: string,
): Promise<string> {
  return page
    .locator(
      `[data-testid="${testid}"] .tab-swipe-slide[data-active="true"]`,
    )
    .evaluate((element) => (element as HTMLElement).style.transform);
}

async function waitForDrag(page: Page, testid: string) {
  await page.waitForFunction(
    (id) => {
      const current = document.querySelector(
        `[data-testid="${id}"] .tab-swipe-slide[data-active="true"]`,
      );
      return (
        current instanceof HTMLElement &&
        current.style.transform !== '' &&
        current.style.transform !== 'translateX(0px)'
      );
    },
    testid,
  );
}

function sampleSlideX(
  page: Page,
  testid: string,
  durationMs: number,
): Promise<(number | null)[]> {
  return page
    .getByTestId(testid)
    .evaluate(
      (element, ms) =>
        new Promise<(number | null)[]>((resolve) => {
          const read = () => {
            const active = element.querySelector(
              '.tab-swipe-slide[data-active="true"]',
            ) as HTMLElement | null;
            const raw = active ? active.style.transform : '';
            const match = /translateX\((-?\d+(?:\.\d+)?)px\)/.exec(raw);
            return match ? Number(match[1]) : null;
          };
          const samples: (number | null)[] = [];
          const start = performance.now();
          const tick = () => {
            const value = read();
            if (value !== null) {
              samples.push(value);
            }
            if (performance.now() - start < ms) {
              requestAnimationFrame(tick);
            } else {
              resolve(samples);
            }
          };
          requestAnimationFrame(tick);
        }),
      durationMs,
    );
}

async function slideX(page: Page, testid: string): Promise<number | null> {
  return page.getByTestId(testid).evaluate((element) => {
    const active = element.querySelector(
      '.tab-swipe-slide[data-active="true"]',
    ) as HTMLElement | null;
    const match = active
      ? /translateX\((-?\d+(?:\.\d+)?)px\)/.exec(active.style.transform)
      : null;
    return match ? Number(match[1]) : null;
  });
}

async function mobileTabStrip(page: Page, testid: string) {
  return page.evaluate((stageId: string) => {
    const stage = document.querySelector(`[data-testid="${stageId}"]`);
    const list = stage?.querySelector('.tab-swipe-tabs [role="tablist"]');
    if (!(list instanceof HTMLElement) || list.offsetParent === null) {
      return null;
    }
    const rect = list.getBoundingClientRect();
    const style = getComputedStyle(list);
    // 隣の面に覆われると座標・表示は変わらず見えなくなるため、上にある要素で確かめる。
    // 右端(最後のタブ)の中心も含め、帯の端から端まで覆われていないこと
    const tabs = list.querySelectorAll('[role="tab"]');
    const lastTab = tabs.length > 0 ? tabs[tabs.length - 1] : null;
    const lastRect = lastTab instanceof HTMLElement ? lastTab.getBoundingClientRect() : null;
    const points: Array<[number, number]> = [0.1, 0.5, 0.9].map((fraction) => [
      rect.x + rect.width * fraction,
      rect.y + rect.height / 2,
    ]);
    if (lastRect !== null) {
      points.push([lastRect.x + lastRect.width / 2, lastRect.y + lastRect.height / 2]);
    }
    const covered = points.some(([x, y]) => {
      const hit = document.elementFromPoint(x, y);
      return hit === null || (hit !== list && !list.contains(hit));
    });
    return {
      x: rect.x,
      width: rect.width,
      display: style.display,
      opacity: style.opacity,
      covered,
    };
  }, testid);
}

async function inkPercent(page: Page, testid: string): Promise<number | null> {
  // PC 用とスマホ用の下線が両方あるため、見えている側を読む
  const transform = await page
    .locator(`[data-testid="${testid}"] .tab-swipe-tabs .tab-swipe-ink`)
    .evaluate((element) => (element as HTMLElement).style.transform);
  return parseTranslateXPercent(transform);
}

// スワイプ中の下線の見え方。選択中タブ側の下線(::after)は隠れ、
// 指に追従する ink が青の1本だけ残ることを確かめる
async function swipeUnderlineState(page: Page, testid: string) {
  return page.getByTestId(testid).evaluate((stage) => {
    const tabs = stage.querySelector('.tab-swipe-tabs');
    const list = tabs?.querySelector('[role="tablist"]');
    const inks = tabs ? Array.from(tabs.querySelectorAll('.tab-swipe-ink')) : [];
    const visibleInks = inks.filter(
      (element) =>
        element instanceof HTMLElement &&
        element.offsetParent !== null &&
        getComputedStyle(element).display !== 'none',
    );
    const selected = list?.querySelector('[role="tab"][aria-selected="true"]');
    const listAfter =
      list instanceof HTMLElement ? getComputedStyle(list, '::after') : null;
    const tabAfter =
      selected instanceof HTMLElement ? getComputedStyle(selected, '::after') : null;
    return {
      inkCount: visibleInks.length,
      inkBackground:
        visibleInks.length > 0
          ? getComputedStyle(visibleInks[0] as HTMLElement).backgroundColor
          : null,
      listAfterDisplay: listAfter?.display ?? null,
      listAfterColor: listAfter?.borderBottomColor ?? null,
      tabAfterContent: tabAfter?.content ?? null,
      tabAfterDisplay: tabAfter?.display ?? null,
    };
  });
}

async function swipePanelGeometry(
  page: Page,
  testid: string,
  summarySelector: string,
  listSelector: string,
) {
  return page.evaluate(
    ({
      stageId,
      summary,
      list,
    }: {
      stageId: string;
      summary: string;
      list: string;
    }) => {
      const stage = document.querySelector(`[data-testid="${stageId}"]`);
      const strip = stage?.querySelector('.tab-swipe-tabs [role="tablist"]');
      // 指で引いている間は隣の面も載るため、前面に絞って測る
      const scope = '.tab-swipe-slide[data-active="true"]';
      const summaryElement = stage?.querySelector(`${scope} ${summary}`);
      const listElement = stage?.querySelector(`${scope} ${list}`);
      if (
        !(strip instanceof HTMLElement) ||
        !(summaryElement instanceof HTMLElement) ||
        !(listElement instanceof HTMLElement)
      ) {
        return null;
      }
      const stripRect = strip.getBoundingClientRect();
      const summaryRect = summaryElement.getBoundingClientRect();
      const listRect = listElement.getBoundingClientRect();
      return {
        stripX: stripRect.x,
        stripBottom: stripRect.bottom,
        summaryLeft: summaryRect.left,
        summaryTop: summaryRect.top,
        listLeft: listRect.left,
      };
    },
    { stageId: testid, summary: summarySelector, list: listSelector },
  );
}

test.describe('下線タブの横スワイプ', () => {
  test.use({ viewport: { width: 390, height: 844 } });

  test('資産管理で左スワイプすると次のタブに切り替わり右スワイプで戻る', async ({
    page,
  }) => {
    await setupAssetMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await swipe(page, 'assetbalance-main-stage', 300, 90);
    await expect(page.getByTestId('asset-kind-tab-fund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await swipe(page, 'assetbalance-main-stage', 90, 300);
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    // 端から始めた接触・短い移動・縦方向の移動では切り替わらない
    await swipe(page, 'assetbalance-main-stage', 5, 200);
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await swipe(page, 'assetbalance-main-stage', 300, 270);
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await swipe(page, 'assetbalance-main-stage', 300, 280, 400, 200);
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    // 端のタブ(投資信託)では外向きのスワイプで動かない
    await swipe(page, 'assetbalance-main-stage', 300, 90);
    await expect(page.getByTestId('asset-kind-tab-fund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await swipe(page, 'assetbalance-main-stage', 300, 90);
    await expect(page.getByTestId('asset-kind-tab-fund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });

  test('指を止めたままでは中身が追従し離すと切り替わる', async ({ page }) => {
    await setupAssetMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await touchStart(page, 'assetbalance-main-stage', 300, 400);
    await touchMove(page, 'assetbalance-main-stage', 220, 400);
    await touchMove(page, 'assetbalance-main-stage', 190, 400);
    await page.waitForFunction(() => {
      const current = document.querySelector(
        '[data-testid="assetbalance-main-stage"] .tab-swipe-slide[data-active="true"]',
      );
      return (
        current instanceof HTMLElement &&
        current.style.transform !== '' &&
        current.style.transform !== 'translateX(0px)'
      );
    });
    // 離す前は切り替わらず、隣の中身が端からのぞく
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    const fundLeft = await page
      .locator('#assetbalance-fund-list')
      .evaluate((element) => element.getBoundingClientRect().left);
    expect(fundLeft).toBeLessThan(390);
    await expect(page.locator('.tab-swipe-ink[data-on="true"]:visible')).toHaveCount(1);

    await touchEnd(page, 'assetbalance-main-stage', 100, 400);
    await expect(page.getByTestId('asset-kind-tab-fund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });

  test('切り替わらない接触では追従しない', async ({ page }) => {
    await setupAssetMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    const currentTransform = () =>
      page
        .locator(
          '[data-testid="assetbalance-main-stage"] .tab-swipe-slide[data-active="true"]',
        )
        .evaluate((element) => (element as HTMLElement).style.transform);

    // 端(ブラウザの戻る領域)から引いても面は動かない
    await touchStart(page, 'assetbalance-main-stage', 5, 400);
    await touchMove(page, 'assetbalance-main-stage', 120, 400);
    await touchMove(page, 'assetbalance-main-stage', 200, 400);
    await page.waitForTimeout(300);
    expect(await currentTransform()).toBe('');
    await touchEnd(page, 'assetbalance-main-stage', 200, 400);
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    // 検索ダイアログを開いている間も面は動かない
    await page.getByTestId('asset-search-sheet-entry').click();
    await expect(page.getByTestId('asset-search-sheet')).toBeVisible();
    await touchStart(page, 'assetbalance-main-stage', 300, 400);
    await touchMove(page, 'assetbalance-main-stage', 190, 400);
    await page.waitForTimeout(300);
    expect(await currentTransform()).toBe('');
    await touchEnd(page, 'assetbalance-main-stage', 190, 400);
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });

  test('取引明細で左スワイプすると次のタブに切り替わり検索ダイアログ中は動かない', async ({
    page,
  }) => {
    await setupReceiptMocks(page);
    await page.goto('/receipts');
    await expect(page.getByRole('tab')).toHaveCount(3);

    await swipe(page, 'receipts-workspace', 300, 90);
    await expect(page.locator('#tab-dividend')).toHaveAttribute(
      'aria-selected',
      'false',
    );

    await page.getByTestId('receipts-search-sheet-entry').click();
    await expect(page.getByTestId('receipts-search-sheet')).toBeVisible();
    await swipe(page, 'receipts-workspace', 300, 200);
    await expect(page.locator('#tab-dividend')).toHaveAttribute(
      'aria-selected',
      'false',
    );
  });

  test('確定時は外側へ一方向にだけ進み切り替わった後は動かない', async ({
    page,
  }) => {
    await setupAssetMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await touchStart(page, 'assetbalance-main-stage', 300, 400);
    await touchMove(page, 'assetbalance-main-stage', 200, 400);
    await touchMove(page, 'assetbalance-main-stage', 90, 400);
    await waitForDrag(page, 'assetbalance-main-stage');

    const sampling = sampleSlideX(page, 'assetbalance-main-stage', 600);
    await touchEnd(page, 'assetbalance-main-stage', 90, 400);
    const samples = await sampling;
    const frames = samples.filter((value): value is number => value !== null);
    expect(frames.length).toBeGreaterThan(2);
    for (const value of frames) {
      expect(value).toBeLessThanOrEqual(5);
    }
    for (let index = 1; index < frames.length; index += 1) {
      expect(frames[index]).toBeLessThanOrEqual(frames[index - 1] + 1);
    }
    expect(Math.min(...frames)).toBeLessThan(-100);

    await expect(page.getByTestId('asset-kind-tab-fund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await page.waitForTimeout(500);
    expect(
      await activeSlideTransform(page, 'assetbalance-main-stage'),
    ).toBe('');
    await expect(page.getByTestId('asset-kind-tab-fund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });

  test('右スワイプの確定時も外側へ一方向にだけ進む', async ({ page }) => {
    await setupAssetMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();

    await swipe(page, 'assetbalance-main-stage', 300, 90);
    await expect(page.getByTestId('asset-kind-tab-fund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await touchStart(page, 'assetbalance-main-stage', 90, 400);
    await touchMove(page, 'assetbalance-main-stage', 200, 400);
    await touchMove(page, 'assetbalance-main-stage', 300, 400);
    await waitForDrag(page, 'assetbalance-main-stage');

    const initialX = await slideX(page, 'assetbalance-main-stage');
    expect(initialX).not.toBeNull();
    const sampling = sampleSlideX(page, 'assetbalance-main-stage', 600);
    await touchEnd(page, 'assetbalance-main-stage', 300, 400);
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    const samples = await sampling;
    const positions = [
      initialX as number,
      ...samples.filter((value): value is number => value !== null),
    ];
    expect(Math.max(...positions)).toBeGreaterThan(100);
    for (let index = 1; index < positions.length; index += 1) {
      expect(positions[index]).toBeGreaterThanOrEqual(positions[index - 1] - 1);
    }

    await page.waitForTimeout(500);
    expect(
      await activeSlideTransform(page, 'assetbalance-main-stage'),
    ).toBe('');
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });

  test('取り消し時は元へ戻り切り替わらない', async ({ page }) => {
    await setupAssetMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await touchStart(page, 'assetbalance-main-stage', 300, 400);
    await touchMove(page, 'assetbalance-main-stage', 270, 400);
    await waitForDrag(page, 'assetbalance-main-stage');
    await touchEnd(page, 'assetbalance-main-stage', 270, 400);

    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await page.waitForFunction(() => {
      const current = document.querySelector(
        '[data-testid="assetbalance-main-stage"] .tab-swipe-slide[data-active="true"]',
      );
      return current instanceof HTMLElement && current.style.transform === '';
    });
  });

  test('下線は指と同じ向きの隣のタブ側へ進む', async ({ page }) => {
    await setupAssetMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    const before = await inkPercent(page, 'assetbalance-main-stage');

    await touchStart(page, 'assetbalance-main-stage', 300, 400);
    await touchMove(page, 'assetbalance-main-stage', 190, 400);
    await page.waitForFunction(() => {
      const ink = document.querySelector('.tab-swipe-ink');
      return (
        ink instanceof HTMLElement && ink.dataset.on === 'true'
      );
    });
    const during = await inkPercent(page, 'assetbalance-main-stage');
    expect(during).not.toBeNull();
    expect(before).not.toBeNull();
    expect(during as number).toBeGreaterThan(before as number);

    await touchEnd(page, 'assetbalance-main-stage', 190, 400);
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });

  test('スワイプ途中で下線は青の1本だけ出る(資産管理)', async ({ page }) => {
    await setupAssetMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    const rest = await swipeUnderlineState(page, 'assetbalance-main-stage');
    expect(rest.inkCount).toBe(0);
    expect(rest.listAfterDisplay).not.toBe('none');
    expect(rest.listAfterColor).not.toBeNull();

    await touchStart(page, 'assetbalance-main-stage', 300, 400);
    await touchMove(page, 'assetbalance-main-stage', 190, 400);
    await waitForDrag(page, 'assetbalance-main-stage');
    const mid = await swipeUnderlineState(page, 'assetbalance-main-stage');
    // 選択中タブ側の下線は隠れ、指に追従する1本だけが残る。色は通常の下線と同じ青
    expect(mid.inkCount).toBe(1);
    expect(mid.inkBackground).toBe(rest.listAfterColor);
    expect(mid.listAfterDisplay).toBe('none');
    expect(
      mid.tabAfterDisplay === 'none' || mid.tabAfterContent === 'none',
      '選択中タブ側の下線は出ない',
    ).toBe(true);

    await touchMove(page, 'assetbalance-main-stage', 90, 400);
    await touchEnd(page, 'assetbalance-main-stage', 90, 400);
    await expect(page.getByTestId('asset-kind-tab-fund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await expect(page.getByTestId('assetbalance-main-stage')).toHaveAttribute(
      'data-settling',
      'false',
      { timeout: 2_000 },
    );
    const after = await swipeUnderlineState(page, 'assetbalance-main-stage');
    expect(after.inkCount).toBe(0);
    expect(after.listAfterDisplay).not.toBe('none');
  });

  test('スワイプ途中で下線は青の1本だけ出る(取引明細)', async ({ page }) => {
    await setupReceiptMocks(page);
    await page.goto('/receipts');
    await expect(page.getByRole('tab')).toHaveCount(3);
    const rest = await swipeUnderlineState(page, 'receipts-workspace');
    expect(rest.inkCount).toBe(0);
    expect(rest.listAfterDisplay).not.toBe('none');
    expect(rest.listAfterColor).not.toBeNull();

    await touchStart(page, 'receipts-workspace', 300, 400);
    await touchMove(page, 'receipts-workspace', 190, 400);
    await waitForDrag(page, 'receipts-workspace');
    const mid = await swipeUnderlineState(page, 'receipts-workspace');
    expect(mid.inkCount).toBe(1);
    expect(mid.inkBackground).toBe(rest.listAfterColor);
    expect(mid.listAfterDisplay).toBe('none');
    expect(
      mid.tabAfterDisplay === 'none' || mid.tabAfterContent === 'none',
      '選択中タブ側の下線は出ない',
    ).toBe(true);

    await touchMove(page, 'receipts-workspace', 90, 400);
    await touchEnd(page, 'receipts-workspace', 90, 400);
    await expect(page.locator('#tab-domesticstock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await expect(page.getByTestId('receipts-workspace')).toHaveAttribute(
      'data-settling',
      'false',
      { timeout: 2_000 },
    );
    const after = await swipeUnderlineState(page, 'receipts-workspace');
    expect(after.inkCount).toBe(0);
    expect(after.listAfterDisplay).not.toBe('none');
  });

  test('動きを減らす設定では即時に切り替わる', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await setupAssetMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await swipe(page, 'assetbalance-main-stage', 300, 90);
    await expect(page.getByTestId('asset-kind-tab-fund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    expect(
      await activeSlideTransform(page, 'assetbalance-main-stage'),
    ).toBe('');
  });

  test('動きの途中のタップと新しいスワイプを無視し終わったら受け付ける', async ({
    page,
  }) => {
    await setupAssetMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await touchStart(page, 'assetbalance-main-stage', 300, 400);
    await touchMove(page, 'assetbalance-main-stage', 200, 400);
    await touchMove(page, 'assetbalance-main-stage', 90, 400);
    await expect(page.getByTestId('assetbalance-main-stage')).toHaveAttribute(
      'data-settling',
      'true',
    );
    // 離した直後のタップと新しいスワイプは同じタスクで続けて起こす。
    // 間に待ちを挟むと動きが終わり、終わった後の入力として受け付けてしまう
    await page.getByTestId('assetbalance-main-stage').evaluate((stage) => {
      const point = (x: number) =>
        new Touch({ identifier: 1, target: stage, clientX: x, clientY: 400 });
      const touch = (type: string, init: TouchEventInit) =>
        stage.dispatchEvent(
          new TouchEvent(type, { ...init, bubbles: true, cancelable: true }),
        );
      touch('touchend', { changedTouches: [point(90)] });
      document
        .querySelector('[data-testid="asset-kind-tab-stock-mobile"]')
        ?.dispatchEvent(
          new MouseEvent('click', {
            bubbles: true,
            cancelable: true,
            detail: 1,
          }),
        );
      touch('touchstart', { touches: [point(90)] });
      touch('touchmove', { touches: [point(200)] });
      touch('touchmove', { touches: [point(300)] });
      touch('touchend', { changedTouches: [point(300)] });
    });

    // 動きの最中の前のタブへのタップと逆向きの新しいスワイプは無視され、隣へ切り替わる
    await expect(page.getByTestId('asset-kind-tab-fund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    // 終わりの合図がなくても一定時間で再開する
    await expect(page.getByTestId('assetbalance-main-stage')).toHaveAttribute(
      'data-settling',
      'false',
      { timeout: 2_000 },
    );
    await page.getByTestId('asset-kind-tab-stock-mobile').click();
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });

  test('引いている最中の2本目の指を無視する', async ({ page }) => {
    await setupAssetMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await touchStart(page, 'assetbalance-main-stage', 300, 400);
    await touchMove(page, 'assetbalance-main-stage', 200, 400);
    await expect(page.getByTestId('assetbalance-main-stage')).toHaveAttribute(
      'data-settling',
      'true',
    );
    // 2本目の指の接触は無視され、1本目の追従が続く
    await touchStart(page, 'assetbalance-main-stage', 250, 400);
    await touchMove(page, 'assetbalance-main-stage', 90, 400);
    await touchEnd(page, 'assetbalance-main-stage', 90, 400);
    await expect(page.getByTestId('asset-kind-tab-fund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });

  test('スワイプ中と直後にタブの帯が見えたまま同じ位置に残る(資産管理)', async ({
    page,
  }) => {
    await setupAssetMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    const rest = await mobileTabStrip(page, 'assetbalance-main-stage');
    expect(rest).not.toBeNull();

    await touchStart(page, 'assetbalance-main-stage', 300, 400);
    await touchMove(page, 'assetbalance-main-stage', 190, 400);
    await waitForDrag(page, 'assetbalance-main-stage');
    const mid = await mobileTabStrip(page, 'assetbalance-main-stage');
    expect(mid).not.toBeNull();
    expect(mid?.display).not.toBe('none');
    expect(mid?.opacity).toBe('1');
    expect(mid?.covered).toBe(false);
    expect(Math.abs((mid?.x ?? 0) - (rest?.x ?? 0))).toBeLessThanOrEqual(2);

    await touchMove(page, 'assetbalance-main-stage', 90, 400);
    await touchEnd(page, 'assetbalance-main-stage', 90, 400);
    await expect(page.getByTestId('asset-kind-tab-fund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await expect(page.getByTestId('assetbalance-main-stage')).toHaveAttribute(
      'data-settling',
      'false',
      { timeout: 2_000 },
    );
    const after = await mobileTabStrip(page, 'assetbalance-main-stage');
    expect(after).not.toBeNull();
    expect(after?.display).not.toBe('none');
    expect(after?.opacity).toBe('1');
    expect(after?.covered).toBe(false);
    expect(Math.abs((after?.x ?? 0) - (rest?.x ?? 0))).toBeLessThanOrEqual(2);
  });

  test('スワイプ中と直後にタブの帯が見えたまま同じ位置に残る(取引明細)', async ({
    page,
  }) => {
    await setupReceiptMocks(page);
    await page.goto('/receipts');
    await expect(page.getByRole('tab')).toHaveCount(3);
    const rest = await mobileTabStrip(page, 'receipts-workspace');
    expect(rest).not.toBeNull();

    await touchStart(page, 'receipts-workspace', 300, 400);
    await touchMove(page, 'receipts-workspace', 190, 400);
    await waitForDrag(page, 'receipts-workspace');
    const mid = await mobileTabStrip(page, 'receipts-workspace');
    expect(mid).not.toBeNull();
    expect(mid?.display).not.toBe('none');
    expect(mid?.opacity).toBe('1');
    expect(mid?.covered).toBe(false);
    expect(Math.abs((mid?.x ?? 0) - (rest?.x ?? 0))).toBeLessThanOrEqual(2);

    await touchMove(page, 'receipts-workspace', 90, 400);
    await touchEnd(page, 'receipts-workspace', 90, 400);
    await expect(page.locator('#tab-dividend')).toHaveAttribute(
      'aria-selected',
      'false',
    );
    const after = await mobileTabStrip(page, 'receipts-workspace');
    expect(after).not.toBeNull();
    expect(after?.display).not.toBe('none');
    expect(after?.opacity).toBe('1');
    expect(after?.covered).toBe(false);
    expect(Math.abs((after?.x ?? 0) - (rest?.x ?? 0))).toBeLessThanOrEqual(2);
  });

  test('タップ切替でもタブの帯は同じ位置に残る', async ({ page }) => {
    await setupAssetMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    const rest = await mobileTabStrip(page, 'assetbalance-main-stage');
    expect(rest).not.toBeNull();

    await page.getByTestId('asset-kind-tab-fund-mobile').click();
    await expect(page.getByTestId('asset-kind-tab-fund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    const after = await mobileTabStrip(page, 'assetbalance-main-stage');
    expect(after).not.toBeNull();
    expect(after?.display).not.toBe('none');
    expect(after?.opacity).toBe('1');
    expect(after?.covered).toBe(false);
    expect(Math.abs((after?.x ?? 0) - (rest?.x ?? 0))).toBeLessThanOrEqual(2);
  });

  test('スワイプ中は帯より下が1枚で動き帯より上にはみ出さない(資産管理)', async ({
    page,
  }) => {
    await setupAssetMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    const summarySelector = '[data-testid="asset-portfolio-summary"]';
    const rest = await swipePanelGeometry(
      page,
      'assetbalance-main-stage',
      summarySelector,
      '#assetbalance-stock-list',
    );
    expect(rest).not.toBeNull();

    await touchStart(page, 'assetbalance-main-stage', 300, 400);
    await touchMove(page, 'assetbalance-main-stage', 190, 400);
    await waitForDrag(page, 'assetbalance-main-stage');
    const mid = await swipePanelGeometry(
      page,
      'assetbalance-main-stage',
      summarySelector,
      '#assetbalance-stock-list',
    );
    expect(mid).not.toBeNull();
    expect(Math.abs((mid?.stripX ?? 0) - (rest?.stripX ?? 0))).toBeLessThanOrEqual(2);
    const summaryDx = (mid?.summaryLeft ?? 0) - (rest?.summaryLeft ?? 0);
    const listDx = (mid?.listLeft ?? 0) - (rest?.listLeft ?? 0);
    expect(summaryDx).toBeLessThan(-10);
    expect(listDx).toBeLessThan(-10);
    expect(Math.abs(summaryDx - listDx)).toBeLessThanOrEqual(4);
    expect(mid?.summaryTop ?? 0).toBeGreaterThanOrEqual((mid?.stripBottom ?? 0) - 1);

    await touchMove(page, 'assetbalance-main-stage', 90, 400);
    await touchEnd(page, 'assetbalance-main-stage', 90, 400);
    await expect(page.getByTestId('asset-kind-tab-fund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });

  test('スワイプ中は帯より下が1枚で動き帯より上にはみ出さない(取引明細)', async ({
    page,
  }) => {
    await setupReceiptRichMocks(page);
    await page.goto('/receipts');
    await expect(page.locator('#receipts-summary')).toBeVisible();
    const rest = await swipePanelGeometry(
      page,
      'receipts-workspace',
      '#receipts-summary',
      '#receipts-list',
    );
    expect(rest).not.toBeNull();

    await touchStart(page, 'receipts-workspace', 300, 400);
    await touchMove(page, 'receipts-workspace', 190, 400);
    await waitForDrag(page, 'receipts-workspace');
    const mid = await swipePanelGeometry(
      page,
      'receipts-workspace',
      '#receipts-summary',
      '#receipts-list',
    );
    expect(mid).not.toBeNull();
    expect(Math.abs((mid?.stripX ?? 0) - (rest?.stripX ?? 0))).toBeLessThanOrEqual(2);
    const summaryDx = (mid?.summaryLeft ?? 0) - (rest?.summaryLeft ?? 0);
    const listDx = (mid?.listLeft ?? 0) - (rest?.listLeft ?? 0);
    expect(summaryDx).toBeLessThan(-10);
    expect(listDx).toBeLessThan(-10);
    expect(Math.abs(summaryDx - listDx)).toBeLessThanOrEqual(4);
    expect(mid?.summaryTop ?? 0).toBeGreaterThanOrEqual((mid?.stripBottom ?? 0) - 1);

    await touchMove(page, 'receipts-workspace', 90, 400);
    await touchEnd(page, 'receipts-workspace', 90, 400);
    await expect(page.locator('#tab-domesticstock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });

  test('タップ切替でも帯は同じ位置に残り帯の下に集計が来る(資産管理)', async ({
    page,
  }) => {
    await setupAssetMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
    const rest = await swipePanelGeometry(
      page,
      'assetbalance-main-stage',
      '[data-testid="asset-portfolio-summary"]',
      '#assetbalance-stock-list',
    );
    expect(rest).not.toBeNull();

    await page.getByTestId('asset-kind-tab-fund-mobile').click();
    await expect(page.getByTestId('asset-kind-tab-fund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    const after = await swipePanelGeometry(
      page,
      'assetbalance-main-stage',
      '[data-testid="asset-portfolio-summary"]',
      '#assetbalance-fund-list',
    );
    expect(after).not.toBeNull();
    expect(Math.abs((after?.stripX ?? 0) - (rest?.stripX ?? 0))).toBeLessThanOrEqual(2);
    expect(after?.stripBottom ?? 0).toBeLessThanOrEqual((after?.summaryTop ?? 0) + 1);
  });

  test('タップ切替でも帯は同じ位置に残り帯の下に集計が来る(取引明細)', async ({
    page,
  }) => {
    await setupReceiptRichMocks(page);
    await page.goto('/receipts');
    await expect(page.locator('#receipts-summary')).toBeVisible();
    const rest = await swipePanelGeometry(
      page,
      'receipts-workspace',
      '#receipts-summary',
      '#receipts-list',
    );
    expect(rest).not.toBeNull();

    await page.locator('#tab-domesticstock-mobile').click();
    await expect(page.locator('#tab-domesticstock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    const after = await swipePanelGeometry(
      page,
      'receipts-workspace',
      '#receipts-summary',
      '#receipts-list',
    );
    expect(after).not.toBeNull();
    expect(Math.abs((after?.stripX ?? 0) - (rest?.stripX ?? 0))).toBeLessThanOrEqual(2);
    expect(after?.stripBottom ?? 0).toBeLessThanOrEqual((after?.summaryTop ?? 0) + 1);
  });
});
