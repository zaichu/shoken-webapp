import type { Page } from '@playwright/test';
import { expect, test, mockSession } from '../support/test';

const MOCK_STOCK = {
  date: '2024-03-01',
  code: '9905',
  name: '架空ゲーム',
  market_category: 'プライム',
  industry_category_33: '情報・通信業',
  industry_category_17: '情報通信・サービスその他',
  size_category: 'TOPIX Large70',
};

const ROUTES = {
  // query string を含む URL でも一致するよう path 部分のみでマッチ
  stock: /\/api\/v1\/stocks(?:\?.*)?$/,
};

async function setupAuthMocks(page: Page) {
  await mockSession(page);
  await page.route(ROUTES.stock, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_STOCK),
    }),
  );
}

test('検索ページの初期表示で検索フォームが表示される', async ({ page }) => {
  await setupAuthMocks(page);

  await page.goto('/search');
  await page.waitForLoadState('networkidle');

  await expect(page.getByRole('searchbox', { name: '銘柄コードまたは銘柄名で検索' })).toBeVisible();
});

test('PC では検索フォームが本文の上部に1つだけ表示される', async ({ page }) => {
  await setupAuthMocks(page);
  await page.setViewportSize({ width: 1440, height: 900 });

  await page.goto('/search');
  await page.waitForLoadState('networkidle');

  // PC は下部操作帯を出さないため、見える検索領域は本文直上の1つだけ
  const search = page.getByRole('search', { name: '銘柄検索' });
  await expect(search).toHaveCount(1);
  await expect(
    search.getByRole('searchbox', { name: '銘柄コードまたは銘柄名で検索' }),
  ).toBeVisible();
});

test('モバイルでも検索フォームが見える', async ({ page }) => {
  await setupAuthMocks(page);
  await page.setViewportSize({ width: 390, height: 844 });

  await page.goto('/search');
  await page.waitForLoadState('networkidle');

  const search = page.getByRole('search', { name: '銘柄検索' });
  await expect(search).toHaveCount(1);
  await expect(
    search.getByRole('searchbox', { name: '銘柄コードまたは銘柄名で検索' }),
  ).toBeVisible();
});

test('検索フォームに文字を入力できる', async ({ page }) => {
  await setupAuthMocks(page);

  await page.goto('/search');
  await page.waitForLoadState('networkidle');

  const searchInput = page.getByRole('searchbox', { name: '銘柄コードまたは銘柄名で検索' });
  await searchInput.fill('架空ゲーム');

  await expect(searchInput).toHaveValue('架空ゲーム');
});

test('検索の取得が失敗したとき再読み込みで再取得できる', async ({ page }) => {
  await mockSession(page);
  // 500 はクライアント側で1回リトライされるので、初回 dispatch は2リクエスト分を失敗させる
  let stockCalls = 0;
  await page.route(ROUTES.stock, (route) => {
    stockCalls += 1;
    if (stockCalls <= 2) {
      return route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'server error' }),
      });
    }
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_STOCK),
    });
  });

  await page.goto('/search?code=9905');

  const error = page.getByTestId('list-load-error');
  await expect(error).toBeVisible();
  await expect(error.getByRole('alert')).toContainText('エラー:');

  await page.getByRole('button', { name: '再読み込み' }).click();
  await expect(page.getByText('架空ゲーム')).toBeVisible();
  expect(stockCalls).toBe(3);
});

test('検索ボタンの読み込み属性がリクエストの開始と完了に連動する', async ({ page }) => {
  await setupAuthMocks(page);
  let finish!: () => void;
  const pending = new Promise<void>((resolve) => { finish = resolve; });
  await page.route(ROUTES.stock, async (route) => {
    await pending;
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_STOCK),
    });
  });

  await page.goto('/search?code=9905');
  const loading = page.getByRole('button', { name: '検索中' });
  await expect(loading).toBeDisabled();
  await expect(loading).toHaveAttribute('data-loading', 'true');

  finish();
  const ready = page.getByRole('button', { name: '銘柄を検索' });
  await expect(ready).toBeEnabled();
  await expect(ready).not.toHaveAttribute('data-loading', 'true');
});

const ASSET_ROUTES = {
  assetBalances: /\/api\/v1\/asset-balances(?:\?.*)?$/,
  dividendBatch: /\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/,
};

function holdingRow(id: string, account: string | null, shares = 100) {
  return {
    id,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: '9905',
    security_name: '架空ゲーム',
    account,
    shares,
    average_purchase_price: 2500,
    total_purchase_amount: 2500 * shares,
    current_price: 2600,
    daily_change: 50,
    market_value: 2600 * shares,
    profit_loss_rate: 4.0,
  };
}

async function setupHoldingMocks(page: Page, rows: ReturnType<typeof holdingRow>[]) {
  await mockSession(page);
  await page.route(ROUTES.stock, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_STOCK),
    }),
  );
  await page.route(ASSET_ROUTES.assetBalances, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ data: rows, total: rows.length, page: 1, per_page: rows.length }),
    }),
  );
  await page.route(ASSET_ROUTES.dividendBatch, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ items: [] }),
    }),
  );
}

