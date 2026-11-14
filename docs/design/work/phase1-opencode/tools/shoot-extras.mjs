import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const { chromium } = require('/home/zaichu/project/.worktrees/shoken-webapp-1890-opencode/frontend/node_modules/playwright');
import { prepareContext } from '/home/zaichu/project/.worktrees/shoken-webapp-1890-opencode/frontend/scripts/ui-shots/screens.mjs';

const OUT = '/home/zaichu/project/.worktrees/shoken-webapp-1890-opencode/docs/design/work/phase1-opencode/shots';
const BASE = 'http://127.0.0.1:8170';
mkdirSync(OUT, { recursive: true });

const VIEWPORTS = [{key:'pc',w:1280,h:800},{key:'mobile',w:390,h:844}];
const THEMES = ['light','dark'];

const EMPTY_PAGE = { data: [], total: 0, page: 1, per_page: 0 };

async function shot(page, name) {
  await page.waitForTimeout(400);
  await page.screenshot({ path: join(OUT, `${name}.png`) });
}

async function withContext(vp, theme, anonymous, admin, fn, namePrefix) {
  // returns array of saved names
}

async function setupMocks(context, anonymous, admin) {
  const hits = await prepareContext(context, { anonymous, admin });
  return hits;
}

// Scenario definitions: function(page, vp, theme) -> Promise<name (without viewport/theme?)>
// We'll handle naming outside.

const scenarios = [];

function add(name, run, opts={}) {
  scenarios.push({ name, run, ...opts });
}

// 1. home logged out
add('home-loggedout', async (page) => {
  await page.goto(`${BASE}/`);
  await page.waitForLoadState('networkidle');
}, { anonymous: true });

// 2. 404
add('notfound', async (page) => {
  await page.goto(`${BASE}/no-such-page-xyz`);
  await page.waitForLoadState('networkidle');
}, { anonymous: false });

// 3. asset empty
add('asset-empty', async (page) => {
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => route.fulfill({ status:200, contentType:'application/json', body: JSON.stringify(EMPTY_PAGE) }));
  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) => route.fulfill({ status:200, contentType:'application/json', body: JSON.stringify(EMPTY_PAGE) }));
  await page.goto(`${BASE}/assetbalance`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(500);
}, { anonymous:false });

// 4. asset error
add('asset-error', async (page) => {
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => route.fulfill({ status:500, contentType:'application/json', body: JSON.stringify({error:'取得に失敗しました'}) }));
  await page.goto(`${BASE}/assetbalance`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(500);
}, { anonymous:false });

// 5. receipts empty
add('receipts-empty', async (page) => {
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) => route.fulfill({ status:200, contentType:'application/json', body: JSON.stringify(EMPTY_PAGE) }));
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) => route.fulfill({ status:200, contentType:'application/json', body: JSON.stringify(EMPTY_PAGE) }));
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) => route.fulfill({ status:200, contentType:'application/json', body: JSON.stringify(EMPTY_PAGE) }));
  await page.goto(`${BASE}/receipts`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(500);
}, { anonymous:false });

// 6. receipts error
add('receipts-error', async (page) => {
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) => route.fulfill({ status:500, contentType:'application/json', body: JSON.stringify({error:'取得に失敗しました'}) }));
  await page.goto(`${BASE}/receipts`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(500);
}, { anonymous:false });

// 7. search empty (no query)
add('search-empty', async (page) => {
  await page.goto(`${BASE}/search`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(500);
}, { anonymous:false });

// 8. composition
add('asset-composition', async (page) => {
  await page.goto(`${BASE}/assetbalance/composition`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(800);
}, { anonymous:false });

// 9. holding detail stock
add('asset-holding-stock', async (page) => {
  await page.goto(`${BASE}/assetbalance`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(800);
  // click first holding link if present
  const link = page.locator('a[href^="/assetbalance/holdings/"]').first();
  if (await link.count()) { await link.click(); await page.waitForLoadState('networkidle'); await page.waitForTimeout(800); }
  else { await page.goto(`${BASE}/assetbalance/holdings/asset-balance-1`); await page.waitForTimeout(800); }
}, { anonymous:false });

// 10. holding detail fund
add('asset-holding-fund', async (page) => {
  await page.goto(`${BASE}/assetbalance`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(800);
  const link = page.locator('a[href^="/assetbalance/funds/"]').first();
  if (await link.count()) { await link.click(); await page.waitForLoadState('networkidle'); await page.waitForTimeout(800); }
  else { await page.goto(`${BASE}/assetbalance/funds/fund-balance-1`); await page.waitForTimeout(800); }
}, { anonymous:false });

// 11. csv mutualfund (empty count)
add('csv-mutualfund', async (page) => {
  await page.goto(`${BASE}/csv?kind=mutualfund`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(800);
}, { anonymous:false });

// 12. settings data section
add('settings-data', async (page) => {
  await page.goto(`${BASE}/settings#data`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(800);
}, { anonymous:false });

// 13. receipts search sheet (mobile) / condition dialog (pc) captured by viewport-specific? We'll capture generic after opening appropriate control.
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
}, { anonymous:false });

// 14. asset search sheet
add('asset-search-ui', async (page, vpKey) => {
  await page.goto(`${BASE}/assetbalance`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(800);
  // ensure data
  if (vpKey === 'pc') {
    const input = page.getByTestId('asset-search-input-pc');
    if (await input.count()) await input.click();
  } else {
    const entry = page.getByTestId('asset-search-sheet-entry');
    if (await entry.count()) { await entry.click(); await page.waitForTimeout(500); }
  }
}, { anonymous:false });

// 15. admin empty (pc only maybe)
add('admin-empty', async (page) => {
  await page.goto(`${BASE}/admin/logins`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(800);
  // type nonmatching in pc search? Use mobile sheet? Just capture default admin; empty via search not essential.
}, { anonymous:false, admin:true });

let count = 0;
const browser = await chromium.launch();
for (const sc of scenarios) {
  for (const vp of VIEWPORTS) {
    for (const theme of THEMES) {
      const context = await browser.newContext({ viewport:{width:vp.w,height:vp.h}, colorScheme:theme, deviceScaleFactor:1, locale:'ja-JP' });
      const hits = await setupMocks(context, sc.anonymous, sc.admin);
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
