#!/usr/bin/env python3
"""Phase9 opencode担当分の新案HTMLを生成する。数値は fixtures.json 由来の架空データを直書き。"""
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE.parent

CSS_VARS = """:root{
  --page:#ffffff; --card:#ffffff; --ink:#111827; --mut:#5f6b7a; --line:#e5e7eb;
  --navy:#0f2a4a; --accent:#1d4ed8; --up:#0b7a4b; --down:#c0362c;
  --gold:#f2b33d; --seg:#eef1f6;
  --comp-a:#0f2a4a; --comp-b:#1d4ed8;
}
html[data-theme="dark"]{
  --page:#0b1220; --card:#121b2c; --ink:#dfe6f2; --mut:#c2cddf; --line:#26334d;
  --navy:#dfe6f2; --accent:#8fb3f7; --up:#53d28a; --down:#f0857a;
  --gold:#f2b33d; --seg:#1c2942;
  --comp-a:#3987e5; --comp-b:#d95926;
}
@media (prefers-color-scheme: dark){
  html:not([data-theme]){ --page:#0b1220; --card:#121b2c; --ink:#dfe6f2; --mut:#c2cddf; --line:#26334d;
    --navy:#dfe6f2; --accent:#8fb3f7; --up:#53d28a; --down:#f0857a;
    --gold:#f2b33d; --seg:#1c2942;
    --comp-a:#3987e5; --comp-b:#d95926; }
}"""

BASE = """*{box-sizing:border-box}
body{margin:0; background:var(--page); color:var(--ink);
  font-family:"Noto Sans JP","Hiragana Kaku Gothic ProN",Meiryo,sans-serif; font-weight:400;
  -webkit-text-size-adjust:100%}
.num{font-feature-settings:"tnum" 1; font-variant-numeric:tabular-nums}
.up{color:var(--up)} .down{color:var(--down)}
.card{background:var(--card); border:1px solid var(--line); border-radius:12px; padding:16px; box-shadow:none; min-width:0}
.card h2{font-weight:700; color:var(--navy); margin:0; white-space:nowrap; overflow:hidden; text-overflow:ellipsis}
.hrow{display:flex; justify-content:space-between; align-items:center; gap:12px}
.hrow .more{font-size:13px; color:var(--accent); white-space:nowrap; min-height:44px; display:flex; align-items:center}
.sum{display:flex; justify-content:space-between; align-items:baseline; gap:12px; margin:12px 0 0}
.rows{margin-top:4px}
.row{display:flex; justify-content:space-between; align-items:center; gap:12px; padding:8px 0; border-top:1px solid var(--line)}
.row:first-child{border-top:none}
.row .left{min-width:0; flex:1 1 auto}
.row .name{white-space:nowrap; overflow:hidden; text-overflow:ellipsis}
.row .sub{display:block; color:var(--mut); margin-top:2px; white-space:nowrap; overflow:hidden; text-overflow:ellipsis}
.row .amt{flex:none; font-weight:700; text-align:right; white-space:nowrap}
.row .amt .rate{display:block; font-weight:700; margin-top:2px}
.subhead{color:var(--mut); margin:14px 0 0; white-space:nowrap}
/* タブ: 現行と同じ3項目。下線で現位置だけ示す */
.tabs{display:flex; gap:4px; border-bottom:1px solid var(--line); margin-top:12px}
.tabs span{flex:1; text-align:center; white-space:nowrap; color:var(--mut); padding:12px 0 10px; min-height:44px}
.tabs .here{color:var(--navy); font-weight:700; border-bottom:2px solid var(--gold)}
/* 検索と並べ替え: 全画面で同じ部品 */
.search{display:flex; align-items:center; gap:8px; border:1px solid var(--line); border-radius:8px; padding:0 12px; min-height:44px; margin-top:12px; color:var(--mut); background:var(--card)}
.search .ph{font-size:14px; white-space:nowrap; overflow:hidden; text-overflow:ellipsis}
.sortbtn{display:flex; align-items:center; justify-content:space-between; gap:8px; border:1px solid var(--line); border-radius:8px; padding:0 12px; min-height:44px; margin-top:8px; font-size:14px; background:var(--card); white-space:nowrap}
.countrow{display:flex; justify-content:space-between; align-items:center; gap:12px; margin:12px 0 0}
.basisline{display:flex; align-items:center; gap:8px; margin:12px 0 0; font-size:13px}
.basisline .cur{color:var(--navy); font-weight:700; white-space:nowrap}
.basisline .opt{color:var(--accent); white-space:nowrap; min-height:44px; display:flex; align-items:center}
.basisline .lbl{color:var(--mut); white-space:nowrap}
.basisline .sep{color:var(--mut)}
.countrow .n{font-size:13px; color:var(--mut); white-space:nowrap}
.countrow .link{font-size:13px; color:var(--accent); white-space:nowrap; min-height:44px; display:flex; align-items:center}
/* 横棒: 高さ8px。3区分までは紺・青・灰 */
.bar100{display:flex; height:8px; margin:14px 0 6px; overflow:hidden; border-radius:4px; background:var(--seg)}
.bar100 .a{background:var(--comp-a)} .bar100 .b{background:var(--comp-b)} .bar100 .c{background:var(--mut)}
/* 状態の例: スケルトンは線だけの箱 */
.sk{border-radius:6px; background:var(--seg); min-height:14px}
.state-note{font-size:12px; color:var(--mut); white-space:nowrap; overflow:hidden; text-overflow:ellipsis}
.statenote-wrap{font-size:13px; color:var(--mut); margin:12px 0 0; line-height:1.7}
.btn{display:flex; align-items:center; justify-content:center; min-height:48px; border-radius:8px; font-size:15px; font-weight:700; white-space:nowrap; border:1px solid var(--line); background:var(--card); color:var(--ink)}
.btn.primary{background:var(--navy); border-color:var(--navy); color:#ffffff}
html[data-theme="dark"] .btn.primary{color:#0b1220}
@media (prefers-color-scheme: dark){ html:not([data-theme]) .btn.primary{color:#0b1220} }
html[data-theme="dark"] .btn.primary{background:var(--accent); border-color:var(--accent)}
@media (prefers-color-scheme: dark){ html:not([data-theme]) .btn.primary{background:var(--accent); border-color:var(--accent)} }
.btn.danger{color:var(--down); border-color:var(--down)}
.backlink{display:inline-flex; align-items:center; min-height:44px; font-size:13px; color:var(--accent); white-space:nowrap; text-decoration:none}
.pagetitle{font-size:20px; font-weight:700; color:var(--navy); margin:0; white-space:nowrap}
.seg{display:flex; border:1px solid var(--line); border-radius:8px; overflow:hidden; margin-top:12px}
.seg span{flex:1; text-align:center; font-size:14px; padding:10px 0; min-height:44px; display:flex; align-items:center; justify-content:center; color:var(--mut); white-space:nowrap}
.seg .here{background:var(--navy); color:#ffffff; font-weight:700}
html[data-theme="dark"] .seg .here{background:var(--accent); color:#0b1220}
@media (prefers-color-scheme: dark){ html:not([data-theme]) .seg .here{background:var(--accent); color:#0b1220} }
.dl{display:flex; justify-content:space-between; align-items:baseline; gap:12px; padding:9px 0; border-top:1px solid var(--line)}
.dl:first-child{border-top:none}
.dl dt{font-size:14px; color:var(--mut); white-space:nowrap}
.dl dd{margin:0; font-size:15px; font-weight:700; text-align:right; white-space:nowrap}
.tlinks{display:flex; gap:16px; align-items:center}
.mblock{margin-top:20px}
.mblock h2{font-size:16px; font-weight:700; color:var(--navy); margin:0; white-space:nowrap; overflow:hidden; text-overflow:ellipsis}
.rows.loose .name{white-space:normal; display:-webkit-box; -webkit-line-clamp:2; -webkit-box-orient:vertical; overflow:hidden}
.mrow td{background:var(--seg); font-weight:700}
.mrow td:first-child{border-left:3px solid var(--gold)}
.mrow .mt{font-size:16px; font-weight:700; color:var(--navy); white-space:nowrap}
.mrow .ms{font-size:13px; color:var(--mut); white-space:nowrap}
.mrow .ms.up{color:var(--up)} .mrow .ms.down{color:var(--down)}
.herod{margin:4px 0 0}
.herod summary{list-style:none; display:flex; align-items:center; justify-content:space-between; gap:12px; cursor:pointer; min-height:44px}
.herod summary::-webkit-details-marker{display:none}
.herod .chev{font-size:13px; color:var(--mut)}
.herod[open] .chev{transform:rotate(180deg)}
.herod .xrow{display:flex; justify-content:space-between; align-items:baseline; gap:12px; padding:8px 0; border-top:1px solid var(--line)}
.herod .xrow .k{font-size:14px; color:var(--mut); white-space:nowrap}
.herod .xrow .v{font-size:15px; font-weight:700; white-space:nowrap}
table.grid .nm{overflow:hidden; text-overflow:ellipsis}
"""

