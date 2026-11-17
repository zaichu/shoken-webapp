#!/usr/bin/env python3
"""Phase9 rest担当分の新案HTMLを生成する。旧見本用の生成スクリプト。
数値は ../phase5-codex/data/fixtures.json の架空データを直書き。
対象: search / settings / login / loginprompt(未ログイン時ホーム) / notfound(404) /
admin-logins / dialogs(9種)。各 *-mobile(390px) / *-pc(1280px)。"""
from pathlib import Path
import re

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
.tabs{display:flex; gap:4px; border-bottom:1px solid var(--line); margin-top:12px}
.tabs span{flex:1; text-align:center; white-space:nowrap; color:var(--mut); padding:12px 0 10px; min-height:44px}
.tabs .here{color:var(--navy); font-weight:700; border-bottom:2px solid var(--gold)}
.search{display:flex; align-items:center; gap:8px; border:1px solid var(--line); border-radius:8px; padding:0 12px; min-height:44px; margin-top:12px; color:var(--mut); background:var(--card)}
.search .ph{font-size:14px; white-space:nowrap; overflow:hidden; text-overflow:ellipsis}
.sortbtn{display:flex; align-items:center; justify-content:space-between; gap:8px; border:1px solid var(--line); border-radius:8px; padding:0 12px; min-height:44px; margin-top:8px; font-size:14px; background:var(--card); white-space:nowrap}
.countrow{display:flex; justify-content:space-between; align-items:center; gap:12px; margin:12px 0 0}
.countrow .n{font-size:13px; color:var(--mut); white-space:nowrap}
.countrow .link{font-size:13px; color:var(--accent); white-space:nowrap; min-height:44px; display:flex; align-items:center}
.bar100{display:flex; height:8px; margin:14px 0 6px; overflow:hidden; border-radius:4px; background:var(--seg)}
.bar100 .a{background:var(--comp-a)} .bar100 .b{background:var(--comp-b)} .bar100 .c{background:var(--mut)}
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
.chips{display:flex; flex-wrap:wrap; gap:8px; margin-top:12px}
.chip{display:inline-flex; align-items:center; gap:6px; border:1px solid var(--line); border-radius:999px; padding:8px 12px; min-height:44px; font-size:13px; white-space:nowrap}
.chip .x{color:var(--mut)}
.field{display:flex; align-items:center; gap:8px; border:1px solid var(--line); border-radius:8px; padding:0 12px; min-height:48px; margin-top:12px; font-size:14px; color:var(--mut); background:var(--card); white-space:nowrap; overflow:hidden; text-overflow:ellipsis}
.badge{display:inline-block; border:1px solid var(--line); border-radius:6px; padding:2px 8px; font-size:12px; color:var(--mut); white-space:nowrap}
.badge.ok{color:var(--up); border-color:var(--up)}
.badge.ng{color:var(--down); border-color:var(--down)}
.linkgrid{display:grid; grid-template-columns:1fr 1fr; gap:8px; margin-top:12px}
.linkgrid span{border:1px solid var(--line); border-radius:8px; min-height:44px; display:flex; align-items:center; justify-content:center; font-size:13px; color:var(--accent); white-space:nowrap}
.center{max-width:480px; margin:0 auto; padding:48px 0 24px}
.toc{display:flex; flex-wrap:wrap; gap:8px; margin-top:12px}
"""

MOBILE_EXTRA = """.top{border-bottom:1px solid var(--line)}
.top .inner{max-width:480px; margin:0 auto; padding:0 16px; display:flex; align-items:center; justify-content:space-between; min-height:52px; gap:12px}
.brand{font-size:15px; font-weight:700; color:var(--navy); white-space:nowrap}
.account{font-size:13px; color:var(--mut); white-space:nowrap; min-height:44px; display:flex; align-items:center}
.wrap{max-width:480px; margin:0 auto; padding:0 16px 120px}
.wrap.nonavi{padding-bottom:48px}
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
.linkgrid{grid-template-columns:1fr 1fr 1fr 1fr}
@media (max-width:900px){ .grid2{grid-template-columns:1fr} .hero{flex-wrap:wrap; gap:16px} .stat + .stat{border-left:none; padding-left:0} .hero .dates{margin-left:0; text-align:left} }
"""


def nav_m(here):
    items = ["ホーム", "資産管理", "取引明細", "銘柄検索"]
    icons = {
        "ホーム": '<path d="M4 11l8-7 8 7"/><path d="M6 10v10h12V10"/>',
        "資産管理": '<path d="M4 20V10"/><path d="M10 20V4"/><path d="M16 20v-8"/><path d="M22 20H2"/>',
        "取引明細": '<path d="M6 3h12v18H6z"/><path d="M9 8h6M9 12h6M9 16h4"/>',
        "銘柄検索": '<circle cx="11" cy="11" r="7"/><path d="M16.5 16.5L21 21"/>',
    }
    out = ['<nav class="bottomnav" aria-label="下ナビ">']
    for label in items:
        cls = ' class="here"' if label == here else ""
        out.append(f'  <a{cls} href="#"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">{icons[label]}</svg>{label}</a>')
    out.append("</nav>")
    return "\n".join(out)


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


def mobile_top(account="アカウント"):
    acc = f'<span class="account">{account}</span>' if account else ""
    return ('<header class="top"><div class="inner">\n'
            '  <span class="brand">証券Web</span>' + acc + '\n'
            '</div></header>\n<div class="wrap">\n')


def mobile_top_nonavi():
    return ('<header class="top"><div class="inner">\n'
            '  <span class="brand">証券Web</span>\n'
            '</div></header>\n<div class="wrap nonavi">\n')


def pc_top(here):
    items = ["ホーム", "資産管理", "取引明細", "銘柄検索"]
    nav = "".join(f'<span class="here">{i}</span>' if i == here else f"<span>{i}</span>" for i in items)
    return ('<header class="top"><div class="inner">\n'
            f'  <span class="brand">証券Web</span>\n  <nav>{nav}</nav>\n'
            '  <span class="account">アカウント</span>\n</div></header>\n<div class="wrap">\n')


def pc_top_nonavi():
    return ('<header class="top"><div class="inner">\n'
            '  <span class="brand">証券Web</span>\n'
            '</div></header>\n<div class="wrap">\n')


def write(name, title, body, css=""):
    is_mobile = name.endswith("-mobile")
    extra = (MOBILE_EXTRA if is_mobile else PC_EXTRA) + css
    html = HEAD.format(title=title, vars=CSS_VARS, base=BASE, extra=extra) + body + "</div>\n" + SCRIPT + "\n</body>\n</html>\n"
    (OUT / f"{name}.html").write_text(html, encoding="utf-8")
    print("wrote", name, len(html))

OVL_M = """.ovl{position:fixed; inset:0; background:rgba(17,24,39,.45); z-index:50; display:flex; align-items:flex-end; justify-content:center}
.ovl .sheet{width:100%; max-width:480px; margin:0 auto; background:var(--page); border-radius:16px 16px 0 0; padding:8px 8px calc(16px + env(safe-area-inset-bottom)); max-height:88vh; overflow:auto}
.ovl .sheet .card{margin-top:8px}"""
OVL_P = """.ovl{position:fixed; inset:0; background:rgba(17,24,39,.45); z-index:50; display:flex; align-items:center; justify-content:center; padding:24px}
.ovl .sheet{width:min(560px,100%); max-height:calc(100vh - 48px); overflow:auto}
.ovl .sheet .card{margin-top:0}"""

def ovl(dialog, label):
    return '<div class="ovl" role="dialog" aria-modal="true" aria-label="' + label + '"><div class="sheet">' + dialog + "</div></div>"

FRAG_CSS = """.dl{display:flex; justify-content:space-between; align-items:baseline; gap:12px; padding:9px 0; border-top:1px solid var(--line)}
.dl:first-child{border-top:none}
.dl dt{font-size:14px; color:var(--mut); white-space:nowrap}
.dl dd{margin:0; font-size:15px; font-weight:700; text-align:right; white-space:nowrap}
.field{display:flex; align-items:center; gap:8px; border:1px solid var(--line); border-radius:8px; padding:0 12px; min-height:48px; margin-top:12px; font-size:14px; color:var(--mut); background:var(--card); white-space:nowrap; overflow:hidden; text-overflow:ellipsis}"""


# ---------- 銘柄検索 ----------
SEARCH_FORM = """<div class="search"><span aria-hidden="true">⌕</span><span class="ph">銘柄コードまたは銘柄名で検索</span></div>
  <div class="btn primary" style="margin-top:8px">銘柄を検索</div>"""

HOLDING_CARD = """<section class="card" aria-label="保有情報">
    <div class="hrow"><h2>保有情報</h2></div>
    <p class="subhead">合計</p>
    <dl style="margin:4px 0 0">
      <div class="dl"><dt>評価額</dt><dd class="num">¥1,200,000</dd></div>
      <div class="dl"><dt>評価損益</dt><dd class="num up">+¥200,000 +20.00%</dd></div>
      <div class="dl"><dt>保有数量</dt><dd class="num">100株</dd></div>
      <div class="dl"><dt>取得総額</dt><dd class="num">¥1,000,000</dd></div>
      <div class="dl"><dt>現在値</dt><dd class="num">¥12,000</dd></div>
      <div class="dl"><dt>価格基準日</dt><dd class="num">2026/11/13</dd></div>
      <div class="dl"><dt>1株配当</dt><dd class="num">¥360</dd></div>
      <div class="dl"><dt>年間配当参考額</dt><dd class="num">¥36,000</dd></div>
      <div class="dl"><dt>取得額基準利回り</dt><dd class="num">3.60%</dd></div>
    </dl>
    <p class="subhead">口座別</p>
    <div class="rows">
      <div class="row"><div class="left"><div class="name">特定口座</div><span class="sub num">50株 ・ 取得単価 ¥10,123.46</span></div><span class="amt num">¥600,000</span></div>
      <div class="row"><div class="left"><div class="name">NISA成長口座</div><span class="sub num">50株 ・ 取得単価 ¥9,876.54</span></div><span class="amt num">¥600,000</span></div>
    </div>
  </section>"""

RESULT_CARD = """<section class="card" aria-label="検索結果">
    <div class="hrow"><h2>9901 架空青空電機</h2><span class="more">銘柄情報 ›</span></div>
    <dl style="margin:4px 0 0">
      <div class="dl"><dt>市場</dt><dd>東証プライム</dd></div>
      <div class="dl"><dt>33業種</dt><dd>電気機器</dd></div>
      <div class="dl"><dt>17業種</dt><dd>電機・精密</dd></div>
      <div class="dl"><dt>規模</dt><dd>大型</dd></div>
    </dl>
    <p class="subhead">外部リンク</p>
    <div class="linkgrid">
      <span>楽天証券</span><span>SBI証券</span><span>株探</span><span>日経</span>
      <span>バフェットコード</span><span>みんかぶ</span><span>銘柄スカウター</span><span>ザイマニ</span>
      <span>Yahoo! Finance</span><span>IR BANK</span><span>JPX Explorer</span>
    </div>
  </section>"""

SEARCH_STATES = """<section class="card" aria-label="状態の例">
    <div class="hrow"><h2>状態の例</h2></div>
    <p class="subhead">初期</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">銘柄を検索</div><span class="sub">銘柄コード（例：7203）または銘柄名を入力してください</span></div></div></div>
    <div class="statenote-wrap">4桁の銘柄コードで検索できます（例：7203, 9984）・ 会社の一部でも検索できます（例：トヨタ）・ 検索結果から各種証券サイトへのリンクを確認できます</div>
    <p class="subhead">該当なし</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">該当する銘柄が見つかりませんでした</div><span class="sub">銘柄コードまたは銘柄名を確認してください</span></div></div></div>
    <p class="subhead">読み込み中</p>
    <div class="rows"><div class="row"><div class="left"><div class="sk" style="width:60%"></div></div><span class="amt"><span class="sk" style="display:inline-block;width:90px">&nbsp;</span></span></div></div>
    <p class="subhead">取得失敗</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">エラー: 読み込みに失敗しました</div><span class="sub">ネットワーク接続を確認してください</span></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>
    <p class="subhead">不正なコード</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">不正な銘柄コードが指定されています</div></div></div></div>
  </section>"""

search_m = (mobile_top() + """
  <h1 class="pagetitle" style="margin-top:12px">銘柄検索</h1>
