import { expect, test } from '@playwright/test';
import path from 'node:path';

const leptosRoot = path.resolve(__dirname, '..');
const expectedBase = path.resolve(
  leptosRoot,
  process.env.LEPTOS_E2E_OUTPUT_DIR || 'test-results',
);

// /tmp 固定や一方の内側への入れ子に戻ると、実行開始時の掃除で互いの成果物を消し合う
test('E2E 出力先は <base>/<suite> の並列構成になっている', ({}, testInfo) => {
  const dir = testInfo.project.outputDir;
  expect(path.dirname(dir)).toBe(expectedBase);
  expect(['leptos', 'receipts']).toContain(path.basename(dir));
});