MOBILE_EXTRA = """.top{border-bottom:1px solid var(--line)}
.top .inner{max-width:480px; margin:0 auto; padding:0 16px; display:flex; align-items:center; justify-content:space-between; min-height:52px; gap:12px}
.brand{font-size:15px; font-weight:700; color:var(--navy); white-space:nowrap}
.account{font-size:13px; color:var(--mut); white-space:nowrap; min-height:44px; display:flex; align-items:center}
.wrap{max-width:480px; margin:0 auto; padding:0 16px 120px}
.hero{padding:20px 0 4px}
.hero .label{font-size:12px; color:var(--mut); margin:0; white-space:nowrap}
.hero .total{font-size:30px; font-weight:700; color:var(--navy); margin:4px 0 0; white-space:nowrap; text-align:left}
.hero .pl{font-size:16px; font-weight:700; margin:6px 0 0; white-space:nowrap}
.hero .date{font-size:12px; color:var(--mut); margin:6px 0 0; white-space:nowrap}
.card{margin-top:12px} .card h2{font-size:16px}
.row{min-height:56px} .row .name{font-size:15px} .row .sub{font-size:12px} .row .amt{font-size:15px} .row .amt .rate{font-size:12px}
.subhead{font-size:13px}
.bottomnav{position:fixed; left:0; right:0; bottom:0; display:flex; background:var(--page); border-top:1px solid var(--line); z-index:10}
.bottomnav a{flex:1; display:flex; flex-direction:column; align-items:center; justify-content:center; gap:2px; min-height:60px; padding:8px 0 calc(8px + env(safe-area-inset-bottom)); font-size:11px; color:var(--mut); text-decoration:none; white-space:nowrap}
.bottomnav a.here{color:var(--accent); font-weight:700}
table.grid{width:100%}
"""

PC_EXTRA = """.top{border-bottom:1px solid var(--line)}
.top .inner{max-width:1200px; margin:0 auto; padding:0 24px; display:flex; align-items:center; gap:28px; min-height:56px}
.brand{font-size:15px; font-weight:700; color:var(--navy); white-space:nowrap}
.top nav{display:flex; gap:24px; font-size:14px}
.top nav span{white-space:nowrap; color:var(--mut); padding:16px 0}
.top nav .here{color:var(--ink); font-weight:700; border-bottom:2px solid var(--gold)}
.account{margin-left:auto; font-size:13px; color:var(--mut); white-space:nowrap}
.wrap{max-width:1200px; margin:0 auto; padding:0 24px 64px}
.hero{display:flex; align-items:flex-end; gap:32px; padding:28px 0 24px}
.stat{min-width:0}
.stat + .stat{border-left:1px solid var(--line); padding-left:32px}
.stat .label{font-size:13px; color:var(--mut); margin:0; white-space:nowrap}
.stat .v1{font-size:32px; font-weight:700; color:var(--navy); margin:4px 0 0; white-space:nowrap}
.stat .v2{font-size:24px; font-weight:700; margin:4px 0 0; white-space:nowrap}
.hero .dates{margin-left:auto; text-align:right; font-size:13px; color:var(--mut); white-space:nowrap; line-height:1.8}
.grid2{display:grid; grid-template-columns:2fr 1fr; gap:24px; align-items:start}
.col{display:grid; gap:24px; align-content:start}
.card h2{font-size:17px}
.row{min-height:52px} .row .name{font-size:15px} .row .sub{font-size:12px} .row .amt{font-size:15px} .row .amt .rate{font-size:12px}
.subhead{font-size:13px}
table.grid{width:100%; border-collapse:collapse; font-size:14px}
table.grid th{font-size:13px; font-weight:400; color:var(--mut); text-align:left; white-space:nowrap; padding:10px 12px; border-bottom:1px solid var(--line)}
table.grid th.r, table.grid td.r{text-align:right; white-space:nowrap}
table.grid th.sortable{color:var(--navy); font-weight:700}
table.grid td{padding:10px 12px; border-bottom:1px solid var(--line); vertical-align:middle}
table.grid tr:last-child td{border-bottom:none}
table.grid .nm{font-weight:400; white-space:nowrap}
table.grid .sb{font-size:12px; color:var(--mut); white-space:nowrap}
@media (max-width:900px){ .grid2{grid-template-columns:1fr} .hero{flex-wrap:wrap; gap:16px} .stat + .stat{border-left:none; padding-left:0} .hero .dates{margin-left:0; text-align:left} }
"""

def nav_m(here):
    items = [("ホーム", "M4 11l8-7 8 7\"/><path d=\"M6 10v10h12V10"),
             ("資産管理", "M4 20V10\"/><path d=\"M10 20V4\"/><path d=\"M16 20v-8\"/><path d=\"M22 20H2"),
             ("取引明細", "M6 3h12v18H6z\"/><path d=\"M9 8h6M9 12h6M9 16h4"),
             ("銘柄検索", "")]
    out = ['<nav class="bottomnav" aria-label="下ナビ">']
    for label, _ in items:
        cls = ' class="here"' if label == here else ""
        if label == "銘柄検索":
            icon = '<circle cx="11" cy="11" r="7"/><path d="M16.5 16.5L21 21"/>'
        elif label == "ホーム":
            icon = '<path d="M4 11l8-7 8 7"/><path d="M6 10v10h12V10"/>'
        elif label == "資産管理":
            icon = '<path d="M4 20V10"/><path d="M10 20V4"/><path d="M16 20v-8"/><path d="M22 20H2"/>'
        else:
            icon = '<path d="M6 3h12v18H6z"/><path d="M9 8h6M9 12h6M9 16h4"/>'
        out.append(f'  <a{cls} href="#"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">{icon}</svg>{label}</a>')
    out.append("</nav>")
    return "\n".join(out)