""" + SEARCH_FORM + "\n  " + HOLDING_CARD + "\n  " + RESULT_CARD + nav_m("銘柄検索"))

SEARCH_CARD_P = """<section class="card" aria-label="検索">
        <div class="hrow"><h2>銘柄コードまたは銘柄名で検索</h2></div>
""" + SEARCH_FORM + """
      </section>"""

search_p = (pc_top("銘柄検索") + """
  <h1 class="pagetitle" style="margin-top:12px">銘柄検索</h1>
  <div class="grid2" style="margin-top:12px">
    <div class="col">
""" + SEARCH_CARD_P + "\n" + HOLDING_CARD + "\n" + RESULT_CARD + """
    </div>
  </div>""")

# ---------- 設定 ----------
SETTINGS_TOC = """<section class="card" aria-label="設定セクション">
    <div class="hrow"><h2>設定セクション</h2></div>
    <div class="rows">
      <div class="row"><div class="left"><div class="name" style="color:var(--accent)">アカウント</div></div></div>
      <div class="row"><div class="left"><div class="name" style="color:var(--accent)">表示</div></div></div>
      <div class="row"><div class="left"><div class="name" style="color:var(--accent)">データ</div></div></div>
      <div class="row"><div class="left"><div class="name" style="color:var(--accent)">セキュリティ</div></div></div>
      <div class="row"><div class="left"><div class="name" style="color:var(--accent)">登録した端末</div></div></div>
      <div class="row"><div class="left"><div class="name" style="color:var(--accent)">ログイン履歴</div></div></div>
      <div class="row"><div class="left"><div class="name" style="color:var(--accent)">危険な操作</div></div></div>
    </div>
  </section>"""

SETTINGS_ACCOUNT = """<section class="card" aria-label="アカウント">
    <div class="hrow"><h2>アカウント</h2></div>
    <div class="rows">
      <div class="row"><div class="left"><div class="name">架空 太郎</div><span class="sub">kakuu.taro@example.jp ・ Google でログイン中</span></div></div>
    </div>
  </section>"""

SETTINGS_DISPLAY = """<section class="card" aria-label="表示">
    <div class="hrow"><h2>表示</h2></div>
    <dl style="margin:4px 0 0">
      <div class="dl"><dt>テーマ</dt><dd>端末に合わせる</dd></div>
      <div class="dl"><dt>資産管理の並び順</dt><dd>評価額の高い順</dd></div>
      <div class="dl"><dt>取引明細の初期タブ</dt><dd>配当金</dd></div>
      <div class="dl"><dt>金額の表示単位</dt><dd>円</dd></div>
    </dl>
  </section>"""

SETTINGS_DATA = """<section class="card" aria-label="データ">
    <div class="hrow"><h2>データ</h2></div>
    <div class="rows">
      <div class="row"><div class="left"><div class="name">CSVでエクスポート</div><span class="sub">配当金 ・ 資産残高 ・ 売買履歴</span></div><span class="amt" style="color:var(--accent)">出力</span></div>
      <div class="row"><div class="left"><div class="name">CSVの取り込み・保存済みデータの削除</div><span class="sub">保存済み 26件</span></div><span class="amt" style="color:var(--accent)">CSV管理へ</span></div>
    </div>
    <div class="statenote-wrap">asset-balance-20261114.csv を保存しました</div>
  </section>"""

SETTINGS_SECURITY = """<section class="card" aria-label="セキュリティ">
    <div class="hrow"><h2>セキュリティ</h2></div>
    <dl style="margin:4px 0 0">
      <div class="dl"><dt>操作がないときの自動ログアウト</dt><dd class="num">30分</dd></div>
      <div class="dl"><dt>新しい端末からログインしたときに通知する</dt><dd>オン</dd></div>
    </dl>
    <div class="statenote-wrap">オフでも、ログインの記録は残ります</div>
  </section>"""

SETTINGS_DEVICES = """<section class="card" aria-label="登録した端末">
    <div class="hrow"><h2>登録した端末</h2></div>
    <div class="rows">
      <div class="row"><div class="left"><div class="name">東京 ・ Chrome</div><span class="sub num">最終利用 12/02 09:12 ・ この端末</span></div></div>
      <div class="row"><div class="left"><div class="name">大阪 ・ Safari</div><span class="sub num">最終利用 11/28 18:04</span></div><span class="amt" style="color:var(--accent)">解除</span></div>
    </div>
    <div class="btn" style="margin-top:12px">すべての端末をログアウト</div>
  </section>"""

SETTINGS_HISTORY = """<section class="card" aria-label="ログイン履歴">
    <div class="hrow"><h2>ログイン履歴</h2><span class="more">もっと見る ›</span></div>
    <div class="rows">
      <div class="row"><div class="left"><div class="name">kakuu.taro@example.jp</div><span class="sub num">12/02 09:12 ・ 東京 ・ 成功</span></div></div>
      <div class="row"><div class="left"><div class="name">kakuu.taro@example.jp</div><span class="sub num">11/28 18:04 ・ 大阪 ・ 新しい端末</span></div></div>
      <div class="row"><div class="left"><div class="name">kakuu.taro@example.jp</div><span class="sub num">11/10 07:55 ・ 東京 ・ <span style="color:var(--down)">失敗</span></span></div></div>
    </div>
    <div class="statenote-wrap">Cookie を消すと同じ端末でも新しい端末として登録されます</div>
  </section>"""

SETTINGS_DANGER = """<section class="card" aria-label="危険な操作">
    <div class="hrow"><h2>危険な操作</h2></div>
    <div class="rows">
      <div class="row"><div class="left"><div class="name">取引明細：配当金を削除</div><span class="sub">取り込んだ配当金データ。取り消せません</span></div><span class="amt down">削除…</span></div>
      <div class="row"><div class="left"><div class="name">取引明細：国内株式を削除</div><span class="sub">取り込んだ国内株式売買データ。取り消せません</span></div><span class="amt down">削除…</span></div>
      <div class="row"><div class="left"><div class="name">取引明細：投資信託を削除</div><span class="sub">取り込んだ投資信託売買データ。取り消せません</span></div><span class="amt down">削除…</span></div>
      <div class="row"><div class="left"><div class="name">資産管理データを削除</div><span class="sub">保存済みの保有銘柄データ。取り消せません</span></div><span class="amt down">削除…</span></div>
      <div class="row"><div class="left"><div class="name">アカウントを削除</div><span class="sub">データもすべて消えます。取り消せません</span></div><span class="amt down">削除…</span></div>
    </div>
  </section>"""

SETTINGS_STATES = """<section class="card" aria-label="状態の例">
    <div class="hrow"><h2>状態の例</h2></div>
    <p class="subhead">読み込み中</p>
    <div class="rows"><div class="row"><div class="left"><div class="sk" style="width:60%"></div></div><span class="amt"><span class="sk" style="display:inline-block;width:90px">&nbsp;</span></span></div></div>
    <p class="subhead">保存中</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">表示の設定</div><span class="sub">保存中…</span></div></div></div>
    <p class="subhead">保存済み</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">表示の設定</div><span class="sub">保存しました</span></div></div></div>
    <p class="subhead">保存失敗</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">エラー: 保存に失敗しました</div></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>
    <p class="subhead">空（端末・履歴なし）</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">登録した端末はありません</div></div></div>
    <div class="row"><div class="left"><div class="name">ログイン履歴はまだありません</div></div></div></div>
  </section>"""

settings_m = (mobile_top() + """
  <h1 class="pagetitle" style="margin-top:12px">設定</h1>
