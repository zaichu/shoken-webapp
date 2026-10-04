import type { Page } from '@playwright/test';
import { expect, test, paginated } from '../support/test';
import * as path from 'path';

test.use({ viewport: { width: 390, height: 844 } });

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const DIVIDENDS = [
  {
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
  },
  {
    id: '00000000-0000-0000-0000-000000000002',
    settlement_date: '2024-03-15',
    product: '特定口座',
    account: '楽天証券',
    security_code: '9432',
    security_name: '日本電信電話',
    unit_price: 10.0,
    shares: 200,
    dividends_before_tax: 2000,
    taxes: 406,
    net_amount_received: 1594,
    created_at: '2024-03-15T00:00:00Z',
    updated_at: '2024-03-15T00:00:00Z',
  },
  {
    id: '00000000-0000-0000-0000-000000000003',
    settlement_date: '2024-02-10',
    product: 'NISA口座',
    account: 'SBI証券',
    security_code: '6758',
    security_name: 'ソニーグループ',
    unit_price: 20.0,
    shares: 50,
    dividends_before_tax: 1000,
    taxes: 203,
    net_amount_received: 797,
    created_at: '2024-02-10T00:00:00Z',
    updated_at: '2024-02-10T00:00:00Z',
  },
];

const DOMESTIC = {
  id: '00000000-0000-0000-0000-000000000004',
  trade_date: '2024-02-01',
  settlement_date: '2024-02-03',
  account: 'SBI証券',
  security_code: '7974',
  security_name: '任天堂',
  shares: 10,
  asked_price: 5000,
  proceeds: 55000,
  purchase_price: 5000,
  realized_profit_and_loss: 5000,
  taxes: 1015,
  realized_profit_and_loss_after_tax: 3985,
  created_at: '2024-02-01T00:00:00Z',
  updated_at: '2024-02-01T00:00:00Z',
};

const FUND = {
  id: '00000000-0000-0000-0000-000000000005',
  trade_date: '2024-02-01',
  settlement_date: '2024-02-03',
  fund_name: 'eMAXIS Slim 米国株式(S&P500)',
  account: 'NISA',
  shares: 1000,
  exchange_rate: 1,
  cancellation_unit_price_yen: 20000,
  cancellation_amount_yen: 20000,
  average_acquisition_price_yen: 18000,
  realized_profit_and_loss: 2000,
  taxes: 406,
  realized_profit_and_loss_after_tax: 1594,
  created_at: '2024-02-01T00:00:00Z',
  updated_at: '2024-02-01T00:00:00Z',
};


async function mockApi(page: Page) {
  await page.route(/\/api\/v1\//, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated([])),
    }),
  );
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_USER),
    }),
  );
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated(DIVIDENDS)),
    }),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated([DOMESTIC])),
    }),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated([FUND])),
    }),
  );
}

// 狭い帯ではパネルが畳んで始まるので、パネル内の UI に触れる前にドロワーを開ける
async function openPanelDrawer(page: Page) {
  const toggle = page.getByTestId('receipt-utility-toggle');
  await expect(toggle).toBeVisible();
  if ((await toggle.getAttribute('aria-expanded')) !== 'true') {
    await toggle.click();
  }
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
}

test('スマホ幅では明細がカード表示になりテーブルは隠れる', async ({ page }) => {  await mockApi(page);
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  await expect(cardList).toBeVisible();
  await expect(page.getByRole('table')).toBeHidden();
  await expect(cardList.getByTestId('receipt-card')).toHaveCount(3);
  await expect(cardList.getByTestId('receipt-card-group')).toHaveCount(2);
});

test('グループ見出しは主要集計を常時表示しタップで全集計を開閉できる', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  await expect(cardList.getByTestId('receipt-card')).toHaveCount(3);

  const toggle = cardList.getByRole('button', { name: /2024年3月 2件 税引後 ¥3,985/ });
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  const region = cardList.getByRole('region', {
    name: /2024年3月 2件 税引後 ¥3,985/,
  });
  await expect(region).toBeHidden();

  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await expect(region).toBeVisible();
  await expect(region.getByText('配当金', { exact: true })).toBeVisible();
  await expect(region.getByText('¥5,000')).toBeVisible();

  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(region).toBeHidden();
});