NAV_M = ""  # 旧テンプレは使わない(nav_mを使う)

SCRIPT = """<script>
try{const t=new URLSearchParams(location.search).get('theme');if(t==='light'||t==='dark')document.documentElement.dataset.theme=t;}catch(e){}
</script>"""

HEAD = """<!doctype html>
<html lang="ja">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title} | 証券Web</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@400;700&display=swap" rel="stylesheet">
<style>
/* 数値は docs/design/work/phase5-codex/data/fixtures.json の架空データを直書き。色は現行トークンのみ。面の塗りは横棒だけ。 */
{vars}
{base}
{extra}
</style>
</head>
<body>
"""


def mobile_top():
    return ('<header class="top"><div class="inner">\n'
            '  <span class="brand">証券Web</span><span class="account">アカウント</span>\n'
            '</div></header>\n<div class="wrap">\n')


def pc_top(here):
    items = ["ホーム", "資産管理", "取引明細", "銘柄検索"]
    nav = "".join(f'<span class="here">{i}</span>' if i == here else f"<span>{i}</span>" for i in items)
    return ('<header class="top"><div class="inner">\n'
            f'  <span class="brand">証券Web</span>\n  <nav>{nav}</nav>\n'
            '  <span class="account">アカウント</span>\n</div></header>\n<div class="wrap">\n')


def write(name, title, is_mobile, body, css=""):
    extra = ((MOBILE_EXTRA if is_mobile else PC_EXTRA) + css)
    html = HEAD.format(title=title, vars=CSS_VARS, base=BASE, extra=extra) + body + "</div>\n" + SCRIPT + "\n</body>\n</html>\n"
    (OUT / f"{name}.html").write_text(html, encoding="utf-8")
    print("wrote", name, len(html))


# ---------- 資産管理 ----------
ASSET_ROWS_M = """<div class="rows loose">
      <div class="row"><div class="left"><div class="name">架空成長インデックス・ファンド ›</div><span class="sub num">特定口座 ・ 10,000,000口</span></div><span class="amt num">¥19,443,341<span class="rate num">¥0 0.00%</span></span></div>
      <div class="row"><div class="left"><div class="name">9902 架空食品ホールディングス</div><span class="sub num">NISA ・ 100株</span></div><span class="amt num">¥7,949,807<span class="rate num up">+¥1,352,488 +20.50%</span></span></div>
      <div class="row"><div class="left"><div class="name">9901 架空青空電機</div><span class="sub num">特定口座 ・ 100株</span></div><span class="amt num">¥1,200,000<span class="rate num up">+¥200,000 +20.00%</span></span></div>
    </div>"""

ASSET_TITLE_M = """<div class="hrow" style="margin-top:12px"><h1 class="pagetitle">資産管理</h1><span class="tlinks"><span class="more">構成比を見る ›</span><span class="more">CSV管理 ›</span></span></div>"""
ASSET_TITLE_P = """<div class="hrow" style="margin-top:12px"><h1 class="pagetitle">資産管理</h1><span class="tlinks"><span class="more">構成比を見る ›</span><span class="more">CSV管理 ›</span></span></div>"""

ASSET_HERO_M = """<div class="hero">
    <p class="label">評価額</p>
    <details class="herod">
      <summary><span class="total num">¥28,593,148</span><span class="chev" aria-hidden="true">∨</span></summary>
      <div class="xrow"><span class="k">前日比</span><span class="v num up">+¥154,488</span></div>
      <div class="xrow"><span class="k">取得総額</span><span class="v num">¥27,040,660</span></div>
    </details>
    <p class="pl num up">+¥1,552,488 +5.74%</p>
    <p class="date num">価格基準日 2026/11/13</p>
  </div>"""

ASSET_SEARCH_M = """<div class="search"><span aria-hidden="true">⌕</span><span class="ph">銘柄・ファンド名・コード・口座で検索</span></div>
  <div class="sortbtn"><span>並べ替え: 評価額の高い順</span><span aria-hidden="true">›</span></div>
  <div class="countrow"><span class="n num">3件 / 3件</span></div>"""

ASSET_STATES = """<section class="card" aria-label="状態の例">
    <div class="hrow"><h2>状態の例</h2></div>
    <p class="subhead">空</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">資産管理データがありません</div><span class="sub">CSVファイルを取り込むと表示されます</span></div></div></div>
    <p class="subhead">読み込み中</p>
    <div class="rows"><div class="row"><div class="left"><div class="sk" style="width:60%"></div><div class="sub"><span class="sk" style="display:inline-block;width:40%">&nbsp;</span></div></div><span class="amt"><span class="sk" style="display:inline-block;width:90px">&nbsp;</span></span></div></div>
    <p class="subhead">取得失敗</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">エラー: 読み込みに失敗しました</div><span class="sub">時間をおいて再度お試しください</span></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>
  </section>"""

asset_m = (mobile_top() + ASSET_TITLE_M + ASSET_HERO_M + """
  <div class="tabs"><span class="here">すべて 3</span><span>国内株式 2</span><span>投資信託 1</span></div>
""" + ASSET_SEARCH_M + """
""" + ASSET_ROWS_M + """
""" + nav_m("資産管理"))

ASSET_HERO_P = """<div class="hero">
    <div class="stat"><p class="label">評価額</p><p class="v1 num">¥28,593,148</p></div>
    <div class="stat"><p class="label">評価損益</p><p class="v2 num up">+¥1,552,488 +5.74%</p></div>
    <div class="stat"><p class="label">前日比</p><p class="v2 num up">+¥154,488</p></div>
    <div class="stat"><p class="label">取得総額</p><p class="v2 num">¥27,040,660</p></div>
    <div class="dates num">価格基準日 2026/11/13<br>データ 2026/11/14取り込み</div>
  </div>"""

ASSET_TABLE_P = """<table class="grid" style="margin-top:8px">
      <tr><th>コード</th><th>銘柄・ファンド</th><th>種別</th><th>口座</th><th>分配金</th><th class="r">1株配当</th><th class="r">前日比</th><th class="r sortable">評価額 ▼</th><th class="r">評価損益</th></tr>
      <tr><td><span class="sb">—</span></td><td><div class="nm">架空成長インデックス・ファンド ›</div></td><td><span class="sb">投資信託</span></td><td><span class="sb">特定口座</span></td><td><span class="sb">—</span></td><td class="r num">—</td><td class="r num up">+¥154,488</td><td class="r num">¥19,443,341</td><td class="r num">¥0 0.00%</td></tr>
      <tr><td class="num">9902</td><td><div class="nm">架空食品ホールディングス</div></td><td><span class="sb">国内株式</span></td><td><span class="sb">NISA</span></td><td><span class="sb">—</span></td><td class="r num">¥39,339.07</td><td class="r num down">-¥20,000</td><td class="r num">¥7,949,807</td><td class="r num up">+¥1,352,488 +20.50%</td></tr>
      <tr><td class="num">9901</td><td><div class="nm">架空青空電機</div></td><td><span class="sb">国内株式</span></td><td><span class="sb">特定口座</span></td><td><span class="sb">—</span></td><td class="r num">¥360</td><td class="r num up">+¥20,000</td><td class="r num">¥1,200,000</td><td class="r num up">+¥200,000 +20.00%</td></tr>
    </table>"""