""" + SETTINGS_TOC + "\n  " + SETTINGS_ACCOUNT + "\n  " + SETTINGS_DISPLAY + "\n  "
 + SETTINGS_DATA + "\n  " + SETTINGS_SECURITY + "\n  " + SETTINGS_DEVICES + "\n  "
 + SETTINGS_HISTORY + "\n  " + SETTINGS_DANGER + nav_m(""))

settings_p = (pc_top("") + """
  <h1 class="pagetitle" style="margin-top:12px">設定</h1>
  <div class="grid2" style="margin-top:12px; grid-template-columns:220px 1fr">
    <div class="col">
""" + SETTINGS_TOC + """
    </div>
    <div class="col">
""" + SETTINGS_ACCOUNT + "\n" + SETTINGS_DISPLAY + "\n" + SETTINGS_DATA + "\n" + SETTINGS_SECURITY + """
    </div>
  </div>
  <div class="grid2" style="margin-top:24px">
    <div class="col">
""" + SETTINGS_DEVICES + "\n" + SETTINGS_HISTORY + """
    </div>
    <div class="col">
""" + SETTINGS_DANGER + """
    </div>
  </div>""")

# ---------- ログイン ----------
LOGIN_CARD = """<section class="card" aria-label="ログイン">
    <div class="hrow"><h2>ログイン</h2></div>
    <div class="statenote-wrap">証券Webを使うにはログインしてください。</div>
    <div class="btn primary" style="margin-top:12px">Googleでログイン</div>
  </section>"""

LOGIN_STATES = """<section class="card" aria-label="状態の例">
    <div class="hrow"><h2>状態の例</h2></div>
    <p class="subhead">読み込み中</p>
    <div class="rows"><div class="row"><div class="left"><div class="sk" style="width:60%"></div></div></div></div>
    <p class="subhead">取得失敗</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">エラー: 読み込みに失敗しました</div><span class="sub">ネットワーク接続を確認してください</span></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>
    <p class="subhead">ログイン済み</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">すでにログインしています</div><span class="sub">ホームに移動します</span></div><span class="amt" style="color:var(--accent)">ホームへ</span></div></div>
  </section>"""

login_m = (mobile_top_nonavi() + '<div class="center">\n  ' + LOGIN_CARD + "\n</div>")
login_p = (pc_top_nonavi() + '<div class="center">\n  ' + LOGIN_CARD + "\n</div>")

# ---------- 未ログイン時のホーム(LoginPrompt) ----------
PROMPT_HERO = """<div class="hero" style="display:block">
    <p class="label">評価額</p>
    <p class="total num">—</p>
    <p class="date num">ログインすると表示されます</p>
  </div>"""

PROMPT_CARD = """<section class="card" aria-label="ログインの案内">
    <div class="hrow"><h2>ログインしてください</h2></div>
    <div class="btn primary" style="margin-top:12px">Googleでログイン</div>
  </section>"""

PROMPT_PUBLIC = """<section class="card" aria-label="銘柄検索の入口">
    <div class="hrow"><h2>銘柄検索</h2><span class="more">開く ›</span></div>
    <div class="search"><span aria-hidden="true">⌕</span><span class="ph">銘柄コードまたは銘柄名で検索</span></div>
  </section>"""

PROMPT_STATES = """<section class="card" aria-label="状態の例">
    <div class="hrow"><h2>状態の例</h2></div>
    <p class="subhead">読み込み中</p>
    <div class="rows"><div class="row"><div class="left"><div class="sk" style="width:60%"></div></div></div></div>
    <p class="subhead">取得失敗</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">エラー: 読み込みに失敗しました</div></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>
  </section>"""

loginprompt_m = (mobile_top_nonavi() + PROMPT_HERO + "\n  " + PROMPT_CARD + "\n  " + PROMPT_PUBLIC)
loginprompt_p = (pc_top_nonavi() + PROMPT_HERO + "\n  " + PROMPT_CARD + "\n  " + PROMPT_PUBLIC)

# ---------- 404 ----------
NOTFOUND_CARD = """<section class="card" aria-label="ページが見つかりません">
    <div class="hrow"><h2>404 - ページが見つかりません</h2></div>
    <div class="statenote-wrap">指定されたリソースが見つかりません。URLを確認してください。</div>
    <div class="btn primary" style="margin-top:12px">ホームに戻る</div>
    <div class="toc">
      <span class="chip">銘柄検索</span><span class="chip">資産管理</span><span class="chip">取引明細</span>
    </div>
  </section>"""

NOTFOUND_AUTH = """<section class="card" aria-label="認証と権限の振り分け">
    <div class="hrow"><h2>認証と権限の振り分け</h2></div>
    <div class="rows">
      <div class="row"><div class="left"><div class="name">未ログイン</div><span class="sub">ログインが必要です</span></div><span class="amt" style="color:var(--accent)">ログインへ</span></div>
      <div class="row"><div class="left"><div class="name">管理者以外</div><span class="sub">このリソースへのアクセス権限がありません</span></div><span class="amt num">404</span></div>
    </div>
  </section>"""

notfound_m = (mobile_top_nonavi() + '<div class="center">\n  ' + NOTFOUND_CARD + "\n  " + NOTFOUND_AUTH + "\n</div>")
notfound_p = (pc_top_nonavi() + '<div class="center">\n  ' + NOTFOUND_CARD + "\n  " + NOTFOUND_AUTH + "\n</div>")

# ---------- ログイン一覧(管理者) ----------
ADMIN_FORM_M = """<div class="search"><span aria-hidden="true">⌕</span><span class="ph">メール・場所で検索</span></div>
  <div class="sortbtn"><span>並べ替え: 日時の新しい順</span><span aria-hidden="true">›</span></div>
  <div class="chips"><span class="chip">メール: kakuu <span class="x">×</span></span><span class="chip">場所: 東京 <span class="x">×</span></span><span class="chip">すべてクリア</span></div>
  <div class="countrow"><span class="n num">3件 / 50件</span><span class="link">この条件で絞る</span></div>"""

ADMIN_ROWS = """<div class="rows">
      <div class="row"><div class="left"><div class="name">kakuu.taro@example.jp</div><span class="sub num">12/02 09:12 ・ 東京 ・ Chrome</span></div><span class="amt"><span class="badge ok">成功</span></span></div>
      <div class="row"><div class="left"><div class="name">suzuki.hanako@example.jp</div><span class="sub num">12/01 21:30 ・ 大阪 ・ Safari</span></div><span class="amt"><span class="badge">新しい端末</span></span></div>
      <div class="row"><div class="left"><div class="name">tanaka.jiro.verylongname@example.jp</div><span class="sub num">11/10 07:55 ・ 名古屋 ・ Edge</span></div><span class="amt"><span class="badge ng">失敗</span></span></div>
    </div>"""

ADMIN_STATES = """<section class="card" aria-label="状態の例">
    <div class="hrow"><h2>状態の例</h2></div>
    <p class="subhead">空</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">条件に一致するログイン記録はありません</div></div></div></div>
    <p class="subhead">読み込み中</p>
    <div class="rows"><div class="row"><div class="left"><div class="sk" style="width:60%"></div></div><span class="amt"><span class="sk" style="display:inline-block;width:90px">&nbsp;</span></span></div></div>
    <p class="subhead">取得失敗</p>
    <div class="rows"><div class="row"><div class="left"><div class="name">エラー: 読み込みに失敗しました</div></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>
  </section>"""

admin_m = (mobile_top() + """
  <h1 class="pagetitle" style="margin-top:12px">ログイン一覧(管理者)</h1>
