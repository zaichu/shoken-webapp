import type { Page } from '@playwright/test';
import { expect, test, json, mockSession } from '../support/test';

function holding(index: number, purchase: number) {
  return {
    id: `asset-balance-${index}`,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: `70${String(index).padStart(2, '0')}`,
    security_name: `銘柄${String(index).padStart(2, '0')}`,
    shares: 100,
    average_purchase_price: 100,
    total_purchase_amount: purchase,
    current_price: 100,
    daily_change: 0,
  };
}

// 22銘柄: 先頭は8桁の取得総額、末尾はマイナス取得額
const HOLDINGS = [
  holding(1, 70_000_000),
  ...Array.from({ length: 20 }, (_, i) => holding(i + 2, (20 - i) * 10_000)),
  holding(22, -10_000),
];

async function setupAssetBalanceMocks(
  page: Page,
  holdings: ReturnType<typeof holding>[] = HOLDINGS,
  dividendItems: unknown[] = [],
) {
  await mockSession(page);

  const totalPurchase = holdings.reduce(
    (sum, h) => sum + h.total_purchase_amount,
    0,
  );

  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => {
    if (route.request().method() !== 'GET') {
      return route.abort();
    }
    return route.fulfill(
      json({
        data: holdings,
        total: holdings.length,
        page: 1,
        per_page: 1000,
        summary: {
          total_purchase_amount: totalPurchase,
          total_market_value: totalPurchase,
          total_daily_change: 0,
          priced_purchase_amount: totalPurchase,
          priced_market_value: totalPurchase,
        },
      }),
    );
  });

  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: dividendItems })),
  );
}

function marketHolding(
  index: number,
  overrides: Record<string, unknown> = {},
) {
  return {
    ...holding(index, 100_000),
    ...overrides,
  };
}

function dividendItem(security_code: string, dividend_per_share: number) {
  return {
    security_code,
    dividend_per_share,
    status: 'ok',
    fetched_at: '2026-01-01T00:00:00Z',
    is_stale: false,
  };
}

// 口座別と商品別で集まる3行: 9904を特定・NISAで2行、9914を1行
const GROUP_HOLDINGS = [
  marketHolding(1, {
    id: 'asset-balance-a',
    security_code: '9904',
    security_name: '架空自動車',
    account: '特定',
    shares: 100,
    current_price: 3000,
  }),
  marketHolding(2, {
    id: 'asset-balance-b',
    security_code: '9904',
    security_name: '架空自動車',
    account: 'NISA',
    shares: 100,
    current_price: 1000,
  }),
  marketHolding(3, {
    id: 'asset-balance-c',
    security_code: '9914',
    security_name: '架空電機',
    account: '特定',
    shares: 100,
    current_price: 2000,
  }),
];

const GROUP_DIVIDENDS = [
  dividendItem('9904', 90),
  dividendItem('9914', 70),
];

