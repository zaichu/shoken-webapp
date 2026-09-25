import { test, type TestInfo } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

/**
 * React 版 e2e/mobile-screenshots.spec.ts の成果物(mobile-receipts-*.png)を
 * 比較用命名 react-receipts-<tab>-<pc|mobile>.png へ複製するユーティリティ。
 * ファイルコピーは副作用専用で、REACT_SHOT_SUFFIX 未指定時はスキップする
 * (ゲートの全 spec 実行を壊さないため)。
 *
 * 使い方:
 *   PC 撮影後:    REACT_SHOT_SUFFIX=pc     npx playwright test --config ../frontend-leptos/playwright.receipts.config.ts rename-receipts-screenshots.spec.ts
 *   スマホ撮影後: REACT_SHOT_SUFFIX=mobile 同コマンド
 */

const TABS = ['dividend', 'domestic-stock', 'mutualfund'] as const;

function shotDir(testInfo: TestInfo): string {
  return path.resolve(testInfo.project.testDir, '../../.playwright-mcp');
}

test('receipts スクショを react- 命名へ複製', async ({}, testInfo) => {
  const suffix = process.env.REACT_SHOT_SUFFIX;
  test.skip(
    suffix !== 'pc' && suffix !== 'mobile',
    'REACT_SHOT_SUFFIX=pc|mobile を指定して実行する',
  );

  const dir = shotDir(testInfo);
  for (const tab of TABS) {
    const src = path.join(dir, `mobile-receipts-${tab}.png`);
    const dst = path.join(dir, `react-receipts-${tab}-${suffix}.png`);
    test.skip(!fs.existsSync(src), `${src} が未取得`);
    fs.copyFileSync(src, dst);
  }
});
