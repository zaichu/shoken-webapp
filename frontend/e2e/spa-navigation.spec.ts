import { expect, test, type Page } from '@playwright/test';
import { json, paginated } from './support/test';

const paginatedAll = (data: unknown[], extra: Record<string, unknown> = {}) =>
  paginated(data, { per_page: 1000, ...extra });

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const DIVIDEND = {
  id: '00000000-0000-0000-0000-000000000001',
  settlement_date: '2024-03-01',
  product: '特定口座',
  account: 'SBI証券',
  security_code: '7203',
  security_name: 'トヨタ自動車',
  unit_price: 30.0,
  shares: 100,
  dividends_before_tax: 3000,
  taxes: 609,
  net_amount_received: 2391,
  created_at: '2024-03-01T00:00:00Z',
  updated_at: '2024-03-01T00:00:00Z',
};

const ASSET = {
  id: 'asset-balance-1',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '7203',
  security_name: 'トヨタ自動車',
  shares: 100,
  executing_shares: 0,
  average_purchase_price: 2500,
  total_purchase_amount: 250000,
  current_price: 2600,
  daily_change: 50,
  market_value: 260000,
  profit_loss_rate: 4.0,
};

const STOCK = {
  date: '2024-03-01',
  code: '7203',
  name: 'トヨタ自動車',
  market_category: 'プライム',
  industry_category_33: '輸送用機器',
  industry_category_17: '自動車・輸送機',
  size_category: '大型',
};



interface ApiCounts {
  session: number;
  dividends: number;
  assetBalances: number;
}

async function setupApiMocks(page: Page, counts: ApiCounts, authed = true) {
  await page.route('**/api/v1/**', (route) => route.fulfill(json(paginatedAll([]))));
  await page.route('**/api/v1/session', (route) => {
    counts.session += 1;
    return route.fulfill(
      authed
        ? json(MOCK_USER)
        : { status: 401, contentType: 'application/json', body: '{}' },
    );
  });
  await page.route('**/api/v1/asset-balances**', (route) => {
    counts.assetBalances += 1;
    return route.fulfill(
      json(
        paginatedAll([ASSET], {
          summary: {
            total_purchase_amount: 250000,
            total_market_value: 260000,
            total_daily_change: 50,
          },
          facets: { securities: [{ value: '7203', label: 'トヨタ自動車', count: 1 }] },
        }),
      ),
    );
  });
  await page.route('**/api/v1/dividends**', (route) => {
    // ホームの集計呼び出し(year 指定)は一覧件数のカウントに含めない
    if (!route.request().url().includes('year=')) {
      counts.dividends += 1;
    }
    return route.fulfill(json(paginatedAll([DIVIDEND])));
  });
  await page.route('**/api/v1/stocks**', (route) => route.fulfill(json(STOCK)));
}

function nav(page: Page, name: string) {
  return page
    .getByRole('navigation', { name: '主要ナビゲーション' })
    .getByRole('link', { name, exact: true });
}

async function marker(page: Page) {
  return page.evaluate(() => (window as unknown as { __spaMarker?: number }).__spaMarker);
}

async function setMarker(page: Page) {
  await page.evaluate(() => {
    (window as unknown as { __spaMarker?: number }).__spaMarker = 42;
  });
}

test('ナビ遷移はページリロードせず session も再取得しない', async ({ page }) => {
  const counts: ApiCounts = { session: 0, dividends: 0, assetBalances: 0 };
  await setupApiMocks(page, counts);

  await page.goto('/');
  await expect(page.getByTestId('home-overview')).toBeVisible();
  await setMarker(page);

  await nav(page, '取引明細').click();
  await expect(page.getByRole('cell', { name: 'トヨタ自動車' }).first()).toBeVisible();
  await expect(page).toHaveURL(/\/receipts$/);
  await expect(page).toHaveTitle('取引明細 - 証券Web');
  // SPA 遷移後はフォーカスを main に移してキーボード操作の文脈を維持する
  expect(await page.evaluate(() => document.activeElement?.id)).toBe('main-content');
  expect(await marker(page)).toBe(42);

  await nav(page, '資産管理').click();
  await expect(
    page.getByTestId('assetbalance-main-stage').getByText('トヨタ自動車').first(),
  ).toBeVisible();
  await expect(page).toHaveURL(/\/assetbalance$/);
  await expect(page).toHaveTitle('資産管理 - 証券Web');

  await nav(page, '銘柄検索').click();
  await expect(page.getByRole('textbox', { name: '銘柄コードまたは銘柄名' })).toBeVisible();
  await expect(page).toHaveURL(/\/search$/);

  // wasm や HTML の再取得が起きていない(マーカーが残る)、session も初回の1回だけ
  expect(await marker(page)).toBe(42);
  expect(counts.session).toBe(1);
});

