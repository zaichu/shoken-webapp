#!/usr/bin/env node
// trunk が生成する nonce 付きインライン init script を外部ファイル化する。
// 静的配信ではレスポンスごとの nonce を発行できないため、
// 外部化して script-src 'self' で通せる形にする。

import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const dist = process.argv[2] ?? 'dist';
const indexPath = join(dist, 'index.html');
const html = readFileSync(indexPath, 'utf8');

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
