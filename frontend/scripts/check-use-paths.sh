#!/usr/bin/env bash
# ui::labels と ui::format の関数・定数は、モジュールを取り込んで
# `labels::名前` / `format::名前` と書く。名前を並べる取り込みは禁止する。
# (型は名前で取り込んでよいので、この検査の対象外)
# 使い方: frontend/ で `bash scripts/check-use-paths.sh`
set -euo pipefail
shopt -s globstar nullglob

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
SCRIPT_PATH="$SCRIPT_DIR/$(basename "$0")"
cd "$SCRIPT_DIR/.."

PATTERN='^[[:space:]]*use[[:space:]]+crate::ui::(labels|format)::'
RS_FILES=(src/**/*.rs)
fail=0

check() {
  local label="$1" hits
  shift
  hits=$(grep -HnE -e "$PATTERN" "$@" 2>/dev/null || true)
  # コメント行(// で始まる行)は違反ではない
  if [ -n "$hits" ]; then
    hits=$(printf '%s\n' "$hits" | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//' || true)
  fi
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

check "ui::labels/format の名前取り込み(src/**/*.rs)" "${RS_FILES[@]}"
[ "$fail" -ne 0 ] && exit 1

# 自己テスト: 違反形は落とし、正規形は通す
tmp_self=$(mktemp -d)
printf 'use crate::ui::labels::{RECEIPTS_TITLE, NO_DATA};\n' >"$tmp_self/bad.rs"
printf 'use crate::ui::format::format_iso_date;\n' >>"$tmp_self/bad.rs"
printf 'use crate::ui::labels;\nuse crate::ui::format;\n' >"$tmp_self/good.rs"
if bash "$SCRIPT_PATH" "$tmp_self/bad.rs" >/dev/null 2>&1; then
  echo "ERROR: use 検査の自己テストが違反形を通しました" >&2
  rm -rf "$tmp_self"
  exit 1
fi
if ! bash "$SCRIPT_PATH" "$tmp_self/good.rs" >/dev/null 2>&1; then
  echo "ERROR: use 検査の自己テストが正規形を落としました" >&2
  rm -rf "$tmp_self"
  exit 1
fi
rm -rf "$tmp_self"
echo "  OK: 違反・正規の自己テスト"
echo "OK: 違反はありません"
