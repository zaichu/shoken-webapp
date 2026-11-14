// Phase 5 OpenCode 画面案の数値生成・検算。
// 元になる値は ../phase5-codex/data/fixtures.json だけに置く(単一データ)。
// HTML は data/templates/*.html を起点に {{KEY}} を埋めて html/ へ生成する
// (手で数字を書かない)。Codex 側 build.mjs と同じ fixtures を読む。
import assert from 'node:assert/strict';
import { readFile, writeFile, readdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

let assertCount = 0;
const eq = (a, b, message) => { assertCount++; assert.equal(a, b, message); };
const ok = (value, message) => { assertCount++; assert.ok(value, message); };

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const tplDir = path.join(root, 'data/templates');
const htmlDir = path.join(root, 'html');
const fixture = JSON.parse(await readFile(path.join(root, '..', 'phase5-codex', 'data/fixtures.json'), 'utf8'));

const yen = value => `¥${Math.round(value).toLocaleString('en-US')}`;
const signedYen = value => `${value > 0 ? '+' : value < 0 ? '-' : ''}¥${Math.abs(Math.round(value)).toLocaleString('en-US')}`;
const number = value => Number(value).toLocaleString('en-US', { maximumFractionDigits: 2 });
const unitYen = value => `¥${Number(value).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
const sum = (rows, key) => rows.reduce((total, row) => total + row[key], 0);
const cls = value => (value > 0 ? 'pos' : value < 0 ? 'neg' : 'zero');
const shortBadge = account => ({ '特定口座': '特定', '一般口座': '一般', NISA: 'NISA成長' }[account] ?? account);
const shortDate = date => date.slice(5);
const jpDate = date => date.replace(/^(\d+)\/(\d+)\/(\d+)$/, (_, y, m, d) => `${y}年${Number(m)}月${Number(d)}日`);
const monthLabel = date => `${date.slice(0, 4)}年${Number(date.slice(5, 7))}月`;
const rangeOf = rows => `${rows.at(-1).date.slice(0, 7).replace('/', '/')} 〜 ${rows[0].date.slice(0, 7)}`.replaceAll('/', '/');
const rangeLabel = rows => {
  const months = [...new Set(rows.map(row => row.date.slice(0, 7)))].sort();
  return `${months[0].replace('/', '/')} 〜 ${months.at(-1).replace('/', '/')}`;
};
const qtyText = row => row.holdingId === 'fund-a'
  ? `${number(row.quantity)}口`
  : `${number(row.quantity)}株`;
const isFundRow = row => row.holdingId === 'fund-a';
const longClass = name => (name.includes('・') || name.length > 11 ? ' class="long"' : '');

// ---- 保有(口座・件数と合計。Codex 側と同じ計算) ----
const holdings = fixture.holdings.map(row => {
  const multiplier = row.type === 'fund' ? 1 / 10000 : 1;
  const value = Math.round(row.quantity * row.currentPrice * multiplier);
  const cost = Math.round(row.quantity * row.costUnitPrice * multiplier);
  return { ...row, value, cost, gain: value - cost };
});
const total = sum(holdings, 'value');
const costTotal = sum(holdings, 'cost');
const gainTotal = total - costTotal;
const stocks = holdings.filter(row => row.type === 'stock');
const funds = holdings.filter(row => row.type === 'fund');
eq(total, 28593148, '評価額合計');
eq(costTotal, 27040660, '取得総額');
eq(holdings.length, 3, '保有件数');
eq(stocks.length, 2, '国内株式件数');
eq(funds.length, 1, '投資信託件数');

// ---- 配当明細行 ----
const dividendTxs = fixture.dividendTransactions.map(row => {
  const holding = holdings.find(h => h.id === row.holdingId);
  ok(holding, `${row.date}: 不明な保有`);
  eq(Math.round(row.quantity * row.unitPrice), row.beforeTax, `${row.date}: 数量×単価`);
  ok(row.tax >= 0 && row.tax <= row.beforeTax, `${row.date}: 税額範囲`);
  return { ...row, holding, code: holding.code, name: holding.name, afterTax: row.beforeTax - row.tax };
});
const dividendBefore = sum(fixture.dividends, 'beforeTax');
const dividendTax = sum(fixture.dividends, 'tax');
const dividendAfter = dividendBefore - dividendTax;
const dividendAnnual = dividendBefore;
for (const month of fixture.dividends) {
  const inMonth = dividendTxs.filter(row => Number(row.date.slice(5, 7)) === month.month);
  ok(inMonth.length >= 1 && inMonth.length <= 4, `${month.month}月行数`);
  eq(sum(inMonth, 'beforeTax'), month.beforeTax, `${month.month}月 税引前`);
  eq(sum(inMonth, 'tax'), month.tax, `${month.month}月 税額`);
}
eq(sum(dividendTxs, 'beforeTax'), dividendBefore, '配当行合計 税引前');
eq(sum(dividendTxs, 'tax'), dividendTax, '配当行合計 税額');
eq(dividendTxs.length, 18, '配当行数');

// ---- 株式売買行・投信解約行 ----
const stockTrades = fixture.stockTrades.map(row => {
  const holding = holdings.find(h => h.id === row.holdingId && h.type === 'stock');
  ok(holding, `${row.date}: 不明な株式`);
  eq(row.sellAmount, row.quantity * row.sellPrice, `${row.date}: 数量×売却単価`);
  eq(row.costAmount, Math.round(row.quantity * holding.costUnitPrice), `${row.date}: 数量×取得単価`);
  eq(row.gain, row.sellAmount - row.costAmount, `${row.date}: 実現損益`);
  return { ...row, code: holding.code, name: holding.name, after: row.gain - row.tax };
});
const fundTrades = fixture.fundTrades.map(row => {
  const holding = holdings.find(h => h.id === row.holdingId && h.type === 'fund');
  ok(holding, `${row.date}: 不明な投信`);
  eq(row.redeemAmount, Math.round(row.quantity * row.redeemNav / 10000), `${row.date}: 口数×基準価額÷10000`);
  eq(row.costNav, holding.costUnitPrice, `${row.date}: 取得単価`);
  eq(row.costAmount, Math.round(row.quantity * row.costNav / 10000), `${row.date}: 口数×取得単価÷10000`);
  eq(row.gain, row.redeemAmount - row.costAmount, `${row.date}: 実現損益`);
  return { ...row, code: holding.code, name: holding.name, after: row.gain - row.tax };
});
const stockGain = sum(stockTrades, 'gain');
const stockTax = sum(stockTrades, 'tax');
const fundGain = sum(fundTrades, 'gain');
const fundTax = sum(fundTrades, 'tax');
eq(stockTrades.length, 3, '株式行数');
eq(fundTrades.length, 2, '投信行数');
// 月は降順、月内は日付昇順(従来の表の並び)
const descMonths = rows => [...rows].sort((x, y) => (x.date < y.date ? 1 : x.date > y.date ? -1 : 0))
  .sort((x, y) => y.date.slice(0, 7).localeCompare(x.date.slice(0, 7)) || x.date.localeCompare(y.date));
const divRows = descMonths(dividendTxs);
const stkRows = [...stockTrades].sort((x, y) => y.date.localeCompare(x.date));
const fndRows = [...fundTrades].sort((x, y) => y.date.localeCompare(x.date));

// ---- 取引明細: 配当金タブ ----
const divMonths = [...new Set(divRows.map(row => row.date.slice(0, 7)))].sort().reverse();
const monthGroup = (ym, rows, kind) => {
  const label = `${ym.slice(0, 4)}年${Number(ym.slice(5))}月`;
  if (kind === 'div') {
    const before = sum(rows, 'beforeTax');
    const tax = sum(rows, 'tax');
    const after = before - tax;
    return {
      pc: `<tr class="grp"><td colspan="6">${label} · ${rows.length}件の合計</td><td class="num">${yen(before)}</td><td class="num neg">-${yen(tax)}</td><td class="num">${yen(after)}</td></tr>`,
      sp: `<li class="mgrp">${label} · ${rows.length}件 <span class="sub">受取 ${yen(after)}</span></li>`,
    };
  }
  const after = sum(rows.map(row => row.gain - row.tax));
  const gain = sum(rows, 'gain');
  const tax = sum(rows, 'tax');
  return {
    pcGain: { gain, tax, after, label, n: rows.length },
    sp: `<li class="mgrp">${label} · ${rows.length}件 <span class="sub">損益 ${signedYen(after)}</span></li>`,
  };
};
// ダイアログ番号は表示順(月降順・月内昇順)に d1 から振る
let dialogNo = 0;
const divPcRows = [];
const divSpRows = [];
const divDialogs = [];
for (const ym of divMonths) {
  const rows = divRows.filter(row => row.date.startsWith(ym));
  const g = monthGroup(ym, rows, 'div');
  divPcRows.push(g.pc);
  divSpRows.push(g.sp);
  for (const row of rows) {
    dialogNo++;
    const id = `rcpt-d${dialogNo}`;
    const selected = dialogNo === divRows.length ? '' : '';
    void selected;
    divPcRows.push(`<tr${dialogNo === 1 ? ' aria-selected="true"' : ''}><td>${row.date}</td><td class="code">${row.code}</td><td${longClass(row.name)}>${row.name}</td><td><span class="badge">${shortBadge(row.account)}</span></td><td class="num">${qtyText(row)}</td><td class="num">${unitYen(row.unitPrice)}</td><td class="num">${yen(row.beforeTax)}</td><td class="num neg">-${yen(row.tax)}</td><td class="num"><a href="#${id}">${yen(row.afterTax)}</a></td></tr>`);
    divSpRows.push(`<li><a class="rowlink" href="#${id}" aria-label="${shortDate(row.date)} ${row.name}の詳細を見る"><div class="r1"><span class="t">${shortDate(row.date)} ${row.name}</span><span class="amt">${yen(row.afterTax)}</span></div><div class="r2"><span>${shortBadge(row.account)} · 税引前 ${yen(row.beforeTax)}</span><span>${isFundRow(row) ? '分配金' : row.code}</span></div></a></li>`);
    divDialogs.push(`<div class="overlay" id="${id}" role="dialog" aria-modal="true" aria-labelledby="${id}-t">
  <div class="dialog">
    <h2 id="${id}-t">行の詳細</h2>
    <p style="margin-top:0"><strong>${row.code} ${row.name}</strong> <span class="badge">${shortBadge(row.account)}</span></p>
    <dl class="kv">
      <dt>入金日</dt><dd class="l">${jpDate(row.date)}</dd>
      <dt>数量</dt><dd>${qtyText(row)}</dd>
      <dt>単価</dt><dd>${unitYen(row.unitPrice)}</dd>
      <dt>配当金(税引前)</dt><dd>${yen(row.beforeTax)}</dd>
      <dt>税額</dt><dd class="neg">-${yen(row.tax)}</dd>
      <dt>税引後</dt><dd>${yen(row.afterTax)}</dd>
    </dl>${isFundRow(row) ? '' : `\n    <p><a class="linkish" href="#">${row.code} の銘柄情報を見る</a></p>`}
    <div class="actions"><a class="btn" href="receipts-opencode.html">閉じる</a></div>
  </div>
</div>`);
  }
}
eq(dialogNo, 18, '配当ダイアログ数');
// 行の詳細(PCの aside)は最新の行
const newest = divRows[0];
const divAside = `<h2>行の詳細</h2>
      <p style="margin-top:0"><strong>${newest.code} ${newest.name}</strong> <span class="badge">${shortBadge(newest.account)}</span></p>
      <dl class="kv">
        <dt>入金日</dt><dd class="l">${jpDate(newest.date)}</dd>
        <dt>数量</dt><dd>${qtyText(newest)}</dd>
        <dt>単価</dt><dd>${unitYen(newest.unitPrice)}</dd>
        <dt>配当金(税引前)</dt><dd>${yen(newest.beforeTax)}</dd>
        <dt>税額</dt><dd class="neg">-${yen(newest.tax)}</dd>
        <dt>税引後</dt><dd>${yen(newest.afterTax)}</dd>
      </dl>
      <p><a class="linkish" href="#">${newest.code} の銘柄情報を見る</a></p>`;

// ---- 取引明細: 国内株式タブ ----
const stkMonths = [...new Set(stkRows.map(row => row.date.slice(0, 7)))].sort().reverse();
const stkPcRows = [];
const stkSpRows = [];
for (const ym of stkMonths) {
  const rows = stkRows.filter(row => row.date.startsWith(ym));
  const gain = sum(rows, 'gain');
  const tax = sum(rows, 'tax');
  const after = gain - tax;
  const g = monthGroup(ym, rows, 'stk');
  stkPcRows.push(`<tr class="grp"><td colspan="8">${g.pcGain.label} · ${rows.length}件の合計</td><td class="num ${cls(gain)}">${signedYen(gain)}</td><td class="num ${cls(tax) === 'pos' ? 'neg' : 'zero'}">${tax === 0 ? '¥0' : `-${yen(tax)}`}</td><td class="num ${cls(after)}">${signedYen(after)}</td></tr>`);
  stkSpRows.push(g.sp);
  for (const [k, row] of rows.entries()) {
    const rAfter = row.gain - row.tax;
    stkPcRows.push(`<tr${stkPcRows.length === 1 ? ' aria-selected="true"' : ''}><td>${row.date}</td><td class="code">${row.code}</td><td${longClass(row.name)}>${row.name}</td><td><span class="badge">${shortBadge(row.account)}</span></td><td>${row.side}</td><td class="num">${number(row.quantity)}株</td><td class="num">${yen(row.sellAmount)}</td><td class="num">${yen(row.costAmount)}</td><td class="num ${cls(row.gain)}">${signedYen(row.gain)}</td><td class="num ${row.tax === 0 ? 'zero' : 'neg'}">${row.tax === 0 ? '¥0' : `-${yen(row.tax)}`}</td><td class="num ${cls(rAfter)}">${signedYen(rAfter)}</td></tr>`);
    stkSpRows.push(`<li><div class="r1"><span class="t">${shortDate(row.date)} ${row.name}</span><span class="amt ${cls(rAfter)}">${signedYen(rAfter)}</span></div><div class="r2"><span>${shortBadge(row.account)} · ${row.side} ${number(row.quantity)}株</span><span>${row.code}</span></div></li>`);
  }
}
const stkNewest = stkRows[0];
const stkAside = `<h2>行の詳細</h2>
      <p style="margin-top:0"><strong>${stkNewest.code} ${stkNewest.name}</strong> <span class="badge">${shortBadge(stkNewest.account)}</span></p>
      <dl class="kv">
        <dt>約定日</dt><dd class="l">${jpDate(stkNewest.date)}</dd>
        <dt>取引</dt><dd class="l">${stkNewest.side} ${number(stkNewest.quantity)}株</dd>
        <dt>売却額</dt><dd>${yen(stkNewest.sellAmount)}</dd>
        <dt>取得額</dt><dd>${yen(stkNewest.costAmount)}</dd>
        <dt>実現損益</dt><dd class="${cls(stkNewest.gain)}">${signedYen(stkNewest.gain)}</dd>
        <dt>税額</dt><dd class="${stkNewest.tax === 0 ? 'zero' : 'neg'}">${stkNewest.tax === 0 ? '¥0' : `-${yen(stkNewest.tax)}`}</dd>
        <dt>税引後</dt><dd class="${cls(stkNewest.gain - stkNewest.tax)}">${signedYen(stkNewest.gain - stkNewest.tax)}</dd>
      </dl>
      <p><a class="linkish" href="#">${stkNewest.code} の銘柄情報を見る</a></p>`;

// ---- 取引明細: 投資信託タブ ----
const fndMonths = [...new Set(fndRows.map(row => row.date.slice(0, 7)))].sort().reverse();
const fndPcRows = [];
const fndSpRows = [];
for (const ym of fndMonths) {
  const rows = fndRows.filter(row => row.date.startsWith(ym));
  const gain = sum(rows, 'gain');
  const tax = sum(rows, 'tax');
  const after = gain - tax;
  const g = monthGroup(ym, rows, 'fnd');
  fndPcRows.push(`<tr class="grp"><td colspan="7">${g.pcGain.label} · ${rows.length}件の合計</td><td class="num ${cls(gain)}">${signedYen(gain)}</td><td class="num neg">-${yen(tax)}</td><td class="num ${cls(after)}">${signedYen(after)}</td></tr>`);
  fndSpRows.push(g.sp);
  for (const [k, row] of rows.entries()) {
    const rAfter = row.gain - row.tax;
    fndPcRows.push(`<tr${fndPcRows.length === 1 ? ' aria-selected="true"' : ''}><td>${row.date}</td><td${longClass(row.name)}>${row.name}</td><td><span class="badge">${shortBadge(row.account)}</span></td><td>${row.side}</td><td class="num">${number(row.quantity)}口</td><td class="num">${yen(row.redeemAmount)}</td><td class="num">${yen(row.costAmount)}</td><td class="num ${cls(row.gain)}">${signedYen(row.gain)}</td><td class="num neg">-${yen(row.tax)}</td><td class="num ${cls(rAfter)}">${signedYen(rAfter)}</td></tr>`);
    fndSpRows.push(`<li><div class="r1"><span class="t">${shortDate(row.date)} ${row.name}</span><span class="amt ${cls(rAfter)}">${signedYen(rAfter)}</span></div><div class="r2"><span>${shortBadge(row.account)} · ${row.side} ${number(row.quantity)}口</span><span>—</span></div></li>`);
  }
}
const fndNewest = fndRows[0];
const fndAside = `<h2>行の詳細</h2>
      <p style="margin-top:0"><strong>${fndNewest.name}</strong> <span class="badge">${shortBadge(fndNewest.account)}</span></p>
      <dl class="kv">
        <dt>約定日</dt><dd class="l">${jpDate(fndNewest.date)}</dd>
        <dt>取引</dt><dd class="l">${fndNewest.side} ${number(fndNewest.quantity)}口</dd>
        <dt>解約額</dt><dd>${yen(fndNewest.redeemAmount)}</dd>
        <dt>取得額</dt><dd>${yen(fndNewest.costAmount)}</dd>
        <dt>実現損益</dt><dd class="${cls(fndNewest.gain)}">${signedYen(fndNewest.gain)}</dd>
        <dt>税額</dt><dd class="neg">-${yen(fndNewest.tax)}</dd>
        <dt>税引後</dt><dd class="${cls(fndNewest.gain - fndNewest.tax)}">${signedYen(fndNewest.gain - fndNewest.tax)}</dd>
      </dl>`;

// 件数(行の数から数える。手で書かない)
const divCount = dividendTxs.length;
const stkCount = stockTrades.length;
const fndCount = fundTrades.length;
const holdStockCount = stocks.length;
const holdFundCount = funds.length;
const holdCount = holdings.length;
const txTotal = divCount + stkCount + fndCount;
const CSV_FUND_OK = 1;
const CSV_FUND_SKIP = 1;
const CSV_PLUNGE_NEXT = 4;
eq(CSV_FUND_OK + CSV_FUND_SKIP, fndCount, '投信の取込+スキップ=行数');
ok(CSV_PLUNGE_NEXT < divCount, '急減ダイアログの取込後件数');

const values = {
  RECEIPTS_HEAD_COUNT: `${divCount}件`,
  DIV_TAB_COUNT: `${divCount}件`, STOCK_TAB_COUNT: `${stkCount}件`, FUND_TAB_COUNT: `${fndCount}件`,
  DIV_BAND_AFTER: yen(dividendAfter), DIV_BAND_BEFORE: yen(dividendBefore), DIV_BAND_TAX: `-${yen(dividendTax)}`,
  DIV_RANGE: rangeLabel(dividendTxs),
  DIV_ROWS_PC: divPcRows.join('\n            '),
  DIV_ROWS_SP: divSpRows.join('\n          '),
  DIV_ASIDE: divAside,
  DIV_DIALOGS: divDialogs.join('\n'),
  STOCK_BAND_AFTER: signedYen(stockGain - stockTax), STOCK_BAND_BEFORE: signedYen(stockGain), STOCK_BAND_TAX: `-${yen(stockTax)}`,
  STOCK_RANGE: rangeLabel(stockTrades),
  STOCK_ROWS_PC: stkPcRows.join('\n            '),
  STOCK_ROWS_SP: stkSpRows.join('\n          '),
  STOCK_ASIDE: stkAside,
  FUND_BAND_AFTER: signedYen(fundGain - fundTax), FUND_BAND_BEFORE: signedYen(fundGain), FUND_BAND_TAX: `-${yen(fundTax)}`,
  FUND_RANGE: rangeLabel(fundTrades),
  FUND_ROWS_PC: fndPcRows.join('\n            '),
  FUND_ROWS_SP: fndSpRows.join('\n          '),
  FUND_ASIDE: fndAside,
  CSV_DIV: String(divCount), CSV_STOCK: String(stkCount), CSV_FUND: String(fndCount),
  CSV_HOLD_STOCK: String(holdStockCount), CSV_HOLD_FUND: String(holdFundCount),
  CSV_HOLD_TOTAL: String(holdCount),
  CSV_FUND_TOTAL: String(fndCount), CSV_FUND_OK: String(CSV_FUND_OK), CSV_FUND_SKIP: String(CSV_FUND_SKIP),
  CSV_PLUNGE_NEXT: String(CSV_PLUNGE_NEXT),
  DIV_RANGE: rangeLabel(dividendTxs), STOCK_RANGE: rangeLabel(stockTrades), FUND_RANGE: rangeLabel(fundTrades),
  HOLD_COUNT: String(holdCount), TX_TOTAL: String(txTotal),
  DIV_TX_COUNT: String(divCount), STOCK_TX_COUNT: String(stkCount), FUND_TX_COUNT: String(fndCount),
  STOCK_COUNT: String(holdStockCount), FUND_COUNT: String(holdFundCount),
  TOTAL: yen(total), COST_TOTAL: yen(costTotal), GAIN_TOTAL: signedYen(gainTotal),
  DIVIDEND_ANNUAL: yen(dividendAnnual),
};

// ---- 生成 ----
const templates = (await readdir(tplDir)).filter(file => file.endsWith('.html'));
ok(templates.length > 0, 'テンプレートなし');
for (const file of templates) {
  let html = await readFile(path.join(tplDir, file), 'utf8');
  html = html.replace(/\{\{([A-Z0-9_]+)\}\}/g, (_, key) => {
    ok(Object.hasOwn(values, key), `${file}: 未定義テンプレート ${key}`);
    return values[key];
  });
  ok(!/\{\{[A-Z0-9_]+\}\}/.test(html), `${file}: 未解決テンプレート`);
  await writeFile(path.join(htmlDir, file), html);
}

// 生成物の検証
const generated = Object.fromEntries(await Promise.all(
  templates.map(async file => [file, await readFile(path.join(htmlDir, file), 'utf8')]),
));
for (const [file, html] of Object.entries(generated)) {
  ok(!/\{\{/.test(html), `${file}: プレースホルダ残存`);
}
const receipts = generated['receipts-opencode.html'];
ok(receipts.includes(values.DIV_BAND_AFTER), 'receipts 集計帯 税引後');
ok(receipts.includes(values.DIV_BAND_BEFORE), 'receipts 集計帯 税引前');
ok(receipts.includes(values.DIV_BAND_TAX), 'receipts 集計帯 税額');
for (const ym of divMonths) {
  const rows = divRows.filter(row => row.date.startsWith(ym));
  const label = `${ym.slice(0, 4)}年${Number(ym.slice(5))}月 · ${rows.length}件の合計`;
  ok(receipts.includes(label), `receipts ${label}`);
  ok(receipts.includes(yen(sum(rows, 'afterTax'))), `receipts ${label}小計`);
}
for (const row of dividendTxs) {
  ok(receipts.includes(row.date) && receipts.includes(yen(row.afterTax)), `receipts 行 ${row.date}`);
}
ok(receipts.includes(values.STOCK_BAND_AFTER) && receipts.includes(values.FUND_BAND_AFTER), 'receipts 株式・投信集計帯');
const csv = generated['csv-opencode.html'];
ok(csv.includes(`資産管理(国内株式)</td><td class="num">${holdStockCount}件`), 'csv 国内株式件数');
ok(csv.includes(`資産管理(投資信託)</td><td class="num">${holdFundCount}件`), 'csv 投資信託件数');
ok(csv.includes(`取引明細(配当金)</td><td class="num">${divCount}件`), 'csv 配当件数');
ok(csv.includes(`取引明細(国内株式)</td><td class="num">${stkCount}件`), 'csv 株式件数');
ok(csv.includes(`取引明細(投資信託)</td><td class="num">${fndCount}件`), 'csv 投信件数');
ok(generated['settings-opencode.html'].includes(`保存済み ${divCount}件`), 'settings 配当件数');
ok(generated['account-opencode.html'].includes(values.DIVIDEND_ANNUAL), 'account 年間配当');
ok(generated['states-opencode.html'].includes(values.TOTAL), 'states 評価額');

// ---- data-check.md ----
const divMonthlyRows = fixture.dividends.map(row => {
  const rows = dividendTxs.filter(tx => Number(tx.date.slice(5, 7)) === row.month);
  return `| 2026年${row.month}月・${rows.length}件 | ${yen(row.beforeTax)} | −${yen(row.tax)} | ${yen(row.beforeTax - row.tax)} | OK |`;
}).join('\n');
const divRowChecks = divRows.map(row => `| ${row.date} | ${row.code} | ${qtyText(row)} | ${unitYen(row.unitPrice)} | ${yen(row.beforeTax)} | ${yen(row.beforeTax)} | ${row.beforeTax}−${row.tax} | ${yen(row.afterTax)} | OK |`).join('\n');
const stkRowChecks = stkRows.map(row => `| ${row.date} | ${row.code} | ${number(row.quantity)}株 | ${yen(row.sellAmount)} | ${yen(row.costAmount)} | ${signedYen(row.gain)} | ${row.tax === 0 ? '¥0' : `−${yen(row.tax)}`} | ${signedYen(row.gain - row.tax)} | OK |`).join('\n');
const fndRowChecks = fndRows.map(row => `| ${row.date} | ${number(row.quantity)}口 | ${yen(row.redeemAmount)} | ${yen(row.costAmount)} | ${signedYen(row.gain)} | −${yen(row.tax)} | ${signedYen(row.gain - row.tax)} | OK |`).join('\n');
const report = `# 取引明細の検算(OpenCode 担当分)\n\n対象: \`docs/design/work/phase5-opencode/html/receipts-opencode.html\`(配当金 ${divCount} 行・国内株式 ${stkCount} 行・投資信託 ${fndCount} 行)。\nこのファイルは \`node docs/design/work/phase5-opencode/data/build-opencode.mjs\` が生成する。\n元データは \`docs/design/work/phase5-codex/data/fixtures.json\`(Codex 側と同一)。全 ${assertCount} assert 通過(不一致 0)。\n確認式: 数量×単価=金額、金額−税額=税引後、行の合計=月小計、月小計の合計=年合計(集計帯)。\n\n## 配当金タブ(${divCount} 行)\n\n| 入金日 | コード | 数量 | 単価 | 数量×単価 | 金額 | 金額−税額 | 税引後 | 判定 |\n|---|---|---:|---:|---:|---:|---:|---:|:---:|\n${divRowChecks}\n\n## 月小計・年合計(配当金)\n\n| 月 | 金額 | 税額 | 税引後 | 判定 |\n|---|---|---:|---:|:---:|\n${divMonthlyRows}\n| 年合計(集計帯) | ${yen(dividendBefore)} | −${yen(dividendTax)} | ${yen(dividendAfter)} | OK |\n\n## 国内株式タブ(${stkCount} 行)\n\n売却額−取得額=実現損益、実現損益−税額=税引後の順に確認。損失行は税額 ¥0。\n\n| 約定日 | コード | 数量 | 売却額 | 取得額 | 実現損益 | 税額 | 税引後 | 判定 |\n|---|---|---:|---:|---:|---:|---:|---:|:---:|\n${stkRowChecks}\n\n年合計: 税引前 ${signedYen(stockGain)}、税額 −${yen(stockTax)}、税引後 ${signedYen(stockGain - stockTax)}。いずれも行の合計と一致。\n\n## 投資信託タブ(${fndCount} 行)\n\n| 約定日 | 数量 | 解約額 | 取得額 | 実現損益 | 税額 | 税引後 | 判定 |\n|---|---|---:|---:|---:|---:|---:|---:|:---:|\n${fndRowChecks}\n\n年合計: 税引前 ${signedYen(fundGain)}、税額 −${yen(fundTax)}、税引後 ${signedYen(fundGain - fundTax)}。いずれも行の合計と一致。\n\n## CSV 管理・設定・その他画面の件数\n\n- 資産管理(国内株式)=保有の国内株式 ${holdStockCount}件、資産管理(投資信託)=保有の投資信託 ${holdFundCount}件\n- 取引明細(配当金)=配当行数 ${divCount}件、取引明細(国内株式)=${stkCount}件、取引明細(投資信託)=${fndCount}件\n- 設定のデータ項目: 資産残高(${holdCount}件)・売買履歴(合計 ${txTotal}件)、危険な操作の各項目に保存済み件数を表示\n- 状態ボードの端数データ表は桁あふれ等の見本のため意図的に架空の大きな値(生成対象外)。実データの集計帯(評価額 ${yen(total)})は fixtures から生成\n`;
await writeFile(path.join(root, 'data-check.md'), report);
console.log(`receipts: 配当 ${divCount}行 / 株式 ${stkCount}行 / 投信 ${fndCount}行; CSV: 国内株式 ${holdStockCount} / 投信 ${holdFundCount} / 配当 ${divCount} / 株式 ${stkCount} / 投信 ${fndCount}`);
console.log(`asserts: ${assertCount} passed, 0 failed`);
