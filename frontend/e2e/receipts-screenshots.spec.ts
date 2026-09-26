import { expect, test, type Page, type TestInfo } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

/**
 * 取引明細(/receipts)のスクリーンショットを PC(1920x1080) / スマホ(390x844)
 * の両幅で撮り、リポジトリルートの .playwright-mcp/ に leptos- プレフィックスで
 * 保存する。API は page.route() でモックするためバックエンド・ログイン不要。
 * 実行: LEPTOS_E2E_PORT=8096 npx playwright test --config playwright.leptos.config.ts receipts-screenshots.spec.ts
 */

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const ROUTES = {
  session: /\/api\/v1\/session$/,
  dividends: /\/api\/v1\/dividends(?:\?.*)?$/,
  domesticStocks: /\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/,
  mutualfunds: /\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/,
  assetBalances: /\/api\/v1\/asset-balances(?:\?.*)?$/,
  dividendPerShare: /\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/,
  stock: /\/api\/v1\/stocks(?:\?.*)?$/,
};

const STOCKS = [
  ['7203', 'トヨタ自動車'],
  ['9432', '日本電信電話'],
  ['6758', 'ソニーグループ'],
  ['8306', '三菱UFJフィナンシャル・グループ'],
  ['9984', 'ソフトバンクグループ'],
  ['4502', '武田薬品工業'],
  ['6861', 'キーエンス'],
  ['8058', '三菱商事'],
];

// Decimal フィールドは JSON 数値としてデシリアライズされるため数値で返す。
const DIVIDENDS = Array.from({ length: 18 }, (_, i) => {
  const [code, name] = STOCKS[i % STOCKS.length];
  const month = String((i % 12) + 1).padStart(2, '0');
  const day = String((i % 27) + 1).padStart(2, '0');
  return {
    id: `dividend-${i}`,
    user_id: MOCK_USER.id,
    settlement_date: `2024-${month}-${day}`,
    product: i % 2 === 0 ? '特定口座' : 'NISA口座',
    account: i % 3 === 0 ? 'SBI証券' : '楽天証券',
    security_code: code,
    security_name: name,
    unit_price: 10 + i * 3,
    shares: 100 * ((i % 5) + 1),
    dividends_before_tax: 3000 + i * 1234,
    taxes: 609 + i * 250,
    net_amount_received: 2391 + i * 984,
    created_at: '2024-03-01T00:00:00Z',
    updated_at: '2024-03-01T00:00:00Z',
  };
});

const DOMESTIC_STOCKS = Array.from({ length: 15 }, (_, i) => {
  const [code, name] = STOCKS[i % STOCKS.length];
  const month = String((i % 12) + 1).padStart(2, '0');
  const day = String((i % 27) + 1).padStart(2, '0');
  return {
    id: `domestic-${i}`,
    trade_date: `2024-${month}-${day}`,
    settlement_date: `2024-${month}-${day}`,
    security_code: code,
    security_name: name,
    account: i % 2 === 0 ? '特定口座' : 'NISA口座',
    shares: 100 * ((i % 4) + 1),
    asked_price: 2800 + i * 150,
    proceeds: 280000 + i * 15000,
    purchase_price: 2500 + i * 120,
    realized_profit_and_loss: 30000 - i * 2500,
    taxes: 6090 - i * 400,
    realized_profit_and_loss_after_tax: 23910 - i * 2100,
    created_at: '2024-05-10T00:00:00Z',
    updated_at: '2024-05-10T00:00:00Z',
  };
});

const MUTUALFUNDS = Array.from({ length: 12 }, (_, i) => {
  const month = String((i % 12) + 1).padStart(2, '0');
  const day = String((i % 27) + 1).padStart(2, '0');
  return {
    id: `mutualfund-${i}`,
    trade_date: `2024-${month}-${day}`,
    settlement_date: `2024-${month}-${day}`,
    fund_name:
      i % 2 === 0
        ? 'eMAXIS Slim 全世界株式（オール・カントリー）'
        : 'SBI・V・S&P500インデックス・ファンド',
    account: i % 2 === 0 ? '特定口座' : 'NISA口座',
    shares: 10000 + i * 1500,
    exchange_rate: 1,
    cancellation_unit_price_yen: 21000 + i * 800,
    cancellation_amount_yen: 210000 + i * 12000,
    average_acquisition_price_yen: 18000 + i * 600,
    dividends: '0',
    realized_profit_and_loss: 30000 - i * 1800,
    taxes: 6090 - i * 350,
    realized_profit_and_loss_after_tax: 23910 - i * 1450,
    created_at: '2024-04-02T00:00:00Z',
    updated_at: '2024-04-02T00:00:00Z',
  };
});