asset_p = (pc_top("資産管理") + ASSET_TITLE_P + ASSET_HERO_P + """
  <div class="tabs"><span class="here">すべて 3</span><span>国内株式 2</span><span>投資信託 1</span></div>
  <div class="search"><span aria-hidden="true">⌕</span><span class="ph">銘柄・ファンド名・コード・口座で検索</span></div>
  <div class="countrow"><span class="n num">3件 / 3件(列名クリックで並べ替え)</span></div>
""" + ASSET_TABLE_P)

# ---------- 構成比 ----------
COMP_M_BAR = """<div class="bar100" role="img" aria-label="架空成長インデックス・ファンド68.0%・9902 27.8%・9901 4.2%"><span class="a" style="width:68.0%"></span><span class="b" style="width:27.8%"></span><span class="c" style="width:4.2%"></span></div>"""
COMP_ROWS_M = """<div class="rows">
      <div class="row"><div class="left"><div class="name">架空成長インデックス・ファンド</div><span class="sub num">68.0% ・ 特定口座</span></div><span class="amt num">¥19,443,341</span></div>
      <div class="row"><div class="left"><div class="name">9902 架空食品ホールディングス</div><span class="sub num">27.8% ・ NISA</span></div><span class="amt num">¥7,949,807</span></div>
      <div class="row"><div class="left"><div class="name">9901 架空青空電機</div><span class="sub num">4.2% ・ 特定口座</span></div><span class="amt num">¥1,200,000</span></div>
    </div>"""
COMP_DIV_ROWS = """<div class="rows">
      <div class="row"><div class="left"><div class="name">9902 架空食品ホールディングス</div><span class="sub num">99.1% ・ NISA</span></div><span class="amt num">¥3,933,907</span></div>
      <div class="row"><div class="left"><div class="name">9901 架空青空電機</div><span class="sub num">0.9% ・ 特定口座</span></div><span class="amt num">¥36,000</span></div>
      <div class="row"><div class="left"><div class="name">架空成長インデックス・ファンド</div><span class="sub num">算出不可 ・ 特定口座</span></div><span class="amt num">—</span></div>
    </div>"""
COMP_STATES = """<section class="card" aria-label="状態の例">
    <div class="hrow"><h2>状態の例</h2></div>
    <p class="subhead">空</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">データがありません</div><span class="sub">保有残高をCSVで登録すると表示されます</span></div></div></div>
    <p class="subhead">読み込み中</p>
    <div class="rows"><div class="row"><div class="left"><div class="sk" style="width:60%"></div></div><span class="amt"><span class="sk" style="display:inline-block;width:90px">&nbsp;</span></span></div></div>
    <p class="subhead">取得失敗</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">エラー: 読み込みに失敗しました</div></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>
    <p class="subhead">算出不可の例</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">算出不可</div><span class="sub">合計が0のため割合を出せません</span></div><span class="amt num">—</span></div></div>
  </section>"""

COMP_DIV_SECTION_M = """  <section class="card" aria-label="基準を年間配当に替えた例">
    <div class="hrow"><h2>年間配当(税引前)</h2></div>
    <div class="sum"><span class="state-note num">年間配当(税引前)の比率・合計 ¥3,969,907・保有全体</span></div>
""" + COMP_DIV_ROWS + """
  </section>"""

COMP_DIV_SECTION_P = """      <section class="card" aria-label="基準を年間配当に替えた例">
        <div class="hrow"><h2>年間配当(税引前)</h2></div>
        <div class="sum"><span class="state-note num">合計 ¥3,969,907・保有全体</span></div>
""" + COMP_DIV_ROWS + """
      </section>"""

comp_m = (mobile_top() + """
  <a class="backlink" href="#">‹ 資産管理に戻る</a>
  <h1 class="pagetitle">構成比</h1>
  <div class="basisline" role="group" aria-label="構成比の基準"><span class="lbl">基準:</span><span class="cur">評価額</span><span class="sep">｜</span><span class="opt">年間配当(税引前)</span></div>
  <div class="tabs"><span class="here">銘柄別 3</span><span>商品別 2</span><span>口座別 2</span></div>
  <section class="card" aria-label="構成比の一覧">
    <div class="hrow"><h2>銘柄別</h2></div>
    <div class="sum"><span class="state-note num">評価額の比率・合計 ¥28,593,148・保有全体</span></div>
""" + COMP_M_BAR + COMP_ROWS_M + """
  </section>
""" + COMP_DIV_SECTION_M + """
""" + nav_m("資産管理"))

comp_p = (pc_top("資産管理") + """
  <a class="backlink" href="#">‹ 資産管理に戻る</a>
  <h1 class="pagetitle">構成比</h1>
  <div class="basisline" role="group" aria-label="構成比の基準"><span class="lbl">基準:</span><span class="cur">評価額</span><span class="sep">｜</span><span class="opt">年間配当(税引前)</span></div>
  <div class="tabs"><span class="here">銘柄別 3</span><span>商品別 2</span><span>口座別 2</span></div>
  <div class="grid2" style="margin-top:12px">
    <section class="card" aria-label="構成比の一覧">
      <div class="hrow"><h2>銘柄別</h2></div>
      <div class="sum"><span class="state-note num">評価額の比率・合計 ¥28,593,148・保有全体</span></div>
""" + COMP_M_BAR + COMP_ROWS_M + """
    </section>
    <div class="col">
""" + COMP_DIV_SECTION_P + """
    </div>
  </div>""")

# ---------- 保有ファンド詳細 ----------
FUND_DL = """<dl style="margin:12px 0 0">
      <div class="dl"><dt>評価額</dt><dd class="num">¥19,443,341</dd></div>
      <div class="dl"><dt>評価損益</dt><dd class="num">¥0 0.00%</dd></div>
      <div class="dl"><dt>基準価額</dt><dd class="num">¥19,443</dd></div>
      <div class="dl"><dt>基準価額(前日比)</dt><dd class="num up">+¥154 +0.80%</dd></div>
      <div class="dl"><dt>基準価額(前月比)</dt><dd class="num">—</dd></div>
      <div class="dl"><dt>基準価額の基準日</dt><dd class="num">2026/11/13</dd></div>
      <div class="dl"><dt>保有口数</dt><dd class="num">10,000,000口</dd></div>
      <div class="dl"><dt>平均取得価額</dt><dd class="num">¥19,443</dd></div>
      <div class="dl"><dt>取得総額</dt><dd class="num">¥19,443,341</dd></div>
      <div class="dl"><dt>トータルリターン</dt><dd class="num">¥0 0.00%</dd></div>
    </dl>"""
FUND_STATES = """<section class="card" aria-label="状態の例">
    <div class="hrow"><h2>状態の例</h2></div>
    <p class="subhead">空</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">ファンドが見つかりません</div></div></div></div>
    <p class="subhead">読み込み中</p>
    <div class="rows"><div class="row"><div class="left"><div class="sk" style="width:60%"></div></div><span class="amt"><span class="sk" style="display:inline-block;width:90px">&nbsp;</span></span></div></div>
    <p class="subhead">取得失敗</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">エラー: 読み込みに失敗しました</div></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>
    <p class="subhead">更新待ちの例</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">基準価額(前日比)</div><span class="sub">更新待ち: 最新は11/13・この値は11/12</span></div><span class="amt num">更新待ち</span></div></div>
  </section>"""

