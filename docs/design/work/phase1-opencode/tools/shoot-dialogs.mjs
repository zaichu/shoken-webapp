import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const { chromium } = require('/home/zaichu/project/.worktrees/shoken-webapp-1890-opencode/frontend/node_modules/playwright');
import { prepareContext } from '/home/zaichu/project/.worktrees/shoken-webapp-1890-opencode/frontend/scripts/ui-shots/screens.mjs';
const OUT='/home/zaichu/project/.worktrees/shoken-webapp-1890-opencode/docs/design/work/phase1-opencode/shots';
const BASE='http://127.0.0.1:8170';
const V=[{key:'pc',w:1280,h:800},{key:'mobile',w:390,h:844}];
const T=['light','dark'];
const browser=await chromium.launch();
// receipt detail dialog
for(const vp of V){for(const th of T){
 const ctx=await browser.newContext({viewport:{width:vp.w,height:vp.h},colorScheme:th,deviceScaleFactor:1,locale:'ja-JP'});
 await prepareContext(ctx,{anonymous:false,admin:false});
 const page=await ctx.newPage();
 try{
   await page.goto(`${BASE}/receipts`); await page.waitForLoadState('networkidle'); await page.waitForTimeout(800);
   const btn=page.getByTestId('receipt-row-button').first();
   if(await btn.count()){ await btn.click(); await page.waitForTimeout(500); }
   await page.screenshot({path:join(OUT,`receipts-detail-dialog-default-${vp.key}-${th}.png`)});
   console.log('receipt detail ok',vp.key,th);
 }catch(e){console.error(e.message);}
 await ctx.close();
}}
// account menu open (click menu button)
for(const vp of V){for(const th of T){
 const ctx=await browser.newContext({viewport:{width:vp.w,height:vp.h},colorScheme:th,deviceScaleFactor:1,locale:'ja-JP'});
 await prepareContext(ctx,{anonymous:false,admin:false});
 const page=await ctx.newPage();
 try{
   await page.goto(`${BASE}/`); await page.waitForLoadState('networkidle'); await page.waitForTimeout(800);
   const menu=page.getByRole('button',{name:'メニュー'});
   if(await menu.count()){ await menu.click(); await page.waitForTimeout(500); }
   await page.screenshot({path:join(OUT,`account-menu-default-${vp.key}-${th}.png`)});
   console.log('menu ok',vp.key,th);
 }catch(e){console.error(e.message);}
 await ctx.close();
}}
await browser.close();
