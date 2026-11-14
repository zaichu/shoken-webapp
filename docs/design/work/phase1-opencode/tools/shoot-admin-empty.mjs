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
for(const vp of V){for(const th of T){
 const ctx=await browser.newContext({viewport:{width:vp.w,height:vp.h},colorScheme:th,deviceScaleFactor:1,locale:'ja-JP'});
 const hits=await prepareContext(ctx,{anonymous:false,admin:true});
 const page=await ctx.newPage();
 try{
   await page.goto(`${BASE}/admin/logins?q=no-matching-login`);
   await page.waitForLoadState('networkidle'); await page.waitForTimeout(800);
   await page.screenshot({path:join(OUT,`admin-empty-default-${vp.key}-${th}.png`)});
   console.log('ok',vp.key,th);
 }catch(e){console.error(e.message);}
 await ctx.close();
}}
await browser.close();
