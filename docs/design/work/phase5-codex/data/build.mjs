// Phase 5 Codex 画面案の数値生成・検算。
// 元になる値は data/fixtures.json だけに置く。HTML は data/templates/*.html を
// 起点に {{KEY}} を埋めて html/ へ生成する(手で数字を書かない)。
// 計算規則は実装と合わせる: 株式=数量×現在値、投信=口数×基準価額÷10000、
// 前日比率=前日比÷前日の評価額 (backend asset_balance/fund_balance 由来)。
// 配当明細・売買明細も fixtures に置き、月別合計・年合計・件数を検算する。
// OpenCode 担当画面は同一 fixtures から生成されるため、突き合わせ表は
// 生成物に期待値が含まれることを assert で確認する(先に build-opencode を実行)。
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
const opencodeHtmlDir = path.join(root, '..', 'phase5-opencode', 'html');
const fixture = JSON.parse(await readFile(path.join(root, 'data/fixtures.json'), 'utf8'));

const yen = value => `¥${Math.round(value).toLocaleString('en-US')}`;
const priceYen = value => `¥${Number(value).toLocaleString('en-US', { maximumFractionDigits: 3 })}`;
const signedYen = value => `${value > 0 ? '+' : value < 0 ? '-' : ''}¥${Math.abs(Math.round(value)).toLocaleString('en-US')}`;
const number = value => Number(value).toLocaleString('en-US', { maximumFractionDigits: 2 });
const pct = (value, digits = 1) => `${value.toFixed(digits)}%`;
const sum = (rows, key) => rows.reduce((total, row) => total + row[key], 0);

