#!/usr/bin/env python3
import re
from pathlib import Path

HERE = Path(__file__).resolve().parent
VISUAL = HERE.parent

EMPTY = {
    "今年の配当金": ("今年の配当データがありません", "配当金明細をCSVで追加してください"),
    "資産の内訳": ("資産データがありません", "CSVファイルを取り込むと表示されます"),
    "最近の配当入金": ("データがありません", "配当金明細をCSVで追加してください"),
    "評価損益の大きい銘柄": ("データがありません", "CSVファイルを取り込むと表示されます"),
}

SKEL = """<div class="rows">
      <div class="row"><div class="left"><div class="sk" style="width:60%;height:14px;border-radius:6px;background:var(--mut);opacity:.25"></div></div><span class="amt"><span class="sk" style="display:inline-block;width:90px;height:14px;border-radius:6px;background:var(--mut);opacity:.25">&nbsp;</span></span></div>
      <div class="row"><div class="left"><div class="sk" style="width:50%;height:14px;border-radius:6px;background:var(--mut);opacity:.25"></div></div><span class="amt"><span class="sk" style="display:inline-block;width:90px;height:14px;border-radius:6px;background:var(--mut);opacity:.25">&nbsp;</span></span></div>
    </div>"""


def section(label, inner):
    return '<section class="card" aria-label="' + label + '">\n' + inner + "\n  </section>"


def head_rows(label, more):
    return '    <div class="hrow"><h2>' + label + "</h2>" + (('<span class="more">' + more + "</span>") if more else "") + "</div>"


MORE = {
    "今年の配当金": "取引明細で見る ›",
    "資産の内訳": "資産管理 ›",
    "最近の配当入金": "すべて見る ›",
    "評価損益の大きい銘柄": "資産管理 ›",
}


def build(label, kind):
    if kind == "empty":
        name, sub = EMPTY[label]
        inner = head_rows(label, MORE[label]) + '\n    <div class="rows">\n      <div class="row"><div class="left"><div class="name">' + name + '</div><span class="sub">' + sub + "</span></div></div>\n    </div>"
    elif kind == "loading":
        inner = head_rows(label, MORE[label]) + "\n" + SKEL
    else:
        inner = head_rows(label, MORE[label]) + '\n    <div class="rows">\n      <div class="row"><div class="left"><div class="name">エラー: 読み込みに失敗しました</div><span class="sub">時間をおいて再度お試しください</span></div><span class="amt" style="color:var(--accent)">再試行</span></div>\n    </div>'
    return section(label, inner)


for dev in ("mobile", "pc"):
    base = VISUAL / ("home-" + dev + ".html")
    html = base.read_text(encoding="utf-8")
    for kind, title in (("empty", "ホーム(データなし) 新案"), ("loading", "ホーム(読み込み中) 新案"), ("failed", "ホーム(読み込み失敗) 新案")):
        out = html
        for label in ("今年の配当金", "資産の内訳", "最近の配当入金", "評価損益の大きい銘柄"):
            out = re.sub(
                r'<section class="card" aria-label="' + label + r'">.*?</section>',
                lambda m, b=build(label, kind): b,
                out,
                count=1,
                flags=re.DOTALL,
            )
        suffix = "スマホ" if dev == "mobile" else "PC"
        out = re.sub(r"<title>.*?</title>", "<title>" + title + " " + suffix + " | 証券Web</title>", out, count=1)
        name = "home-state-" + kind + "-" + dev
        (VISUAL / (name + ".html")).write_text(out, encoding="utf-8")
        print("wrote", name, len(out))