test.describe('スマホの構成比', () => {
  test.use({ viewport: { width: 390, height: 844 } });

  test('区分の切り替えで銘柄別・商品別・口座別の集まりが変わる', async ({
    page,
  }) => {
    await setupAssetBalanceMocks(page, GROUP_HOLDINGS, GROUP_DIVIDENDS);
    await page.goto('/assetbalance/composition');

    const tabs = page.getByTestId('asset-composition-tabs');
    await expect(tabs.getByRole('tab', { name: '銘柄別' })).toHaveAttribute(
      'aria-selected',
      'true',
    );
    const list = page.getByTestId('asset-composition-mobile-list');
    await expect(list.getByTestId('comp-row')).toHaveCount(3);
    await expect(page.getByTestId('asset-composition-donut')).toBeVisible();

    await tabs.getByRole('tab', { name: '商品別' }).click();
    await expect(list.getByTestId('comp-row')).toHaveCount(2);
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('架空自動車');
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('66.7%');

    await tabs.getByRole('tab', { name: '口座別' }).click();
    await expect(list.getByTestId('comp-row')).toHaveCount(2);
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('特定');
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('83.3%');
  });

  test('基準の切り替えで注記と金額の集まりが変わる', async ({ page }) => {
    await setupAssetBalanceMocks(page, GROUP_HOLDINGS, GROUP_DIVIDENDS);
    await page.goto('/assetbalance/composition');

    const list = page.getByTestId('asset-composition-mobile-list');
    await expect(list.getByTestId('comp-row')).toHaveCount(3);
    await expect(page.getByTestId('asset-composition-note')).toContainText(
      '評価額の比率',
    );
    await expect(page.getByTestId('asset-composition-note')).toContainText(
      '¥600,000',
    );

    await page.getByTestId('comp-basis').getByRole('button', { name: '年間配当' }).click();
    await expect(page.getByTestId('asset-composition-note')).toContainText(
      '年間配当(税引前)の比率',
    );
    await expect(page.getByTestId('asset-composition-note')).toContainText(
      '¥25,000',
    );
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('36.0%');

    await page
      .getByTestId('asset-composition-tabs')
      .getByRole('tab', { name: '商品別' })
      .click();
    await expect(list.getByTestId('comp-row')).toHaveCount(2);
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('72.0%');
  });

  test('小さい区分はその他にまとまる', async ({ page }) => {
    const holdings = [
      marketHolding(1, {
        id: 'asset-balance-big',
        security_code: '9901',
        security_name: '架空自動車',
        shares: 900,
        current_price: 1000,
      }),
      ...Array.from({ length: 12 }, (_, i) =>
        marketHolding(i + 2, {
          id: `asset-balance-small-${i}`,
          security_code: `99${String(10 + i)}`,
          security_name: `銘柄${String(i).padStart(2, '0')}`,
          shares: 50,
          current_price: 100,
        }),
      ),
    ];
    await setupAssetBalanceMocks(page, holdings);
    await page.goto('/assetbalance/composition');

    const list = page.getByTestId('asset-composition-mobile-list');
    await expect(list.getByText('その他 12銘柄')).toHaveCount(1);
    await expect(
      page.getByTestId('asset-composition-donut').getByTestId('comp-slice-other'),
    ).toHaveCount(1);
    await expect(
      page.getByTestId('asset-composition-donut').getByText('その他 12銘柄'),
    ).toBeVisible();
  });

  test('1銘柄だけでも件数と100%を出す', async ({ page }) => {
    await setupAssetBalanceMocks(page, [
      marketHolding(1, {
        security_code: '9904',
        security_name: '架空自動車',
        shares: 100,
        current_price: 3000,
      }),
    ]);
    await page.goto('/assetbalance/composition');

    await expect(page.getByTestId('asset-composition-mobile')).toContainText(
      '1銘柄',
    );
    const list = page.getByTestId('asset-composition-mobile-list');
    await expect(list.getByTestId('comp-row')).toHaveCount(1);
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('100.0%');
  });

  test('評価額が出せない行は割合を出さずに一覧へ残す', async ({ page }) => {
    await setupAssetBalanceMocks(page, [
      marketHolding(1, {
        security_code: '9904',
        security_name: '架空自動車',
        shares: 100,
        current_price: 3000,
      }),
      marketHolding(2, {
        security_code: '9914',
        security_name: '価格なし銘柄',
        shares: 100,
        current_price: 0,
      }),
    ]);
    await page.goto('/assetbalance/composition');

    const list = page.getByTestId('asset-composition-mobile-list');
    await expect(list.getByTestId('comp-row')).toHaveCount(2);
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('100.0%');
    const other = list.getByTestId('comp-row').filter({ hasText: 'その他 1銘柄' });
    await expect(other).toHaveCount(1);
    await expect(other).toContainText('—');
  });

  test('一覧の見出しから構成比へ進め、帯の入口は無い', async ({ page }) => {
    await setupAssetBalanceMocks(page, GROUP_HOLDINGS, GROUP_DIVIDENDS);
    await page.goto('/assetbalance');
    await page.getByTestId('asset-kind-tab-stock').click();

    const heading = page.getByTestId('asset-stock-heading');
    await expect(heading).toContainText('保有銘柄');
    await expect(heading).toContainText('3件');
    await expect(page.getByText('構成比 ›', { exact: true })).toHaveCount(0);
    await heading.getByRole('link', { name: '構成比を見る' }).click();
    await expect(page).toHaveURL('/assetbalance/composition');
  });
});

test('構成比詳細は直接 URL で開け、基準と対象を示して取得額順に全保有を並べる', async ({
  page,
}) => {
  await setupAssetBalanceMocks(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/assetbalance/composition');

  const desktop = page.getByTestId('asset-composition-desktop');
  await expect(desktop.getByText('対象: 保存済みの保有全体')).toBeVisible();
  await expect(desktop.getByText('基準(取得総額):')).toBeVisible();
  await expect(desktop.getByText('¥72,110,000')).toBeVisible();

  const items = page.getByTestId('asset-composition-list').getByRole('listitem');
  await expect(items).toHaveCount(22);
  await expect(items.nth(0)).toContainText('銘柄01');
  await expect(items.nth(0)).toContainText('97.1%');
  // 取得額が少ない順に末尾へ並ぶ。0・マイナスも一覧には残る
  await expect(items.nth(21)).toContainText('銘柄22');
  // 構成比の棒は本番 CSP 下でも描画される
  const bar = items.nth(0).locator('.bg-accent');
  expect((await bar.boundingBox())?.width).toBeGreaterThan(100);

  // トップからの導線
  await page.goto('/assetbalance');
  await page.getByRole('link', { name: '構成比を見る' }).click();
  await expect(page).toHaveURL('/assetbalance/composition');
});

test('取得額が0の銘柄は構成比を出さずに一覧へ残す', async ({ page }) => {
  const gifted = { ...holding(2, 0), security_name: '贈与銘柄' };
  await setupAssetBalanceMocks(page, [holding(1, 100_000), gifted]);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/assetbalance/composition');

  const items = page.getByTestId('asset-composition-list').getByRole('listitem');
  await expect(items).toHaveCount(2);
  await expect(items.nth(0)).toContainText('銘柄01');
  await expect(items.nth(0)).toContainText('100.0%');
  const giftedItem = items.filter({ hasText: '贈与銘柄' });
  await expect(giftedItem).toHaveCount(1);
  await expect(giftedItem).toContainText('—');
});

test('取得総額の合計が0のときは算出不可を表示する', async ({ page }) => {
  const holdings = [holding(1, 0), holding(2, 0)];
  await setupAssetBalanceMocks(page, holdings);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/assetbalance/composition');

  await expect(page.getByText('算出不可')).toBeVisible();
});
