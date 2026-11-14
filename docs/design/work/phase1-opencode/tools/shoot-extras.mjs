import { mkdirSync } from 'node:fs';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '../../../../..');
const require = createRequire(import.meta.url);
const { chromium } = require(resolve(ROOT, 'frontend/node_modules/playwright'));

const { prepareContext } = await import(resolve(ROOT, 'frontend/scripts/ui-shots/screens.mjs'));

const OUT = resolve(__dirname, '../shots');
const BASE = 'http://127.0.0.1:8170';
mkdirSync(OUT, { recursive: true });

const VIEWPORTS = [{ key: 'pc', w: 1280, h: 800 }, { key: 'mobile', w: 390, h: 844 }];
const THEMES = ['light', 'dark'];

const EMPTY_PAGE = { data: [], total: 0, page: 1, per_page: 0 };

async function shot(page, name) {
  await page.waitForTimeout(400);
  await page.screenshot({ path: join(OUT, `${name}.png`) });
}

function add(name, run, opts = {}) {
  scenarios.push({ name, run, ...opts });
}

const scenarios = [];

add('home-loggedout', async (page) => {
  await page.goto(`${BASE}/`);
  await page.waitForLoadState('networkidle');
}, { anonymous: true });

add('notfound', async (page) => {
  await page.goto(`${BASE}/no-such-page-xyz`);
  await page.waitForLoadState('networkidle');
}, { anonymous: false });

add('asset-empty', async (page) => {
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(EMPTY_PAGE) }));
  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) => route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(EMPTY_PAGE) }));
  await page.goto(`${BASE}/assetbalance`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(500);
}, { anonymous: false });

add('asset-error', async (page) => {
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => route.fulfill({ status: 500, contentType: 'application/json', body: JSON.stringify({ error: '取得に失敗しました' }) }));
  await page.goto(`${BASE}/assetbalance`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(500);
}, { anonymous: false });

add('receipts-empty', async (page) => {
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) => route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(EMPTY_PAGE) }));
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) => route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(EMPTY_PAGE) }));
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) => route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(EMPTY_PAGE) }));
  await page.goto(`${BASE}/receipts`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(500);
}, { anonymous: false });

add('receipts-error', async (page) => {
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) => route.fulfill({ status: 500, contentType: 'application/json', body: JSON.stringify({ error: '取得に失敗しました' }) }));
  await page.goto(`${BASE}/receipts`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(500);
}, { anonymous: false });

add('search-empty', async (page) => {
  await page.goto(`${BASE}/search`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(500);
}, { anonymous: false });

add('asset-composition', async (page) => {
  await page.goto(`${BASE}/assetbalance/composition`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(800);
}, { anonymous: false });

add('asset-holding-stock', async (page) => {
  await page.goto(`${BASE}/assetbalance`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(800);
  const link = page.locator('a[href^="/assetbalance/holdings/"]').first();
  if (await link.count()) { await link.click(); await page.waitForLoadState('networkidle'); await page.waitForTimeout(800); }
  else { await page.goto(`${BASE}/assetbalance/holdings/asset-balance-1`); await page.waitForTimeout(800); }
}, { anonymous: false });

add('asset-holding-fund', async (page) => {
  await page.goto(`${BASE}/assetbalance`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(800);
  const link = page.locator('a[href^="/assetbalance/funds/"]').first();
  if (await link.count()) { await link.click(); await page.waitForLoadState('networkidle'); await page.waitForTimeout(800); }
  else { await page.goto(`${BASE}/assetbalance/funds/fund-balance-1`); await page.waitForTimeout(800); }
}, { anonymous: false });

add('csv-mutualfund', async (page) => {
  await page.goto(`${BASE}/csv?kind=mutualfund`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(800);
}, { anonymous: false });

add('settings-data', async (page) => {
  await page.goto(`${BASE}/settings#data`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(800);
}, { anonymous: false });

add('receipts-search-ui', async (page, vpKey) => {
  await page.goto(`${BASE}/receipts`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(800);
  if (vpKey === 'pc') {
    const btn = page.getByRole('button', { name: '＋ 口座' });
    if (await btn.count()) { await btn.click(); await page.waitForTimeout(500); }
  } else {
    const entry = page.getByTestId('receipts-search-sheet-entry');
    if (await entry.count()) { await entry.click(); await page.waitForTimeout(500); }
  }
}, { anonymous: false });

add('asset-search-ui', async (page, vpKey) => {
  await page.goto(`${BASE}/assetbalance`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(800);
  if (vpKey === 'pc') {
    const input = page.getByTestId('asset-search-input-pc');
    if (await input.count()) await input.click();
  } else {
    const entry = page.getByTestId('asset-search-sheet-entry');
    if (await entry.count()) { await entry.click(); await page.waitForTimeout(500); }
  }
}, { anonymous: false });

let count = 0;
const browser = await chromium.launch();
for (const sc of scenarios) {
  for (const vp of VIEWPORTS) {
    for (const theme of THEMES) {
      const context = await browser.newContext({ viewport: { width: vp.w, height: vp.h }, colorScheme: theme, deviceScaleFactor: 1, locale: 'ja-JP' });
      const hits = await (async () => prepareContext(context, { anonymous: sc.anonymous, admin: Boolean(sc.admin) }))();
      const page = await context.newPage();
      const name = `${sc.name}-default-${vp.key}-${theme}`;
      try {
        await sc.run(page, vp.key);
        await shot(page, name);
        count++;
      } catch (e) {
        console.error('FAIL', name, e.message);
      } finally {
        if (hits && hits.length) console.error('unexpected', hits);
        await context.close();
      }
    }
  }
  console.log('done', sc.name);
}
await browser.close();
console.log('captured', count);