#!/usr/bin/env python3
"""fund_codes の seed を再生成する。

資産運用業協会が公開する「成長投資枠対象商品リスト(非上場ファンド)」の xlsx から
運用会社が三菱UFJアセットマネジメントのファンドを選び、
backend/migrations/0013_fund_nav.sql 内の seed ブロック
(-- gen:fund-codes-seed begin/end で囲まれた VALUES)を書き換える。

標準ライブラリだけで動く(openpyxl 等は不要)。ファンド名のセルに埋め込まれた
ふりがな(<rPh>)は読み飛ばす。

使い方:
    python3 scripts/update_fund_codes_seed.py
    python3 scripts/update_fund_codes_seed.py --xlsx /path/to/unlisted_fund_for_investor.xlsx
"""

import argparse
import re
import unicodedata
import urllib.request
import zipfile
import xml.etree.ElementTree as ET
from pathlib import Path

SOURCE_URL = (
    "https://www.imaj.or.jp/find/nisa_growth_productslist/"
    "xlsx-cms/unlisted_fund_for_investor.xlsx"
)
MIGRATION = Path(__file__).resolve().parent.parent / "backend/migrations/0013_fund_nav.sql"
SHEET_NAME = "対象商品一覧"
OPERATOR_PREFIX = "三菱UFJアセット"
BEGIN = "-- gen:fund-codes-seed begin"
END = "-- gen:fund-codes-seed end"
NS = "{http://schemas.openxmlformats.org/spreadsheetml/2006/main}"


def shared_strings(zf):
    root = ET.fromstring(zf.read("xl/sharedStrings.xml"))
    out = []
    for si in root.findall(f"{NS}si"):
        parts = []
        for child in si:
            if child.tag == f"{NS}t":
                parts.append(child.text or "")
            elif child.tag == f"{NS}r":
                t = child.find(f"{NS}t")
                if t is not None and t.text:
                    parts.append(t.text)
        out.append("".join(parts))
    return out


def sheet_path(zf, name):
    workbook = ET.fromstring(zf.read("xl/workbook.xml"))
    rels = ET.fromstring(zf.read("xl/_rels/workbook.xml.rels"))
    rid = None
    for sheet in workbook.iter(f"{NS}sheet"):
        if sheet.get("name") == name:
            rid = sheet.get("{http://schemas.openxmlformats.org/officeDocument/2006/relationships}id")
    if rid is None:
        raise SystemExit(f"シート {name} が見つかりません")
    for rel in rels:
        if rel.get("Id") == rid:
            return "xl/" + rel.get("Target")
    raise SystemExit(f"シート {name} のリレーションが見つかりません")


def col_index(cell_ref):
    return ord(cell_ref[0]) - ord("A")


def load_rows(zf, path, strings):
    sheet = ET.fromstring(zf.read(path))
    rows = []
    for row in sheet.iter(f"{NS}row"):
        vals = {}
        for cell in row.findall(f"{NS}c"):
            v = cell.find(f"{NS}v")
            if v is None or v.text is None:
                continue
            text = strings[int(v.text)] if cell.get("t") == "s" else v.text
            vals[col_index(cell.get("r"))] = text
        rows.append(vals)
    return rows


def extract_funds(xlsx):
    with zipfile.ZipFile(xlsx) as zf:
        strings = shared_strings(zf)
        rows = load_rows(zf, sheet_path(zf, SHEET_NAME), strings)
    header = next(
        r for r in rows if any("投信協会ファンドコード" in v for v in r.values())
    )
    cols = {name: i for name in ("投信協会ファンドコード", "ファンド名称", "運用会社名")
            for i, v in header.items() if name in v}
    if len(cols) != 3:
        raise SystemExit(f"必要な列が見つかりません: {header}")
    funds = {}
    for row in rows:
        operator = unicodedata.normalize("NFKC", row.get(cols["運用会社名"], ""))
        if not operator.startswith(OPERATOR_PREFIX):
            continue
        code = row.get(cols["投信協会ファンドコード"], "").strip()
        name = row.get(cols["ファンド名称"], "").strip()
        if code and name:
            funds.setdefault(name, code)
    return sorted(funds.items())


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--xlsx", default=SOURCE_URL,
                    help="対象商品リスト xlsx の URL またはローカルパス")
    ap.add_argument("--migration", default=str(MIGRATION))
    args = ap.parse_args()
    source = urllib.request.urlretrieve(args.xlsx)[0] if "://" in args.xlsx else args.xlsx
    funds = extract_funds(source)
    lines = [
        f"    ('{name.replace(chr(39), chr(39) * 2)}','{code}')" + ("," if i < len(funds) - 1 else "")
        for i, (name, code) in enumerate(funds)
    ]
    text = Path(args.migration).read_text()
    block = re.sub(
        f"{re.escape(BEGIN)}\n.*{re.escape(END)}",
        f"{BEGIN}\n" + "\n".join(lines) + f"\n{END}",
        text, flags=re.S,
    )
    if lines[0] not in block:
        raise SystemExit("seed ブロックのマーカーが見つかりません")
    Path(args.migration).write_text(block)
    print(f"{len(funds)} 件を {args.migration} に書き込みました")


if __name__ == "__main__":
    main()