""" + ADMIN_FORM_M + """
  <section class="card" aria-label="ログイン記録の一覧">
    <div class="hrow"><h2>ログイン記録</h2></div>
""" + ADMIN_ROWS + """
  </section>
""" + nav_m(""))

ADMIN_COND_P = """<section class="card" aria-label="検索パネル">
        <div class="hrow"><h2>条件</h2><span class="more">クリア</span></div>
        <div class="field">メール: kakuu</div>
        <div class="field">場所: 東京</div>
        <div class="field">並べ替え: 日時の新しい順</div>
        <div class="btn primary" style="margin-top:12px">この条件で絞る</div>
      </section>"""

admin_p = (pc_top("") + """
  <h1 class="pagetitle" style="margin-top:12px">ログイン一覧(管理者)</h1>
  <div class="grid2" style="margin-top:12px">
    <section class="card" aria-label="ログイン記録の一覧">
      <div class="hrow"><h2>ログイン記録</h2><span class="more">この条件で絞る</span></div>
      <div class="search"><span aria-hidden="true">⌕</span><span class="ph">メール・場所で検索</span></div>
      <div class="countrow"><span class="n num">3件 / 50件(列名クリックで並べ替え)</span></div>
      <table class="grid" style="margin-top:8px">
        <tr><th class="sortable">日時 ▼</th><th>メール</th><th>結果</th><th>場所</th><th>ブラウザ・OS</th></tr>
        <tr><td class="num">12/02 09:12</td><td><div class="nm">kakuu.taro@example.jp</div></td><td><span class="sb" style="color:var(--up)">成功</span></td><td><span class="sb">東京</span></td><td><span class="sb">Chrome</span></td></tr>
        <tr><td class="num">12/01 21:30</td><td><div class="nm">suzuki.hanako@example.jp</div></td><td><span class="sb">新しい端末</span></td><td><span class="sb">大阪</span></td><td><span class="sb">Safari</span></td></tr>
        <tr><td class="num">11/10 07:55</td><td><div class="nm">tanaka.jiro.verylongname@example.jp</div></td><td><span class="sb" style="color:var(--down)">失敗</span></td><td><span class="sb">名古屋</span></td><td><span class="sb">Edge</span></td></tr>
      </table>
    </section>
    <div class="col">
