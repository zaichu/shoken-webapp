#!/usr/bin/env node
// vercel.json の rewrites / headers を最小実装で再現する静的配信サーバ。
// ローカルで配信設定(SPA fallback・CSP・Cache-Control・WASM MIME)を検証するためのもの。

import { createReadStream, existsSync, readFileSync, statSync } from 'node:fs';
import { createServer } from 'node:http';
import { extname, isAbsolute, join, normalize, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = fileURLToPath(new URL('.', import.meta.url));
const root = process.argv[2] ?? 'dist';
const port = Number(process.argv[3] ?? process.env.PORT ?? 8190);
const config = JSON.parse(readFileSync(join(scriptDir, '..', 'vercel.json'), 'utf8'));

// この vercel.json で使う構文だけを正規表現に変換する
//   /(.*)        -> 全パス
//   /:name*.ext  -> 末尾が拡張子に一致(ネストしたパスも含む)
//   /:name.ext   -> ルート直下の 1 セグメントのみ
//   それ以外      -> 完全一致
function sourceToRegex(source) {
  if (source === '/(.*)') return /^\/.*/;
  const nested = source.match(/^\/:[A-Za-z]+\*(\..+)$/);
  if (nested) {
    const suffix = nested[1].replace(/[.+?^${}()|[\]\\]/g, '\\$&');
    return new RegExp(`^/.*${suffix}$`);
  }
  const single = source.match(/^\/:[A-Za-z]+(\..+)$/);
  if (single) {
    const suffix = single[1].replace(/[.+?^${}()|[\]\\]/g, '\\$&');
    return new RegExp(`^/[^/]+${suffix}$`);
  }
  const literal = source.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  return new RegExp(`^${literal}$`);
}

const headerRules = (config.headers ?? []).map((rule) => ({
  pattern: sourceToRegex(rule.source),
  headers: rule.headers,
}));

// SPA fallback は vercel.json の rewrite が決める。無ければデフォルトを補わない
const spaDestination = (config.rewrites ?? []).find(
  (rule) => rule.source === '/(.*)',
)?.destination;

// .wasm はあえて含めない。application/wasm は vercel.json の headers 規則から来る
const MIME_TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.ico': 'image/x-icon',
  '.map': 'application/json; charset=utf-8',
  '.txt': 'text/plain; charset=utf-8',
};

const rootResolved = resolve(root);

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
  for (const rule of headerRules) {
    if (rule.pattern.test(pathname)) {
      for (const { key, value } of rule.headers) headers[key] = value;
    }
  }

  res.writeHead(200, headers);
  createReadStream(filePath).pipe(res);
}).listen(port, '127.0.0.1', () => {
  console.log(`serving ${root} at http://127.0.0.1:${port} (SPA fallback -> ${spaDestination})`);
});
