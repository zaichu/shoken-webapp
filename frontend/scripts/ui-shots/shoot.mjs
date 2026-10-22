#!/usr/bin/env node
// 撮影本体。起動済みの配信に Playwright の Chromium で接続して撮る。
// 配信の起動・停止は ui-shots.sh が担う。
//
// 使い:
//   node scripts/ui-shots/shoot.mjs --port 8160 --out test-results/shots [--only home,asset-stock]

import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { chromium } from '@playwright/test';
import { SCREENS, THEMES, VIEWPORTS, prepareContext } from './screens.mjs';

function parseArgs(argv) {
  const options = { only: null };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--port') {
      options.port = Number(argv[(index += 1)]);
    } else if (arg === '--out') {
      options.out = argv[(index += 1)];
    } else if (arg === '--only') {
      options.only = argv[(index += 1)];
    } else {
      throw new Error(`不明な引数: ${arg}`);
    }
  }
  if (!options.port || !options.out) {
    throw new Error('--port --out は必須です');
  }
  return options;
}

// 画面が準備できた要素が DOM に現れるまで待つ。
// 表とカード一覧は幅で切り替わるので候補を複数持つ
async function waitForReady(page, selectors) {
  await page.waitForFunction(
    (candidates) => candidates.some((selector) => document.querySelector(selector) !== null),
    selectors,
    { timeout: 30_000 },
  );
}

// 本文(main.app-scroll)を最下端まで送る。スクロール領域は窓ではない
async function scrollToBottom(page) {
  await page.evaluate(() => {
    const scroller = document.querySelector('.app-scroll');
    if (scroller) {
      scroller.scrollTop = scroller.scrollHeight - scroller.clientHeight;
    }
  });
  await page.waitForTimeout(300);
}

const options = parseArgs(process.argv.slice(2));
const only = options.only ? new Set(options.only.split(',').map((key) => key.trim())) : null;
const screens = only ? SCREENS.filter((screen) => only.has(screen.key)) : SCREENS;
if (screens.length === 0) {
  throw new Error('--only に一致する画面がありません');
}

mkdirSync(options.out, { recursive: true });
const baseUrl = `http://127.0.0.1:${options.port}`;
const browser = await chromium.launch();
const files = [];
const unexpected = [];

try {
  for (const screen of screens) {
    for (const viewport of VIEWPORTS) {
      for (const theme of THEMES) {
        const context = await browser.newContext({
          viewport: { width: viewport.width, height: viewport.height },
          colorScheme: theme,
          deviceScaleFactor: 1,
          locale: 'ja-JP',
        });
        const hits = await prepareContext(context, { anonymous: Boolean(screen.anonymous) });
        const page = await context.newPage();
        const name = `${screen.key}-${viewport.width}x${viewport.height}-${theme}`;
        try {
          await page.goto(`${baseUrl}${screen.path}`, { waitUntil: 'load' });
          if (screen.prepare) {
            await waitForReady(page, screen.loaded ?? screen.ready);
            await screen.prepare(page);
          }
          await waitForReady(page, screen.ready);
          await page.waitForTimeout(500);
          await page.screenshot({ path: join(options.out, `${name}.png`) });
          files.push(`${name}.png`);
          // 表・一覧がある画面はスクロール後も見せる(見出しが追従するか確かめる)
          if (screen.scroll && viewport.width > 768) {
            await scrollToBottom(page);
            await page.screenshot({ path: join(options.out, `${name}-scrolled.png`) });
            files.push(`${name}-scrolled.png`);
          }
        } finally {
          unexpected.push(...hits);
          await context.close();
        }
      }
    }
    console.log(`撮影: ${screen.label}`);
  }
} finally {
  await browser.close();
}

if (unexpected.length > 0) {
  console.error('モック以外の通信がありました:');
  for (const url of unexpected) {
    console.error(`  ${url}`);
  }
  process.exitCode = 1;
}
console.log(`${files.length} 枚を撮りました: ${options.out}`);
