// icons/icon.svg から各サイズの PNG を生成する。実行: node scripts/render-icons.mjs
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const svg = readFileSync(join(root, 'icons/icon.svg'), 'utf8');
const NAVY = '#1e3560';

const targets = [
  { file: 'favicon-16.png', size: 16, opaque: false },
  { file: 'favicon-32.png', size: 32, opaque: false },
  { file: 'apple-touch-icon.png', size: 180, opaque: true },
  { file: 'icon-192.png', size: 192, opaque: true },
  { file: 'icon-512.png', size: 512, opaque: true },
];

const browser = await chromium.launch();
try {
  for (const { file, size, opaque } of targets) {
    const page = await browser.newPage({
      viewport: { width: size, height: size },
      deviceScaleFactor: 1,
    });
    const body = svg.replace('<svg ', `<svg width="${size}" height="${size}" `);
    await page.setContent(
      `<!doctype html><body style="margin:0;${opaque ? `background:${NAVY};` : ''}">${body}</body>`,
    );
    const png = await page.screenshot({ omitBackground: !opaque });
    writeFileSync(join(root, 'icons', file), png);
    await page.close();
    console.log(`${file} (${size}px${opaque ? ', opaque' : ''})`);
  }
} finally {
  await browser.close();
}