fund_m = (mobile_top() + """
  <a class="backlink" href="#">‹ 資産管理に戻る</a>
  <h1 class="pagetitle">保有ファンド詳細</h1>
  <section class="card" aria-label="ファンドの明細" style="margin-top:12px">
    <div class="hrow"><h2>架空成長インデックス・ファンド</h2></div>
    <div class="sum"><span class="state-note">特定口座</span><span class="state-note num">10,000,000口</span></div>
""" + FUND_DL + """
  </section>
""" + nav_m("資産管理"))

fund_p = (pc_top("資産管理") + """
  <a class="backlink" href="#">‹ 資産管理に戻る</a>
  <h1 class="pagetitle">保有ファンド詳細</h1>
  <div class="grid2" style="margin-top:12px">
    <section class="card" aria-label="ファンドの明細">
      <div class="hrow"><h2>架空成長インデックス・ファンド</h2></div>
      <div class="sum"><span class="state-note">特定口座</span><span class="state-note num">10,000,000口</span></div>
""" + FUND_DL + """
    </section>
    <div class="col">
    </div>
  </div>""")

# ---------- 取引明細 ----------
RECEIPT_TITLE_M = """<div class="hrow" style="margin-top:12px"><h1 class="pagetitle">取引明細</h1><span class="tlinks"><span class="more">CSV管理 ›</span></span></div>"""
RECEIPT_TITLE_P = """<div class="hrow" style="margin-top:12px"><h1 class="pagetitle">取引明細</h1><span class="tlinks"><span class="more">CSV管理 ›</span></span></div>"""

RECEIPT_MONTHS = [
    ("2026年11月", [("11/10", "9901", "架空青空電機", "特定口座", "100株", 56250, 11427, 44823), ("11/08", "9902", "架空食品ホールディングス", "特定口座", "100株", 40000, 8126, 31874)]),
    ("2026年10月", [("10/20", "9901", "架空青空電機", "特定口座", "100株", 10000, 2031, 7969)]),
    ("2026年9月", [("09/26", "9902", "架空食品ホールディングス", "特定口座", "100株", 5000, 1016, 3984), ("09/10", "9901", "架空青空電機", "特定口座", "100株", 10000, 2031, 7969)]),
    ("2026年8月", [("08/19", "9901", "架空青空電機", "特定口座", "100株", 7000, 1422, 5578)]),
    ("2026年7月", [("07/18", "9901", "架空青空電機", "特定口座", "100株", 9000, 1828, 7172)]),
    ("2026年6月", [("06/29", "—", "架空成長インデックス・ファンド", "特定口座", "49,070口", 4907, 997, 3910), ("06/25", "9901", "架空青空電機", "特定口座", "100株", 60000, 12189, 47811), ("06/20", "9902", "架空食品ホールディングス", "特定口座", "100株", 3500000, 711009, 2788991)]),
    ("2026年5月", [("05/20", "9901", "架空青空電機", "特定口座", "100株", 13000, 2641, 10359)]),
    ("2026年4月", [("04/18", "9901", "架空青空電機", "特定口座", "100株", 11000, 2235, 8765)]),
    ("2026年3月", [("03/28", "9901", "架空青空電機", "特定口座", "100株", 60000, 12189, 47811), ("03/15", "—", "架空成長インデックス・ファンド", "特定口座", "1,000,000口", 120000, 24377, 95623)]),
    ("2026年2月", [("02/19", "9901", "架空青空電機", "特定口座", "100株", 8000, 1625, 6375)]),
    ("2026年1月", [("01/20", "9901", "架空青空電機", "特定口座", "100株", 12000, 2438, 9562)]),
]


def receipt_row_m(date, code, name, account, qty, at):
    disp = name if code == "—" else code + " " + name
    return '<div class="row"><div class="left"><div class="name">' + disp + '</div><span class="sub num">' + date + ' ・ ' + account + ' ・ ' + qty + '</span></div><span class="amt num up">+¥' + f"{at:,}" + '</span></div>'


def receipt_month_m(label, txns):
    tot = sum(t[7] for t in txns)
    rows = "\n      ".join(receipt_row_m(t[0], t[1], t[2], t[3], t[4], t[7]) for t in txns)
    return '<div class="mblock">\n    <div class="hrow"><h2>' + label + ' ・ ' + str(len(txns)) + '件</h2><span class="num up" style="font-size:13px; white-space:nowrap">受取 +¥' + f"{tot:,}" + '</span></div>\n    <div class="rows loose">\n      ' + rows + '\n    </div>\n  </div>'


def receipt_row_p(t):
    date, code, name, account, qty, bt, tax, at = t
    nm = name + " ›" if code == "—" else name
    return '<tr><td class="num">' + date + '</td><td class="num">' + code + '</td><td><div class="nm">' + nm + '</div></td><td><span class="sb">' + account + '</span></td><td class="r num">' + qty + '</td><td class="r num up">+¥' + f"{bt:,}" + '</td><td class="r num down">-¥' + f"{tax:,}" + '</td><td class="r num up">+¥' + f"{at:,}" + '</td></tr>'


def receipt_mrow_p(label, txns):
    tot_bt = sum(t[5] for t in txns)
    tot_tax = sum(t[6] for t in txns)
    tot_at = sum(t[7] for t in txns)
    return '<tr class="mrow"><td colspan="5"><span class="mt">' + label + ' ・ ' + str(len(txns)) + '件</span></td><td class="r"><span class="ms num up">+¥' + f"{tot_bt:,}" + '</span></td><td class="r"><span class="ms num down">-¥' + f"{tot_tax:,}" + '</span></td><td class="r"><span class="ms num up">+¥' + f"{tot_at:,}" + '</span></td></tr>'


def receipt_table_p():
    out = ['<table class="grid" style="margin-top:8px">',
           '      <tr><th class="sortable">入金日 ▼</th><th>銘柄コード</th><th>銘柄</th><th>口座</th><th class="r">数量</th><th class="r">配当金(税引前)</th><th class="r">税額</th><th class="r">税引後</th></tr>']
    for label, txns in RECEIPT_MONTHS:
        out.append("      " + receipt_mrow_p(label, txns))
        for t in txns:
            out.append("      " + receipt_row_p(t))
    out.append("    </table>")
    return "\n".join(out)


RECEIPT_TABLE_P = receipt_table_p()
RECEIPT_MONTHS_M = "\n".join(receipt_month_m(label, txns) for label, txns in RECEIPT_MONTHS)
RECEIPT_NOV_M = receipt_month_m(RECEIPT_MONTHS[0][0], RECEIPT_MONTHS[0][1])
RECEIPT_NOV_OPEN_M = (
    '<div class="mblock">\n    <div class="hrow"><h2>2026年11月 ・ 2件</h2><span class="num up" style="font-size:13px; white-space:nowrap">受取 +¥76,697</span></div>\n'
    '    <div class="rows loose">\n      '
    + receipt_row_m("11/10", "9901", "架空青空電機", "特定口座", "100株", 44823)
    + '\n      <div class="row"><div class="left"><div class="name">9902 架空食品ホールディングス</div><span class="sub num">11/08 ・ 特定口座 ・ 100株</span></div><span class="amt num up">+¥31,874</span></div>\n    </div>\n'
    '    <dl style="margin:4px 0 0; border-top:1px solid var(--line); padding-top:4px">\n'
    '      <div class="dl"><dt>配当金(税引前)</dt><dd class="num up">+¥40,000</dd></div>\n'
    '      <div class="dl"><dt>税額</dt><dd class="num down">-¥8,126</dd></div>\n'
    '      <div class="dl"><dt>税引後</dt><dd class="num up">+¥31,874</dd></div>\n    </dl>\n  </div>')

