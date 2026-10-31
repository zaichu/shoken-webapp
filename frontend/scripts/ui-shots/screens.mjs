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
  { key: 'pc-1024', width: 1024, height: 768, label: 'PC 1024×768' },
  { key: 'pc-1280', width: 1280, height: 800, label: 'PC 1280×800' },
  { key: 'pc-1440', width: 1440, height: 900, label: 'PC 1440×900' },
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
    key: 'admin-logins',
    label: 'ログイン一覧(管理者)',
    path: '/admin/logins',
    ready: ['[data-testid="admin-login-table"] tbody tr', '[data-testid="admin-login-list"] li'],
    admin: true,
    scroll: true,
  },
  {
    key: 'admin-logins-empty',
    label: 'ログイン一覧(管理者)・検索0件',
    path: '/admin/logins',
    loaded: ['[data-testid="admin-login-table"] tbody tr', '[data-testid="admin-login-list"] li'],
    ready: ['[data-testid="empty-state"]'],
    admin: true,
    prepare: async (page) => {
      await page.getByRole('searchbox', { name: 'メール・場所で検索' }).fill('no-matching-login');
    },
  },
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
    ready: ['#login-history tbody tr', '#devices li'],
    scroll: true,
  },
  {
    key: 'settings-login-history',
    label: '設定・ログイン履歴と登録端末',
    path: '/settings',
    ready: ['#login-history tbody tr', '#devices li'],
    prepare: async (page) => {
      await page.locator('#login-history').scrollIntoViewIfNeeded();
    },
    scroll: true,
  },
  {
    key: 'csv',
    label: 'CSV管理・取引明細',
    path: '/csv?kind=dividend',
    ready: ['[data-testid="csv-file-trigger"]'],
    scroll: true,
  },
  {
    key: 'csv-asset',
    label: 'CSV管理・資産管理',
    path: '/csv?kind=assetbalance',
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

async function installMocks(context, { anonymous, admin }) {
  const stocks = assetBalances();
  const funds = fundBalances();
  const currentDividends = dividends(CURRENT_YEAR);
  const previousDividends = dividends(PREVIOUS_YEAR);
  const domesticStocks = domesticStockTransactions();
  const mutualFunds = mutualFundTransactions();

  await context.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill(anonymous ? json({ message: 'unauthorized' }, 401) : json({ ...MOCK_USER, is_admin: Boolean(admin) })),
  );
  await context.route(/\/api\/v1\/admin\/logins(?:\?.*)?$/, (route) => {
    const params = new URL(route.request().url()).searchParams;
    if (!admin) return route.fulfill(json({ error: { code: 'FORBIDDEN', message: '管理者権限が必要です' } }, 403));
    const all = adminLogins();
    const q = (params.get('q') ?? '').toLowerCase().split(/\s+/).filter(Boolean);
    const email = (params.get('email') ?? '').toLowerCase();
    const place = (params.get('place') ?? '').toLowerCase();
    const rows = all.filter((row) => {
      const location = `${row.country ?? ''} ${row.region ?? ''}`.toLowerCase();
      return row.email.toLowerCase().includes(email) && location.includes(place)
        && q.every((token) => `${row.email} ${location}`.toLowerCase().includes(token));
    });
    const key = params.get('sort_by') ?? 'occurred_at';
    const value = (row) => key === 'place' ? `${row.country ?? ''} ${row.region ?? ''}` : row[key];
    rows.sort((a, b) => String(value(a)).localeCompare(String(value(b))) * (params.get('sort_order') === 'asc' ? 1 : -1));
    const page = Number(params.get('page') ?? 1);
    const per_page = Number(params.get('per_page') ?? 50);
    return route.fulfill(json({ data: rows.slice((page - 1) * per_page, page * per_page), total: rows.length, page, per_page }));
  });
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
  await context.route(/\/api\/v1\/login-history(?:\?.*)?$/, (route) =>
    route.fulfill(json(loginHistory())),
  );
  await context.route(/\/api\/v1\/devices$/, (route) => route.fulfill(json(deviceList())));
  await context.route(/\/api\/v1\/notification-settings$/, (route) =>
    route.fulfill(json({ login_notify_enabled: true })),
  );
}

