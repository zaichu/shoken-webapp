#!/usr/bin/env bash
# 関数と定数はモジュールを取り込んで `モジュール::名前` と書く。
# 型(struct・enum・trait・コンポーネント)は名前で取り込んでよい。
# 小文字の名前(関数・モジュール)と大文字だけの名前(定数)を { } や直接で
# 名前取り込みしたら失敗する。ただし小文字でも対応するモジュールファイルが
# ある場合はモジュールの取り込みなので通す。
# (例: `use crate::session::pending_logout;` は pending_logout.rs があるので可、
# `use crate::session::local_storage;` は関数なので不可)
# 使い方: frontend/ で `bash scripts/check-use-paths.sh`
set -euo pipefail
shopt -s globstar nullglob

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
SCRIPT_PATH="$SCRIPT_DIR/$(basename "$0")"
cd "$SCRIPT_DIR/.."

RS_FILES=(src/**/*.rs)
fail=0

check() {
  local label="$1"
  shift
  local hits
  hits=$(python3 - "$@" <<'PY' 2>/dev/null || true
import re, sys, pathlib
SRC = pathlib.Path("src")

def split_top_commas(s):
    parts, depth, cur = [], 0, ""
    for ch in s:
        if ch == "{":
            depth += 1
            cur += ch
        elif ch == "}":
            depth -= 1
            cur += ch
        elif ch == "," and depth == 0:
            parts.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        parts.append(cur)
    return parts

def expand_use(tree):
    tree = tree.strip()
    if "{" not in tree:
        # `A as B` は元の名前で判定する
        base = tree.split(" as ")[0].strip()
        return [base] if base else []
    # 最初の `{...}` に対応する `}` を探す
    start = tree.index("{")
    depth = 0
    end = -1
    for i in range(start, len(tree)):
        if tree[i] == "{":
            depth += 1
        elif tree[i] == "}":
            depth -= 1
            if depth == 0:
                end = i
                break
    if end == -1:
        return [tree]
    prefix = tree[:start].rstrip(":")
    inner = tree[start + 1:end]
    suffix = tree[end + 1:]
    out = []
    for part in split_top_commas(inner):
        part = part.strip()
        if not part:
            continue
        out.extend(expand_use(prefix + "::" + part + suffix))
    return out

def is_module(parent_parts, leaf):
    base = SRC.joinpath(*parent_parts) if parent_parts else SRC
    return (base / (leaf + ".rs")).exists() or (base / leaf / "mod.rs").exists()

violations = []
for path in sys.argv[1:]:
    try:
        text = pathlib.Path(path).read_text()
    except OSError:
        continue
    # 複数行 use に対応するため全文から `use ...;` を抜き出す
    for m in re.finditer(r'(?m)^[ \t]*use\s+(.*?);', text, re.DOTALL):
        stmt = m.group(0)
        # コメント行(// で始まる行)は違反ではない
        first = stmt.strip().splitlines()[0] if stmt.strip() else ""
        if first.startswith("//"):
            continue
        if "crate::" not in stmt:
            continue
        lineno = text[:m.start()].count("\n") + 1
        tree_m = re.match(r'\s*use\s+(.*);', stmt, re.DOTALL)
        if not tree_m:
            continue
        tree = re.sub(r'\s+', ' ', tree_m.group(1)).strip()
        # 先頭の `crate::` を含む木だけ調べる
        if not tree.startswith("crate::") and "crate::" not in tree:
            continue
        oneline = " ".join(stmt.strip().split())
        for expanded in expand_use(tree):
            exp = expanded.split(" as ")[0].strip()
            if not exp.startswith("crate::"):
                continue
            segs = exp.split("::")
            if len(segs) < 2:
                continue
            leaf = segs[-1].strip()
            if not re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', leaf or ""):
                continue  # `*` など
            if leaf in ("self", "super", "crate"):
                continue
            if leaf == "*":
                continue
            parent_parts = segs[1:-1]
            # 型(UpperCamel)は通す
            if re.match(r'^[A-Z]', leaf) and not (leaf.upper() == leaf and re.fullmatch(r'[A-Z][A-Z0-9_]*', leaf)):
                continue
            # 大文字だけの名前(定数)は失敗
            if re.fullmatch(r'[A-Z][A-Z0-9_]*', leaf) and leaf.upper() == leaf:
                violations.append(f"{path}:{lineno}:{oneline}")
                break
            # 小文字の名前はモジュールファイルがあれば通す(モジュールの取り込み)
            if re.match(r'^[a-z_]', leaf):
                if is_module(parent_parts, leaf):
                    continue
                violations.append(f"{path}:{lineno}:{oneline}")
                break
print("\n".join(violations))
PY
)
  [ -z "$hits" ] && { echo "  OK: $label"; return 0; }
  echo "ERROR: $label" >&2
  printf '%s\n' "$hits" >&2
  fail=1
}

if [ "$#" -gt 0 ]; then
  check "名前取り込みの use(自己テスト)" "$@"
  [ "$fail" -ne 0 ] && exit 1
  exit 0
fi

check "関数・定数の名前取り込み(src/**/*.rs)" "${RS_FILES[@]}"
[ "$fail" -ne 0 ] && exit 1

# 自己テスト: 違反形は落とし、正規形は通す
tmp_self=$(mktemp -d)
# まとめた形・直接の形の違反
printf 'use crate::ui::{labels::{RECEIPTS_TITLE}, format::{format_iso_date}};\n' >"$tmp_self/bad_nested.rs"
printf 'use crate::ui::labels::RECEIPTS_TITLE;\n' >>"$tmp_self/bad_nested.rs"
printf 'use crate::support::pagination::{ListEndpoint, fetch_all_pages};\n' >>"$tmp_self/bad_nested.rs"
printf 'use crate::ui::labels;\nuse crate::ui::format;\n' >"$tmp_self/good_module.rs"
printf 'use crate::ui::button::{Button, ButtonVariant};\n' >>"$tmp_self/good_module.rs"
printf 'use crate::session::pending_logout;\n' >>"$tmp_self/good_module.rs"
if bash "$SCRIPT_PATH" "$tmp_self/bad_nested.rs" >/dev/null 2>&1; then
  echo "ERROR: use 検査の自己テストが違反形(まとめた形・直接)を通しました" >&2
  rm -rf "$tmp_self"
  exit 1
fi
if ! bash "$SCRIPT_PATH" "$tmp_self/good_module.rs" >/dev/null 2>&1; then
  echo "ERROR: use 検査の自己テストが正規形(モジュール・型)を落としました" >&2
  rm -rf "$tmp_self"
  exit 1
fi
rm -rf "$tmp_self"
echo "  OK: 違反・正規の自己テスト"
echo "OK: 違反はありません"
