import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const { chromium } = require('/home/zaichu/project/.worktrees/shoken-webapp-1890-opencode/frontend/node_modules/playwright');
import { SCREENS, prepareContext } from '/home/zaichu/project/.worktrees/shoken-webapp-1890-opencode/frontend/scripts/ui-shots/screens.mjs';

const VIEWPORTS = [
  { key: 'pc', width: 1280, height: 800 },
  { key: 'mobile', width: 390, height: 844 },
];
const THEMES = ['light', 'dark'];

const options = { port: 8170, out: '/home/zaichu/project/.worktrees/shoken-webapp-1890-opencode/docs/design/work/phase1-opencode/shots' };
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
            await page.evaluate(() => { const s=document.querySelector('.app-scroll'); if(s) s.scrollTop=s.scrollHeight-s.clientHeight; });
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