test('持っている銘柄は結果の上に保有情報と資産管理への戻りが出る', async ({ page }) => {
  await setupHoldingMocks(page, [holdingRow('holding-1', '特定')]);

  await page.goto('/search?code=9905');
  await expect(page.getByText('架空ゲーム')).toBeVisible();

  await expect(page.getByRole('heading', { name: '保有情報' })).toBeVisible();
  await expect(page.getByRole('link', { name: '資産管理に戻る' })).toBeVisible();
  const section = page.getByLabel('保有情報', { exact: true });
  await expect(section.getByText('評価額')).toBeVisible();
  await expect(section.getByText('評価損益')).toBeVisible();
  await expect(section.getByText('取得単価')).toBeVisible();
});

test('持っていない銘柄は保有情報が出ない', async ({ page }) => {
  await setupHoldingMocks(page, []);

  await page.goto('/search?code=9905');
  await expect(page.getByText('架空ゲーム')).toBeVisible();

  await expect(page.getByRole('heading', { name: '保有情報' })).toHaveCount(0);
  await expect(page.getByRole('link', { name: '資産管理に戻る' })).toHaveCount(0);
});

test('口座が複数の銘柄は合計と口座ごとの保有情報が出る', async ({ page }) => {
  await setupHoldingMocks(page, [
    holdingRow('holding-1', '特定'),
    holdingRow('holding-2', 'NISA成長', 50),
  ]);

  await page.goto('/search?code=9905');
  await expect(page.getByRole('heading', { name: '保有情報' })).toBeVisible();

  await expect(page.getByText('合計')).toBeVisible();
  const section = page.getByLabel('保有情報', { exact: true });
  await expect(section.getByText('150株')).toBeVisible();
  await expect(section.getByText('取得単価')).toHaveCount(2);
});

test('資産管理の行から銘柄画面を開ける', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await setupHoldingMocks(page, [holdingRow('holding-1', '特定')]);

  await page.goto('/assetbalance');
  await expect(page.getByTestId('asset-holdings-table')).toBeVisible();
  await page.getByTestId('asset-holdings-table').getByText('架空ゲーム').click();

  await expect(page).toHaveURL('/search?code=9905');
  await expect(page.getByRole('heading', { name: '保有情報' })).toBeVisible();
});

test('旧保有詳細URLは同じ銘柄の銘柄画面を開く', async ({ page }) => {
  await setupHoldingMocks(page, [holdingRow('holding-1', '特定')]);

  await page.goto('/assetbalance/holdings/holding-1');
  await expect(page).toHaveURL('/search?code=9905');
  await expect(page.getByRole('heading', { name: '保有情報' })).toBeVisible();
});

test('銘柄名で検索しても成功した銘柄コードがURLに載る', async ({ page }) => {
  await setupHoldingMocks(page, [holdingRow('holding-1', '特定')]);

  await page.goto('/search');
  await page.getByRole('searchbox', { name: '銘柄コードまたは銘柄名で検索' }).fill('架空ゲーム');
  await page.getByRole('button', { name: '銘柄を検索' }).click();

  await expect(page.getByText('架空ゲーム')).toBeVisible();
  await expect(page).toHaveURL('/search?code=9905');
  await expect(page.getByRole('heading', { name: '保有情報' })).toBeVisible();
});

test('保有一覧の取得失敗は再試行できる', async ({ page }) => {
  await mockSession(page);
  await page.route(ROUTES.stock, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_STOCK),
    }),
  );
  let listCalls = 0;
  await page.route(ASSET_ROUTES.assetBalances, (route) => {
    listCalls += 1;
    // 500 はクライアント側でリトライされ、銘柄画面では取得が2回走るため、
    // 初回表示の2回分(最大4リクエスト)を失敗させる
    if (listCalls <= 4) {
      return route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'server error' }),
      });
    }
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        data: [holdingRow('holding-1', '特定')],
        total: 1,
        page: 1,
        per_page: 1,
      }),
    });
  });
  await page.route(ASSET_ROUTES.dividendBatch, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ items: [] }),
    }),
  );

  await page.goto('/search?code=9905');
  await expect(page.getByText('架空ゲーム')).toBeVisible();
  await expect(page.getByRole('heading', { name: '保有情報' })).toHaveCount(0);

  await page.getByRole('button', { name: '再読み込み' }).click();
  await expect(page.getByRole('heading', { name: '保有情報' })).toBeVisible();
});

test('銘柄マスタに無い保有銘柄も保有情報が出る', async ({ page }) => {
  await mockSession(page);
  await page.route(ROUTES.stock, (route) =>
    route.fulfill({
      status: 404,
      contentType: 'application/json',
      body: JSON.stringify({ error: 'not found' }),
    }),
  );
  await page.route(ASSET_ROUTES.assetBalances, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        data: [holdingRow('holding-1', '特定')],
        total: 1,
        page: 1,
        per_page: 1,
      }),
    }),
  );
  await page.route(ASSET_ROUTES.dividendBatch, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ items: [] }),
    }),
  );

  await page.goto('/search?code=9905');
  await expect(page.getByRole('heading', { name: '保有情報' })).toBeVisible();
  await expect(page.getByText('該当する銘柄が見つかりませんでした')).toBeVisible();
});
