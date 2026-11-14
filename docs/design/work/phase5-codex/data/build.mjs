// Phase 5 Codex 画面案の数値生成・検算。
// 元になる値は data/fixtures.json だけに置く。HTML は data/templates/*.html を
// 起点に {{KEY}} を埋めて html/ へ生成する(手で数字を書かない)。
// 計算規則は実装と合わせる: 株式=数量×現在値、投信=口数×基準価額÷10000、
// 前日比率=前日比÷前日の評価額 (backend asset_balance/fund_balance 由来)。
import assert from 'node:assert/strict';
import { readFile, writeFile, readdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const tplDir = path.join(root, 'data/templates');
const htmlDir = path.join(root, 'html');
const fixture = JSON.parse(await readFile(path.join(root, 'data/fixtures.json'), 'utf8'));

const yen = value => `¥${Math.round(value).toLocaleString('en-US')}`;
const signedYen = value => `${value > 0 ? '+' : value < 0 ? '-' : ''}¥${Math.abs(Math.round(value)).toLocaleString('en-US')}`;
const number = value => Number(value).toLocaleString('en-US', { maximumFractionDigits: 2 });
const pct = (value, digits = 1) => `${value.toFixed(digits)}%`;
const sum = (rows, key) => rows.reduce((total, row) => total + row[key], 0);

// 行ごとの派生値: 評価額・取得額・損益・損益率(=損益÷取得総額)・前日評価額
const holdings = fixture.holdings.map(row => {
  assert.ok(row.type === 'stock' || row.type === 'fund', `${row.id}: 不明な資産種別`);
  assert.ok(row.quantity > 0 && row.currentPrice > 0 && row.costUnitPrice > 0, `${row.id}: 数量・価格`);
  const multiplier = row.type === 'fund' ? 1 / 10000 : 1;
  const value = Math.round(row.quantity * row.currentPrice * multiplier);
  const cost = Math.round(row.quantity * row.costUnitPrice * multiplier);
  const previousValue = row.type === 'fund'
    ? Math.round(row.quantity * row.previousNav * multiplier)
    : row.previousValue;
  return { ...row, value, cost, gain: value - cost, gainRate: (value - cost) / cost * 100, previousValue };
});
const total = sum(holdings, 'value');
const costTotal = sum(holdings, 'cost');
const gainTotal = total - costTotal;
const previousValue = sum(holdings, 'previousValue');
const dailyChange = total - previousValue;
const dailyRate = dailyChange / previousValue * 100;
const stocks = holdings.filter(row => row.type === 'stock');
const funds = holdings.filter(row => row.type === 'fund');
const stockValue = sum(stocks, 'value');
const fundValue = sum(funds, 'value');
const dividendBefore = sum(fixture.dividends, 'beforeTax');
const dividendTax = sum(fixture.dividends, 'tax');
const dividendAfter = dividendBefore - dividendTax;
const annualByHolding = Object.fromEntries(stocks.map(row => [row.id, row.annualDividendPerShare * row.quantity]));
const dividendAnnual = dividendBefore;
const recentDividends = fixture.recentDividends.map(row => ({
  ...row, holding: holdings.find(holding => holding.id === row.holdingId), afterTax: row.beforeTax - row.tax,
}));
const percentage = (value, base, digits = 1) => `${(value / base * 100).toFixed(digits)}%`;
const byId = Object.fromEntries(holdings.map(row => [row.id, row]));
const a = byId['stock-a'];
const b = byId['stock-b'];
const fund = byId['fund-a'];
const fundNavChange = fund.currentPrice - fund.previousNav;
const stockADayChange = a.currentPrice - a.previousValue / a.quantity;
const specificValue = holdings.filter(row => row.account === '特定口座').reduce((t, row) => t + row.value, 0);
const nisaValue = holdings.filter(row => row.account === 'NISA').reduce((t, row) => t + row.value, 0);
const novMonthly = fixture.dividends.find(row => row.month === 11);
const novRecentBefore = recentDividends.filter(row => row.month === 11).reduce((t, row) => t + row.beforeTax, 0);
const values = {
  TOTAL: yen(total), COST_TOTAL: yen(costTotal), GAIN_TOTAL: signedYen(gainTotal),
  GAIN_RATE_TOTAL: pct(gainTotal / costTotal * 100, 2),
  STOCK_VALUE: yen(stockValue), FUND_VALUE: yen(fundValue),
  DAILY_CHANGE: signedYen(dailyChange), DAILY_RATE: `+${pct(dailyRate, 1)}`,
  DAILY_RATE_EXACT: `${dailyRate.toFixed(4)}%`, PREVIOUS_VALUE: yen(previousValue),
  STOCK_A_VALUE: yen(a.value), STOCK_A_COST: yen(a.cost), STOCK_A_GAIN: signedYen(a.gain),
  STOCK_A_QUANTITY: number(a.quantity), STOCK_A_PRICE: yen(a.currentPrice),
  STOCK_B_VALUE: yen(b.value), STOCK_B_COST: yen(b.cost), STOCK_B_GAIN: signedYen(b.gain),
  STOCK_B_QUANTITY: number(b.quantity),
  FUND_GAIN: signedYen(fund.gain), FUND_NAV: yen(fund.currentPrice),
  FUND_NAV_ROUND: yen(fund.currentPrice), FUND_NAV_CHANGE: signedYen(fundNavChange),
  FUND_QUANTITY: number(fund.quantity),
  DIVIDEND_ANNUAL: yen(dividendAnnual), DIVIDEND_BEFORE: yen(dividendBefore),
  DIVIDEND_TAX: yen(dividendTax), DIVIDEND_AFTER: yen(dividendAfter),
  STOCK_A_DIVIDEND: yen(annualByHolding[a.id]), STOCK_B_DIVIDEND: yen(annualByHolding[b.id]),
  STOCK_A_DIVIDEND_PER_SHARE: yen(a.annualDividendPerShare),
  STOCK_A_YIELD: pct(a.annualDividendPerShare / a.currentPrice * 100, 2),
  STOCK_A_DAY_CHANGE: signedYen(stockADayChange),
  STOCK_A_DAY_RATE: `+${pct(stockADayChange / (a.previousValue / a.quantity) * 100, 1)}`,
  STOCK_CLASS_SHARE: percentage(stockValue, total), FUND_CLASS_SHARE: percentage(fundValue, total),
  DONUT_PCT: (stockValue / total * 100).toFixed(1),
  STOCK_A_CLASS_SHARE: percentage(a.value, stockValue), STOCK_B_CLASS_SHARE: percentage(b.value, stockValue),
  STOCK_A_DIVIDEND_SHARE: percentage(annualByHolding[a.id], dividendAnnual),
  STOCK_B_DIVIDEND_SHARE: percentage(annualByHolding[b.id], dividendAnnual),
  COUNT: String(holdings.length), STOCK_COUNT: String(stocks.length),
  FUND_COUNT: String(funds.length), DIVIDEND_COUNT: String(stocks.length),
  RECENT_A_BEFORE: yen(recentDividends[0].beforeTax), RECENT_A_TAX_SIGNED: `-${yen(recentDividends[0].tax)}`,
  RECENT_A_AFTER: yen(recentDividends[0].afterTax),
  RECENT_B_AFTER: yen(recentDividends[1].afterTax),
  ACCOUNT_SPECIFIC: yen(specificValue), ACCOUNT_NISA: yen(nisaValue),
  COST_PARTS: holdings.map(row => yen(row.cost)).join(' + '),
  MONTHLY_CHART: fixture.dividends.map(row => {
    const max = Math.max(...fixture.dividends.map(r => r.beforeTax));
    return `<div class="month"><i style="height:${Math.max(2, Math.round(row.beforeTax / max * 100))}%"></i><small>${row.month}月</small></div>`;
  }).join(''),
  STOCK_ROWS: stocks.map((row, index) => `<div class="row"><span>${index + 1}　${row.code} ${row.name}</span><b class="numeric">${yen(row.value)}　${percentage(row.value, stockValue)}</b></div><div class="bar"><i style="width:${percentage(row.value, stockValue)}"></i></div>`).join(''),
  DIVIDEND_ROWS: stocks.map((row, index) => `<div class="legend"><i class="swatch${index ? ' blue' : ''}"></i>${row.code} ${row.name} <b class="numeric" style="margin-left:auto">${yen(annualByHolding[row.id])}　${percentage(annualByHolding[row.id], dividendAnnual)}</b></div><div class="bar"><i style="width:${percentage(annualByHolding[row.id], dividendAnnual)}${index ? ';background:var(--chart-2)' : ''}"></i></div>`).join(''),
  RECENT_ROWS: recentDividends.map(row => `<a class="row" href="dialogs.html"><span>${row.holding.code}　${row.holding.name}<br><small>${row.date} ・ ${row.holding.account}</small></span><b class="numeric">${yen(row.afterTax)}</b></a>`).join(''),
};

// ---- 検算: 不整合があれば失敗する ----
// Phase 3 の集計値に合うこと
assert.equal(total, 28593148, '評価額合計');
assert.equal(costTotal, 27040660, '取得総額');
assert.equal(gainTotal, 1552488, '評価損益');
assert.equal(gainTotal, total - costTotal, '合計損益');
assert.equal(stockValue, 9149807, '国内株式評価額');
assert.equal(fundValue, 19443341, '投資信託評価額');
assert.equal(total, stockValue + fundValue, '分類別評価額');
assert.equal(total, specificValue + nisaValue, '口座別評価額');
// 行の数量×単価=評価額(投信は口数×基準価額÷10000)
assert.equal(a.value, a.quantity * a.currentPrice, '9901 数量×現在値');
assert.equal(a.value, 1200000, '9901 評価額');
assert.equal(b.value, Math.round(b.quantity * b.currentPrice), '9902 数量×現在値');
assert.equal(fund.value, Math.round(fund.quantity * fund.currentPrice / 10000), '投信 口数×基準価額÷10000');
assert.equal(sum(stocks, 'value'), stockValue, '国内株式内訳');
assert.equal(sum(funds, 'value'), fundValue, '投資信託内訳');
// 前日比の率=前日比÷前日の評価額
assert.equal(dailyChange, 154488, '前日比額');
assert.equal(Math.round(dailyRate * 10) / 10, 0.5, '前日比率');
assert.equal(previousValue + dailyChange, total, '前日評価額+前日比=評価額');
// 配当: 月別合計=年合計=銘柄別合計、税引前−税額=税引後、月別は個別受取の合計以上
assert.equal(dividendAnnual, dividendBefore, '月別配当合計と年合計');
assert.equal(dividendAfter, dividendBefore - dividendTax, '配当税引後');
assert.equal(dividendAnnual, Object.values(annualByHolding).reduce((t, v) => t + v, 0), '年間配当と銘柄別合計');
assert.equal(novMonthly.beforeTax, novRecentBefore, '11月合計と11月個別の合計');
assert.ok(fixture.dividends.every(row => row.beforeTax >= row.tax), '月別税引後非負');
assert.ok(recentDividends.every(row => row.holding && row.afterTax === row.beforeTax - row.tax), '最近の配当内訳');
// 件数・ID重複・スマホ/PC共通(同じ fixtures から同じ数量・金額)
assert.equal(holdings.length, 3, '保有件数');
assert.equal(new Set(holdings.map(row => row.id)).size, holdings.length, '保有ID重複');
assert.equal(fixture.dividends.length, 12, '配当月数');

// ---- 生成: テンプレートに埋める ----
const templates = (await readdir(tplDir)).filter(file => file.endsWith('.html'));
assert.ok(templates.length > 0, 'テンプレートなし');
for (const file of templates) {
  let html = await readFile(path.join(tplDir, file), 'utf8');
  html = html.replace(/\{\{([A-Z0-9_]+)\}\}/g, (_, key) => {
    assert.ok(Object.hasOwn(values, key), `${file}: 未定義テンプレート ${key}`);
    return values[key];
  });
  assert.ok(!/\{\{[A-Z0-9_]+\}\}/.test(html), `${file}: 未解決テンプレート`);
  await writeFile(path.join(htmlDir, file), html);
}

// 生成物の抜き打ち検証(手書き数字の混入防止): 主要な値は全HTMLで一致する
const generated = Object.fromEntries(await Promise.all(
  templates.map(async file => [file, await readFile(path.join(htmlDir, file), 'utf8')]),
));
for (const [file, html] of Object.entries(generated)) {
  assert.ok(!/\{\{/.test(html), `${file}: プレースホルダ残存`);
}
assert.ok(generated['assets.html'].includes(values.TOTAL), 'assets 集計');
assert.ok(generated['home.html'].includes(`前日比 ${values.DAILY_CHANGE} (${values.DAILY_RATE})`), 'home 前日比');
assert.ok(generated['stock-search.html'].includes(values.STOCK_A_DAY_CHANGE), 'search 前日比');
assert.ok(generated['holding-route.html'].includes(values.STOCK_A_DAY_CHANGE), 'route 前日比');
assert.ok(generated['dialogs.html'].includes(values.RECENT_A_BEFORE), 'dialogs 配当');
assert.ok(!generated['index.html'].includes('¥¥'), 'index 二重円');
assert.ok(!generated['states-codex.html'].includes('++'), 'states 二重符号');

const checks = [
  ['評価額', holdings.map(row => `${row.code} ${number(row.quantity)}${row.type === 'fund' ? '口' : '株'} × ${yen(row.currentPrice)}${row.type === 'fund' ? '/10,000口' : ''} = ${yen(row.value)}`).join(' / ') + `; 合計 ${yen(stockValue)} + ${yen(fundValue)} = ${yen(total)}`],
  ['取得総額', `${values.COST_PARTS} = ${yen(costTotal)}`],
  ['評価損益・損益率', `${yen(total)} − ${yen(costTotal)} = ${signedYen(gainTotal)} (全体 ${values.GAIN_RATE_TOTAL}); 行別率 ${holdings.map(row => `${row.code} ${pct(row.gainRate, 2)}`).join(' / ')}`],
  ['分類別割合', `${yen(stockValue)} ÷ ${yen(total)} = ${values.STOCK_CLASS_SHARE}; ${yen(fundValue)} ÷ ${yen(total)} = ${values.FUND_CLASS_SHARE}`],
  ['前日比', `${yen(previousValue)} ${signedYen(dailyChange)} = ${yen(total)}; ${yen(dailyChange)} ÷ ${yen(previousValue)} = ${values.DAILY_RATE_EXACT} (表示 ${values.DAILY_RATE})`],
  ['配当', `月別税引前 ${yen(dividendBefore)} − 税額 ${yen(dividendTax)} = 税引後 ${yen(dividendAfter)}; 年間配当 ${yen(dividendAnnual)}; 9901利回り ${a.annualDividendPerShare} ÷ ${yen(a.currentPrice)} = ${values.STOCK_A_YIELD}`],
  ['件数', `保有 ${holdings.length} 件 (国内株式 ${stocks.length}・投資信託 ${funds.length}); 配当銘柄 ${stocks.length} 件`],
];
const report = `# Phase 5 Codex 数値検算\n\nこのファイルは \`node docs/design/work/phase5-codex/data/build.mjs\` が生成する。金額は円単位に四捨五入し、投資信託は口数×基準価額÷10,000 で計算。全 assert 通過。\n\n| 項目 | 計算・内訳 | 判定 |\n|---|---|---|\n${checks.map(([name, detail]) => `| ${name} | ${detail} | OK |`).join('\n')}\n\n| 保有 | 数量・価格 | 評価額 | 取得総額 | 評価損益 | 損益率 | 前日評価額 |\n|---|---:|---:|---:|---:|---:|---:|\n${holdings.map(row => `| ${row.code} ${row.name} | ${number(row.quantity)} ${row.type === 'fund' ? '口' : '株'} × ${yen(row.currentPrice)}${row.type === 'fund' ? '/10,000口' : ''}; 取得単価 ${yen(row.costUnitPrice)} | ${yen(row.value)} | ${yen(row.cost)} | ${signedYen(row.gain)} | ${pct(row.gainRate, 2)} | ${yen(row.previousValue)} |`).join('\n')}\n\n全画面はこの fixture から生成される。前日評価額は株の previousValue と投信の previousNav から合算。\n`;
await writeFile(path.join(root, 'data-check.md'), report);
console.log(checks.map(([name, detail]) => `${name}: ${detail}`).join('\n'));
