import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const { chromium } = require('/home/zaichu/project/.worktrees/shoken-webapp-1890-opencode/frontend/node_modules/playwright');
import { prepareContext } from '/home/zaichu/project/.worktrees/shoken-webapp-1890-opencode/frontend/scripts/ui-shots/screens.mjs';
const OUT='/home/zaichu/project/.worktrees/shoken-webapp-1890-opencode/docs/design/work/phase1-opencode/shots';
const BASE='http://127.0.0.1:8170';
const browser=await chromium.launch();
for(const th of ['light','dark']){
 const ctx=await browser.newContext({viewport:{width:390,height:844},colorScheme:th,deviceScaleFactor:1,locale:'ja-JP'});
 await prepareContext(ctx,{anonymous:false,admin:true});
 const page=await ctx.newPage();
 try{
   await page.goto(`${BASE}/admin/logins?q=no-matching-login`);
   await page.waitForLoadState('networkidle'); await page.waitForTimeout(800);
   await page.screenshot({path:join(OUT,`admin-logins-empty-default-mobile-${th}.png`)});
   console.log('ok',th);
 }catch(e){console.error(e.message);}
 await ctx.close();
}
await browser.close();