""" + ADMIN_COND_P + """
    </div>
  </div>""")

# ---------- ダイアログ9種 ----------
D1 = """<section class="card" aria-label="保有株詳細ダイアログ">
    <div class="hrow"><h2>9902 架空食品ホールディングス</h2><span class="more">閉じる</span></div>
    <dl style="margin:4px 0 0">
      <div class="dl"><dt>評価額</dt><dd class="num">¥7,949,807</dd></div>
      <div class="dl"><dt>評価損益</dt><dd class="num up">+¥1,352,488 +20.50%</dd></div>
      <div class="dl"><dt>数量</dt><dd class="num">100株</dd></div>
      <div class="dl"><dt>取得単価</dt><dd class="num">¥65,973.19</dd></div>
      <div class="dl"><dt>現在値</dt><dd class="num">¥79,498.07</dd></div>
      <div class="dl"><dt>価格基準日</dt><dd class="num">2026/11/13</dd></div>
    </dl>
  </section>"""

D2 = """<section class="card" aria-label="取引明細の行詳細">
    <div class="hrow"><h2>9902 架空食品ホールディングス</h2><span class="more">閉じる</span></div>
    <div class="sum"><span class="state-note num">11/08 ・ 特定口座 ・ 100株</span><span class="state-note num">税引後 ¥31,874</span></div>
    <dl style="margin:4px 0 0">
      <div class="dl"><dt>配当金(税引前)</dt><dd class="num">¥40,000</dd></div>
      <div class="dl"><dt>税額</dt><dd class="num">¥8,126</dd></div>
      <div class="dl"><dt>税引後</dt><dd class="num">¥31,874</dd></div>
    </dl>
  </section>"""

D3 = """<section class="card" aria-label="ホーム配当詳細ダイアログ">
    <div class="hrow"><h2>配当金の詳細</h2><span class="more">閉じる</span></div>
    <dl style="margin:4px 0 0">
      <div class="dl"><dt>配当金(税引前)</dt><dd class="num">¥56,250</dd></div>
      <div class="dl"><dt>税額</dt><dd class="num">¥11,427</dd></div>
      <div class="dl"><dt>税引後</dt><dd class="num">¥44,823</dd></div>
      <div class="dl"><dt>口座</dt><dd>特定口座</dd></div>
      <div class="dl"><dt>入金日</dt><dd class="num">2026/11/10</dd></div>
    </dl>
  </section>"""

D4 = """<section class="card" aria-label="検索・条件追加パネル">
    <div class="hrow"><h2>条件</h2><span class="more">クリア</span></div>
    <p class="subhead">期間の種類</p>
    <div class="tabs" role="group" aria-label="期間"><span>全期間</span><span class="here">年</span><span>年月</span><span>日</span><span>範囲</span></div>
    <div class="field">年を選択: 2026年</div>
    <div class="field">口座: 特定口座 ・ NISA成長口座</div>
    <div class="field">商品: 国内株式 ・ 投資信託</div>
    <div class="btn primary" style="margin-top:12px">この期間で絞る</div>
  </section>"""

D5 = """<section class="card" aria-label="CSV行メニュー">
    <div class="hrow"><h2>取引明細(配当金)</h2><span class="more">閉じる</span></div>
    <div class="rows">
      <div class="row"><div class="left"><div class="name">楽天証券から再取得</div></div></div>
      <div class="row"><div class="left"><div class="name">ファイルから取り込み</div></div></div>
      <div class="row"><div class="left"><div class="name" style="color:var(--down)">削除</div></div></div>
    </div>
  </section>"""

D6 = """<section class="card" aria-label="削除確認">
    <div class="hrow"><h2>取引明細(配当金)を削除しますか?</h2></div>
    <p class="statenote-wrap">対象: 取引明細(配当金)のデータ 18件を削除します。この操作は取り消せません。</p>
    <div style="display:flex; gap:12px; margin-top:12px">
      <div class="btn" style="flex:1">キャンセル</div>
      <div class="btn danger" style="flex:1">18件を削除する</div>
    </div>
  </section>"""

D7 = """<section class="card" aria-label="サイト共通アカウントメニュー">
    <div class="hrow"><h2>kakuu.taro@example.jp</h2><span class="more">閉じる</span></div>
    <div class="rows">
      <div class="row"><div class="left"><div class="name">設定</div></div></div>
      <div class="row"><div class="left"><div class="name">CSVでエクスポート</div></div></div>
      <div class="row"><div class="left"><div class="name">ログイン履歴</div></div></div>
      <div class="row"><div class="left"><div class="name">ログイン一覧(管理者)</div></div></div>
      <div class="row"><div class="left"><div class="name">ログアウト</div></div></div>
      <div class="row"><div class="left"><div class="name" style="color:var(--down)">アカウントを削除</div></div></div>
    </div>
  </section>"""

D8 = """<section class="card" aria-label="アカウント削除確認">
    <div class="hrow"><h2>アカウント削除の確認</h2><span class="more">閉じる</span></div>
    <p class="statenote-wrap">アカウントを削除すると、資産管理・配当金・取引履歴などすべてのデータが削除されます。この操作は取り消せません。削除されたデータは復元できません。</p>
    <div style="display:flex; gap:12px; margin-top:12px">
      <div class="btn" style="flex:1">キャンセル</div>
      <div class="btn danger" style="flex:1">削除する</div>
    </div>
  </section>"""

D9 = """<section class="card" aria-label="管理者検索パネル">
    <div class="hrow"><h2>管理者の検索</h2><span class="more">クリア</span></div>
    <div class="field">メール: kakuu</div>
    <div class="field">場所: 東京</div>
    <div class="field">並べ替え: 日時の新しい順</div>
    <div class="btn primary" style="margin-top:12px">この条件で絞る</div>
  </section>"""

D10 = """<section class="card" aria-label="通知リンクのログアウト確認">
    <div class="hrow"><h2>心当たりのないログインがありましたか</h2><span class="more">閉じる</span></div>
    <p class="statenote-wrap">次の操作でこのアカウントのすべての端末をログアウトします。</p>
    <div class="btn primary" style="margin-top:12px">すべての端末をログアウトする</div>
  </section>"""

OP = OUT.parent / "opencode"
VISUAL = OUT.parent.parent / "phase8-visual"

def dialog_state(name, title, parent, dialog, label, css):
    html = parent.read_text(encoding="utf-8")
    html = html.replace("</style>", css + "\n" + FRAG_CSS + "\n</style>", 1)
    html = re.sub(r"<title>.*?</title>", "<title>" + title + " | 証券Web</title>", html, count=1)
    html = html.replace("</body>", ovl(dialog, label) + "\n</body>", 1)
    (OUT / (name + ".html")).write_text(html, encoding="utf-8")
    print("wrote", name, len(html))

for key, title, parent_tpl, dialog, label in [
    ("holding", "ダイアログ(保有株詳細) 新案", str(OP / "assetbalance-{}.html"), D1, "保有株詳細ダイアログ"),
    ("receipt-detail", "ダイアログ(取引明細の行詳細) 新案", str(OP / "receipts-{}.html"), D2, "取引明細の行詳細"),
    ("dividend-detail", "ダイアログ(ホーム配当詳細) 新案", str(VISUAL / "home-{}.html"), D3, "ホーム配当詳細ダイアログ"),
    ("filter", "ダイアログ(条件追加パネル) 新案", str(OP / "receipts-{}.html"), D4, "検索・条件追加パネル"),
    ("csv-menu", "ダイアログ(CSV行メニュー) 新案", str(OP / "csv-{}.html"), D5, "CSV行メニュー"),
    ("delete-confirm", "ダイアログ(削除確認) 新案", str(OP / "csv-{}.html"), D6, "削除確認"),
    ("account-menu", "ダイアログ(アカウントメニュー) 新案", str(VISUAL / "home-{}.html"), D7, "サイト共通アカウントメニュー"),
    ("account-delete", "ダイアログ(アカウント削除確認) 新案", str(OUT / "settings-{}.html"), D8, "アカウント削除確認"),
    ("admin-search", "ダイアログ(管理者検索パネル) 新案", str(OUT / "admin-logins-{}.html"), D9, "管理者検索パネル"),
    ("logout-all", "ダイアログ(全端末ログアウト確認) 新案", str(OUT / "settings-{}.html"), D10, "通知リンクのログアウト確認"),
]:
    for dev in ("mobile", "pc"):
        dialog_state(
            "dialogs-state-" + key + "-" + dev,
            title + (" スマホ" if dev == "mobile" else " PC"),
            Path(parent_tpl.format(dev)),
            dialog,
            label,
            OVL_M if dev == "mobile" else OVL_P,
        )

write("search-mobile", "銘柄検索 新案 スマホ", search_m)
write("search-pc", "銘柄検索 新案 PC", search_p)
write("settings-mobile", "設定 新案 スマホ", settings_m)
write("settings-pc", "設定 新案 PC", settings_p)
write("login-mobile", "ログイン 新案 スマホ", login_m)
write("login-pc", "ログイン 新案 PC", login_p)
write("loginprompt-mobile", "未ログイン時のホーム 新案 スマホ", loginprompt_m)
write("loginprompt-pc", "未ログイン時のホーム 新案 PC", loginprompt_p)
write("notfound-mobile", "ページが見つかりません 新案 スマホ", notfound_m)
write("notfound-pc", "ページが見つかりません 新案 PC", notfound_p)
write("admin-logins-mobile", "ログイン一覧(管理者) 新案 スマホ", admin_m)
write("admin-logins-pc", "ログイン一覧(管理者) 新案 PC", admin_p)

SEARCH_INITIAL = """<section class="card" aria-label="検索結果">
    <div class="hrow"><h2>銘柄を検索</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="name">銘柄コードまたは銘柄名を入力してください</div><span class="sub">4桁の銘柄コードで検索できます（例：7203, 9984）</span></div></div></div>
  </section>"""
SEARCH_NOMATCH = """<section class="card" aria-label="検索結果">
    <div class="hrow"><h2>検索結果</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="name">該当する銘柄が見つかりませんでした</div><span class="sub">銘柄コードまたは銘柄名を確認してください</span></div></div></div>
  </section>"""
SEARCH_LOADING = """<section class="card" aria-label="検索結果">
    <div class="hrow"><h2>検索結果</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="sk" style="width:60%"></div></div><span class="amt"><span class="sk" style="display:inline-block;width:90px">&nbsp;</span></span></div></div>
  </section>"""
SEARCH_FAILED = """<section class="card" aria-label="検索結果">
    <div class="hrow"><h2>検索結果</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="name">エラー: 読み込みに失敗しました</div><span class="sub">ネットワーク接続を確認してください</span></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>
  </section>"""
SEARCH_INVALID = """<section class="card" aria-label="検索結果">
    <div class="hrow"><h2>検索結果</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="name">不正な銘柄コードが指定されています</div></div></div></div>
  </section>"""

SETTINGS_SKELETON = """<section class="card" aria-label="表示">
    <div class="hrow"><h2>表示</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="sk" style="width:60%"></div></div><span class="amt"><span class="sk" style="display:inline-block;width:90px">&nbsp;</span></span></div></div>
  </section>"""
SETTINGS_SAVING = """<section class="card" aria-label="表示">
    <div class="hrow"><h2>表示</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="name">表示の設定</div><span class="sub">保存中…</span></div></div></div>
  </section>"""
SETTINGS_SAVED = """<section class="card" aria-label="表示">
    <div class="hrow"><h2>表示</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="name">表示の設定</div><span class="sub">保存しました</span></div></div></div>
  </section>"""
SETTINGS_SAVEFAILED = """<section class="card" aria-label="表示">
    <div class="hrow"><h2>表示</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="name">エラー: 保存に失敗しました</div></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>
  </section>"""
SETTINGS_EMPTY_DEVICES = """<section class="card" aria-label="登録した端末">
    <div class="hrow"><h2>登録した端末</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="name">登録した端末はありません</div></div></div></div>
  </section>"""
SETTINGS_EMPTY_HISTORY = """<section class="card" aria-label="ログイン履歴">
    <div class="hrow"><h2>ログイン履歴</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="name">ログイン履歴はまだありません</div></div></div></div>
  </section>"""

LOGIN_LOADING = """<section class="card" aria-label="ログイン">
    <div class="hrow"><h2>ログイン</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="sk" style="width:60%"></div></div></div></div>
  </section>"""
LOGIN_FAILED = """<section class="card" aria-label="ログイン">
    <div class="hrow"><h2>ログイン</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="name">エラー: 読み込みに失敗しました</div><span class="sub">ネットワーク接続を確認してください</span></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>
  </section>"""
LOGIN_LOGGEDIN = """<section class="card" aria-label="ログイン">
    <div class="hrow"><h2>ログイン</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="name">すでにログインしています</div><span class="sub">ホームに移動します</span></div><span class="amt" style="color:var(--accent)">ホームへ</span></div></div>
  </section>"""

PROMPT_LOADING = """<section class="card" aria-label="ログインの案内">
    <div class="hrow"><h2>ログインしてください</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="sk" style="width:60%"></div></div></div></div>
  </section>"""
PROMPT_FAILED = """<section class="card" aria-label="ログインの案内">
    <div class="hrow"><h2>ログインしてください</h2></div>
    <div class="rows"><div class="row"><div class="left"><div class="name">エラー: 読み込みに失敗しました</div></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>
  </section>"""

ADMIN_LIST_OPEN = '<section class="card" aria-label="ログイン記録の一覧">\n    <div class="hrow"><h2>ログイン記録</h2></div>\n'
ADMIN_EMPTY = ADMIN_LIST_OPEN + """<div class="rows"><div class="row"><div class="left"><div class="name">条件に一致するログイン記録はありません</div></div></div></div>
  </section>"""
ADMIN_LOADING = ADMIN_LIST_OPEN + """<div class="rows"><div class="row"><div class="left"><div class="sk" style="width:60%"></div></div><span class="amt"><span class="sk" style="display:inline-block;width:90px">&nbsp;</span></span></div></div>
  </section>"""
ADMIN_FAILED = ADMIN_LIST_OPEN + """<div class="rows"><div class="row"><div class="left"><div class="name">エラー: 読み込みに失敗しました</div></div><span class="amt" style="color:var(--accent)">再試行</span></div></div>
  </section>"""

write("search-state-initial-mobile", "銘柄検索(初期) 新案 スマホ", mobile_top() + """
  <h1 class="pagetitle" style="margin-top:12px">銘柄検索</h1>
