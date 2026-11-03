import type { Page } from '@playwright/test';
import { expect, test, json, mockSession } from '../support/test';

function holding(index: number, purchase: number) {
  return {
    id: `asset-balance-${index}`,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: `99${String(index).padStart(2, '0')}`,
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
  funds: unknown[] = [],
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

  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) => {
    if (route.request().method() !== 'GET') {
      return route.abort();
    }
    return route.fulfill(
      json({ data: funds, total: funds.length, page: 1, per_page: 1000 }),
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

function groupFund(
  id: string,
  fund_name: string,
  account: string,
  market_value: number,
) {
  return {
    id,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    fund_name,
    dividends_course: '再投資型',
    account,
    shares: 10000,
    average_purchase_price: 10000,
    total_purchase_amount: 100000,
    base_price: 11000,
    base_price_day_change: null,
    base_price_month_change: null,
    market_value,
    profit_loss: 10000,
    profit_loss_rate: 10.0,
    total_return: null,
    price_as_of: '2026-10-20',
    price_source: 'api',
    nav_is_latest: true,
    expected_nav_date: null,
  };
}

// 商品別は国内株式・投資信託の2区分、口座別は特定・NISAの2区分に集まる
const GROUP_FUNDS = [
  groupFund('fund-a', 'ファンドA', '特定', 400_000),
  groupFund('fund-b', 'ファンドB', 'NISA', 100_000),
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
    await setupAssetBalanceMocks(page, GROUP_HOLDINGS, GROUP_DIVIDENDS, GROUP_FUNDS);
    await page.goto('/assetbalance/composition');

    const tabs = page.getByTestId('asset-composition-tabs');
    await expect(tabs.getByRole('tab', { name: '銘柄別' })).toHaveAttribute(
      'aria-selected',
      'true',
    );
    const list = page.getByTestId('asset-composition-list');
    await expect(list.getByTestId('comp-row')).toHaveCount(5);
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('ファンドA');
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('36.4%');
    await expect(page.getByTestId('asset-composition')).toContainText(
      '5銘柄',
    );
    await expect(page.getByTestId('asset-composition-donut')).toBeVisible();

    await tabs.getByRole('tab', { name: '商品別' }).click();
    await expect(list.getByTestId('comp-row')).toHaveCount(2);
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('国内株式');
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('54.5%');
    await expect(page.getByTestId('asset-composition')).toContainText(
      '2種類',
    );

    await tabs.getByRole('tab', { name: '口座別' }).click();
    await expect(list.getByTestId('comp-row')).toHaveCount(2);
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('特定');
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('81.8%');
    await expect(page.getByTestId('asset-composition')).toContainText(
      '2口座',
    );
  });

  test('基準の切り替えで注記と金額の集まりが変わる', async ({ page }) => {
    await setupAssetBalanceMocks(page, GROUP_HOLDINGS, GROUP_DIVIDENDS, GROUP_FUNDS);
    await page.goto('/assetbalance/composition');

    const list = page.getByTestId('asset-composition-list');
    await expect(list.getByTestId('comp-row')).toHaveCount(5);
    await expect(page.getByTestId('page-skeleton-note-text')).toContainText(
      '評価額の比率',
    );
    await expect(page.getByTestId('page-skeleton-note-text')).toContainText(
      '¥1,100,000',
    );

    await page.getByTestId('comp-basis').getByRole('button', { name: '年間配当' }).click();
    await expect(page.getByTestId('page-skeleton-note-text')).toContainText(
      '年間配当(税引前)の比率',
    );
    await expect(page.getByTestId('page-skeleton-note-text')).toContainText(
      '¥25,000',
    );
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('36.0%');

    await page
      .getByTestId('asset-composition-tabs')
      .getByRole('tab', { name: '商品別' })
      .click();
    await expect(list.getByTestId('comp-row')).toHaveCount(2);
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('100.0%');
  });

  test('小さい区分も個別の切れと行で出す', async ({ page }) => {
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

    const list = page.getByTestId('asset-composition-list');
    await expect(list.getByTestId('comp-row')).toHaveCount(13);
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('93.8%');
    await expect(list.getByText('その他')).toHaveCount(0);
    // 13銘柄すべてを1切れずつ出す
    await expect(
      page.getByTestId('asset-composition-donut').getByTestId('comp-slice'),
    ).toHaveCount(13);
    // 凡例は比率の大きい順に上位5件、残りは「ほか8件」にまとめる
    const legend = page.getByTestId('asset-composition-legend');
    await expect(legend.getByTestId('comp-legend-item')).toHaveCount(5);
    await expect(legend.getByTestId('comp-legend-item').nth(0)).toContainText('93.8%');
    await expect(page.getByTestId('comp-legend-others')).toHaveText('ほか8件');
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

    await expect(page.getByTestId('asset-composition')).toContainText(
      '1銘柄',
    );
    const list = page.getByTestId('asset-composition-list');
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

    const list = page.getByTestId('asset-composition-list');
    await expect(list.getByTestId('comp-row')).toHaveCount(2);
    await expect(list.getByTestId('comp-row').nth(0)).toContainText('100.0%');
    // 算出不可の行も個別に残し、「その他」にはまとめない
    const missing = list.getByTestId('comp-row').nth(1);
    await expect(missing).toContainText('価格なし銘柄');
    await expect(missing).toContainText('—');
    await expect(list.getByText('その他')).toHaveCount(0);
  });

  test('切れの数は行と同じで凡例は上位5件まで出す', async ({
    page,
  }) => {
    await setupAssetBalanceMocks(page, GROUP_HOLDINGS, GROUP_DIVIDENDS, GROUP_FUNDS);
    await page.goto('/assetbalance/composition');

    const list = page.getByTestId('asset-composition-list');
    const rows = list.getByTestId('comp-row');
    await expect(rows).toHaveCount(5);
    const donut = page.getByTestId('asset-composition-donut');
    await expect(donut.getByTestId('comp-slice')).toHaveCount(5);
    const legend = page.getByTestId('asset-composition-legend');
    const items = legend.getByTestId('comp-legend-item');
    await expect(items).toHaveCount(5);
    await expect(items.nth(0)).toContainText('ファンドA');
    await expect(items.nth(0)).toContainText('36.4%');
    // 5件以下なので「ほか」の行は出さない
    await expect(page.getByTestId('comp-legend-others')).toHaveCount(0);
  });

  test('件数行の右端から構成比へ進め、見出し行は無い', async ({ page }) => {
    await setupAssetBalanceMocks(page, GROUP_HOLDINGS, GROUP_DIVIDENDS, GROUP_FUNDS);
    await page.goto('/assetbalance');

    // 『保有銘柄 ◯件』の見出し行は無い
    await page.getByTestId('asset-kind-tab-stock').click();
    await expect(page.getByTestId('asset-stock-heading')).toHaveCount(0);
    await expect(page.getByText('保有銘柄')).toHaveCount(0);

    // 3タブとも件数と同じ行の右端に入口があり、押せる高さを持つ
    const tabs = [
      { tab: 'asset-kind-tab-stock', count: '3件 / 3件' },
      { tab: 'asset-kind-tab-fund', count: '2件 / 2件' },
      { tab: 'asset-kind-tab-all', count: '5件 / 5件' },
    ] as const;
    for (const { tab, count } of tabs) {
      await page.getByTestId(tab).click();
      const countLine = page.getByTestId('asset-search-count-mobile');
      await expect(countLine).toContainText(count);
      const entry = page.getByTestId('asset-composition-link');
      await expect(entry).toBeVisible();
      await expect(entry).toContainText('構成比を見る');
      const countBox = await countLine.boundingBox();
      const entryBox = await entry.boundingBox();
      expect(countBox).not.toBeNull();
      expect(entryBox).not.toBeNull();
      if (countBox && entryBox) {
        expect(entryBox.x).toBeGreaterThan(countBox.x);
        expect(entryBox.y).toBeLessThanOrEqual(countBox.y + countBox.height);
        expect(entryBox.y + entryBox.height).toBeGreaterThanOrEqual(countBox.y);
        expect(entryBox.height).toBeGreaterThanOrEqual(44);
      }
    }

    await page.getByTestId('asset-kind-tab-stock').click();
    await page.getByTestId('asset-composition-link').click();
    await expect(page).toHaveURL('/assetbalance/composition');
    await expect(page.getByRole('link', { name: '資産管理に戻る' })).toBeVisible();
    await page.getByRole('link', { name: '資産管理に戻る' }).click();
    await expect(page).toHaveURL('/assetbalance');
  });
});

