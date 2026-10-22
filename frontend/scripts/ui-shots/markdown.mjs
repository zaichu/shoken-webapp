#!/usr/bin/env node
// PR 本文に貼る Markdown(変更前後の表)を作る。
// screenshots ブランチの pr<番号>/<before|after>/ にある画像を見て、
// ある側だけをリンクする。無い側は「—」にする。
//
// 使い:
//   node scripts/ui-shots/markdown.mjs --pr 1484 --sha <commit> \
//     --repo zaichu/shoken-webapp --dir <screenshots の worktree>

import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { SCREENS, THEMES, VIEWPORTS } from './screens.mjs';

function parseArgs(argv) {
  const options = {};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--pr') {
      options.pr = argv[(index += 1)];
    } else if (arg === '--sha') {
      options.sha = argv[(index += 1)];
    } else if (arg === '--repo') {
      options.repo = argv[(index += 1)];
    } else if (arg === '--dir') {
      options.dir = argv[(index += 1)];
    } else {
      throw new Error(`不明な引数: ${arg}`);
    }
  }
  if (!options.pr || !options.sha || !options.repo || !options.dir) {
    throw new Error('--pr --sha --repo --dir は必須です');
  }
  return options;
}

const options = parseArgs(process.argv.slice(2));
const base = `pr${options.pr}`;

function imageUrl(side, file) {
  return `https://github.com/${options.repo}/blob/${options.sha}/${base}/${side}/${file}?raw=true`;
}

const rows = [];
for (const screen of SCREENS) {
  for (const viewport of VIEWPORTS) {
    for (const theme of THEMES) {
      const name = `${screen.key}-${viewport.width}x${viewport.height}-${theme}`;
      rows.push({
        screen: screen.label,
        size: `${viewport.label} ${theme === 'light' ? 'ライト' : 'ダーク'}`,
        file: `${name}.png`,
      });
      if (screen.scroll && viewport.width > 768) {
        rows.push({
          screen: `${screen.label}(スクロール後)`,
          size: `${viewport.label} ${theme === 'light' ? 'ライト' : 'ダーク'}`,
          file: `${name}-scrolled.png`,
        });
      }
    }
  }
}

const lines = ['| 画面 | 幅・配色 | 変更前 | 変更後 |', '| --- | --- | --- | --- |'];
let count = 0;
for (const row of rows) {
  const before = existsSync(join(options.dir, base, 'before', row.file));
  const after = existsSync(join(options.dir, base, 'after', row.file));
  if (!before && !after) {
    continue;
  }
  count += 1;
  const cell = (side, exists) =>
    exists ? `[![${row.screen}](${imageUrl(side, row.file)})](${imageUrl(side, row.file)})` : '—';
  lines.push(`| ${row.screen} | ${row.size} | ${cell('before', before)} | ${cell('after', after)} |`);
}

console.log(`## 画面(${count} 枚)`);
console.log('');
console.log(lines.join('\n'));
