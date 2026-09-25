import { expect, test, type TestInfo } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

/**
 * mobile-receipts-*.png を react-receipts-<tab>-<pc|mobile>.png へ複製する。
 * REACT_SHOT_SUFFIX 未指定時はスキップする(ゲートの全 spec 実行を壊さないため)。
 *
 * 入力名の mobile- は元 spec の固定命名で幅を表さないため、PNG の幅を
 * 検査してから複製する(pc=1920、mobile=390x3)。
 *
 * 使い方:
 *   PC 撮影後:    REACT_SHOT_SUFFIX=pc     npx playwright test --config ../frontend-leptos/playwright.receipts.config.ts rename-receipts-screenshots.spec.ts
 *   スマホ撮影後: REACT_SHOT_SUFFIX=mobile 同コマンド
 */

const TABS = ['dividend', 'domestic-stock', 'mutualfund'] as const;
const EXPECTED_WIDTH = { pc: 1920, mobile: 1170 } as const;

function shotDir(testInfo: TestInfo): string {
  return path.resolve(testInfo.project.testDir, '../../.playwright-mcp');
}

function pngWidth(file: string): number {
  return fs.readFileSync(file).readUInt32BE(16);
}

test('receipts スクショを react- 命名へ複製', async ({}, testInfo) => {
  const raw = process.env.REACT_SHOT_SUFFIX;
  test.skip(raw !== 'pc' && raw !== 'mobile', 'REACT_SHOT_SUFFIX=pc|mobile を指定して実行する');
  const suffix = raw as 'pc' | 'mobile';

  const dir = shotDir(testInfo);
  const sources = TABS.map((tab) => path.join(dir, `mobile-receipts-${tab}.png`));
  expect(
    sources.filter((src) => !fs.existsSync(src)),
    'ソース画像の欠落',
  ).toEqual([]);
  for (const src of sources) {
    expect(pngWidth(src), `${path.basename(src)} の幅`).toBe(EXPECTED_WIDTH[suffix]);
  }
  for (let i = 0; i < TABS.length; i += 1) {
    fs.copyFileSync(sources[i], path.join(dir, `react-receipts-${TABS[i]}-${suffix}.png`));
  }
});
