#!/usr/bin/env node
// latest/ の README.md を作る。画面名・何が写っているかの一覧と、撮った main の
// コミット・日時を書く(Muse に見せて横断レビューをもらうための目録)。
// screens.mjs から機械生成するため、画面の増減に追従する。
//
// 使い:
//   node scripts/ui-shots/latest-readme.mjs --dir <latest のディレクトリ> --commit <main の SHA> --date <ISO日時>

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { SCREENS } from './screens.mjs';

function parseArgs(argv) {
  const options = {};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--dir') {
      options.dir = argv[(index += 1)];
    } else if (arg === '--commit') {
      options.commit = argv[(index += 1)];
    } else if (arg === '--date') {
      options.date = argv[(index += 1)];
    } else {
      throw new Error(`不明な引数: ${arg}`);
    }
  }
  if (!options.dir || !options.commit || !options.date) {
    throw new Error('--dir --commit --date は必須です');
  }
  return options;
}

const options = parseArgs(process.argv.slice(2));

// ファイル名の約束: <画面キー>-<幅>x<高さ>-<light|dark>[-scrolled].png
// 例: asset-stock-1280x800-dark.png / asset-stock-1280x800-dark-scrolled.png
// 幅は 390・1280・1920、テーマは light・dark。表・一覧がある画面はスクロール後(PC だけ)も撮る
const lines = [
  '# 最新の全画面スクリーンショット',
  '',
  `撮った main のコミット: ${options.commit}`,
  `撮った日時(UTC): ${options.date}`,
  '',
  'ファイル名の約束: `<画面キー>-<幅>x<高さ>-<light|dark>[-scrolled].png`',
  '(例: `asset-stock-1280x800-dark.png`。`-scrolled` は表・一覧のスクロール後で PC 幅だけ)',
  '幅は 390・1280・1920、テーマは light・dark。データはすべて架空のモック。',
  '',
  '| 画面キー | 画面名 | パス | スクロール後あり |',
  '| --- | --- | --- | --- |',
];
for (const screen of SCREENS) {
  lines.push(`| ${screen.key} | ${screen.label} | ${screen.path} | ${screen.scroll ? 'あり' : 'なし'} |`);
}
lines.push('');

writeFileSync(join(options.dir, 'README.md'), `${lines.join('\n')}`);
console.log(`README.md を書きました: ${options.dir}`);
