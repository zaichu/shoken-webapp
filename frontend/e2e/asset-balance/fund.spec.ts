import path from 'node:path';
import type { Page } from '@playwright/test';
import { expect, test, json, mockSession } from '../support/test';

const CSV_FIXTURE_PATH = path.resolve(
  process.cwd(),
  'e2e/fixtures/csv/assetbalance-invst-sample.csv',
);

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
    price_as_of: '2026-01-05',
    price_source: 'api',
    nav_is_latest: true,
    expected_nav_date: null,
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
    price_source: 'csv',
    nav_is_latest: false,
    expected_nav_date: '2026-01-05',
  },
];

const ROUTES = {
  assetBalances: /\/api\/v1\/asset-balances(?:\?.*)?$/,
  fundBalances: /\/api\/v1\/fund-balances(?:\?.*)?$/,
  fundPreview: /\/api\/v1\/fund-balance-import-validations$/,
  fundUpload: /\/api\/v1\/fund-balance-imports$/,
  dividendPerShareBatch: /\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/,
};

async function setupMocks(page: Page) {
  let funds = FUND_ROWS.map((row) => ({ ...row }));
  await mockSession(page);
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
  await page.getByTestId('asset-search-clear').click();

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

test('前日比は最新の基準価額の変動だけを出し、タイトルには件数だけを出す', async ({
  page,
}) => {
  await setupMocks(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/assetbalance');
  await page.getByTestId('asset-kind-tab-fund').click();
  const table = page.getByTestId('asset-fund-table');
  await expect(table).toBeVisible();

  // 共有値が最新の行は基準価額の前日比をそのまま出す(50円/1万口あたり)。
  // 基準日は前日比の欄に出さず title にだけ入れる(2行の基準日は違う日にしてある)
  const latest = table.getByRole('row', { name: /架空バランスファンドA号/ });
  await expect(latest).toContainText('+¥50');
  await expect(latest).not.toContainText('更新待ち');
  await expect(latest).not.toContainText(/\(\d{1,2}\/\d{1,2}\)/);

  // 更新待ちの行は古い日の前日比(-30円)を出さず「更新待ち」とだけ示す
  const pending = table.getByRole('row', { name: /架空インデックスファンドB号/ });
  await expect(pending).toContainText('更新待ち');
  await expect(pending).not.toContainText('-¥30');

  // タイトルには件数だけを出す(前日比の集計は出さない)
  const subtitle = page.getByTestId('page-title-count');
  await expect(subtitle).toContainText('2件');
  await expect(subtitle).not.toContainText('前日比');
  await expect(page.getByText(/前日比が最新/)).toHaveCount(0);
  await expect(page.getByText(/更新待ち \d件/)).toHaveCount(0);

  // 「前日比」列の見出しに説明(title と abbr でキーボードからも読める)
  const header = table.getByRole('columnheader', { name: '前日比' });
  await expect(header).toHaveAttribute('title', /最新の基準価額の日の前日比/);
  await expect(header).toHaveAttribute('abbr', /最新の基準価額の日の前日比/);

  // 合計の前日比は最新の行だけの合計(更新待ちの -60円は入らない)
  const note = page.getByText(/前日比は \d+件\/\d+件\(最新の前日比が分かるファンドだけ\)/);
  await expect(note).toBeVisible();

  // 注記は帯の直下にあり、表の末尾までスクロールしなくても読める
  const band = await page.getByTestId('asset-portfolio-summary').boundingBox();
  const noteBox = await note.boundingBox();
  const tableBox = await table.boundingBox();
  expect(band).not.toBeNull();
  expect(noteBox).not.toBeNull();
  expect(tableBox).not.toBeNull();
  if (!band || !noteBox || !tableBox) {
    return;
  }
  expect(noteBox.y).toBeGreaterThan(band.y + band.height - 1);
  expect(noteBox.y + noteBox.height).toBeLessThanOrEqual(tableBox.y + 1);

  // 「すべて」タブでも更新待ちの行は前日比を出さない
  await page.getByTestId('asset-kind-tab-all').click();
  const merged = page.getByTestId('asset-merged-table');
  await expect(merged).toBeVisible();
  const mergedPending = merged.getByRole('row', {
    name: /架空インデックスファンドB号/,
  });
  await expect(mergedPending).toContainText('更新待ち');
  await expect(mergedPending).not.toContainText('-¥30');

  // 「すべて」タブでもタイトルに前日比の集計を出さない
  await expect(page.getByTestId('page-title-count')).not.toContainText('前日比が最新');
  await expect(page.getByTestId('page-title-count')).not.toContainText('更新待ち');
  await expect(page.getByText(/前日比が最新/)).toHaveCount(0);
  await expect(page.getByText(/更新待ち \d件/)).toHaveCount(0);

  // 「すべて」タブも、合計から投信の行を除外する注記を PC で帯の近くに見せる
  await expect(
    page.getByText(/投資信託の前日比は \d+件\/\d+件/),
  ).toBeVisible();

  // 「すべて」タブでも投信の行は前日比の横に基準日を出さない
  const mergedLatest = merged.getByRole('row', { name: /架空バランスファンドA号/ });
  await expect(mergedLatest).toContainText('+¥50');
  await expect(mergedLatest).not.toContainText(/\(\d{1,2}\/\d{1,2}\)/);
});

test('投資信託タブのCSVはプレビューから全件置換で保存できる', async ({
  page,
}) => {
  await setupMocks(page);
  await page.goto('/csv?kind=fundbalance');

  const input = page.getByLabel('CSVファイルを選択');
  await expect(input).toBeEnabled();
  await input.setInputFiles(CSV_FIXTURE_PATH);

  // 投資信託タブの入力に投したので、ファンドのプレビューが出る
  const preview = page.getByTestId('csv-preview');
  await expect(preview).toContainText('架空バランスファンドA号');

  await page.getByRole('button', { name: '2件 全件置換で保存' }).click();
  // 成功したらダイアログが閉じる
  await expect(page.getByTestId('csv-import-dialog')).toBeHidden();
});
