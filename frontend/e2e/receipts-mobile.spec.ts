import { expect, test, type Page } from '@playwright/test';
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

function paginatedResponse(data: unknown[]) {
  return { data, total: data.length, page: 1, per_page: data.length };
}

async function mockApi(page: Page) {
  await page.route(/\/api\/v1\//, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginatedResponse([])),
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
      body: JSON.stringify(paginatedResponse(DIVIDENDS)),
    }),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginatedResponse([DOMESTIC])),
    }),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginatedResponse([FUND])),
    }),
  );
}

test('スマホ幅では明細がカード表示になりテーブルは隠れる', async ({ page }) => {
  await mockApi(page);
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

  const toggle = cardList.getByRole('button', { name: /2024年3月 2件 税引後 ¥ 3,985/ });
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  const region = cardList.getByRole('region', {
    name: /2024年3月 2件 税引後 ¥ 3,985/,
  });
  await expect(region).toBeHidden();

  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await expect(region).toBeVisible();
  await expect(region.getByText('配当金', { exact: true })).toBeVisible();
  await expect(region.getByText('¥ 5,000')).toBeVisible();

  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(region).toBeHidden();
});

test('カードはタップで全列の明細を開閉できる', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  const cardButton = cardList.getByRole('button', { name: 'トヨタ自動車 ¥ 2,391' });
  await expect(cardButton).toHaveAttribute('aria-expanded', 'false');
  await expect(cardList.getByText('入金日')).toHaveCount(0);

  await cardButton.click();
  const region = cardList.getByRole('region', { name: 'トヨタ自動車 ¥ 2,391' });
  await expect(cardButton).toHaveAttribute('aria-expanded', 'true');
  await expect(region).toBeVisible();
  for (const label of ['入金日', '商品', '口座', '銘柄コード', '銘柄名', '単価', '数量', '配当金', '税額', '受取額']) {
    await expect(region.getByText(label, { exact: true })).toBeVisible();
  }
  await expect(region.getByText('2024/03/01', { exact: true })).toBeVisible();
  const codeLink = region.getByRole('link', { name: '7203' });
  await expect(codeLink).toHaveAttribute('href', '/search?code=7203');
  await expect(
    region.getByRole('button', { name: 'トヨタ自動車(7203) をコピー' }),
  ).toBeVisible();

  await cardButton.click();
  await expect(cardButton).toHaveAttribute('aria-expanded', 'false');
  await expect(region).toBeHidden();
});

test('タブは1行のまま横スクロール可能', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');
  await expect(page.getByRole('tab')).toHaveCount(3);

  const tablist = page.getByRole('tablist');
  const { overflowX, flexWrap } = await tablist.evaluate((element) => {
    const style = getComputedStyle(element);
    return { overflowX: style.overflowX, flexWrap: style.flexWrap };
  });
  expect(overflowX).toBe('auto');
  expect(flexWrap).toBe('nowrap');

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

test('集計は主要指標のみ常時表示しタップで全項目を開く', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const toggle = page.getByTestId('receipt-summary-compact-toggle');
  await expect(toggle).toBeVisible();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(toggle).toHaveAttribute('aria-label', /配当金\(税引\) ¥ 4,782/);
  await expect(page.getByTestId('receipt-summary-desktop')).toBeHidden();

  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  const region = page.getByRole('region', { name: '集計情報' });
  await expect(region).toBeVisible();
  await expect(region.getByText('配当金', { exact: true })).toBeVisible();
  await expect(region.getByText('配当金(税引)', { exact: true })).toBeVisible();

  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(region).toBeHidden();
});

test('国内株式タブでもカードが開き銘柄リンクとコピーが使える', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  await page.getByRole('tab', { name: '国内株式' }).click();
  const cardList = page.getByTestId('receipt-card-list');
  const cardButton = cardList.getByRole('button', { name: '任天堂 ¥ 3,985' });
  await cardButton.click();
  const region = cardList.getByRole('region', { name: '任天堂 ¥ 3,985' });
  await expect(region).toBeVisible();
  await expect(region.getByText('銘柄コード', { exact: true })).toBeVisible();
  await expect(region.getByRole('link', { name: '7974' })).toHaveAttribute(
    'href',
    '/search?code=7974',
  );
  await expect(
    region.getByRole('button', { name: '任天堂(7974) をコピー' }),
  ).toBeVisible();
  await expect(region.getByText('税引後', { exact: true })).toBeVisible();
});