const ASSET_BALANCES = Array.from({ length: 8 }, (_, i) => {
  const [code, name] = STOCKS[i % STOCKS.length];
  return {
    id: `asset-balance-${i}`,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: code,
    security_name: name,
    shares: 100 * (i + 1),
    executing_shares: 0,
    average_purchase_price: 2500 + i * 300,
    total_purchase_amount: 250000 + i * 120000,
    current_price: 2600 + i * 310,
    daily_change: 50 - i * 12,
    market_value: 260000 + i * 125000,
    profit_loss_rate: 4.0 - i * 0.7,
  };
});

function paginated(data: unknown[]) {
  return { data, total: data.length, page: 1, per_page: Math.max(data.length, 1) };
}

function json(body: unknown) {
  return { status: 200, contentType: 'application/json', body: JSON.stringify(body) };
}

async function mockApi(page: Page) {
  // Playwright は後から登録した route が優先されるため、catch-all を最初に登録する
  await page.route(/\/api\/v1\//, (route) => route.fulfill(json(paginated([]))));

  await page.route(ROUTES.session, (route) => route.fulfill(json(MOCK_USER)));
  await page.route(ROUTES.dividends, (route) => route.fulfill(json(paginated(DIVIDENDS))));
  await page.route(ROUTES.domesticStocks, (route) =>
    route.fulfill(json(paginated(DOMESTIC_STOCKS))),
  );
  await page.route(ROUTES.mutualfunds, (route) => route.fulfill(json(paginated(MUTUALFUNDS))));
  await page.route(ROUTES.assetBalances, (route) => route.fulfill(json(paginated(ASSET_BALANCES))));
  await page.route(ROUTES.dividendPerShare, (route) =>
    route.fulfill(
      json({
        items: STOCKS.map(([code]) => ({
          security_code: code,
          dividend_per_share: 30,
          status: 'ok',
          is_stale: false,
        })),
      }),
    ),
  );
  await page.route(ROUTES.stock, (route) =>
    route.fulfill(
      json({
        date: '2024-01-01',
        code: '7203',
        name: 'トヨタ自動車',
        market_category: 'プライム',
        industry_code_33: '3050',
        industry_category_33: '輸送用機器',
        industry_code_17: '6',
        industry_category_17: '自動車・輸送機',
        size_code: '1',
        size_category: 'TOPIX Core30',
      }),
    ),
  );
}

function shotDir(testInfo: TestInfo): string {
  const dir = path.resolve(testInfo.project.testDir, '../../.playwright-mcp');
  fs.mkdirSync(dir, { recursive: true });
  return dir;
}

const TABS = [
  { name: '配当金', slug: 'dividend', testid: 'dividend', count: DIVIDENDS.length, header: '入金日' },
  {
    name: '国内株式',
    slug: 'domestic-stock',
    testid: 'domesticstock',
    count: DOMESTIC_STOCKS.length,
    header: '売却単価',
  },
  {
    name: '投資信託',
    slug: 'mutualfund',
    testid: 'mutualfund',
    count: MUTUALFUNDS.length,
    header: 'ファンド名',
  },
] as const;

async function shootReceiptTabs(page: Page, testInfo: TestInfo, suffix: 'pc' | 'mobile') {
  const dir = shotDir(testInfo);
  const mobile = suffix === 'mobile';
  const cardList = page.getByTestId('receipt-card-list');
  const table = page.getByRole('table');

  await page.goto('/receipts');

  for (const [index, tab] of TABS.entries()) {
    if (index > 0) {
      await page.getByRole('tab', { name: tab.name }).click();
    }
    await expect(page.getByRole('tab', { name: tab.name })).toHaveAttribute(
      'aria-selected',
      'true',
    );
    // タブの件数バッジは各タブのフェッチ完了後に確定するため待機条件に使う
    await expect(page.getByTestId(`tab-count-${tab.testid}`)).toHaveText(String(tab.count));
    if (mobile) {
      await expect(cardList.getByTestId('receipt-card')).toHaveCount(tab.count);
    } else {
      await expect(table.getByText(tab.header, { exact: true })).toBeVisible();
      await expect(table.getByRole('row').first()).toBeVisible();
    }
    await page.waitForTimeout(700);
    await page.screenshot({
      path: path.join(dir, `leptos-receipts-${tab.slug}-${suffix}.png`),
      fullPage: true,
    });
  }
}

test('取引明細 PC幅スクリーンショット', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1920, height: 1080 });
  await mockApi(page);
  await shootReceiptTabs(page, testInfo, 'pc');
});

test('取引明細 スマホ幅スクリーンショット', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await mockApi(page);
  await shootReceiptTabs(page, testInfo, 'mobile');
});
