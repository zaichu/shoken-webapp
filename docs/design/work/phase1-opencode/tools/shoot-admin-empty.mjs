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
const V = [{ key: 'pc', w: 1280, h: 800 }, { key: 'mobile', w: 390, h: 844 }];
const T = ['light', 'dark'];

const browser = await chromium.launch();

for (const vp of V) {
  for (const th of T) {
    const ctx = await browser.newContext({ viewport: { width: vp.w, height: vp.h }, colorScheme: th, deviceScaleFactor: 1, locale: 'ja-JP' });
    await prepareContext(ctx, { anonymous: false, admin: true });
    const page = await ctx.newPage();
    try {
      await page.goto(`${BASE}/admin/logins`);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(800);
      const pcSearch = page.getByRole('searchbox', { name: 'メール・場所で検索' });
      if (await pcSearch.count()) {
        await pcSearch.fill('no-matching-login');
      } else {
        await page.getByTestId('admin-login-search-entry').click();
        await page.getByTestId('admin-login-search-input').fill('no-matching-login');
        await page.getByTestId('search-dialog-search').click();
      }
      await page.waitForTimeout(800);
      await page.screenshot({ path: join(OUT, `admin-logins-empty-default-${vp.key}-${th}.png`) });
      console.log('ok', vp.key, th);
    } catch (e) { console.error(e.message); }
    await ctx.close();
  }
}

await browser.close();