#!/usr/bin/env node
// _headers の CSP connect-src から本番 backend の URL を読む唯一の入口。
// シェル・スクリプト・spec はすべてここ経由で受け取る。

const { readFileSync } = require('node:fs');
const { join } = require('node:path');

// CSP の connect-src は 'self' の直後に本番 origin を1つだけ置く決まり。
// 末尾スラッシュ・空白を含む値は埋め込み先とずれるため弾く
function loadBackendOrigin() {
  const headersPath = join(__dirname, '..', '_headers');
  const csp = readFileSync(headersPath, 'utf8')
    .split('\n')
    .map((line) => line.trim())
    .find((line) => line.startsWith('Content-Security-Policy:'))
    ?.slice('Content-Security-Policy:'.length)
    .trim();
  const origin = csp?.match(/connect-src 'self' (\S+?);/)?.[1];
  if (!origin || !/^https:\/\/\S+$/.test(origin) || origin.endsWith('/')) {
    throw new Error(
      '_headers: CSP connect-src must contain exactly one https origin without a trailing slash',
    );
  }
  return origin;
}

module.exports = { loadBackendOrigin };

if (require.main === module) {
  console.log(loadBackendOrigin());
}