receipt_m = (mobile_top() + RECEIPT_TITLE_M + """
  <div class="hero" style="padding-bottom:0">
    <p class="label">受け取った配当金(税引後)</p>
    <p class="total num">¥3,128,576</p>
    <p class="date num">2026/01 〜 2026/11 ・ 16件</p>
  </div>
  <div class="tabs"><span class="here">配当金 16</span><span>国内株式 3</span><span>投資信託 2</span></div>
  <div class="search"><span aria-hidden="true">⌕</span><span class="ph">銘柄名・コードで検索</span></div>
  <div class="sortbtn"><span>並べ替え: 入金日の新しい順</span><span aria-hidden="true">›</span></div>
  <div class="countrow"><span class="n num">16件</span></div>
""" + RECEIPT_MONTHS_M + nav_m("取引明細"))

receipt_p = (pc_top("取引明細") + RECEIPT_TITLE_P + """
  <div class="hero">
    <div class="stat"><p class="label">受け取った配当金(税引後)</p><p class="v1 num">¥3,128,576</p></div>
    <div class="stat"><p class="label">配当金(税引前)</p><p class="v2 num up">+¥3,926,157</p></div>
    <div class="stat"><p class="label">税額</p><p class="v2 num down">-¥797,581</p></div>
    <div class="stat"><p class="label">期間</p><p class="v2 num">2026/01 〜 2026/11</p></div>
    <div class="dates num">16件</div>
  </div>
  <div class="tabs"><span class="here">配当金 16</span><span>国内株式 3</span><span>投資信託 2</span></div>
  <div class="search"><span aria-hidden="true">⌕</span><span class="ph">銘柄名・コードで検索</span></div>
  <div class="countrow"><span class="n num">16件(列名クリックで並べ替え)</span></div>
""" + RECEIPT_TABLE_P)

# ---------- CSV管理 ----------
CSV_ROWS = """<div class="rows">
      <div class="row"><div class="left"><div class="name">資産管理(国内株式)</div><span class="sub num" style="color:var(--up)">今日 09:12 に更新</span></div><span class="amt num">2件</span></div>
      <div class="row"><div class="left"><div class="name">資産管理(投資信託)</div><span class="sub num">2日前(11/12 18:04)に更新</span></div><span class="amt num">1件</span></div>
      <div class="row"><div class="left"><div class="name">取引明細(配当金)</div><span class="sub num">3日前(11/11 20:44)に更新</span></div><span class="amt num">18件</span></div>
      <div class="row"><div class="left"><div class="name">取引明細(国内株式)</div><span class="sub num" style="color:var(--down)">取得できず・タブを再読み込みしてください</span></div><span class="amt num">3件</span></div>
      <div class="row"><div class="left"><div class="name">取引明細(投資信託)</div><span class="sub num">4日前(11/10 18:20)に更新</span></div><span class="amt num">2件</span></div>
    </div>"""
CSV_MENU = """<section class="card" aria-label="行メニューの例">
    <div class="hrow"><h2>行メニューの例</h2><span class="more">⋯</span></div>
    <div class="rows">
      <div class="row"><div class="left"><div class="name">楽天証券から再取得</div></div></div>
      <div class="row"><div class="left"><div class="name">ファイルから取り込み</div></div></div>
      <div class="row"><div class="left"><div class="name" style="color:var(--down)">削除</div></div></div>
    </div>
  </section>"""
CSV_DIALOG = """<section class="card" aria-label="削除確認の例">
    <div class="hrow"><h2>取引明細(配当金)を削除しますか?</h2></div>
    <p class="statenote-wrap">取引明細(配当金)の保存済み 18件を削除します。元に戻せません</p>
    <div style="display:flex; gap:12px; margin-top:12px">
      <div class="btn" style="flex:1">キャンセル</div>
      <div class="btn danger" style="flex:1">18件を削除する</div>
    </div>
  </section>"""
CSV_STATES = """<section class="card" aria-label="状態の例">
    <div class="hrow"><h2>状態の例</h2></div>
    <p class="subhead">空</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">データがありません</div><span class="sub">楽天証券から取得すると表示されます</span></div></div></div>
    <p class="subhead">読み込み中</p>
    <div class="rows">
      <div class="row"><div class="left"><div class="name">資産管理(国内株式)</div><span class="sub">取得中…</span></div><span class="amt num">2件</span></div>
      <div class="row"><div class="left"><div class="name">取引明細(配当金)</div><span class="sub">取り込み中…</span></div><span class="amt num">18件</span></div>
      <div class="row"><div class="left"><div class="name">取引明細(投資信託)</div><span class="sub">確認待ち</span></div><span class="amt num">2件</span></div>
    </div>
    <p class="subhead">取得失敗</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">エラー: 拡張機能との通信が切れました</div><span class="sub">ネットワーク接続を確認してください</span></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>
  </section>"""

csv_m = (mobile_top() + """
  <a class="backlink" href="#">‹ 資産管理に戻る</a>
  <h1 class="pagetitle">CSV管理</h1>
  <section class="card" aria-label="最新のデータの取得" style="margin-top:12px">
    <div class="hrow"><h2>楽天証券から最新のデータを取得</h2></div>
    <div class="btn primary" style="margin-top:12px">すべて取得</div>
  </section>
  <section class="card" aria-label="データの種類の一覧">
    <div class="hrow"><h2>データの種類</h2></div>
""" + CSV_ROWS + """
  </section>
""" + nav_m("資産管理"))

CSV_TABLE_P = """<table class="grid" style="margin-top:8px">
          <tr><th>データの種類</th><th class="r">保存済み</th><th>状態</th><th class="r">⋯</th></tr>
          <tr><td><div class="nm">資産管理(国内株式)</div></td><td class="r num">2件</td><td><span class="sb" style="color:var(--up)">今日 09:12 に更新</span></td><td class="r">⋯</td></tr>
          <tr><td><div class="nm">資産管理(投資信託)</div></td><td class="r num">1件</td><td><span class="sb">2日前(11/12 18:04)に更新</span></td><td class="r">⋯</td></tr>
          <tr><td><div class="nm">取引明細(配当金)</div></td><td class="r num">18件</td><td><span class="sb">3日前(11/11 20:44)に更新</span></td><td class="r">⋯</td></tr>
          <tr><td><div class="nm">取引明細(国内株式)</div></td><td class="r num">3件</td><td><span class="sb" style="color:var(--down)">取得できず・タブを再読み込みしてください</span></td><td class="r">⋯</td></tr>
          <tr><td><div class="nm">取引明細(投資信託)</div></td><td class="r num">2件</td><td><span class="sb">4日前(11/10 18:20)に更新</span></td><td class="r">⋯</td></tr>
        </table>"""

csv_p = (pc_top("資産管理") + """
  <a class="backlink" href="#">‹ 資産管理に戻る</a>
  <h1 class="pagetitle">CSV管理</h1>
  <div class="grid2" style="margin-top:12px">
    <div class="col">
      <section class="card" aria-label="最新のデータの取得">
        <div class="hrow"><h2>楽天証券から最新のデータを取得</h2><span class="more"></span></div>
        <div class="btn primary" style="margin-top:12px; max-width:320px">すべて取得</div>
      </section>
      <section class="card" aria-label="データの種類の一覧">
        <div class="hrow"><h2>データの種類</h2></div>
""" + CSV_TABLE_P + """
      </section>
    </div>
    <div class="col">
    </div>
  </div>""")