""" + SEARCH_FORM + "\n  " + SEARCH_INITIAL + nav_m("銘柄検索"))
write("search-state-initial-pc", "銘柄検索(初期) 新案 PC", pc_top("銘柄検索") + """
  <h1 class="pagetitle" style="margin-top:12px">銘柄検索</h1>
  <div class="grid2" style="margin-top:12px">
    <div class="col">
""" + SEARCH_CARD_P + "\n" + SEARCH_INITIAL + """
    </div>
  </div>""")
write("search-state-nomatch-mobile", "銘柄検索(該当なし) 新案 スマホ", mobile_top() + """
  <h1 class="pagetitle" style="margin-top:12px">銘柄検索</h1>
""" + SEARCH_FORM + "\n  " + SEARCH_NOMATCH + nav_m("銘柄検索"))
write("search-state-nomatch-pc", "銘柄検索(該当なし) 新案 PC", pc_top("銘柄検索") + """
  <h1 class="pagetitle" style="margin-top:12px">銘柄検索</h1>
  <div class="grid2" style="margin-top:12px">
    <div class="col">
""" + SEARCH_CARD_P + "\n" + SEARCH_NOMATCH + """
    </div>
  </div>""")
write("search-state-loading-mobile", "銘柄検索(読み込み中) 新案 スマホ", mobile_top() + """
  <h1 class="pagetitle" style="margin-top:12px">銘柄検索</h1>