test('PCでもスマホと同じ部品で評価額順に全保有を並べる', async ({
  page,
}) => {
  await setupAssetBalanceMocks(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/assetbalance/composition');

  // PCでもスマホと同じ部品(タブ・基準切り替え・ドーナツ・一覧)を使う
  await expect(page.getByTestId('asset-composition')).toBeVisible();
  await expect(page.getByTestId('asset-composition-tabs')).toBeVisible();
  await expect(page.getByTestId('asset-composition-donut')).toBeVisible();
  await expect(page.getByTestId('page-skeleton-note-text')).toContainText(
    '評価額の比率',
  );
  await expect(page.getByTestId('page-skeleton-note-text')).toContainText(
    '¥220,000',
  );

  const items = page.getByTestId('asset-composition-list').getByRole('listitem');
  await expect(items).toHaveCount(22);
  await expect(items.nth(0)).toContainText('銘柄01');
  await expect(items.nth(0)).toContainText('4.5%');
  await expect(items.nth(21)).toContainText('銘柄22');
  // 切れの数は行と同じ
  await expect(
    page.getByTestId('asset-composition-donut').getByTestId('comp-slice'),
  ).toHaveCount(22);
  // 構成比の棒は本番 CSP 下でも描画される(幅の固定値は見ない)
  const bar = items.nth(0).locator('.comp-bar-fill');
  await expect(bar).toBeVisible();
  expect((await bar.boundingBox())?.width).toBeGreaterThan(0);

  // トップからの導線
  await page.goto('/assetbalance');
  await page.getByRole('link', { name: '構成比を見る' }).click();
  await expect(page).toHaveURL('/assetbalance/composition');
});

test('評価額が出せない銘柄は構成比を出さずに一覧へ残す', async ({ page }) => {
  const gifted = {
    ...holding(2, 0),
    security_name: '贈与銘柄',
    current_price: 0,
  };
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

test('評価額の合計が0のときは算出不可を表示する', async ({ page }) => {
  const holdings = [
    { ...holding(1, 0), current_price: 0 },
    { ...holding(2, 0), current_price: 0 },
  ];
  await setupAssetBalanceMocks(page, holdings);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/assetbalance/composition');

  await expect(page.getByText('算出不可')).toBeVisible();
});