test('カードは開閉せず見出しと全項目を最初から表示する', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  const card = cardList.getByTestId('receipt-card').first();
  await expect(card).toBeVisible();

  // 開閉トリガーを持たない(コピー用のボタンのみ)
  await expect(card.locator('[aria-expanded]')).toHaveCount(0);
  await expect(card.getByRole('button')).toHaveCount(1);

  // 見出しは銘柄名・日付・口座(先頭は入金日順で日本電信電話)
  await expect(
    card.getByRole('button', { name: '日本電信電話(9432) をコピー' }),
  ).toBeVisible();
  await expect(card.getByText('03/15', { exact: true })).toBeVisible();
  await expect(card.getByText('楽天証券', { exact: true })).toBeVisible();

  // 見出しと重複しない残り全項目を、表の列順で2列の格子に出す
  const grid = card.locator('dl');
  const labels = ['商品', '銘柄コード', '単価', '数量', '配当金', '税額', '税引後'];
  await expect(grid.locator('dt')).toHaveCount(labels.length);
  for (const label of labels) {
    await expect(grid.locator('dt', { hasText: label })).toBeVisible();
  }
  for (const label of ['入金日', '口座', '銘柄名']) {
    await expect(grid.locator('dt', { hasText: label })).toHaveCount(0);
  }

  await expect(grid.getByText('¥1,594', { exact: true })).toBeVisible();
  const clipped = await grid
    .locator('dd, dt')
    .evaluateAll((cells) =>
      cells.filter((cell) => cell.scrollWidth > cell.clientWidth + 1),
    );
  expect(clipped, '格子の項目が見切れない').toEqual([]);

  const codeLink = card.getByRole('link', { name: '9432' });
  await expect(codeLink).toHaveAttribute('href', '/search?code=9432');
});

test('タブは1行のまま横スクロール可能', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');
  await expect(page.getByRole('tab')).toHaveCount(3);

  const tops = await page
    .getByRole('tab')
    .evaluateAll((tabs) => tabs.map((tab) => tab.getBoundingClientRect().top));
  expect(new Set(tops).size).toBe(1);
});

test('矢印キーとHome/Endでタブを移動しフォーカスも追従する', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const dividend = page.getByRole('tab', { name: /配当金/ });
  const domestic = page.getByRole('tab', { name: /国内株式/ });
  const fund = page.getByRole('tab', { name: /投資信託/ });
  await expect(dividend).toHaveAttribute('aria-selected', 'true');

  await dividend.press('ArrowRight');
  await expect(domestic).toHaveAttribute('aria-selected', 'true');
  await expect(domestic).toBeFocused();
  await expect(domestic).toHaveAttribute('tabindex', '0');
  await expect(dividend).toHaveAttribute('tabindex', '-1');

  await domestic.press('ArrowRight');
  await expect(fund).toHaveAttribute('aria-selected', 'true');
  await fund.press('ArrowRight');
  await expect(dividend).toHaveAttribute('aria-selected', 'true');
  await expect(dividend).toBeFocused();

  await dividend.press('End');
  await expect(fund).toHaveAttribute('aria-selected', 'true');
  await fund.press('Home');
  await expect(dividend).toHaveAttribute('aria-selected', 'true');
});

