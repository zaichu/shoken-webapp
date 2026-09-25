#!/usr/bin/env node
// connect-src を 'self' と本番 backend だけに閉じるため、置換対象は {{BACKEND_ORIGIN}} のみ

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
