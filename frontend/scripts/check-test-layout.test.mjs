import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import test from 'node:test';
import { checkTestLayout } from './check-test-layout.mjs';

function fixture(t, files) {
  const root = mkdtempSync(join(tmpdir(), 'frontend-test-layout-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(join(root, 'src'));
  mkdirSync(join(root, 'e2e'));
  for (const file of files) {
    const path = join(root, file);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, '');
  }
  return root;
}

test('単一ファイルと分割テスト、機能別E2Eは併存できる', (t) => {
  const root = fixture(t, [
    'src/ui/button/tests.rs',
    'src/features/receipts/tests/suite.rs',
    'src/features/receipts/tests/csv.rs',
    'e2e/receipts/desktop-layout.spec.ts',
    'e2e/deploy/deploy-config.spec.ts',
    'e2e/support/test.ts',
    'e2e/fixtures/receipts-print.ts',
  ]);
  assert.deepEqual(checkTestLayout(root), []);
});

for (const [name, files, message] of [
  ['tests.rsとtests/の並存', ['src/foo/tests.rs', 'src/foo/tests/suite.rs'], /tests\.rs.*tests\//],
  ['E2E直下のspec', ['e2e/idle.spec.ts'], /e2e\/idle\.spec\.ts/],
  ['i接頭辞のIssue番号名', ['e2e/receipts/i1144-toggle.spec.ts'], /Issue番号/],
  ['Issue接頭辞の番号名', ['e2e/receipts/issue-1163-layout.spec.ts'], /Issue番号/],
  ['番号だけの名前', ['e2e/receipts/1163-layout.spec.ts'], /Issue番号/],
  ['旧migrated配置', ['e2e/migrated/flow.spec.ts'], /migrated/],
  ['撮影専用spec', ['e2e/receipts/screenshots.spec.ts'], /撮影/],
  ['補助配下のspec', ['e2e/support/idle.spec.ts'], /support/],
  ['深すぎる配置', ['e2e/receipts/mobile/flow.spec.ts'], /機能/],
]) {
  test(`${name}を検出する`, (t) => {
    const violations = checkTestLayout(fixture(t, files));
    assert.ok(violations.some((violation) => message.test(violation)), violations.join('\n'));
  });
}
