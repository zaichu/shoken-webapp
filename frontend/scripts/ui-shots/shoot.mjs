#!/usr/bin/env node
// 撮影本体。起動済みの配信に Playwright の Chromium で接続して撮る。
// 配信の起動・停止は ui-shots.sh が担う。
//
// 使い:
//   node scripts/ui-shots/shoot.mjs --port 8160 --out test-results/shots [--only home,asset-stock] [--viewports 390,1280,1920]

import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { chromium } from '@playwright/test';
import {
  assertCandidateUsed,
  collectManifest,
  compareManifest,
  installCandidate,
  parseNumerals,
  specimenHtml,
} from './numerals.mjs';
import { SCREENS, THEMES, VIEWPORTS, prepareContext } from './screens.mjs';

// 書体比較で拡大して撮る部位。ホームの大きな金額と、表の金額
const ZOOM_TARGETS = {
  home: { name: 'overview', selectors: ['[data-testid="home-overview"]'] },
  'asset-stock': {
    name: 'table',
    selectors: ['[data-testid="asset-holdings-table"]', '[data-testid="asset-holdings-list"]'],
    rows: 'tbody tr, li',
    rowCount: 6,
  },
};
const ZOOM_SCALE = 2;

function parseArgs(argv) {
  const options = { only: null, viewportWidths: null, numerals: null };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--port') {
      options.port = Number(argv[(index += 1)]);
    } else if (arg === '--out') {
      options.out = argv[(index += 1)];
    } else if (arg === '--only') {
      options.only = argv[(index += 1)];
    } else if (arg === '--viewports') {
      options.viewportWidths = argv[(index += 1)].split(',').map((width) => Number(width.trim()));
    } else if (arg === '--numerals') {
      options.numerals = parseNumerals(argv[(index += 1)]);
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
// latest/ は 390・1280・1920 だけに絞って軽くする。PR 確認用は既定の全幅のまま
const viewports = options.viewportWidths
  ? VIEWPORTS.filter((viewport) => options.viewportWidths.includes(viewport.width))
  : VIEWPORTS;
if (viewports.length === 0) {
  throw new Error('--viewports に一致する幅がありません');
}

mkdirSync(options.out, { recursive: true });
const baseUrl = `http://127.0.0.1:${options.port}`;
const browser = await chromium.launch();
const files = [];
const unexpected = [];

// 画面を開いて撮影できる状態にする。candidate を渡すと Amount の書体だけを候補に替える
async function openScreen(screen, viewport, theme, { candidate = null, deviceScaleFactor = 1 } = {}) {
  const context = await browser.newContext({
    viewport: { width: viewport.width, height: viewport.height },
    colorScheme: theme,
    deviceScaleFactor,
    locale: 'ja-JP',
  });
  const hits = await prepareContext(context, { anonymous: Boolean(screen.anonymous), admin: Boolean(screen.admin) });
  if (candidate) {
    await installCandidate(context, candidate);
  }
  const page = await context.newPage();
  await page.goto(`${baseUrl}${screen.path}`, { waitUntil: 'load' });
  if (screen.prepare) {
    await waitForReady(page, screen.loaded ?? screen.ready);
    await screen.prepare(page);
  }
  await waitForReady(page, screen.ready);
  await page.evaluate(() => document.fonts.ready);
  if (candidate) {
    await assertCandidateUsed(page, candidate);
  }
  await page.waitForTimeout(500);
  return { context, page, hits };
}

async function shootScreens(prefix, candidate) {
  const report = [];
  for (const screen of screens) {
    for (const viewport of viewports) {
      for (const theme of THEMES) {
        const name = `${prefix}${screen.key}-${viewport.width}x${viewport.height}-${theme}`;
        const { context, page, hits } = await openScreen(screen, viewport, theme, { candidate });
        try {
          await page.screenshot({ path: join(options.out, `${name}.png`) });
          files.push(`${name}.png`);
          if (candidate) {
            report.push({ screen: screen.key, viewport: viewport.width, theme, manifest: await collectManifest(page) });
          }
          // 表・一覧がある画面はスクロール後も見せる(見出しが追従するか確かめる)
          if (screen.scroll) {
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
    console.log(`撮影: ${prefix}${screen.label}`);
  }
  return report;
}

async function shootSpecimen(candidate) {
  for (const theme of THEMES) {
    const context = await browser.newContext({
      viewport: { width: 1280, height: 800 },
      colorScheme: theme,
      deviceScaleFactor: 1,
      locale: 'ja-JP',
    });
    const hits = await prepareContext(context, { anonymous: true, admin: false });
    const page = await context.newPage();
    const url = `${baseUrl}/__numerals-specimen`;
    await page.route(url, (route) =>
      route.fulfill({ status: 200, contentType: 'text/html; charset=utf-8', body: specimenHtml(candidate) }),
    );
    try {
      await page.goto(url, { waitUntil: 'load' });
      const faces = await page.evaluate(async (family) => {
        await document.fonts.ready;
        return [...document.fonts]
          .filter((face) => face.family.replaceAll('"', '') === family)
          .map((face) => face.status);
      }, candidate.family);
      if (faces.length === 0 || faces.some((status) => status !== 'loaded')) {
        throw new Error(`${candidate.key}: 字形見本の書体が読み込めていません ${JSON.stringify(faces)}`);
      }
      const name = `${candidate.key}-specimen-${theme}.png`;
      await page.screenshot({ path: join(options.out, name), fullPage: true });
      files.push(name);
    } finally {
      unexpected.push(...hits);
      await context.close();
    }
  }
}

// 実画面の対象を高い解像度で切り出す。大きさや配置は変えない
async function shootZoom(candidate) {
  for (const screen of screens) {
    const target = ZOOM_TARGETS[screen.key];
    if (!target) {
      continue;
    }
    for (const viewport of viewports) {
      for (const theme of THEMES) {
        const { context, page, hits } = await openScreen(screen, viewport, theme, {
          candidate,
          deviceScaleFactor: ZOOM_SCALE,
        });
        try {
          const clip = await page.evaluate(({ selectors, rows, rowCount }) => {
            // 表と一覧は幅で片方が非表示になるので、見えているほうを使う
            const element = selectors
              .map((selector) => document.querySelector(selector))
              .find((candidate) => candidate && candidate.getBoundingClientRect().width > 0);
            if (!element) {
              return null;
            }
            const lastRow = () => {
              const items = rows ? element.querySelectorAll(rows) : [];
              return items[Math.min(rowCount, items.length) - 1] ?? element;
            };
            // 画面内に収まっていればそのまま切り出す。送ると PC の追従する見出しが表に重なる
            if (lastRow().getBoundingClientRect().bottom > window.innerHeight) {
              element.scrollIntoView({ block: 'start' });
            }
            const box = element.getBoundingClientRect();
            const bottom = rows ? lastRow().getBoundingClientRect().bottom : box.bottom;
            const top = Math.max(box.top, 0);
            return {
              x: Math.max(box.left, 0),
              y: top,
              width: Math.min(box.width, window.innerWidth - Math.max(box.left, 0)),
              height: Math.min(bottom, window.innerHeight) - top,
            };
          }, target);
          if (!clip) {
            throw new Error(`${screen.key}: 拡大する部位が見つかりません`);
          }
          await page.waitForTimeout(300);
          const name = `${candidate.key}-zoom-${screen.key}-${target.name}-${viewport.width}x${viewport.height}-${theme}.png`;
          await page.screenshot({ path: join(options.out, name), clip });
          files.push(name);
        } finally {
          unexpected.push(...hits);
          await context.close();
        }
      }
    }
  }
}

// 候補と同じ状態の baseline(書体を替えない)を記録し、変えてはいけない値が同じか比べる
async function baselineManifests() {
  const manifests = new Map();
  for (const screen of screens) {
    for (const viewport of viewports) {
      for (const theme of THEMES) {
        const { context, page, hits } = await openScreen(screen, viewport, theme);
        try {
          manifests.set(`${screen.key}-${viewport.width}-${theme}`, await collectManifest(page));
        } finally {
          unexpected.push(...hits);
          await context.close();
        }
      }
    }
  }
  return manifests;
}

try {
  if (options.numerals) {
    const baseline = await baselineManifests();
    const summary = [];
    for (const candidate of options.numerals) {
      await shootSpecimen(candidate);
      const report = await shootScreens(`${candidate.key}-`, candidate);
      await shootZoom(candidate);
      for (const entry of report) {
        const diff = compareManifest(baseline.get(`${entry.screen}-${entry.viewport}-${entry.theme}`), entry.manifest);
        summary.push({ candidate: candidate.key, screen: entry.screen, viewport: entry.viewport, theme: entry.theme, ...diff });
      }
    }
    writeFileSync(join(options.out, 'numerals-report.json'), `${JSON.stringify(summary, null, 2)}\n`);
    const styleDiffs = summary.filter((entry) => entry.styleDiffs.length > 0);
    console.log(
      `書体比較: 変えてはいけない値の差 ${styleDiffs.length} 件 / 位置が動いた枠のある状態 ${
        summary.filter((entry) => entry.containerDiffs.length > 0).length
      } 件(numerals-report.json)`,
    );
    if (styleDiffs.length > 0) {
      process.exitCode = 1;
    }
  } else {
    await shootScreens('', null);
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
