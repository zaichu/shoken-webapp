import { existsSync, readdirSync } from 'node:fs';
import { join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

function walk(directory, visit) {
  const entries = readdirSync(directory, { withFileTypes: true });
  visit(directory, entries);
  for (const entry of entries) {
    if (entry.isDirectory()) walk(join(directory, entry.name), visit);
  }
}

export function checkTestLayout(root) {
  const violations = [];
  const display = (path) => relative(root, path).split(sep).join('/');
  walk(join(root, 'src'), (directory, entries) => {
    if (entries.some((entry) => entry.name === 'tests.rs' && entry.isFile()) &&
        entries.some((entry) => entry.name === 'tests' && entry.isDirectory())) {
      violations.push(`${display(directory)}: tests.rs と tests/ を同じ階層に置かない`);
    }
  });

  const e2e = join(root, 'e2e');
  const issueName = /^(?:(?:i[-_]?|issue[-_]?|pr[-_]?|#)\d+|\d+)(?:[-_.]|$)/i;
  walk(e2e, (directory, entries) => {
    for (const entry of entries) {
      if (!entry.isFile() || !entry.name.endsWith('.spec.ts')) continue;
      const path = join(directory, entry.name);
      const parts = relative(e2e, path).split(sep);
      if (parts.length !== 2) {
        violations.push(`${display(path)}: E2E は e2e/<機能>/<内容>.spec.ts に置く`);
      }
      if (['support', 'fixtures', '__fixtures__', 'migrated'].includes(parts[0])) {
        violations.push(`${display(path)}: ${parts[0]} ではなく機能フォルダに置く`);
      }
      if (parts.some((part) => issueName.test(part))) {
        violations.push(`${display(path)}: Issue番号ではなく内容で命名する`);
      }
      if (/(?:^|[-_])screenshots?(?:[-_.]|$)/i.test(entry.name)) {
        violations.push(`${display(path)}: 撮影専用 spec は Git 管理外の .local-e2e/ に置く`);
      }
    }
  });
  for (const oldPath of ['e2e/migrated', 'e2e/__fixtures__']) {
    const path = join(root, oldPath);
    if (existsSync(path) && readdirSync(path).length > 0) {
      violations.push(`${oldPath}: 旧配置を使わない`);
    }
  }
  return violations;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = fileURLToPath(new URL('..', import.meta.url));
  const violations = checkTestLayout(root);
  if (violations.length) {
    console.error(violations.join('\n'));
    process.exitCode = 1;
  } else {
    console.log('テストの配置検査: OK');
  }
}