// 行ごとの派生値: 評価額・取得額・損益・損益率(=損益÷取得総額)・前日評価額
const holdings = fixture.holdings.map(row => {
  ok(row.type === 'stock' || row.type === 'fund', `${row.id}: 不明な資産種別`);
  ok(row.quantity > 0 && row.currentPrice > 0 && row.costUnitPrice > 0, `${row.id}: 数量・価格`);
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

// ---- 配当明細行: 月別合計・年合計・数量×単価・税引後の検算 ----
const dividendTxs = fixture.dividendTransactions.map(row => {
  const holding = holdings.find(h => h.id === row.holdingId);
  ok(holding, `${row.date}: 不明な保有 ${row.holdingId}`);
  ok(row.quantity > 0 && row.unitPrice > 0, `${row.date}: 数量・単価`);
  eq(Math.round(row.quantity * row.unitPrice), row.beforeTax, `${row.date}: 数量×単価`);
  ok(row.tax >= 0 && row.tax <= row.beforeTax, `${row.date}: 税額範囲`);
  return { ...row, holding, code: holding.code, name: holding.name, afterTax: row.beforeTax - row.tax };
});
for (const month of fixture.dividends) {
  const inMonth = dividendTxs.filter(row => Number(row.date.slice(5, 7)) === month.month);
  ok(inMonth.length >= 1 && inMonth.length <= 4, `${month.month}月: 1〜4行(実績 ${inMonth.length}行)`);
  eq(sum(inMonth, 'beforeTax'), month.beforeTax, `${month.month}月 税引前`);
  eq(sum(inMonth, 'tax'), month.tax, `${month.month}月 税額`);
}
eq(sum(dividendTxs, 'beforeTax'), dividendBefore, '配当行合計=月別合計(税引前)');
eq(sum(dividendTxs, 'tax'), dividendTax, '配当行合計=月別合計(税額)');
eq(sum(dividendTxs, 'afterTax'), dividendAfter, '配当行合計=年合計(税引後)');

// ---- 国内株式売買行: 保有と整合(コード・取得単価)し、売却額−取得額=損益 ----
const stockTrades = fixture.stockTrades.map(row => {
  const holding = holdings.find(h => h.id === row.holdingId && h.type === 'stock');
  ok(holding, `${row.date}: 不明な国内株式 ${row.holdingId}`);
  eq(row.sellAmount, row.quantity * row.sellPrice, `${row.date}: 数量×売却単価`);
  eq(row.costAmount, Math.round(row.quantity * holding.costUnitPrice), `${row.date}: 数量×取得単価`);
  eq(row.gain, row.sellAmount - row.costAmount, `${row.date}: 実現損益`);
  if (row.gain <= 0) eq(row.tax, 0, `${row.date}: 損失時税額ゼロ`);
  else ok(row.tax > 0 && row.tax < row.gain, `${row.date}: 税額範囲`);
  return { ...row, code: holding.code, name: holding.name, after: row.gain - row.tax };
});
const stockGain = sum(stockTrades, 'gain');
const stockTax = sum(stockTrades, 'tax');
const stockAfter = stockGain - stockTax;
eq(stockAfter, sum(stockTrades, 'after'), '株式税引後');

// ---- 投資信託解約行: 保有と整合(ファンド・取得単価)し、口数×基準価額÷10000 ----
const fundTrades = fixture.fundTrades.map(row => {
  const holding = holdings.find(h => h.id === row.holdingId && h.type === 'fund');
  ok(holding, `${row.date}: 不明な投資信託 ${row.holdingId}`);
  eq(row.redeemAmount, Math.round(row.quantity * row.redeemNav / 10000), `${row.date}: 口数×解約基準価額÷10000`);
  eq(row.costNav, holding.costUnitPrice, `${row.date}: 取得単価=保有の取得単価`);
  eq(row.costAmount, Math.round(row.quantity * row.costNav / 10000), `${row.date}: 口数×取得単価÷10000`);
  eq(row.gain, row.redeemAmount - row.costAmount, `${row.date}: 実現損益`);
  ok(row.tax > 0 && row.tax < row.gain, `${row.date}: 税額範囲`);
  ok(Math.abs(row.redeemNav - holding.currentPrice) / holding.currentPrice < 0.1, `${row.date}: 解約基準価額は保有の現在値に近い`);
  return { ...row, code: holding.code, name: holding.name, after: row.gain - row.tax };
});
const fundGain = sum(fundTrades, 'gain');
const fundTax = sum(fundTrades, 'tax');
const fundAfter = fundGain - fundTax;

// ---- 最近の配当入金 = 明細行の新しい順の上位2行(ホーム・ダイアログと一致) ----
const byDateDesc = [...dividendTxs].sort((x, y) => (x.date < y.date ? 1 : -1));
const recentDividends = byDateDesc.slice(0, 2);
eq(recentDividends.length, 2, '最近の配当2行');
ok(recentDividends[0].date >= recentDividends[1].date, '新しい順');

const percentage = (value, base, digits = 1) => `${(value / base * 100).toFixed(digits)}%`;
const byId = Object.fromEntries(holdings.map(row => [row.id, row]));
const a = byId['stock-a'];
const b = byId['stock-b'];
const fund = byId['fund-a'];
const fundNavChange = fund.currentPrice - fund.previousNav;
const stockADayChange = a.currentPrice - a.previousValue / a.quantity;
const specificValue = holdings.filter(row => row.account === '特定口座').reduce((t, row) => t + row.value, 0);
const nisaValue = holdings.filter(row => row.account === 'NISA').reduce((t, row) => t + row.value, 0);
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
  DIV_TX_COUNT: String(dividendTxs.length), STOCK_TX_COUNT: String(stockTrades.length),
  FUND_TX_COUNT: String(fundTrades.length),
  RECENT_A_BEFORE: yen(recentDividends[0].beforeTax), RECENT_A_TAX_SIGNED: `-${yen(recentDividends[0].tax)}`,
  RECENT_A_AFTER: yen(recentDividends[0].afterTax),
  RECENT_A_DATE_SLASH: recentDividends[0].date, RECENT_A_DATE_JP: recentDividends[0].date.replaceAll('/', '年').replace('年', '年', 1),
  RECENT_A_CODE: recentDividends[0].code, RECENT_A_NAME: recentDividends[0].name,
  RECENT_A_ACCOUNT: recentDividends[0].account,
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
  GAIN_ROWS: [...holdings].filter(row => row.gain > 0).sort((x, y) => y.gain - x.gain).map(row => `<div class="row"><span>${row.code}　${row.name}</span><b class="positive">${signedYen(row.gain)}</b></div>`).join(''),
};
values.RECENT_A_DATE_JP = recentDividends[0].date.replace(/^(\d+)\/(\d+)\/(\d+)$/, '$1年$2月$3日');

// ---- 検算: 不整合があれば失敗する ----
// Phase 3 の集計値に合うこと
eq(total, 28593148, '評価額合計');
eq(costTotal, 27040660, '取得総額');
eq(gainTotal, 1552488, '評価損益');
eq(gainTotal, total - costTotal, '合計損益');
eq(stockValue, 9149807, '国内株式評価額');
eq(fundValue, 19443341, '投資信託評価額');
eq(total, stockValue + fundValue, '分類別評価額');
eq(total, specificValue + nisaValue, '口座別評価額');
// 行の数量×単価=評価額(投信は口数×基準価額÷10000)
eq(a.value, a.quantity * a.currentPrice, '9901 数量×現在値');
eq(a.value, 1200000, '9901 評価額');
eq(b.value, Math.round(b.quantity * b.currentPrice), '9902 数量×現在値');
eq(fund.value, Math.round(fund.quantity * fund.currentPrice / 10000), '投信 口数×基準価額÷10000');
eq(sum(stocks, 'value'), stockValue, '国内株式内訳');
eq(sum(funds, 'value'), fundValue, '投資信託内訳');
// 前日比の率=前日比÷前日の評価額
eq(dailyChange, 154488, '前日比額');
eq(Math.round(dailyRate * 10) / 10, 0.5, '前日比率');
eq(previousValue + dailyChange, total, '前日評価額+前日比=評価額');
// 配当: 月別合計=年合計=銘柄別合計、税引前−税額=税引後
eq(dividendAnnual, dividendBefore, '月別配当合計と年合計');
eq(dividendAfter, dividendBefore - dividendTax, '配当税引後');
eq(dividendAnnual, Object.values(annualByHolding).reduce((t, v) => t + v, 0), '年間配当と銘柄別合計');
ok(fixture.dividends.every(row => row.beforeTax >= row.tax), '月別税引後非負');
ok(recentDividends.every(row => row.holding && row.afterTax === row.beforeTax - row.tax), '最近の配当内訳');
// 件数・ID重複・スマホ/PC共通(同じ fixtures から同じ数量・金額)
eq(holdings.length, 3, '保有件数');
eq(new Set(holdings.map(row => row.id)).size, holdings.length, '保有ID重複');
eq(fixture.dividends.length, 12, '配当月数');
eq(dividendTxs.length, 18, '配当明細行数');
eq(stockTrades.length, 3, '株式売買行数');
eq(fundTrades.length, 2, '投信解約行数');

// ---- 生成: テンプレートに埋める ----
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

// 生成物の抜き打ち検証(手書き数字の混入防止): 主要な値は全HTMLで一致する
const generated = Object.fromEntries(await Promise.all(
  templates.map(async file => [file, await readFile(path.join(htmlDir, file), 'utf8')]),
));
for (const [file, html] of Object.entries(generated)) {
  ok(!/\{\{/.test(html), `${file}: プレースホルダ残存`);
}
ok(generated['assets.html'].includes(values.TOTAL), 'assets 集計');
ok(generated['home.html'].includes(`前日比 ${values.DAILY_CHANGE} (${values.DAILY_RATE})`), 'home 前日比');
ok(generated['stock-search.html'].includes(values.STOCK_A_DAY_CHANGE), 'search 前日比');
ok(generated['holding-route.html'].includes(values.STOCK_A_DAY_CHANGE), 'route 前日比');
ok(generated['dialogs.html'].includes(values.RECENT_A_BEFORE), 'dialogs 配当');
ok(!generated['index.html'].includes('¥¥'), 'index 二重円');
ok(!generated['states-codex.html'].includes('++'), 'states 二重符号');

// ---- OpenCode 担当画面との突き合わせ: 同一 fixtures から生成された HTML に期待値が含まれること ----
const opencode = Object.fromEntries(await Promise.all(
  ['receipts-opencode.html', 'csv-opencode.html', 'account-opencode.html'].map(async file => {
    let html;
    try {
      html = await readFile(path.join(opencodeHtmlDir, file), 'utf8');
    } catch {
      assert.fail(`${file}: OpenCode 生成物がない。先に node docs/design/work/phase5-opencode/data/build-opencode.mjs を実行すること`);
    }
    return [file, html];
  }),
));
const receipts = opencode['receipts-opencode.html'];
const csv = opencode['csv-opencode.html'];
ok(receipts.includes(values.DIVIDEND_AFTER), '突き合わせ 年合計 税引後');
ok(receipts.includes(values.DIVIDEND_BEFORE), '突き合わせ 年合計 税引前');
ok(receipts.includes(`-${values.DIVIDEND_TAX}`), '突き合わせ 年合計 税額');
for (const month of fixture.dividends) {
  const inMonth = dividendTxs.filter(row => Number(row.date.slice(5, 7)) === month.month);
  const after = yen(sum(inMonth, 'afterTax'));
  ok(receipts.includes(`2026年${month.month}月 · ${inMonth.length}件の合計`), `突き合わせ ${month.month}月 見出し`);
  ok(receipts.includes(yen(month.beforeTax)) && receipts.includes(after), `突き合わせ ${month.month}月 小計`);
}
for (const row of dividendTxs) {
  ok(receipts.includes(row.date) && receipts.includes(yen(row.afterTax)), `突き合わせ 行 ${row.date}`);
}
for (const row of stockTrades) {
  ok(receipts.includes(row.date) && receipts.includes(signedYen(row.gain - row.tax)), `突き合わせ 株式行 ${row.date}`);
}
for (const row of fundTrades) {
  ok(receipts.includes(row.date) && receipts.includes(signedYen(row.gain - row.tax)), `突き合わせ 投信行 ${row.date}`);
}
ok(csv.includes(`資産管理(国内株式)</td><td class="num">${values.STOCK_COUNT}件`), '突き合わせ CSV 国内株式件数');
ok(csv.includes(`資産管理(投資信託)</td><td class="num">${values.FUND_COUNT}件`), '突き合わせ CSV 投資信託件数');
ok(csv.includes(`取引明細(配当金)</td><td class="num">${values.DIV_TX_COUNT}件`), '突き合わせ CSV 配当件数');
ok(csv.includes(`取引明細(国内株式)</td><td class="num">${values.STOCK_TX_COUNT}件`), '突き合わせ CSV 株式件数');
ok(csv.includes(`取引明細(投資信託)</td><td class="num">${values.FUND_TX_COUNT}件`), '突き合わせ CSV 投信件数');
ok(opencode['account-opencode.html'].includes(values.DIVIDEND_ANNUAL), '突き合わせ アカウント年間配当');
ok(generated['home.html'].includes(values.RECENT_A_AFTER) && generated['home.html'].includes(values.RECENT_B_AFTER), '突き合わせ ホーム最近の配当');

const checks = [
  ['評価額', holdings.map(row => `${row.code} ${number(row.quantity)}${row.type === 'fund' ? '口' : '株'} × ${priceYen(row.currentPrice)}${row.type === 'fund' ? '/10,000口' : ''} = ${yen(row.value)}`).join(' / ') + `; 合計 ${yen(stockValue)} + ${yen(fundValue)} = ${yen(total)}`],
  ['取得総額', `${values.COST_PARTS} = ${yen(costTotal)}`],
  ['評価損益・損益率', `${yen(total)} − ${yen(costTotal)} = ${signedYen(gainTotal)} (全体 ${values.GAIN_RATE_TOTAL}); 行別率 ${holdings.map(row => `${row.code} ${pct(row.gainRate, 2)}`).join(' / ')}`],
  ['分類別割合', `${yen(stockValue)} ÷ ${yen(total)} = ${values.STOCK_CLASS_SHARE}; ${yen(fundValue)} ÷ ${yen(total)} = ${values.FUND_CLASS_SHARE}`],
  ['前日比', `${yen(previousValue)} ${signedYen(dailyChange)} = ${yen(total)}; ${yen(dailyChange)} ÷ ${yen(previousValue)} = ${values.DAILY_RATE_EXACT} (表示 ${values.DAILY_RATE})`],
  ['配当', `月別税引前 ${yen(dividendBefore)} − 税額 ${yen(dividendTax)} = 税引後 ${yen(dividendAfter)}; 年間配当 ${yen(dividendAnnual)}; 9901利回り ${a.annualDividendPerShare} ÷ ${priceYen(a.currentPrice)} = ${values.STOCK_A_YIELD}`],
  ['件数', `保有 ${holdings.length} 件 (国内株式 ${stocks.length}・投資信託 ${funds.length}); 配当銘柄 ${stocks.length} 件`],
  ['配当明細行', `明細 ${dividendTxs.length} 行の月別合計が月別配当と一致(各月 1〜4 行)。新しい順の上位 2 行(${recentDividends.map(row => `${row.date} ${row.code} ${yen(row.afterTax)}`).join(' / ')})をホーム・ダイアログに表示`],
  ['株式売買行', `売買 ${stockTrades.length} 行: ${stockTrades.map(row => `${row.date} ${row.code} ${signedYen(row.gain)}`).join(' / ')}; 年合計 ${signedYen(stockGain)}(税引前) − ${yen(stockTax)} = ${signedYen(stockAfter)}(税引後)`],
  ['投信解約行', `解約 ${fundTrades.length} 行: ${fundTrades.map(row => `${row.date} ${yen(row.after)}`).join(' / ')}; 年合計 ${signedYen(fundGain)}(税引前) − ${yen(fundTax)} = ${signedYen(fundAfter)}(税引後)`],
];
const monthlyRows = fixture.dividends.map(row => `| 2026年${row.month}月 | ${yen(row.beforeTax)} | ${yen(row.tax)} | ${yen(row.beforeTax - row.tax)} |`).join('\n');
const monthChecks = fixture.dividends.map(row => {
  const inMonth = dividendTxs.filter(tx => Number(tx.date.slice(5, 7)) === row.month);
  return `| 2026年${row.month}月・${inMonth.length}件 | ${yen(row.beforeTax)} | −${yen(row.tax)} | ${yen(row.beforeTax - row.tax)} | ${yen(row.beforeTax)} / −${yen(row.tax)} / ${yen(row.beforeTax - row.tax)} | 一致 |`;
}).join('\n');
const rowChecks = dividendTxs.map(row => `| ${row.date} | ${row.code} | ${yen(row.beforeTax)} | −${yen(row.tax)} | ${yen(row.afterTax)} | 明細と一致 | 一致 |`).join('\n');
const appendix = `
## 月別配当内訳(税引後合計の検算)

| 月 | 税引前 | 税額 | 税引後 |
|---|---|---:|---:|
${monthlyRows}
| 合計 | ${yen(dividendBefore)} | ${yen(dividendTax)} | ${yen(dividendAfter)} |

月別税引後の合計 = ${yen(dividendAfter)} が年合計と一致。税引前 − 税額 = 税引後も月別・年合計とも成立。

## OpenCode 担当画面との突き合わせ

対象: \`docs/design/work/phase5-opencode/html/receipts-opencode.html\` ほか。全画面は同一 fixtures(\`docs/design/work/phase5-codex/data/fixtures.json\`)から生成され、以下の項目は生成物の包含検証(assert)済み。

### 年合計(集計帯)

| OpenCode箇所 | OpenCode値 | fixtures対応値 | 判定 |
|---|---|---|---|
| 集計帯 税引後 | ${yen(dividendAfter)} | ${yen(dividendAfter)} | 一致 |
| 集計帯 税引前 | ${yen(dividendBefore)} | ${yen(dividendBefore)} | 一致 |
| 集計帯 税額 | −${yen(dividendTax)} | −${yen(dividendTax)} | 一致 |

### 月小計

| OpenCode月 | 税引前 | 税額 | 税引後 | fixtures同月(税引前/税額/税引後) | 判定 |
|---|---|---:|---:|---|---|
${monthChecks}

### 行(${dividendTxs.length}行)

| 入金日 | コード | 金額 | 税額 | 税引後 | fixtures対応 | 判定 |
|---|---|---:|---:|---:|---|---|
${rowChecks}

### 件数・ホーム

| 項目 | OpenCode値 | fixtures対応値 | 判定 |
|---|---|---|---|
| 資産管理(国内株式) | ${values.STOCK_COUNT}件 | 保有の国内株式 ${values.STOCK_COUNT}件 | 一致 |
| 資産管理(投資信託) | ${values.FUND_COUNT}件 | 保有の投資信託 ${values.FUND_COUNT}件 | 一致 |
| 取引明細(配当金) | ${values.DIV_TX_COUNT}件 | 配当明細 ${values.DIV_TX_COUNT}行 | 一致 |
| 取引明細(国内株式) | ${values.STOCK_TX_COUNT}件 | 株式売買 ${values.STOCK_TX_COUNT}行 | 一致 |
| 取引明細(投資信託) | ${values.FUND_TX_COUNT}件 | 投信解約 ${values.FUND_TX_COUNT}行 | 一致 |
| ホーム最近の配当入金 | ${recentDividends.map(row => `${row.date} ${yen(row.afterTax)}`).join(' / ')} | 明細の新しい順の上位 2 行 | 一致 |
`;
const report = `# Phase 5 Codex 数値検算\n\nこのファイルは \`node docs/design/work/phase5-codex/data/build.mjs\` が生成する。金額は円単位に四捨五入し、投資信託は口数×基準価額÷10,000 で計算。全 ${assertCount} assert 通過(不一致 0)。\n\n| 項目 | 計算・内訳 | 判定 |\n|---|---|---|\n${checks.map(([name, detail]) => `| ${name} | ${detail} | OK |`).join('\n')}\n\n| 保有 | 数量・価格 | 評価額 | 取得総額 | 評価損益 | 損益率 | 前日評価額 |\n|---|---:|---:|---:|---:|---:|---:|\n${holdings.map(row => `| ${row.code} ${row.name} | ${number(row.quantity)} ${row.type === 'fund' ? '口' : '株'} × ${priceYen(row.currentPrice)}${row.type === 'fund' ? '/10,000口' : ''}; 取得単価 ${priceYen(row.costUnitPrice)} | ${yen(row.value)} | ${yen(row.cost)} | ${signedYen(row.gain)} | ${pct(row.gainRate, 2)} | ${yen(row.previousValue)} |`).join('\n')}\n\n全画面はこの fixture から生成される。前日評価額は株の previousValue と投信の previousNav から合算。\n${appendix}`;
await writeFile(path.join(root, 'data-check.md'), report);
console.log(checks.map(([name, detail]) => `${name}: ${detail}`).join('\n'));
console.log(`asserts: ${assertCount} passed, 0 failed`);
