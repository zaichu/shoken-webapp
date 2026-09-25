#!/usr/bin/env node
// backend-origin.json を読む唯一の入口。シェル・スクリプト・spec は
// すべてここ経由で本番 backend の URL を受け取る。

const { readFileSync } = require('node:fs');
const { join } = require('node:path');

// connect-src の origin 形式に合わせ、末尾スラッシュ・空白を含む値は埋め込み先とずれるため弾く
function loadBackendOrigin() {
  const config = JSON.parse(
    readFileSync(join(__dirname, '..', 'backend-origin.json'), 'utf8'),
  );
  const origin = config.backendOrigin;
  if (typeof origin !== 'string' || !/^https:\/\/\S+$/.test(origin) || origin.endsWith('/')) {
    throw new Error(
      'backend-origin.json: backendOrigin must be an https URL without a trailing slash',
    );
  }
  return origin;
}

module.exports = { loadBackendOrigin };

if (require.main === module) {
  console.log(loadBackendOrigin());
}