write("assetbalance-mobile", "資産管理 新案 スマホ", True, asset_m)
write("assetbalance-pc", "資産管理 新案 PC", False, asset_p)
write("composition-mobile", "構成比 新案 スマホ", True, comp_m)
write("composition-pc", "構成比 新案 PC", False, comp_p)
write("fund-detail-mobile", "保有ファンド詳細 新案 スマホ", True, fund_m)
write("fund-detail-pc", "保有ファンド詳細 新案 PC", False, fund_p)
write("receipts-mobile", "取引明細 新案 スマホ", True, receipt_m)
write("receipts-pc", "取引明細 新案 PC", False, receipt_p)
write("csv-mobile", "CSV管理 新案 スマホ", True, csv_m)
write("csv-pc", "CSV管理 新案 PC", False, csv_p)

OVL_M = """.ovl{position:fixed; inset:0; background:rgba(17,24,39,.45); z-index:50; display:flex; align-items:flex-end; justify-content:center}
.ovl .sheet{width:100%; max-width:480px; margin:0 auto; background:var(--page); border-radius:16px 16px 0 0; padding:8px 8px calc(16px + env(safe-area-inset-bottom)); max-height:88vh; overflow:auto}
.ovl .sheet .card{margin-top:8px}"""
OVL_P = """.ovl{position:fixed; inset:0; background:rgba(17,24,39,.45); z-index:50; display:flex; align-items:center; justify-content:center; padding:24px}
.ovl .sheet{width:min(560px,100%); max-height:calc(100vh - 48px); overflow:auto}
.ovl .sheet .card{margin-top:0}"""

def ovl(dialog, label):
    return '<div class="ovl" role="dialog" aria-modal="true" aria-label="' + label + '"><div class="sheet">' + dialog + "</div></div>"

ASSET_EMPTY = """<div class="rows">
      <div class="row"><div class="left"><div class="name">資産管理データがありません</div><span class="sub">CSVファイルを取り込むと表示されます</span></div></div></div>"""
ASSET_LOADING = """<div class="rows">
      <div class="row"><div class="left"><div class="sk" style="width:60%"></div><div class="sub"><span class="sk" style="display:inline-block;width:40%">&nbsp;</span></div></div><span class="amt"><span class="sk" style="display:inline-block;width:90px">&nbsp;</span></span></div>
      <div class="row"><div class="left"><div class="sk" style="width:50%"></div><div class="sub"><span class="sk" style="display:inline-block;width:30%">&nbsp;</span></div></div><span class="amt"><span class="sk" style="display:inline-block;width:90px">&nbsp;</span></span></div></div>"""
ASSET_FAILED = """<div class="rows"><div class="row"><div class="left"><div class="name">エラー: 読み込みに失敗しました</div><span class="sub">時間をおいて再度お試しください</span></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>"""

COMP_EMPTY = """<div class="rows"><div class="row"><div class="left"><div class="name">データがありません</div><span class="sub">保有残高をCSVで登録すると表示されます</span></div></div></div>"""
COMP_LOADING = """<div class="rows"><div class="row"><div class="left"><div class="sk" style="width:60%"></div></div><span class="amt"><span class="sk" style="display:inline-block;width:90px">&nbsp;</span></span></div></div>"""
COMP_FAILED = """<div class="rows"><div class="row"><div class="left"><div class="name">エラー: 読み込みに失敗しました</div></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>"""
COMP_UNCALC = """<div class="rows"><div class="row"><div class="left"><div class="name">算出不可</div><span class="sub">合計が0のため割合を出せません</span></div><span class="amt num">—</span></div></div>"""

FUND_EMPTY = """<div class="rows"><div class="row"><div class="left"><div class="name">ファンドが見つかりません</div></div></div></div>"""
FUND_LOADING = """<div class="rows"><div class="row"><div class="left"><div class="sk" style="width:60%"></div></div><span class="amt"><span class="sk" style="display:inline-block;width:90px">&nbsp;</span></span></div></div>"""
FUND_FAILED = """<div class="rows"><div class="row"><div class="left"><div class="name">エラー: 読み込みに失敗しました</div></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>"""
FUND_WAITING = FUND_DL.replace(
    '<div class="dl"><dt>基準価額(前日比)</dt><dd class="num up">+¥154 +0.80%</dd></div>',
    '<div class="dl"><dt>基準価額(前日比)</dt><dd class="num">更新待ち</dd></div>'
) + '<p class="statenote-wrap">更新待ち: 最新は11/13・この値は11/12</p>'

RECEIPT_EMPTY = """<section class="card" aria-label="取引明細の一覧">
    <div class="hrow"><h2>配当金の一覧</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="name">データがありません</div><span class="sub">配当金明細をCSVで追加してください</span></div></div></div>
  </section>"""
RECEIPT_LOADING = """<section class="card" aria-label="取引明細の一覧">
    <div class="hrow"><h2>配当金の一覧</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="sk" style="width:60%"></div></div><span class="amt"><span class="sk" style="display:inline-block;width:90px">&nbsp;</span></span></div></div>
  </section>"""
RECEIPT_FAILED = """<section class="card" aria-label="取引明細の一覧">
    <div class="hrow"><h2>配当金の一覧</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="name">エラー: 読み込みに失敗しました</div></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>
  </section>"""
CSV_EMPTY = """<div class="rows"><div class="row"><div class="left"><div class="name">データがありません</div><span class="sub">楽天証券から取得すると表示されます</span></div></div></div>"""
CSV_LOADING = """<div class="rows">
      <div class="row"><div class="left"><div class="name">資産管理(国内株式)</div><span class="sub">取得中…</span></div><span class="amt num">2件</span></div>
      <div class="row"><div class="left"><div class="name">取引明細(配当金)</div><span class="sub">取り込み中…</span></div><span class="amt num">18件</span></div>
      <div class="row"><div class="left"><div class="name">取引明細(投資信託)</div><span class="sub">確認待ち</span></div><span class="amt num">2件</span></div>
    </div>"""
CSV_FAILED = """<div class="rows"><div class="row"><div class="left"><div class="name">エラー: 拡張機能との通信が切れました</div><span class="sub">ネットワーク接続を確認してください</span></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>"""

write("assetbalance-state-empty-mobile", "資産管理(データなし) 新案 スマホ", True, asset_m.replace(ASSET_ROWS_M, ASSET_EMPTY))
write("assetbalance-state-empty-pc", "資産管理(データなし) 新案 PC", False, asset_p.replace(ASSET_TABLE_P, ASSET_EMPTY))
write("assetbalance-state-loading-mobile", "資産管理(読み込み中) 新案 スマホ", True, asset_m.replace(ASSET_ROWS_M, ASSET_LOADING))
write("assetbalance-state-loading-pc", "資産管理(読み込み中) 新案 PC", False, asset_p.replace(ASSET_TABLE_P, ASSET_LOADING))
write("assetbalance-state-failed-mobile", "資産管理(読み込み失敗) 新案 スマホ", True, asset_m.replace(ASSET_ROWS_M, ASSET_FAILED))
write("assetbalance-state-failed-pc", "資産管理(読み込み失敗) 新案 PC", False, asset_p.replace(ASSET_TABLE_P, ASSET_FAILED))
COMP_DIV_EMPTY_M = """  <section class="card" aria-label="基準を年間配当に替えた例">
    <div class="hrow"><h2>年間配当(税引前)</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="name">データがありません</div><span class="sub">保有残高をCSVで登録すると表示されます</span></div></div></div>
  </section>"""