test('アクティブなナビリンクの aria-current が遷移に追随する', async ({ page }) => {
  const counts: ApiCounts = { session: 0, dividends: 0, assetBalances: 0 };
  await setupApiMocks(page, counts);

  await page.goto('/');
  await nav(page, '取引明細').click();
  await expect(page.getByRole('cell', { name: 'トヨタ自動車' }).first()).toBeVisible();

  await expect(nav(page, '取引明細')).toHaveAttribute('aria-current', 'page');
  await expect(nav(page, '資産管理')).not.toHaveAttribute('aria-current');
});

test('再訪では表示済みデータを残したまま裏で取り直す', async ({ page }) => {
  const counts: ApiCounts = { session: 0, dividends: 0, assetBalances: 0 };
  await setupApiMocks(page, counts);
  // 2回目以降の配当一覧取得を遅らせ、裏取得中に既存表示が残ることを確かめる
  await page.unroute('**/api/v1/dividends**');
  await page.route('**/api/v1/dividends**', async (route) => {
    if (!route.request().url().includes('year=')) {
      counts.dividends += 1;
      if (counts.dividends > 1) {
        await new Promise((resolve) => setTimeout(resolve, 800));
      }
    }
    return route.fulfill(json(paginatedAll([DIVIDEND])));
  });

  await page.goto('/');
  await nav(page, '取引明細').click();
  await expect(page.getByRole('cell', { name: 'トヨタ自動車' }).first()).toBeVisible();
  expect(counts.dividends).toBe(1);

  await nav(page, '資産管理').click();
  await expect(
    page.getByTestId('assetbalance-main-stage').getByText('トヨタ自動車').first(),
  ).toBeVisible();

  const secondFetch = page.waitForRequest(/\/api\/v1\/dividends/);
  await nav(page, '取引明細').click();
  // 裏再取得の応答を待たずにキャッシュ済みの行が見えている(スケルトンに戻らない)
  await expect(page.getByRole('cell', { name: 'トヨタ自動車' }).first()).toBeVisible();
  await secondFetch;
  await expect.poll(() => counts.dividends).toBe(2);
  await expect(page.getByRole('cell', { name: 'トヨタ自動車' }).first()).toBeVisible();
});

test('裏再取得が失敗しても表示済みの行を残してエラーを知らせる', async ({ page }) => {
  const counts: ApiCounts = { session: 0, dividends: 0, assetBalances: 0 };
  await setupApiMocks(page, counts);
  // 再訪時の裏再取得だけを失敗させる
  await page.unroute('**/api/v1/dividends**');
  await page.route('**/api/v1/dividends**', (route) => {
    if (route.request().url().includes('year=')) {
      return route.fulfill(json(paginatedAll([DIVIDEND])));
    }
    counts.dividends += 1;
    return counts.dividends === 1
      ? route.fulfill(json(paginatedAll([DIVIDEND])))
      : route.fulfill({ status: 500, contentType: 'application/json', body: '{}' });
  });

  await page.goto('/');
  await nav(page, '取引明細').click();
  await expect(page.getByRole('cell', { name: 'トヨタ自動車' }).first()).toBeVisible();

  await nav(page, '資産管理').click();
  await expect(
    page.getByTestId('assetbalance-main-stage').getByText('トヨタ自動車').first(),
  ).toBeVisible();

  const refresh = page.waitForResponse(/\/api\/v1\/dividends/);
  await nav(page, '取引明細').click();
  await refresh;
  // 失敗しても表示済みの行は消さず、エラーを alert で知らせる
  await expect(page.getByRole('cell', { name: 'トヨタ自動車' }).first()).toBeVisible();
  await expect(page.getByRole('alert')).toContainText('サーバーエラーが発生しました');
});