// ログイン履歴・登録端末の撮影用データ。12件以上の履歴・長いブラウザ名・5台以上の端末
function loginHistory() {
  const rows = [
    ['2026-10-31T00:12:00Z', 'success', 'JP', 'Tokyo', 'chrome', 'windows'],
    ['2026-10-30T15:03:00Z', 'new_device', 'JP', 'Osaka', 'safari', 'ios'],
    ['2026-10-29T09:44:00Z', 'success', 'JP', 'Tokyo', 'edge', 'windows'],
    ['2026-10-28T22:18:00Z', 'location_changed', 'US', 'California', 'firefox', 'linux'],
    ['2026-10-27T11:05:00Z', 'success', 'JP', 'Fukuoka', 'chrome', 'android'],
    ['2026-10-26T08:31:00Z', 'failure', null, null, 'chrome', 'windows'],
    ['2026-10-25T19:52:00Z', 'success', 'JP', 'Hokkaido', 'safari', 'macos'],
    [
      '2026-10-24T13:20:00Z',
      'success',
      'JP',
      'Tokyo',
      'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0 とても長いブラウザ名の表示確認用',
      'Windows 10 Pro 64ビット版とても長いOS名の表示確認用',
    ],
    ['2026-10-23T07:11:00Z', 'new_device', 'JP', 'Aichi', 'firefox', 'linux'],
    ['2026-10-22T18:40:00Z', 'success', 'JP', 'Kyoto', 'chrome', 'macos'],
    ['2026-10-21T12:26:00Z', 'location_changed', 'KR', 'Seoul', 'safari', 'ios'],
    ['2026-10-20T05:59:00Z', 'success', null, null, null, null],
    ['2026-10-19T21:14:00Z', 'success', 'JP', 'Okinawa', 'edge', 'windows'],
  ];
  return {
    data: rows.map(([occurred_at, result, country, region, browser, os]) => ({
      occurred_at,
      result,
      country,
      region,
      browser,
      os,
    })),
  };
}

function deviceList() {
  const rows = [
    ['device-1', '2026-10-31T00:12:00Z', 'JP', 'Tokyo', 'chrome', 'windows', true],
    ['device-2', '2026-10-30T15:03:00Z', 'JP', 'Osaka', 'safari', 'ios', false],
    ['device-3', '2026-10-28T22:18:00Z', 'US', 'California', 'firefox', 'linux', false],
    ['device-4', '2026-10-25T19:52:00Z', 'JP', 'Hokkaido', 'safari', 'macos', false],
    [
      'device-5',
      '2026-10-24T13:20:00Z',
      'JP',
      'Tokyo',
      'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0 とても長いブラウザ名の表示確認用',
      'Windows 10 Pro 64ビット版とても長いOS名の表示確認用',
      false,
    ],
    ['device-6', '2026-10-21T12:26:00Z', 'KR', 'Seoul', 'chrome', 'android', false],
  ];
  return {
    data: rows.map(([id, last_used_at, country, region, browser, os, is_current]) => ({
      id,
      last_used_at,
      country,
      region,
      browser,
      os,
      is_current,
    })),
  };
}

export async function prepareContext(context, { anonymous, admin }) {
  const unexpected = await installGuards(context);
  await installMocks(context, { anonymous, admin });
  return unexpected;
}

export function adminLogins() {
  return Array.from({ length: 64 }, (_, index) => ({
    occurred_at: new Date(Date.UTC(2026, 9, 31, 0, 12) - index * 3600000).toISOString(),
    email: index % 5 === 0
      ? `long.email.for.administrator.login.history.display.check.${index}@example.com`
      : `user${String(index).padStart(2, '0')}@example.com`,
    result: ['success', 'new_device', 'location_changed', 'failure'][index % 4],
    country: index % 7 === 0 ? null : 'JP',
    region: [null, 'Tokyo', 'Osaka', 'Hokkaido'][index % 4],
    browser: ['chrome', 'safari', 'edge', 'firefox'][index % 4],
    os: ['windows', 'ios', 'macos', 'linux'][index % 4],
  }));
}
