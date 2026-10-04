#!/usr/bin/env node
// vercel.json の CSP connect-src から本番 backend の URL を読む唯一の入口。
// シェル・スクリプト・spec はすべてここ経由で受け取る。

const { readFileSync } = require('node:fs');
const { join } = require('node:path');

// CSP の connect-src は 'self' の直後に本番 origin を1つだけ置く決まり。
// 末尾スラッシュ・空白を含む値は埋め込み先とずれるため弾く
function loadBackendOrigin() {
  const vercelJsonPath = join(__dirname, '..', 'vercel.json');
  const config = JSON.parse(readFileSync(vercelJsonPath, 'utf8'));
  const csp = (config.headers ?? [])
    .find((rule) => rule.source === '/(.*)')
    ?.headers.find((header) => header.key === 'Content-Security-Policy')?.value;
  const origin = csp?.match(/connect-src 'self' (\S+?);/)?.[1];
  if (!origin || !/^https:\/\/\S+$/.test(origin) || origin.endsWith('/')) {
    throw new Error(
      'vercel.json: CSP connect-src must contain exactly one https origin without a trailing slash',
    );
  }
  return origin;
}

module.exports = { loadBackendOrigin };

if (require.main === module) {
  console.log(loadBackendOrigin());
}
