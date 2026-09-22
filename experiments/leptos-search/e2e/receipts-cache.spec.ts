import { expect, test, type Page, type Route } from '@playwright/test';

const USER_A = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const DIVIDEND_A = {
  id: '00000000-0000-0000-0000-000000000001',
  user_id: USER_A.id,
  settlement_date: '2024-03-01',
  product: '特定口座',
  account: 'SBI証券',
  security_code: '7203',
  security_name: 'トヨタ自動車',
  unit_price: '30.0',
  shares: '100',
  dividends_before_tax: '3000',
  taxes: '609',
  net_amount_received: '2391',
};

const DIVIDEND_B = {
  ...DIVIDEND_A,
  id: '00000000-0000-0000-0000-000000000003',
  user_id: '00000000-0000-0000-0000-000000000009',
  security_code: '6758',
  security_name: 'ソニーグループ',
};

const DOMESTIC = {
  id: '00000000-0000-0000-0000-000000000004',
  user_id: USER_A.id,
  trade_date: '2024-02-01',
  account: 'SBI証券',
  security_code: '7974',
  security_name: '任天堂',
  shares: '10',
  realized_profit_and_loss: '5000',
  taxes: '1015',
};

const FUND = {
  id: '00000000-0000-0000-0000-000000000005',
  user_id: USER_A.id,
  trade_date: '2024-01-15',
  account: '楽天証券',
  fund_name: 'eMAXIS Slim 全世界株式',
  shares: '10000',
  realized_profit_and_loss: '12000',
  taxes: '2437',
};

function paginatedResponse(data: unknown[]) {
  return { data, total: data.length, page: 1, per_page: data.length };
}

interface RequestLog {
  start: number;
  count: number;
}

async function fulfillDelayed(route: Route, body: unknown, delayMs: number, log: RequestLog) {
  log.count += 1;
  if (log.count === 1) {
    log.start = Date.now();
  }
  await new Promise((resolve) => setTimeout(resolve, delayMs));
  await route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify(body),
  });
}

async function setupMocks(
  page: Page,
  logs: Record<'dividends' | 'domestic' | 'funds', RequestLog>,
  delays: { dividends: (callCount: number) => number; domestic: number; funds: number },
  dividendResponder: (callCount: number) => unknown[],
) {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(USER_A),
    }),
  );
  let dividendCalls = 0;
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) => {
    dividendCalls += 1;
    return fulfillDelayed(
      route,
      paginatedResponse(dividendResponder(dividendCalls)),
      delays.dividends(dividendCalls),
      logs.dividends,
    );
  });
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    fulfillDelayed(route, paginatedResponse([DOMESTIC]), delays.domestic, logs.domestic),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    fulfillDelayed(route, paginatedResponse([FUND]), delays.funds, logs.funds),
  );
}

function freshLogs() {
  return {
    dividends: { start: 0, count: 0 },
    domestic: { start: 0, count: 0 },
    funds: { start: 0, count: 0 },
  };
}

test('選択中タブを優先取得し、完了後に非表示タブをバックグラウンド取得する', async ({ page }) => {
  const logs = freshLogs();
  await setupMocks(page, logs, { dividends: () => 400, domestic: 50, funds: 50 }, () => [DIVIDEND_A]);

  await page.goto('/receipts');

  await expect(page.getByRole('cell', { name: 'トヨタ自動車' })).toBeVisible();
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');
  await expect(page.getByTestId('tab-count-domesticstock')).toHaveText('1');
  await expect(page.getByTestId('tab-count-mutualfund')).toHaveText('1');

  expect(logs.dividends.count).toBe(1);
  expect(logs.domestic.count).toBe(1);
  expect(logs.funds.count).toBe(1);
  expect(logs.domestic.start).toBeGreaterThanOrEqual(logs.dividends.start + 300);
  expect(logs.domestic.start).toBeGreaterThanOrEqual(logs.dividends.start + 300);
  expect(logs.funds.start).toBeGreaterThanOrEqual(logs.dividends.start + 300);
});

test('タブ切替で再フェッチが走らずキャッシュが使われる', async ({ page }) => {
  const logs = freshLogs();
  await setupMocks(page, logs, { dividends: () => 50, domestic: 50, funds: 50 }, () => [DIVIDEND_A]);

  await page.goto('/receipts');
  await expect(page.getByTestId('tab-count-mutualfund')).toHaveText('1');

  await page.getByRole('tab', { name: /国内株式/ }).click();
  await expect(page.getByRole('cell', { name: '任天堂' })).toBeVisible();

  await page.getByRole('tab', { name: /投資信託/ }).click();
  await expect(page.getByRole('cell', { name: 'eMAXIS Slim 全世界株式' })).toBeVisible();

  await page.getByRole('tab', { name: /配当金/ }).click();
  await expect(page.getByRole('cell', { name: 'トヨタ自動車' })).toBeVisible();

  expect(logs.dividends.count).toBe(1);
  expect(logs.domestic.count).toBe(1);
  expect(logs.funds.count).toBe(1);
});

test('ログアウトでキャッシュが消え、別ユーザーでは再取得する', async ({ page }) => {
  const logs = freshLogs();
  await setupMocks(page, logs, { dividends: () => 50, domestic: 50, funds: 50 }, (callCount) =>
    callCount === 1 ? [DIVIDEND_A] : [DIVIDEND_B],
  );

  await page.goto('/receipts');
  await expect(page.getByRole('cell', { name: 'トヨタ自動車' })).toBeVisible();

  await page.getByTestId('poc-logout').click();
  await expect(page.getByTestId('poc-user-id')).toHaveText('未ログイン');

  await page.getByTestId('poc-login-b').click();
  await expect(page.getByRole('cell', { name: 'ソニーグループ' })).toBeVisible();
  await expect(page.getByRole('cell', { name: 'トヨタ自動車' })).toBeHidden();

  expect(logs.dividends.count).toBe(2);
});

test('ログアウト→再ログイン中の古いレスポンスは新ユーザーの表示を上書きしない', async ({ page }) => {
  const logs = freshLogs();
  await setupMocks(
    page,
    logs,
    { dividends: (callCount) => (callCount === 1 ? 800 : 50), domestic: 800, funds: 800 },
    (callCount) => (callCount === 1 ? [DIVIDEND_A] : [DIVIDEND_B]),
  );

  await page.goto('/receipts');
  await page.waitForRequest(/\/api\/v1\/dividends/);

  await page.getByTestId('poc-logout').click();
  await page.getByTestId('poc-login-b').click();

  await expect(page.getByRole('cell', { name: 'ソニーグループ' })).toBeVisible();

  await page.waitForTimeout(1200);
  await expect(page.getByRole('cell', { name: 'ソニーグループ' })).toBeVisible();
  await expect(page.getByRole('cell', { name: 'トヨタ自動車' })).toBeHidden();

  expect(logs.dividends.count).toBe(2);
});