test('投資信託タブでもカードが開きファンド名をコピーできる', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  await page.getByRole('tab', { name: '投資信託' }).click();
  const cardList = page.getByTestId('receipt-card-list');
  const cardButton = cardList.getByRole('button', {
    name: /eMAXIS Slim 米国株式\(S&P500\)/,
  });
  await cardButton.click();
  const region = cardList.getByRole('region', {
    name: /eMAXIS Slim 米国株式\(S&P500\)/,
  });
  await expect(region).toBeVisible();
  await expect(region.getByText('ファンド名', { exact: true })).toBeVisible();
  await expect(region.getByText('銘柄コード', { exact: true })).toHaveCount(0);
  await expect(
    region.getByRole('button', {
      name: 'eMAXIS Slim 米国株式(S&P500) をコピー',
    }),
  ).toBeVisible();
  await expect(region.getByText('税引損益', { exact: true })).toBeVisible();
});

test('検索条件を変えても同じカードと集計は閉じない', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  const firstCard = cardList.getByTestId('receipt-card').first();
  await expect(firstCard.getByRole('button')).toHaveAttribute(
    'aria-label',
    '日本電信電話 ¥ 1,594',
  );
  const cardButton = cardList.getByRole('button', {
    name: '日本電信電話 ¥ 1,594',
  });
  await cardButton.click();
  await expect(
    cardList.getByRole('region', { name: '日本電信電話 ¥ 1,594' }),
  ).toBeVisible();

  const groupToggle = cardList.getByRole('button', {
    name: /2024年3月 2件 税引後/,
  });
  await groupToggle.click();
  const summaryToggle = page.getByTestId('receipt-summary-compact-toggle');
  await summaryToggle.click();

  await page.getByTestId('search-card-header').click();
  await page.locator('#securities-search').selectOption('9432');
  await expect(cardList.getByTestId('receipt-card')).toHaveCount(1);

  await expect(cardButton).toHaveAttribute('aria-expanded', 'true');
  await expect(
    cardList.getByRole('region', { name: '日本電信電話 ¥ 1,594' }),
  ).toBeVisible();
  await expect(
    cardList.getByRole('button', { name: /1件 税引後 ¥ 1,594/ }),
  ).toHaveAttribute('aria-expanded', 'false');
  await expect(summaryToggle).toHaveAttribute('aria-expanded', 'true');
});

test('開いたカードは絞り込みで位置が変わっても開いたまま追従する', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  const sony = cardList.getByRole('button', { name: 'ソニーグループ ¥ 797' });
  await sony.click();
  await expect(sony).toHaveAttribute('aria-expanded', 'true');

  const searchCard = page.getByTestId('search-card');
  await searchCard.getByTestId('search-card-header').click();
  await searchCard.getByRole('button', { name: 'NISA口座' }).click();
  await expect(cardList.getByTestId('receipt-card')).toHaveCount(1);
  await expect(sony).toHaveAttribute('aria-expanded', 'true');
  await expect(
    cardList.getByRole('region', { name: 'ソニーグループ ¥ 797' }),
  ).toBeVisible();
});

test('CSV操作レールはスマホ幅で折り畳み開閉できる', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

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
      body: JSON.stringify(paginatedResponse([])),
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
      body: JSON.stringify(paginatedResponse(rows)),
    });
  });
  await page.goto('/receipts');

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
      body: JSON.stringify(paginatedResponse([])),
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
      body: JSON.stringify(paginatedResponse(rows)),
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

  const toggle = page.getByTestId('receipt-csv-toggle');
  await toggle.click();
  const fileInput = page.getByTestId('csv-file-input');
  // バックグラウンドの他タブ取得中は input が disabled で、change イベントが捨てられる。
  // setInputFiles は enabled を待たないので、先に有効化を待つ
  await expect(fileInput).toBeEnabled();
  await fileInput.setInputFiles(
      path.resolve(
        test.info().project.testDir,
        '__fixtures__/csv/dividend-base.csv',
      ),
    );
  await page.getByRole('button', { name: '1件 追加で保存' }).click();

  const notice = page.getByTestId('csv-save-result-notice');
  await expect(notice).toBeVisible();
  await expect(notice).toContainText('1件反映');
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
});