""" + SEARCH_FORM + "\n  " + SEARCH_LOADING + nav_m("銘柄検索"))
write("search-state-loading-pc", "銘柄検索(読み込み中) 新案 PC", pc_top("銘柄検索") + """
  <h1 class="pagetitle" style="margin-top:12px">銘柄検索</h1>
  <div class="grid2" style="margin-top:12px">
    <div class="col">
""" + SEARCH_CARD_P + "\n" + SEARCH_LOADING + """
    </div>
  </div>""")
write("search-state-failed-mobile", "銘柄検索(取得失敗) 新案 スマホ", mobile_top() + """
  <h1 class="pagetitle" style="margin-top:12px">銘柄検索</h1>
""" + SEARCH_FORM + "\n  " + SEARCH_FAILED + nav_m("銘柄検索"))
write("search-state-failed-pc", "銘柄検索(取得失敗) 新案 PC", pc_top("銘柄検索") + """
  <h1 class="pagetitle" style="margin-top:12px">銘柄検索</h1>
  <div class="grid2" style="margin-top:12px">
    <div class="col">
""" + SEARCH_CARD_P + "\n" + SEARCH_FAILED + """
    </div>
  </div>""")
write("search-state-invalid-mobile", "銘柄検索(不正なコード) 新案 スマホ", mobile_top() + """
  <h1 class="pagetitle" style="margin-top:12px">銘柄検索</h1>
""" + SEARCH_FORM + "\n  " + SEARCH_INVALID + nav_m("銘柄検索"))
write("search-state-invalid-pc", "銘柄検索(不正なコード) 新案 PC", pc_top("銘柄検索") + """
  <h1 class="pagetitle" style="margin-top:12px">銘柄検索</h1>
  <div class="grid2" style="margin-top:12px">
    <div class="col">
""" + SEARCH_CARD_P + "\n" + SEARCH_INVALID + """
    </div>
  </div>""")

write("settings-state-loading-mobile", "設定(読み込み中) 新案 スマホ", mobile_top() + """
  <h1 class="pagetitle" style="margin-top:12px">設定</h1>
""" + SETTINGS_TOC + "\n  " + SETTINGS_SKELETON + nav_m(""))
write("settings-state-loading-pc", "設定(読み込み中) 新案 PC", pc_top("") + """
  <h1 class="pagetitle" style="margin-top:12px">設定</h1>
  <div class="grid2" style="margin-top:12px; grid-template-columns:220px 1fr">
    <div class="col">
""" + SETTINGS_TOC + """
    </div>
    <div class="col">
""" + SETTINGS_SKELETON + """
    </div>
  </div>""")
write("settings-state-saving-mobile", "設定(保存中) 新案 スマホ", mobile_top() + """
  <h1 class="pagetitle" style="margin-top:12px">設定</h1>
