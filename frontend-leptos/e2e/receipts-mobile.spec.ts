import { expect, test, type Page } from '@playwright/test';

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

  const toggle = cardList.getByRole('button', { name: /2024年3月 2件 受取額 ¥ 3,985/ });
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  const region = cardList.getByRole('region', {
    name: /2024年3月 2件 受取額 ¥ 3,985/,
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
  await expect(toggle).toHaveAttribute('aria-label', /受取金額 ¥ 4,782/);
  await expect(page.getByTestId('receipt-summary-desktop')).toBeHidden();

  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  const region = page.getByRole('region').filter({ has: page.getByTestId('kpi-grid') });
  await expect(region).toBeVisible();
  await expect(region.getByText('配当金', { exact: true })).toBeVisible();
  await expect(region.getByText('受取金額', { exact: true })).toBeVisible();
});
