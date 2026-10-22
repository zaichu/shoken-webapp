// 撮影する画面の一覧と、画面のモック設定。
// 集合はホーム・資産管理3タブ・取引明細3タブ・銘柄検索・設定・CSV管理・ログイン。
//
// API は全て架空の応答を返す。本番の backend には繋がない。

import {
  MOCK_USER,
  assetBalances,
  dividends,
  domesticStockTransactions,
  dividendEstimates,
  fundBalances,
  mutualFundTransactions,
  stock,
} from './data.mjs';

export const VIEWPORTS = [
  { key: 'pc-1280', width: 1280, height: 800, label: 'PC 1280×800' },
  { key: 'pc-1920', width: 1920, height: 1080, label: 'PC 1920×1080' },
  { key: 'sp-390', width: 390, height: 844, label: 'スマホ 390×844' },
];

export const THEMES = ['light', 'dark'];

const CURRENT_YEAR = 2026;
const PREVIOUS_YEAR = 2025;

function paginated(data, extra = {}) {
  return {
    data,
    total: data.length,
    page: 1,
    per_page: Math.max(data.length, 1),
    ...extra,
  };
}

function json(body, status = 200) {
  return { status, contentType: 'application/json', body: JSON.stringify(body) };
}

// 取引明細のタブ。ラベルは画面の表示と同じ
const RECEIPT_TAB = {
  stock: '国内株式',
  fund: '投資信託',
};

export const SCREENS = [
  {
    key: 'home',
    label: 'ホーム',
    path: '/',
    ready: ['[data-testid="home-overview"]'],
  },
  {
    key: 'asset-all',
    label: '資産管理・すべて',
    path: '/assetbalance',
    loaded: ['[data-testid="asset-kind-tabs"]'],
    ready: ['[data-testid="asset-merged-table"] tbody tr', '[data-testid="asset-merged-list"] li'],
    prepare: async (page) => {
      await page.getByTestId('asset-kind-tab-all').click();
    },
    scroll: true,
  },
  {
    key: 'asset-stock',
    label: '資産管理・国内株式',
    path: '/assetbalance',
    loaded: ['[data-testid="asset-kind-tabs"]'],
    ready: ['[data-testid="asset-holdings-table"] tbody tr', '[data-testid="asset-holdings-list"] li'],
    prepare: async (page) => {
      await page.getByTestId('asset-kind-tab-stock').click();
    },
    scroll: true,
  },
  {
    key: 'asset-fund',
    label: '資産管理・投資信託',
    path: '/assetbalance',
    loaded: ['[data-testid="asset-kind-tabs"]'],
    ready: ['[data-testid="asset-fund-table"] tbody tr', '[data-testid="asset-fund-list"] li'],
    prepare: async (page) => {
      await page.getByTestId('asset-kind-tab-fund').click();
    },
    scroll: true,
  },
  {
    key: 'receipts-dividend',
    label: '取引明細・配当金',
    path: '/receipts',
    ready: ['[data-testid="receipt-table-card"] tbody tr'],
    scroll: true,
  },
  {
    key: 'receipts-stock',
    label: '取引明細・国内株式',
    path: '/receipts',
    loaded: ['[data-testid="receipts-workspace"]'],
    ready: ['[data-testid="receipt-table-card"] tbody tr'],
    prepare: async (page) => {
      await page.getByRole('tab', { name: new RegExp(RECEIPT_TAB.stock) }).click();
    },
    scroll: true,
  },
  {
    key: 'receipts-fund',
    label: '取引明細・投資信託',
    path: '/receipts',
    loaded: ['[data-testid="receipts-workspace"]'],
    ready: ['[data-testid="receipt-table-card"] tbody tr'],
    prepare: async (page) => {
      await page.getByRole('tab', { name: new RegExp(RECEIPT_TAB.fund) }).click();
    },
    scroll: true,
  },
  {
    key: 'search',
    label: '銘柄検索',
    path: '/search?code=9901',
    // 検索結果は銘柄名の見出し(h2)で出る
    ready: ['h2'],
  },
  {
    key: 'settings',
    label: '設定',
    path: '/settings',
    ready: ['h1'],
  },
  {
    key: 'csv',
    label: 'CSV管理',
    path: '/receipts/csv?kind=dividend',
    ready: ['[data-testid="csv-file-trigger"]'],
    scroll: true,
  },
  {
    key: 'login',
    label: 'ログイン(ログアウト状態)',
    path: '/login',
    ready: ['h1'],
    // ログアウト状態は session が 401 のとき
    anonymous: true,
  },
];

// モック以外の通信を失敗させる。本番への通信が混ざらないようにする
async function installGuards(context) {
  const unexpected = [];
  // 登録が後のものが優先される。全部の通信を一旦通し、その後に API だけを止める
  await context.route('**/*', async (route) => {
    const url = route.request().url();
    if (!url.startsWith('http://127.0.0.1')) {
      unexpected.push(url);
      await route.abort();
      return;
    }
    await route.continue();
  });
  await context.route(/\/api\//, async (route) => {
    unexpected.push(route.request().url());
    await route.abort();
  });
  return unexpected;
}

async function installMocks(context, { anonymous }) {
  const stocks = assetBalances();
  const funds = fundBalances();
  const currentDividends = dividends(CURRENT_YEAR);
  const previousDividends = dividends(PREVIOUS_YEAR);
  const domesticStocks = domesticStockTransactions();
  const mutualFunds = mutualFundTransactions();

  await context.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill(anonymous ? json({ message: 'unauthorized' }, 401) : json(MOCK_USER)),
  );
  await context.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(stocks.rows, { summary: stocks.summary }))),
  );
  await context.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(funds.rows, { summary: funds.summary }))),
  );
  await context.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) => {
    const url = new URL(route.request().url());
    const year = url.searchParams.get('year');
    const perPage = Number(url.searchParams.get('per_page') ?? '0');
    const rows =
      year === String(PREVIOUS_YEAR) ? previousDividends.rows : currentDividends.rows;
    const summary =
      year === String(PREVIOUS_YEAR) ? previousDividends.summary : currentDividends.summary;
    // per_page を指定する要求(直近の入金)は先頭から切り出す
    const data = perPage > 0 ? rows.slice(0, perPage) : rows;
    return route.fulfill(json(paginated(data, { summary })));
  });
  await context.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(domesticStocks.rows, { summary: domesticStocks.summary }))),
  );
  await context.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(mutualFunds.rows, { summary: mutualFunds.summary }))),
  );
  await context.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json(dividendEstimates())),
  );
  await context.route(/\/api\/v1\/stocks(?:\?.*)?$/, (route) => {
    const url = new URL(route.request().url());
    return route.fulfill(json(stock(url.searchParams.get('query'))));
  });
}

export async function prepareContext(context, { anonymous }) {
  const unexpected = await installGuards(context);
  await installMocks(context, { anonymous });
  return unexpected;
}