""" + SETTINGS_TOC + "\n  " + SETTINGS_SAVING + nav_m(""))
write("settings-state-saving-pc", "設定(保存中) 新案 PC", pc_top("") + """
  <h1 class="pagetitle" style="margin-top:12px">設定</h1>
  <div class="grid2" style="margin-top:12px; grid-template-columns:220px 1fr">
    <div class="col">
""" + SETTINGS_TOC + """
    </div>
    <div class="col">
""" + SETTINGS_SAVING + """
    </div>
  </div>""")
write("settings-state-saved-mobile", "設定(保存済み) 新案 スマホ", mobile_top() + """
  <h1 class="pagetitle" style="margin-top:12px">設定</h1>
""" + SETTINGS_TOC + "\n  " + SETTINGS_SAVED + nav_m(""))
write("settings-state-saved-pc", "設定(保存済み) 新案 PC", pc_top("") + """
  <h1 class="pagetitle" style="margin-top:12px">設定</h1>
  <div class="grid2" style="margin-top:12px; grid-template-columns:220px 1fr">
    <div class="col">
""" + SETTINGS_TOC + """
    </div>
    <div class="col">
""" + SETTINGS_SAVED + """
    </div>
  </div>""")
write("settings-state-savefailed-mobile", "設定(保存失敗) 新案 スマホ", mobile_top() + """
  <h1 class="pagetitle" style="margin-top:12px">設定</h1>
""" + SETTINGS_TOC + "\n  " + SETTINGS_SAVEFAILED + nav_m(""))
write("settings-state-savefailed-pc", "設定(保存失敗) 新案 PC", pc_top("") + """
  <h1 class="pagetitle" style="margin-top:12px">設定</h1>
  <div class="grid2" style="margin-top:12px; grid-template-columns:220px 1fr">
    <div class="col">
""" + SETTINGS_TOC + """
    </div>
    <div class="col">
""" + SETTINGS_SAVEFAILED + """
    </div>
  </div>""")
write("settings-state-empty-mobile", "設定(空) 新案 スマホ", mobile_top() + """
  <h1 class="pagetitle" style="margin-top:12px">設定</h1>
""" + SETTINGS_TOC + "\n  " + SETTINGS_EMPTY_DEVICES + "\n  " + SETTINGS_EMPTY_HISTORY + nav_m(""))
write("settings-state-empty-pc", "設定(空) 新案 PC", pc_top("") + """
  <h1 class="pagetitle" style="margin-top:12px">設定</h1>
  <div class="grid2" style="margin-top:12px; grid-template-columns:220px 1fr">
    <div class="col">
""" + SETTINGS_TOC + """
    </div>
    <div class="col">
""" + SETTINGS_EMPTY_DEVICES + "\n" + SETTINGS_EMPTY_HISTORY + """
    </div>
  </div>""")

write("login-state-loading-mobile", "ログイン(読み込み中) 新案 スマホ", mobile_top_nonavi() + '<div class="center">\n  ' + LOGIN_LOADING + "\n</div>")
write("login-state-loading-pc", "ログイン(読み込み中) 新案 PC", pc_top_nonavi() + '<div class="center">\n  ' + LOGIN_LOADING + "\n</div>")
write("login-state-failed-mobile", "ログイン(取得失敗) 新案 スマホ", mobile_top_nonavi() + '<div class="center">\n  ' + LOGIN_FAILED + "\n</div>")
write("login-state-failed-pc", "ログイン(取得失敗) 新案 PC", pc_top_nonavi() + '<div class="center">\n  ' + LOGIN_FAILED + "\n</div>")
write("login-state-loggedin-mobile", "ログイン(ログイン済み) 新案 スマホ", mobile_top_nonavi() + '<div class="center">\n  ' + LOGIN_LOGGEDIN + "\n</div>")
write("login-state-loggedin-pc", "ログイン(ログイン済み) 新案 PC", pc_top_nonavi() + '<div class="center">\n  ' + LOGIN_LOGGEDIN + "\n</div>")

write("loginprompt-state-loading-mobile", "未ログイン時のホーム(読み込み中) 新案 スマホ", mobile_top_nonavi() + PROMPT_HERO + "\n  " + PROMPT_LOADING + "\n  " + PROMPT_PUBLIC)
write("loginprompt-state-loading-pc", "未ログイン時のホーム(読み込み中) 新案 PC", pc_top_nonavi() + PROMPT_HERO + "\n  " + PROMPT_LOADING + "\n  " + PROMPT_PUBLIC)
write("loginprompt-state-failed-mobile", "未ログイン時のホーム(取得失敗) 新案 スマホ", mobile_top_nonavi() + PROMPT_HERO + "\n  " + PROMPT_FAILED + "\n  " + PROMPT_PUBLIC)
write("loginprompt-state-failed-pc", "未ログイン時のホーム(取得失敗) 新案 PC", pc_top_nonavi() + PROMPT_HERO + "\n  " + PROMPT_FAILED + "\n  " + PROMPT_PUBLIC)

write("admin-logins-state-empty-mobile", "ログイン一覧(データなし) 新案 スマホ", mobile_top() + """
  <h1 class="pagetitle" style="margin-top:12px">ログイン一覧(管理者)</h1>
""" + ADMIN_FORM_M + "\n  " + ADMIN_EMPTY + nav_m(""))
write("admin-logins-state-empty-pc", "ログイン一覧(データなし) 新案 PC", pc_top("") + """
  <h1 class="pagetitle" style="margin-top:12px">ログイン一覧(管理者)</h1>
  <div class="grid2" style="margin-top:12px">
""" + ADMIN_EMPTY + """
    <div class="col">
""" + ADMIN_COND_P + """
    </div>
  </div>""")
write("admin-logins-state-loading-mobile", "ログイン一覧(読み込み中) 新案 スマホ", mobile_top() + """
  <h1 class="pagetitle" style="margin-top:12px">ログイン一覧(管理者)</h1>
""" + ADMIN_FORM_M + "\n  " + ADMIN_LOADING + nav_m(""))
write("admin-logins-state-loading-pc", "ログイン一覧(読み込み中) 新案 PC", pc_top("") + """
  <h1 class="pagetitle" style="margin-top:12px">ログイン一覧(管理者)</h1>
  <div class="grid2" style="margin-top:12px">
""" + ADMIN_LOADING + """
    <div class="col">
""" + ADMIN_COND_P + """
    </div>
  </div>""")
write("admin-logins-state-failed-mobile", "ログイン一覧(取得失敗) 新案 スマホ", mobile_top() + """
  <h1 class="pagetitle" style="margin-top:12px">ログイン一覧(管理者)</h1>
""" + ADMIN_FORM_M + "\n  " + ADMIN_FAILED + nav_m(""))
write("admin-logins-state-failed-pc", "ログイン一覧(取得失敗) 新案 PC", pc_top("") + """
  <h1 class="pagetitle" style="margin-top:12px">ログイン一覧(管理者)</h1>
  <div class="grid2" style="margin-top:12px">
""" + ADMIN_FAILED + """
    <div class="col">
""" + ADMIN_COND_P + """
    </div>
  </div>""")