test('検索・集計・CSVは同じツールバーからキーボードで開閉できる', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');
  await openPanelDrawer(page);
  const toolbar = page.getByTestId('receipt-mobile-toolbar');
  await expect(toolbar).toBeVisible();
  const search = toolbar.getByTestId('receipt-search-toggle');
  const summary = toolbar.getByTestId('receipt-summary-compact-toggle');
  const csv = toolbar.getByTestId('receipt-csv-toggle');
  const buttons = [search, summary, csv];
  for (const button of buttons) {
    await expect(button).toHaveAttribute('aria-expanded', 'false');
    const id = await button.getAttribute('aria-controls');
    await expect(page.locator(`[id="${id}"]`)).toBeAttached();
  }
  const boxes = await Promise.all(buttons.map((button) => button.boundingBox()));
  expect(boxes.every((box) => box !== null)).toBe(true);
  expect(Math.max(...boxes.map((box) => box!.y))).toBeLessThan(
    Math.min(...boxes.map((box) => box!.y + box!.height)),
  );
  await expect(page.locator('#search-options-body')).toBeHidden();
  await expect(page.getByRole('region', { name: '集計情報', exact: true })).toBeHidden();
  await expect(page.getByRole('region', { name: 'CSV取り込み・削除' })).toBeHidden();
  for (const button of buttons) {
    for (let step = 0; step < 50 && !(await button.evaluate((el) => el === document.activeElement)); step++) {
      await page.keyboard.press('Tab');
    }
    await expect(button).toBeFocused();
    await page.keyboard.press('Enter');
    await expect(button).toHaveAttribute('aria-expanded', 'true');
    const id = await button.getAttribute('aria-controls');
    await expect(page.locator(`[id="${id}"]`)).toBeVisible();
    await page.keyboard.press('Space');
    await expect(button).toHaveAttribute('aria-expanded', 'false');
    await expect(page.locator(`[id="${id}"]`)).toBeHidden();
  }
});

test('集計はツールバーから全項目を開く', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');
  await openPanelDrawer(page);

  const toggle = page.getByTestId('receipt-summary-compact-toggle');
  await expect(toggle).toBeVisible();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(toggle).toHaveAttribute('aria-label', '集計情報');
  await expect(page.getByTestId('receipt-summary-desktop')).toBeHidden();

  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  const region = page.getByRole('region', { name: '集計情報' });
  await expect(region).toBeVisible();
  const mobileBody = page.locator('#receipt-summary-mobile-body');
  await expect(mobileBody.getByText('配当金', { exact: true })).toBeVisible();
  await expect(mobileBody.getByText('税引後', { exact: true })).toBeVisible();

  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(region).toBeHidden();
});

test('国内株式タブでも全項目が見え、銘柄リンクとコピーが使える', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  await page.getByRole('tab', { name: '国内株式' }).click();
  const card = page
    .getByTestId('receipt-card-list')
    .getByTestId('receipt-card')
    .first();
  await expect(card.locator('[aria-expanded]')).toHaveCount(0);

  // 見出し(約定日・銘柄名・口座)以外の全項目を列順で出す
  const grid = card.locator('dl');
  await expect(grid.locator('dt')).toHaveCount(8);
  for (const label of [
    '銘柄コード',
    '数量',
    '売却単価',
    '売却額',
    '取得価額',
    '実現損益',
    '税額',
    '税引後',
  ]) {
    await expect(grid.locator('dt', { hasText: label })).toBeVisible();
  }

  await expect(card.getByRole('link', { name: '7974' })).toHaveAttribute(
    'href',
    '/search?code=7974',
  );
  await expect(
    card.getByRole('button', { name: '任天堂(7974) をコピー' }),
  ).toBeVisible();
});

test('投資信託タブでも全項目が見え、ファンド名をコピーできる', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  await page.getByRole('tab', { name: '投資信託' }).click();
  const card = page
    .getByTestId('receipt-card-list')
    .getByTestId('receipt-card')
    .first();
  await expect(card.locator('[aria-expanded]')).toHaveCount(0);

  // 見出し(約定日・ファンド名・口座)以外の全項目を列順で出す
  const grid = card.locator('dl');
  await expect(grid.locator('dt')).toHaveCount(7);
  for (const label of [
    '数量',
    '解約単価',
    '解約額',
    '取得価額',
    '実現損益',
    '税額',
    '税引後',
  ]) {
    await expect(grid.locator('dt', { hasText: label })).toBeVisible();
  }

  await expect(
    card.getByRole('button', {
      name: 'eMAXIS Slim 米国株式(S&P500) をコピー',
    }),
  ).toBeVisible();
});

