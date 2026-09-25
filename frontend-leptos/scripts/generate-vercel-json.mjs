#!/usr/bin/env node
// vercel.template.json + backend-origin.json から vercel.json を生成する。
// connect-src が 'self' と本番 backend だけを許可するよう、置換は
// {{BACKEND_ORIGIN}} のみを対象にする。--check は生成せず一致だけ検証する(CI 用)。

import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadBackendOrigin } from './backend-origin.cjs';

const appDir = fileURLToPath(new URL('..', import.meta.url));
const templatePath = join(appDir, 'vercel.template.json');
const outputPath = join(appDir, 'vercel.json');

const rendered = readFileSync(templatePath, 'utf8').replaceAll(
  '{{BACKEND_ORIGIN}}',
  loadBackendOrigin(),
);

if (process.argv.includes('--check')) {
  if (readFileSync(outputPath, 'utf8') !== rendered) {
    console.error('vercel.json is out of date. Run: node scripts/generate-vercel-json.mjs');
    process.exit(1);
  }
  console.log('vercel.json is up to date');
} else {
  writeFileSync(outputPath, rendered);
  console.log('generated vercel.json');
}
