#!/usr/bin/env node
// dist 直下の `_headers` / `_redirects`(Cloudflare Pages 形式)を最小実装で再現する静的配信サーバ。
// ローカルで配信設定(SPA fallback・CSP・Cache-Control・WASM MIME)を検証するためのもの。

import { createReadStream, existsSync, readFileSync, statSync } from 'node:fs';
import { createServer } from 'node:http';
import { extname, isAbsolute, join, normalize, relative, resolve } from 'node:path';

const root = process.argv[2] ?? 'dist';
const port = Number(process.argv[3] ?? process.env.PORT ?? 8190);
const rootResolved = resolve(root);

// このリポジトリの _headers で使うパターンだけを正規表現に変換する
//   /*         -> 全パス
//   /:name.ext -> ルート直下の 1 セグメントのみ
//   それ以外    -> 完全一致
function patternToRegex(pattern) {
  if (pattern === '/*') return /^\/.*/;
  const single = pattern.match(/^\/:[A-Za-z]+(\..+)$/);
  if (single) {
    const suffix = single[1].replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    return new RegExp(`^/[^/]+${suffix}$`);
  }
  const literal = pattern.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  return new RegExp(`^${literal}$`);
}

// Pages の _headers は「一致した全ルールのヘッダーを順に適用」し、同名ヘッダーは
// カンマ連結される。`! Name` はそのルール内で継承分を外す指定(Detach)
function loadHeaderRules(file) {
  if (!existsSync(file)) return [];
  const rules = [];
  let current = null;
  for (const line of readFileSync(file, 'utf8').split('\n')) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;
    if (/^\s/.test(line)) {
      if (!current) continue;
      if (trimmed.startsWith('!')) {
        current.ops.push({ detach: trimmed.slice(1).trim().toLowerCase() });
      } else {
        const idx = trimmed.indexOf(':');
        current.ops.push({ key: trimmed.slice(0, idx).trim(), value: trimmed.slice(idx + 1).trim() });
      }
    } else {
      current = { pattern: patternToRegex(trimmed), ops: [] };
      rules.push(current);
    }
  }
  return rules;
}

function headersFor(rules, pathname) {
  const headers = new Map();
  for (const rule of rules) {
    if (!rule.pattern.test(pathname)) continue;
    for (const op of rule.ops) {
      if (op.detach) {
        headers.delete(op.detach);
      } else {
        const lower = op.key.toLowerCase();
        const existing = headers.get(lower);
        headers.set(lower, existing ? `${existing.value}, ${op.value}` : { name: op.key, value: op.value });
      }
    }
  }
  return Object.fromEntries([...headers.values()].map((h) => [h.name, h.value]));
}

const headerRules = loadHeaderRules(join(rootResolved, '_headers'));

// SPA fallback は _redirects の `/* <target> 200` が決める。無ければデフォルトを補わない
function loadSpaDestination(file) {
  if (!existsSync(file)) return undefined;
  for (const line of readFileSync(file, 'utf8').split('\n')) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;
    const [pattern, destination, status] = trimmed.split(/\s+/);
    if (pattern === '/*' && status === '200') return destination;
  }
  return undefined;
}
const spaDestination = loadSpaDestination(join(rootResolved, '_redirects'));

// .wasm はあえて含めない。application/wasm は _headers の規則から来る
const MIME_TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.ico': 'image/x-icon',
  '.map': 'application/json',
  '.txt': 'text/plain; charset=utf-8',
};

createServer((req, res) => {
  let pathname;
  try {
    pathname = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
  } catch {
    res.writeHead(400).end('bad request');
    return;
  }
  const safePath = normalize(pathname).replace(/^(\.\.[/\\])+/, '');
  let filePath = resolve(rootResolved, `.${safePath}`);
  const fromRoot = relative(rootResolved, filePath);
  if (fromRoot.startsWith('..') || isAbsolute(fromRoot)) {
    res.writeHead(404).end('not found');
    return;
  }

  if (!existsSync(filePath) || !statSync(filePath).isFile()) {
    // '/' への index.html 割当は静的ホストの既定動作、それ以外は rewrite 設定に従う
    filePath = pathname === '/' ? join(root, 'index.html') : spaDestination ? join(root, spaDestination) : '';
  }
  if (!filePath || !existsSync(filePath)) {
    res.writeHead(404).end('not found');
    return;
  }

  const headers = { 'Content-Type': MIME_TYPES[extname(filePath)] ?? 'application/octet-stream' };
  Object.assign(headers, headersFor(headerRules, pathname));

  res.writeHead(200, headers);
  createReadStream(filePath).pipe(res);
}).listen(port, '127.0.0.1', () => {
  console.log(`serving ${root} at http://127.0.0.1:${port} (SPA fallback -> ${spaDestination})`);
});
