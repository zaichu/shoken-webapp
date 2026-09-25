#!/usr/bin/env node
// backend-origin.json を読む唯一の入口。シェル・スクリプト・spec は
// すべてここ経由で本番 backend の URL を受け取る。

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const appDir = fileURLToPath(new URL('..', import.meta.url));

export function loadBackendOrigin() {
  const config = JSON.parse(readFileSync(join(appDir, 'backend-origin.json'), 'utf8'));
  if (!config.backendOrigin) {
    throw new Error('backend-origin.json must define backendOrigin');
  }
  return config.backendOrigin;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  console.log(loadBackendOrigin());
}
