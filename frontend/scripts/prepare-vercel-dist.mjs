#!/usr/bin/env node
// trunk が生成する nonce 付きインライン init script を外部ファイル化する。
// 静的配信ではレスポンスごとの nonce を発行できないため、
// 外部化して script-src 'self' で通せる形にする。
// あわせて shoken-api-origin の meta に backend の URL を埋める(第2引数で差し替え可能。
// E2E では '' を渡して同一オリジンに戻し、モック外の通信が本番に出ないようにする)。
// session-probe.js は内容ハッシュ付きの名前に差し替える
// (固定名のままだと vercel.json の immutable キャッシュでデプロイ後も古いプローブが使われる)。
// _headers と _redirects を dist 直下に生成して Cloudflare Pages 相当の配信動作を再現する。

import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const { loadBackendOrigin } = require('./backend-origin.cjs');

const dist = process.argv[2] ?? 'dist';
const indexPath = join(dist, 'index.html');
let html = readFileSync(indexPath, 'utf8');

const apiOrigin = process.argv[3] ?? loadBackendOrigin();
const metaRe = /<meta\s+name="shoken-api-origin"\s+content="[^"]*"\s*\/?\s*>/;
if (!metaRe.test(html)) {
  console.error('shoken-api-origin meta not found in index.html');
  process.exit(1);
}
// 再実行(serve-dist 前の2度目)でも同じ値に置き直すだけで通る
html = html.replace(metaRe, `<meta name="shoken-api-origin" content="${apiOrigin}" />`);

const probeBody = readFileSync(join(dist, 'session-probe.js'));
const probeHash = createHash('sha256').update(probeBody).digest('hex').slice(0, 8);
const probeName = `session-probe.${probeHash}.js`;
html = html.replace(/src="\/session-probe(?:\.[0-9a-f]+)?\.js"/, `src="/${probeName}"`);
writeFileSync(join(dist, probeName), probeBody);
writeFileSync(indexPath, html);

if (html.includes('<script type="module" src="/init-')) {
  console.log('init script is already externalized');
  process.exit(0);
}

const match = html.match(/<script type="module" nonce="[^"]*">([\s\S]*?)<\/script>/);
if (!match) {
  console.error(`inline init script not found in ${indexPath}`);
  process.exit(1);
}

const body = match[1];
const hash = createHash('sha256').update(body).digest('hex').slice(0, 16);
const sri = createHash('sha384').update(body).digest('base64');
const initName = `init-${hash}.js`;

writeFileSync(join(dist, initName), body);
writeFileSync(
  indexPath,
  html.replace(
    match[0],
    `<script type="module" src="/${initName}" integrity="sha384-${sri}" crossorigin="anonymous"></script>`
  )
);
console.log(`externalized init script -> ${initName}`);

function sourceToCloudflarePattern(source) {
  if (source === '/(.*)') return '/*';
  if (source === '/:name.wasm') return '/*.wasm';
  if (source === '/:name.js') return '/*.js';
  if (source === '/:name.css') return '/*.css';
  return source;
}

function generateCloudflareConfigFiles(distDir) {
  const vercelPath = join(import.meta.dirname, '..', 'vercel.json');
  const vercel = JSON.parse(readFileSync(vercelPath, 'utf8'));

  // _headers の生成
  const headerRules = (vercel.headers ?? []).map((rule) => {
    const pattern = sourceToCloudflarePattern(rule.source);
    const headerLines = rule.headers.map((h) => `  ${h.key}: ${h.value}`).join('\n');
    return `${pattern}\n${headerLines}`;
  });

  // _redirects の生成 (rewrites から)
  const redirectRules = (vercel.rewrites ?? []).map((rule) => {
    if (rule.source === '/(.*)' && rule.destination === '/index.html') {
      return '/* /index.html 200';
    }
    return null;
  }).filter(Boolean);

  const headersContent = ['# Headers from vercel.json', ...headerRules, ''].join('\n') + '\n';
  const redirectsContent = ['# Redirects from vercel.json', ...redirectRules, ''].join('\n') + '\n';

  writeFileSync(join(distDir, '_headers'), headersContent);
  writeFileSync(join(distDir, '_redirects'), redirectsContent);
  console.log('generated _headers and _redirects for Cloudflare Pages');
}

generateCloudflareConfigFiles(dist);