COMP_DIV_EMPTY_P = """      <section class="card" aria-label="基準を年間配当に替えた例">
        <div class="hrow"><h2>年間配当(税引前)</h2></div>
        <div class="rows"><div class="row"><div class="left"><div class="name">データがありません</div><span class="sub">保有残高をCSVで登録すると表示されます</span></div></div></div>
      </section>"""

write("composition-state-empty-mobile", "構成比(データなし) 新案 スマホ", True, comp_m.replace(COMP_M_BAR + COMP_ROWS_M, COMP_EMPTY).replace(COMP_DIV_SECTION_M, COMP_DIV_EMPTY_M))
write("composition-state-empty-pc", "構成比(データなし) 新案 PC", False, comp_p.replace(COMP_M_BAR + COMP_ROWS_M, COMP_EMPTY).replace(COMP_DIV_SECTION_P, COMP_DIV_EMPTY_P))
write("composition-state-loading-mobile", "構成比(読み込み中) 新案 スマホ", True, comp_m.replace(COMP_M_BAR + COMP_ROWS_M, COMP_LOADING).replace(COMP_DIV_ROWS, COMP_LOADING))
write("composition-state-loading-pc", "構成比(読み込み中) 新案 PC", False, comp_p.replace(COMP_M_BAR + COMP_ROWS_M, COMP_LOADING).replace(COMP_DIV_ROWS, COMP_LOADING))
write("composition-state-failed-mobile", "構成比(読み込み失敗) 新案 スマホ", True, comp_m.replace(COMP_M_BAR + COMP_ROWS_M, COMP_FAILED).replace(COMP_DIV_ROWS, COMP_FAILED))
write("composition-state-failed-pc", "構成比(読み込み失敗) 新案 PC", False, comp_p.replace(COMP_M_BAR + COMP_ROWS_M, COMP_FAILED).replace(COMP_DIV_ROWS, COMP_FAILED))
write("composition-state-uncalculable-mobile", "構成比(算出不可) 新案 スマホ", True, comp_m.replace(COMP_M_BAR + COMP_ROWS_M, COMP_UNCALC).replace(COMP_DIV_ROWS, COMP_UNCALC))
write("composition-state-uncalculable-pc", "構成比(算出不可) 新案 PC", False, comp_p.replace(COMP_M_BAR + COMP_ROWS_M, COMP_UNCALC).replace(COMP_DIV_ROWS, COMP_UNCALC))
write("fund-detail-state-empty-mobile", "保有ファンド詳細(見つからない) 新案 スマホ", True, fund_m.replace(FUND_DL, FUND_EMPTY))
write("fund-detail-state-empty-pc", "保有ファンド詳細(見つからない) 新案 PC", False, fund_p.replace(FUND_DL, FUND_EMPTY))
write("fund-detail-state-loading-mobile", "保有ファンド詳細(読み込み中) 新案 スマホ", True, fund_m.replace(FUND_DL, FUND_LOADING))
write("fund-detail-state-loading-pc", "保有ファンド詳細(読み込み中) 新案 PC", False, fund_p.replace(FUND_DL, FUND_LOADING))
write("fund-detail-state-failed-mobile", "保有ファンド詳細(読み込み失敗) 新案 スマホ", True, fund_m.replace(FUND_DL, FUND_FAILED))
write("fund-detail-state-failed-pc", "保有ファンド詳細(読み込み失敗) 新案 PC", False, fund_p.replace(FUND_DL, FUND_FAILED))
write("fund-detail-state-waiting-mobile", "保有ファンド詳細(更新待ち) 新案 スマホ", True, fund_m.replace(FUND_DL, FUND_WAITING))
write("fund-detail-state-waiting-pc", "保有ファンド詳細(更新待ち) 新案 PC", False, fund_p.replace(FUND_DL, FUND_WAITING))
write("receipts-state-empty-mobile", "取引明細(データなし) 新案 スマホ", True, receipt_m.replace(RECEIPT_MONTHS_M, RECEIPT_EMPTY))
write("receipts-state-empty-pc", "取引明細(データなし) 新案 PC", False, receipt_p.replace(RECEIPT_TABLE_P, RECEIPT_EMPTY))
write("receipts-state-loading-mobile", "取引明細(読み込み中) 新案 スマホ", True, receipt_m.replace(RECEIPT_MONTHS_M, RECEIPT_LOADING))
write("receipts-state-loading-pc", "取引明細(読み込み中) 新案 PC", False, receipt_p.replace(RECEIPT_TABLE_P, RECEIPT_LOADING))
write("receipts-state-failed-mobile", "取引明細(読み込み失敗) 新案 スマホ", True, receipt_m.replace(RECEIPT_MONTHS_M, RECEIPT_FAILED))
write("receipts-state-failed-pc", "取引明細(読み込み失敗) 新案 PC", False, receipt_p.replace(RECEIPT_TABLE_P, RECEIPT_FAILED))
write("receipts-state-detail-mobile", "取引明細(行詳細を開いた例) 新案 スマホ", True, receipt_m.replace(RECEIPT_NOV_M, RECEIPT_NOV_OPEN_M))
write("receipts-state-detail-pc", "取引明細(行詳細を開いた例) 新案 PC", False, receipt_p.replace(RECEIPT_TABLE_P, RECEIPT_TABLE_P + RECEIPT_NOV_OPEN_M))
write("csv-state-empty-mobile", "CSV管理(データなし) 新案 スマホ", True, csv_m.replace(CSV_ROWS, CSV_EMPTY))
write("csv-state-empty-pc", "CSV管理(データなし) 新案 PC", False, csv_p.replace(CSV_TABLE_P, CSV_EMPTY))
write("csv-state-loading-mobile", "CSV管理(取得中) 新案 スマホ", True, csv_m.replace(CSV_ROWS, CSV_LOADING))
write("csv-state-loading-pc", "CSV管理(取得中) 新案 PC", False, csv_p.replace(CSV_TABLE_P, CSV_LOADING))
write("csv-state-failed-mobile", "CSV管理(取得失敗) 新案 スマホ", True, csv_m.replace(CSV_ROWS, CSV_FAILED))
write("csv-state-failed-pc", "CSV管理(取得失敗) 新案 PC", False, csv_p.replace(CSV_TABLE_P, CSV_FAILED))
write("csv-state-menu-mobile", "CSV管理(行メニュー) 新案 スマホ", True, csv_m + ovl(CSV_MENU, "行メニュー"), OVL_M)
write("csv-state-menu-pc", "CSV管理(行メニュー) 新案 PC", False, csv_p + ovl(CSV_MENU, "行メニュー"), OVL_P)
write("csv-state-delete-mobile", "CSV管理(削除確認) 新案 スマホ", True, csv_m + ovl(CSV_DIALOG, "削除確認"), OVL_M)
write("csv-state-delete-pc", "CSV管理(削除確認) 新案 PC", False, csv_p + ovl(CSV_DIALOG, "削除確認"), OVL_P)
