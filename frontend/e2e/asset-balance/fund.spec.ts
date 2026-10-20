import path from 'node:path';
import type { Page } from '@playwright/test';
import { expect, test, json } from '../support/test';

const CSV_FIXTURE_PATH = path.resolve(
  process.cwd(),
  'e2e/fixtures/csv/assetbalance-invst-sample.csv',
);

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000010',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const FUND_ROWS = [
  {
    id: 'fund-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    fund_name: '架空バランスファンドA号',
    account: '特定',
    dividends_course: '受取型',
    shares: 10000,
    average_purchase_price: 10000,
    total_purchase_amount: 100000,
    base_price: 11000,
    base_price_day_change: 50,
    base_price_month_change: null,
    market_value: 110000,
    profit_loss: 10000,
    profit_loss_rate: 10.0,
    total_return: 10000,
    price_as_of: '2026-01-01',
  },
  {
    id: 'fund-2',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    fund_name: '架空インデックスファンドB号',
    account: 'NISA',
    dividends_course: '再投資型',
    shares: 20000,
    average_purchase_price: 9500,
    total_purchase_amount: 190000,
    base_price: 9800,
    base_price_day_change: -30,
    base_price_month_change: 120,
    market_value: 196000,
    profit_loss: 6000,
    profit_loss_rate: 3.16,
    total_return: 6000,
    price_as_of: '2026-01-01',
  },
];

const ROUTES = {
  authMe: /\/api\/v1\/session$/,
  assetBalances: /\/api\/v1\/asset-balances(?:\?.*)?$/,
  fundBalances: /\/api\/v1\/fund-balances(?:\?.*)?$/,
  fundPreview: /\/api\/v1\/fund-balance-import-validations$/,
  fundUpload: /\/api\/v1\/fund-balance-imports$/,
  dividendPerShareBatch: /\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/,
};

async function setupMocks(page: Page) {
  let funds = FUND_ROWS.map((row) => ({ ...row }));
  await page.route(ROUTES.authMe, (route) => route.fulfill(json(MOCK_USER)));
  await page.route(ROUTES.dividendPerShareBatch, (route) =>
    route.fulfill(json({ items: [] })),
  );
  await page.route(ROUTES.assetBalances, (route) =>
    route.fulfill(json({ data: [], total: 0, page: 1, per_page: 0 })),
  );
  await page.route(ROUTES.fundPreview, (route) =>
    route.fulfill(
      json({
        total_rows: funds.length,
        valid_rows: funds.length,
        errors: [],
        rows: funds,
      }),
    ),
  );
  await page.route(ROUTES.fundUpload, (route) =>
    route.fulfill(json({ inserted: funds.length, skipped: 0, errors: [] })),
  );
  await page.route(ROUTES.fundBalances, (route) => {
    if (route.request().method() === 'DELETE') {
      funds = [];
      return route.fulfill({ status: 204 });
    }
    const data = funds.map((row) => ({ ...row }));
    return route.fulfill(
      json({ data, total: data.length, page: 1, per_page: data.length }),
    );
  });
}

test('PC: 「条件を足す」の小窓はどのボタンでもその行の直下に出る', async ({
  page,
}) => {
  await setupMocks(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/assetbalance');
  await page.getByTestId('asset-kind-tab-fund').click();
  await expect(page.getByTestId('asset-fund-table')).toBeVisible();

  for (const testid of ['asset-condition-account', 'asset-condition-course']) {
    const trigger = page.getByTestId(testid);
    await trigger.click();
    const dialog = page.getByRole('dialog');
    await expect(dialog).toBeVisible();

    const button = await trigger.boundingBox();
    const panel = await dialog.boundingBox();
    expect(button).not.toBeNull();
    expect(panel).not.toBeNull();
    if (!button || !panel) {
      return;
    }
    // ボタンのすぐ下に出る(Issue #1416: 画面左下に離れて出ていた)
    expect(panel.y).toBeGreaterThanOrEqual(button.y + button.height - 1);
    expect(panel.y - (button.y + button.height)).toBeLessThan(40);
    // 横方向はボタンと重なる(直下に出ていること)
    expect(panel.x).toBeLessThan(button.x + button.width);
    expect(panel.x + panel.width).toBeGreaterThan(button.x);

    await page.keyboard.press('Escape');
    await expect(dialog).toBeHidden();
  }
});

test('スマホ: 「条件を足す」の小窓は下から出るシートのまま', async ({ page }) => {
  await setupMocks(page);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/assetbalance');
  await page.getByTestId('asset-kind-tab-fund').click();
  await expect(page.getByTestId('asset-fund-list')).toBeVisible();

  await page.getByTestId('asset-condition-account').click();
  const sheet = page.getByRole('dialog', { name: '口座を選ぶ' });
  await expect(sheet).toBeVisible();
  const panel = await sheet.boundingBox();
  expect(panel).not.toBeNull();
  if (!panel) {
    return;
  }
  // 画面下端に接するシートのまま(PC の小窓位置にはならない)
  expect(panel.y + panel.height).toBeGreaterThanOrEqual(844 - 2);
});

test('投資信託タブでファンド一覧が表示され、検索と並べ替えが効く', async ({
  page,
}) => {
  await setupMocks(page);
  await page.goto('/assetbalance');

  await page.getByTestId('asset-kind-tab-fund').click();
  const table = page.getByTestId('asset-fund-table');
  await expect(table).toBeVisible();
  await expect(table).toContainText('架空バランスファンドA号');
  await expect(table).toContainText('架空インデックスファンドB号');

  // ファンド名の部分一致で絞り込む
  await page.getByTestId('asset-search-input-pc').fill('B号');
  await expect(table).not.toContainText('架空バランスファンドA号');
  await expect(table).toContainText('架空インデックスファンドB号');
  await page.getByTestId('asset-search-clear-pc').click();

  // 口座チップで絞り込む
  await page.getByTestId('asset-condition-account').click();
  await page.getByRole('button', { name: 'NISA' }).click();
  await page.getByRole('button', { name: '閉じる' }).click();
  await expect(table).not.toContainText('架空バランスファンドA号');
  await expect(table).toContainText('架空インデックスファンドB号');
  await page.getByRole('button', { name: '口座 NISA を解除' }).click();
  await expect(table).toContainText('架空バランスファンドA号');

  // 列見出しクリックで評価額順が切り替わる
  await page
    .getByRole('button', { name: /^評価額/ })
    .click();
  await expect(table).toContainText('架空バランスファンドA号');

  // ファンド詳細ページに遷移する
  await table.getByRole('link', { name: '架空バランスファンドA号' }).click();
  await expect(page).toHaveURL('/assetbalance/funds/fund-1');
  await expect(
    page.getByRole('heading', { name: '架空バランスファンドA号' }),
  ).toBeVisible();
  await expect(page.getByText('トータルリターン')).toBeVisible();
});

test('投資信託CSVは見出しで判別されプレビューから全件置換で保存できる', async ({
  page,
}) => {
  await setupMocks(page);
  await page.goto('/assetbalance/csv');

  const input = page.getByLabel('CSVファイルを選択');
  await expect(input).toBeEnabled();
  await input.setInputFiles(CSV_FIXTURE_PATH);

  // 投資信託と判別されてプレビューが出る
  const preview = page.getByTestId('csv-preview');
  await expect(preview).toContainText('架空バランスファンドA号');

  await page.getByRole('button', { name: '2件 全件置換で保存' }).click();
  await expect(page.getByRole('link', { name: '一覧に戻る' })).toBeVisible();
});
