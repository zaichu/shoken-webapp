#!/usr/bin/env python3
"""金額・率の数字と記号だけの Inter subset を作り、検証する。

対象は `0-9` `,` `.` `¥` `%` `+` `−`(U+2212)の 16 字。日本語・単位(円・株・口・万・千)・
括弧・空白・`—` などは入れない。`@font-face` の `unicode-range` と合わせ、
範囲外の文字は Noto Sans JP に落ちる(設計は docs/design/numerals.md)。

使い:
  python3 frontend/scripts/build-numerals-fonts.py [--cache <dir>] [--verify-only]
  (fonttools・brotli が要る。pip install fonttools brotli)

元ファイルは google/fonts をコミット SHA で固定して取得する。
出力は frontend/fonts/numerals/。
"""

import argparse
import hashlib
import io
import json
import sys
import urllib.request
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

GOOGLE_FONTS_SHA = "5bd87dfc525c8a0d1d8529d3684716e00c3fc59a"
RAW = f"https://raw.githubusercontent.com/google/fonts/{GOOGLE_FONTS_SHA}"
SOURCE_DIR = "ofl/inter"
SOURCE_FILE = "Inter[opsz,wght].ttf"
# 軸はアプリの2段(本文 400・見出し 700)に絞り、opsz は 14 に固定する
AXES = {"opsz": 14, "wght": (400, 700)}
LABEL = "Shoken Numerals"
OUTPUT = "inter-400-700.woff2"
LICENSE_FILE = "OFL-inter.txt"

CODEPOINTS = [0x25, 0x2B, 0x2C, 0x2E, *range(0x30, 0x3A), 0xA5, 0x2212]
DIGITS = [ord(c) for c in "0123456789"]
LAYOUT_FEATURES = ["tnum", "lnum", "kern"]


def fetch(url, dest):
    if not dest.exists():
        dest.parent.mkdir(parents=True, exist_ok=True)
        with urllib.request.urlopen(url) as response:
            dest.write_bytes(response.read())
    return dest


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rename(font):
    name = font["name"]
    ps = LABEL.replace(" ", "")
    for record in list(name.names):
        if record.nameID in (16, 17, 21, 22, 25):
            name.removeNames(nameID=record.nameID)
    style = "Regular"
    for record in name.names:
        if record.nameID == 2:
            style = record.toUnicode()
            break
    name.setName(LABEL, 1, 3, 1, 0x409)
    name.setName(f"{LABEL} {style}", 4, 3, 1, 0x409)
    name.setName(f"{ps}-{style.replace(' ', '')}", 6, 3, 1, 0x409)
    name.setName(f"{ps}-{style.replace(' ', '')};subset-2042", 3, 3, 1, 0x409)
    for record in list(name.names):
        if record.platformID != 3 and record.nameID in (1, 3, 4, 6):
            name.removeNames(nameID=record.nameID, platformID=record.platformID)


def has_feature(font, tag):
    for table in ("GSUB", "GPOS"):
        if table in font and font[table].table.FeatureList:
            if any(r.FeatureTag == tag for r in font[table].table.FeatureList.FeatureRecord):
                return True
    return False


def tnum_map(font):
    mapping = {}
    if "GSUB" not in font or not font["GSUB"].table.FeatureList:
        return mapping
    table = font["GSUB"].table
    for record in table.FeatureList.FeatureRecord:
        if record.FeatureTag != "tnum":
            continue
        for index in record.Feature.LookupListIndex:
            lookup = table.LookupList.Lookup[index]
            for sub in lookup.SubTable:
                if lookup.LookupType == 7:
                    sub = sub.ExtSubTable
                mapping.update(getattr(sub, "mapping", {}) or {})
    return mapping


def tabular_widths(font):
    cmap = font.getBestCmap()
    mapping = tnum_map(font)
    hmtx = font["hmtx"]
    return sorted({hmtx[mapping.get(cmap[c], cmap[c])][0] for c in DIGITS})


def verify(path, weights):
    font = TTFont(path)
    cmap = set(font.getBestCmap())
    result = {
        "file": path.name,
        "bytes": path.stat().st_size,
        "cmap_ok": cmap == set(CODEPOINTS),
        "cmap_extra": sorted(hex(c) for c in cmap - set(CODEPOINTS)),
        "cmap_missing": sorted(hex(c) for c in set(CODEPOINTS) - cmap),
        "gsub_tnum": has_feature(font, "tnum"),
        "family": font["name"].getDebugName(1),
        "digit_widths": {},
    }
    if "fvar" in font:
        for weight in weights:
            instance = instancer.instantiateVariableFont(TTFont(path), {"wght": weight})
            result["digit_widths"][weight] = tabular_widths(instance)
    else:
        result["digit_widths"]["static"] = tabular_widths(font)
    result["ok"] = (
        result["cmap_ok"]
        and result["gsub_tnum"]
        and all(len(widths) == 1 for widths in result["digit_widths"].values())
    )
    return result


def build(cache, out):
    out.mkdir(parents=True, exist_ok=True)
    url = f"{RAW}/{SOURCE_DIR}/{SOURCE_FILE}"
    license_url = f"{RAW}/{SOURCE_DIR}/OFL.txt"
    license_path = fetch(license_url, cache / "OFL.txt")
    (out / LICENSE_FILE).write_bytes(license_path.read_bytes())
    source = fetch(url, cache / SOURCE_FILE)
    font = TTFont(source)
    source_check = {
        "gsub_tnum": has_feature(font, "tnum"),
        "cmap_missing": sorted(hex(c) for c in set(CODEPOINTS) - set(font.getBestCmap())),
    }
    font = instancer.instantiateVariableFont(font, AXES)
    # 部分的に固定した gvar は遅延読み込みのまま subset すると壊れるため読み直す
    buffer = io.BytesIO()
    font.save(buffer)
    buffer.seek(0)
    font = TTFont(buffer)
    options = subset.Options()
    options.unicodes = CODEPOINTS
    options.layout_features = LAYOUT_FEATURES
    options.name_IDs = ["*"]
    options.name_legacy = True
    options.name_languages = ["*"]
    options.notdef_outline = True
    options.flavor = "woff2"
    subsetter = subset.Subsetter(options)
    subsetter.populate(unicodes=CODEPOINTS)
    subsetter.subset(font)
    rename(font)
    font.flavor = "woff2"
    font.save(out / OUTPUT)
    sources = [
        {
            "label": LABEL,
            "source": "Inter",
            "url": url,
            "sha256": sha256(source),
            "axes": {key: list(value) if isinstance(value, tuple) else value for key, value in AXES.items()},
            "source_check": source_check,
            "output": OUTPUT,
            "license": LICENSE_FILE,
            "license_url": license_url,
        }
    ]
    (out / "sources.json").write_text(json.dumps(sources, ensure_ascii=False, indent=2) + "\n")
    return sources


def main():
    parser = argparse.ArgumentParser()
    root = Path(__file__).resolve().parents[1]
    parser.add_argument("--cache", type=Path, default=root / "test-results" / "numerals-src")
    parser.add_argument("--out", type=Path, default=root / "fonts" / "numerals")
    parser.add_argument("--verify-only", action="store_true")
    args = parser.parse_args()
    if not args.verify_only:
        build(args.cache, args.out)
    result = verify(args.out / OUTPUT, [400, 700])
    print(json.dumps(result, ensure_ascii=False))
    if not result["ok"]:
        sys.exit(1)


if __name__ == "__main__":
    main()
