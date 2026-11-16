import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import stylelint from 'stylelint';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const configFile = join(root, '.stylelintrc.json');

async function lintFixture(name) {
  const code = readFileSync(join(root, 'scripts', 'fixtures', name), 'utf8');
  const result = await stylelint.lint({ code, configFile });
  return result.results[0].warnings;
}

test('違反fixtureは色・文字サイズ・print後の直書きで止まる', async () => {
  const warnings = await lintFixture('css-strict-invalid.css');
  const texts = warnings.map((warning) => warning.text);
  assert.ok(texts.some((text) => text.includes('#c0362c')), texts.join('\n'));
  assert.ok(texts.some((text) => text.includes('14px')), texts.join('\n'));
  assert.ok(texts.some((text) => text.includes('4px')), texts.join('\n'));
  assert.equal(warnings.length, 3, JSON.stringify(warnings, null, 2));
});

test('1行で閉じる @media print の中は止めない', async () => {
  const warnings = await lintFixture('css-strict-invalid.css');
  assert.ok(
    warnings.every((warning) => !warning.text.includes('9px')),
    JSON.stringify(warnings, null, 2),
  );
});

test('正常fixtureは止めない', async () => {
  const warnings = await lintFixture('css-strict-valid.css');
  assert.deepEqual(warnings, []);
});