test('ブラウザの戻る・進むでページが切り替わる', async ({ page }) => {
  const counts: ApiCounts = { session: 0, dividends: 0, assetBalances: 0 };
  await setupApiMocks(page, counts);

  await page.goto('/');
  await setMarker(page);
  await nav(page, '取引明細').click();
  await expect(page.getByRole('cell', { name: 'トヨタ自動車' }).first()).toBeVisible();

  await page.goBack();
  await expect(page).toHaveURL(/\/$/);
  await expect(page.getByTestId('home-overview')).toBeVisible();
  expect(await marker(page)).toBe(42);

  await page.goForward();
  await expect(page).toHaveURL(/\/receipts$/);
  await expect(page.getByRole('cell', { name: 'トヨタ自動車' }).first()).toBeVisible();
});

test('未ログインで保護ページに行くと /login に置き換え遷移する', async ({ page }) => {
  const counts: ApiCounts = { session: 0, dividends: 0, assetBalances: 0 };
  await setupApiMocks(page, counts, false);

  await page.goto('/');
  await expect(page.getByTestId('home-overview')).toHaveCount(0);
  await setMarker(page);

  await nav(page, '取引明細').click();
  await expect(page).toHaveURL(/\/login$/);
  await expect(page.getByRole('button', { name: 'Googleでログイン' })).toBeVisible();
  expect(await marker(page)).toBe(42);
  expect(counts.session).toBe(1);

  await page.goBack();
  await expect(page).toHaveURL(/\/$/);
});

test('明細の銘柄コードリンクから検索結果へ遷移する', async ({ page }) => {
  const counts: ApiCounts = { session: 0, dividends: 0, assetBalances: 0 };
  await setupApiMocks(page, counts);

  await page.goto('/receipts');
  await expect(page.getByRole('cell', { name: 'トヨタ自動車' }).first()).toBeVisible();

  await page.locator('a[href="/search?code=7203"]').first().click();
  await expect(page).toHaveURL(/\/search\?code=7203$/);
  await expect(page.getByRole('textbox', { name: '銘柄コードまたは銘柄名' })).toBeVisible();
  await expect(page.getByText('トヨタ自動車').first()).toBeVisible();
});

test('スキップリンクは同一ページ内の移動のまま余計な API を出さない', async ({ page }) => {
  const counts: ApiCounts = { session: 0, dividends: 0, assetBalances: 0 };
  await setupApiMocks(page, counts);

  await page.goto('/');
  await expect(page.getByTestId('home-overview')).toBeVisible();
  const apisBefore = counts.session + counts.dividends + counts.assetBalances;

  // スキップリンクはフォーカスで初めて表示されるためキーボードで起動する
  await page.locator('.skip-link').focus();
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/\/#main-content$/);
  await expect(page.getByTestId('home-overview')).toBeVisible();
  await page.waitForTimeout(300);
  expect(counts.session + counts.dividends + counts.assetBalances).toBe(apisBefore);
});

test('修飾キー付きクリックはブラウザ既定の新規タブで開く', async ({ page, context }) => {
  const counts: ApiCounts = { session: 0, dividends: 0, assetBalances: 0 };
  await setupApiMocks(page, counts);

  await page.goto('/');
  await expect(page.getByTestId('home-overview')).toBeVisible();

  const [popup] = await Promise.all([
    context.waitForEvent('page'),
    nav(page, '銘柄検索').click({ modifiers: ['Control'] }),
  ]);
  try {
    await popup.waitForLoadState('domcontentloaded');
    expect(new URL(popup.url()).pathname).toBe('/search');
    // 元のページは遷移していない
    await expect(page).toHaveURL(/\/$/);
    await expect(page.getByTestId('home-overview')).toBeVisible();
  } finally {
    await popup.close();
  }
});