test('検索条件を変えても集計カードの開閉状態は保たれ、残ったカードは全項目のまま', async ({
  page,
}) => {
  await mockApi(page);
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  const groupToggle = cardList.getByRole('button', {
    name: /2024年3月 2件 税引後/,
  });
  await groupToggle.click();
  // パネル操作はドロワーを開けてから行う
  await openPanelDrawer(page);
  const summaryToggle = page.getByTestId('receipt-summary-compact-toggle');
  await summaryToggle.click();

  await page.getByTestId('receipt-search-toggle').click();
  await page.locator('#securities-search').selectOption('9432');
  await expect(cardList.getByTestId('receipt-card')).toHaveCount(1);

  // 行カードは開閉しないため絞り込み後も全項目が見えている
  const card = cardList.getByTestId('receipt-card').first();
  await expect(card.locator('dl')).toBeVisible();
  await expect(card.locator('dt')).toHaveCount(7);
  await expect(
    cardList.getByRole('button', { name: /1件 税引後 ¥1,594/ }),
  ).toHaveAttribute('aria-expanded', 'false');
  await expect(summaryToggle).toHaveAttribute('aria-expanded', 'true');
});

test('CSV操作レールはスマホ幅で折り畳み開閉できる', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');
  await openPanelDrawer(page);

  const toggle = page.getByTestId('receipt-csv-toggle');
  const region = page.getByRole('region', { name: 'CSV取り込み・削除' });
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(region).toBeHidden();

  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await expect(region).toBeVisible();
  await expect(page.getByTestId('csv-file-input')).toBeAttached();

  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(region).toBeHidden();
});

test('640px境界でカードとテーブルが切り替わる', async ({ page }) => {
  await mockApi(page);
  await page.setViewportSize({ width: 639, height: 844 });
  await page.goto('/receipts');
  await expect(page.getByTestId('receipt-card-list')).toBeVisible();
  await expect(page.getByRole('table')).toBeHidden();

  await page.setViewportSize({ width: 640, height: 844 });
  await expect(page.getByRole('table')).toBeVisible();
  await expect(page.getByTestId('receipt-card-list')).toBeHidden();
});

test('全件削除は確認モーダル経由で実行される', async ({ page }) => {
  let rows: unknown[] = [...DIVIDENDS];
  let deleteCount = 0;
  await page.route(/\/api\/v1\//, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated([])),
    }),
  );
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_USER),
    }),
  );
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) => {
    if (route.request().method() === 'DELETE') {
      deleteCount += 1;
      rows = [];
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: '{}',
      });
    }
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated(rows)),
    });
  });
  await page.goto('/receipts');

  await openPanelDrawer(page);
  await page.getByTestId('receipt-csv-toggle').click();
  await page.getByRole('button', { name: /全件削除/ }).click();
  const dialog = page.getByRole('dialog');
  await expect(dialog).toContainText('この操作は取り消せません');
  await expect(dialog).toContainText('3件');
  expect(deleteCount).toBe(0);

  await dialog.getByRole('button', { name: '削除する' }).click();
  await expect(page.getByText('データがありません')).toBeVisible();
  expect(deleteCount).toBe(1);
});

test('CSV取込の保存結果は一覧再取得後もレールが開いて見える', async ({ page }) => {
  const imported = { ...DIVIDENDS[0], id: '00000000-0000-0000-0000-000000000009' };
  const rows: unknown[] = [...DIVIDENDS];
  await page.route(/\/api\/v1\//, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated([])),
    }),
  );
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_USER),
    }),
  );
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated(rows)),
    }),
  );
  await page.route(/\/api\/v1\/dividend-import-validations$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        total_rows: 1,
        valid_rows: 1,
        errors: [],
        rows: [imported],
      }),
    }),
  );
  await page.route(/\/api\/v1\/dividend-imports$/, (route) => {
    rows.push(imported);
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ inserted: 1, skipped: 0, errors: [] }),
    });
  });
  await page.goto('/receipts');

  await openPanelDrawer(page);
  const toggle = page.getByTestId('receipt-csv-toggle');
  await toggle.click();
  const fileInput = page.getByTestId('csv-file-input');
  // バックグラウンドの他タブ取得中は input が disabled で、change イベントが捨てられる。
  // setInputFiles は enabled を待たないので、先に有効化を待つ
  await expect(fileInput).toBeEnabled();
  await fileInput.setInputFiles(
      path.resolve(
        test.info().project.testDir,
        'fixtures/csv/dividend-base.csv',
      ),
    );
  await page.getByRole('button', { name: '1件 追加で保存' }).click();

  const notice = page.getByTestId('csv-save-result-notice');
  await expect(notice).toBeVisible();
  await expect(notice).toContainText('1件反映');
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
});
