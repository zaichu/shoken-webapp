import { mkdirSync } from 'node:fs';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '../../../../..');
const require = createRequire(import.meta.url);
const { chromium } = require(resolve(ROOT, 'frontend/node_modules/playwright'));

const { SCREENS, prepareContext } = await import(
  resolve(ROOT, 'frontend/scripts/ui-shots/screens.mjs')
);

const VIEWPORTS = [
  { key: 'pc', width: 1280, height: 800 },
  { key: 'mobile', width: 390, height: 844 },
];
const THEMES = ['light', 'dark'];

const options = { port: 8170, out: resolve(__dirname, '../shots') };
mkdirSync(options.out, { recursive: true });
const baseUrl = `http://127.0.0.1:${options.port}`;
const browser = await chromium.launch();
let count = 0;
for (const screen of SCREENS) {
  for (const vp of VIEWPORTS) {
    for (const theme of THEMES) {
      const context = await browser.newContext({
        viewport: { width: vp.width, height: vp.height },
        colorScheme: theme,
        deviceScaleFactor: 1,
        locale: 'ja-JP',
      });
      const hits = await prepareContext(context, { anonymous: Boolean(screen.anonymous), admin: Boolean(screen.admin) });
      const page = await context.newPage();
      const state = 'default';
      const name = `${screen.key}-${state}-${vp.key}-${theme}`;
      try {
        await page.goto(`${baseUrl}${screen.path}`, { waitUntil: 'load' });
        if (screen.prepare) {
          await page.waitForFunction((sels) => sels.some((s) => document.querySelector(s) !== null), screen.loaded ?? screen.ready, { timeout: 30000 });
          await screen.prepare(page);
        }
        await page.waitForFunction((sels) => sels.some((s) => document.querySelector(s) !== null), screen.ready, { timeout: 30000 });
        await page.waitForTimeout(500);
        await page.screenshot({ path: join(options.out, `${name}.png`) });
        count++;
        if (screen.scroll) {
          const scroller = await page.$('.app-scroll');
          if (scroller) {
            await page.evaluate(() => { const s = document.querySelector('.app-scroll'); if (s) s.scrollTop = s.scrollHeight - s.clientHeight; });
            await page.waitForTimeout(300);
            await page.screenshot({ path: join(options.out, `${name}-scrolled.png`) });
            count++;
          }
        }
      } catch (e) {
        console.error('FAIL', name, e.message);
      } finally {
        if (hits.length) console.error('unexpected', hits);
        await context.close();
      }
    }
  }
  console.log('done', screen.key);
}
await browser.close();
console.log('captured', count);