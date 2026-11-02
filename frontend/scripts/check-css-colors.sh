#!/usr/bin/env bash
# Rust 側・CSS 側の色の直書きと、Rust の style 属性での色・寸法の指定を検出する。
# 色は style/input.css の @theme トークン経由(var(--color-*))だけを使い、
# 16進・rgb/hsl 系の直書きと style 属性での色・寸法は増やさないこと。
# 既存の違反は css-color-baseline.txt(検査種別・ファイル・件数)で許し、
# 新しい違反(基準超え)だけを落とす。基準は減る方向にだけ変える。
# 使い方: frontend/ で `bash scripts/check-css-colors.sh`
set -euo pipefail
shopt -s globstar nullglob

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
SCRIPT_PATH="$SCRIPT_DIR/$(basename "$0")"
cd "$SCRIPT_DIR/.."

BASELINE="scripts/css-color-baseline.txt"
HEX='#[0-9a-fA-F]{3,8}\b'
FUNC_COLOR='(rgba?|hsla?|oklch|oklab|color-mix)\('
STYLE_PROP='style:(background|background-color|color|border-color|border-top-color|border-right-color|border-bottom-color|border-left-color|fill|stroke|outline-color|width|min-width|max-width|height|min-height|max-height|top|left|right|bottom|transform|margin|margin-top|margin-right|margin-bottom|margin-left|padding|padding-top|padding-right|padding-bottom|padding-left|font-size)\b'
STYLE_ATTR='(^|[^[:alnum:]_:.-])style[[:space:]]*=|(^|[^[:alnum:]_])attr:style[[:space:]]*='

RS_FILES=(src/**/*.rs)
CSS_FILE="style/input.css"

fail=0

is_comment() {
  grep -Eq '^[[:space:]]*(//|/\*|\*)' <<<"$1"
}

filter_comments() {
  while IFS= read -r line; do
    [ -z "$line" ] && continue
    if [[ "$line" =~ ^[^:]*:[0-9]+: ]]; then
      text=${line#*:*:}
    else
      text=${line#*:}
    fi
    is_comment "$text" && continue
    printf '%s\n' "$line"
  done
}

check_hex_src() {
  local hits deny
  hits=$(grep -HnE -e "$HEX" "${RS_FILES[@]}" 2>/dev/null || true)
  hits=$(printf '%s\n' "$hits" | filter_comments || true)
  deny="$hits"
  [ -z "$deny" ] && { echo "  OK: 16進色直書き(src)"; return 0; }
  echo "  INFO: 16進色直書き(src: 基準で管理)" >&2
  printf '%s\n' "$deny" >&2
}

check_func_src() {
  local hits deny
  hits=$(grep -HnE -e "$FUNC_COLOR" "${RS_FILES[@]}" 2>/dev/null || true)
  hits=$(printf '%s\n' "$hits" | filter_comments || true)
  deny=$(printf '%s\n' "$hits" | grep -vF 'var(--color-' || true)
  [ -z "$deny" ] && { echo "  OK: 関数色直書き(src)"; return 0; }
  echo "  INFO: 関数色直書き(src: 基準で管理)" >&2
  printf '%s\n' "$deny" >&2
}

check_hex_css() {
  local hits deny
  hits=$(grep -HnE -e "$HEX" "$CSS_FILE" 2>/dev/null || true)
  deny=$(printf '%s\n' "$hits" | grep -vE ':[0-9]+:[[:space:]]*--' || true)
  [ -z "$deny" ] && { echo "  OK: 16進色直書き(style)"; return 0; }
  echo "ERROR: 16進色直書き(style)" >&2
  printf '%s\n' "$deny" >&2
  fail=1
}

check_func_css() {
  local hits deny
  hits=$(grep -HnE -e "$FUNC_COLOR" "$CSS_FILE" 2>/dev/null || true)
  deny=$(printf '%s\n' "$hits" | grep -vE ':[0-9]+:[[:space:]]*--' | grep -vF 'var(--color-' || true)
  [ -z "$deny" ] && { echo "  OK: 関数色直書き(style)"; return 0; }
  echo "ERROR: 関数色直書き(style)" >&2
  printf '%s\n' "$deny" >&2
  fail=1
}

check_style_prop() {
  local hits deny
  hits=$(grep -HnE -e "$STYLE_PROP" "${RS_FILES[@]}" 2>/dev/null || true)
  hits=$(printf '%s\n' "$hits" | filter_comments || true)
  deny="$hits"
  [ -z "$deny" ] && { echo "  OK: style指定の色・寸法(src)"; return 0; }
  echo "  INFO: style指定の色・寸法(src: 基準で管理)" >&2
  printf '%s\n' "$deny" >&2
}

check_style_attr() {
  local hits deny filtered=""
  hits=$(grep -HnE -e "$STYLE_ATTR" "${RS_FILES[@]}" 2>/dev/null || true)
  while IFS= read -r line; do
    [ -z "$line" ] && continue
    text=${line#*:*:}
    is_comment "$text" && continue
    grep -Eq 'DisclosureStyle|\.style\(\)|let style' <<<"$text" && continue
    filtered+="$line"$'\n'
  done <<<"$hits"
  deny="${filtered%$'\n'}"
  [ -z "$deny" ] && { echo "  OK: style属性(src)"; return 0; }
  echo "  INFO: style属性(src: 基準で管理)" >&2
  printf '%s\n' "$deny" >&2
}

count_per_file() {
  awk -F: '{n=$1; c[n]++} END {for (f in c) print f"\t"c[f]}' <<<"$1"
}

check_baseline() {
  local kind="$1" hits="$2" tmp_current tmp_base
  [ -z "$hits" ] && return 0
  tmp_current=$(mktemp)
  tmp_base=$(mktemp)
  count_per_file "$hits" | sort >"$tmp_current"
  awk -v k="$kind" -F'\t' '$1==k {print $2"\t"$3}' "$BASELINE" | sort >"$tmp_base"
  local over=0
  while IFS=$'\t' read -r file cnt; do
    [ -z "$file" ] && continue
    base=$(awk -F'\t' -v f="$file" '$1==f {print $2}' "$tmp_base")
    base=${base:-0}
    if [ "$cnt" -gt "$base" ]; then
      echo "ERROR: 基準超え($kind $file: 基準 $base 件 < 現状 $cnt 件)" >&2
      over=1
    fi
  done <"$tmp_current"
  rm -f "$tmp_current" "$tmp_base"
  if [ "$over" -ne 0 ]; then
    fail=1
    return 1
  fi
  echo "  OK: 基準内($kind)"
  return 0
}

collect_hex_src() {
  local hits
  hits=$(grep -HnE -e "$HEX" "${RS_FILES[@]}" 2>/dev/null || true)
  printf '%s\n' "$hits" | filter_comments || true
}

collect_func_src() {
  local hits
  hits=$(grep -HnE -e "$FUNC_COLOR" "${RS_FILES[@]}" 2>/dev/null || true)
  printf '%s\n' "$hits" | filter_comments | grep -vF 'var(--color-' || true
}

collect_hex_css() {
  local hits
  hits=$(grep -HnE -e "$HEX" "$CSS_FILE" 2>/dev/null || true)
  printf '%s\n' "$hits" | grep -vE ':[0-9]+:[[:space:]]*--' || true
}

collect_func_css() {
  local hits
  hits=$(grep -HnE -e "$FUNC_COLOR" "$CSS_FILE" 2>/dev/null || true)
  printf '%s\n' "$hits" | grep -vE ':[0-9]+:[[:space:]]*--' | grep -vF 'var(--color-' || true
}

collect_style_prop() {
  local hits
  hits=$(grep -HnE -e "$STYLE_PROP" "${RS_FILES[@]}" 2>/dev/null || true)
  printf '%s\n' "$hits" | filter_comments || true
}

collect_style_attr() {
  local hits filtered=""
  hits=$(grep -HnE -e "$STYLE_ATTR" "${RS_FILES[@]}" 2>/dev/null || true)
  while IFS= read -r line; do
    [ -z "$line" ] && continue
    text=${line#*:*:}
    is_comment "$text" && continue
    grep -Eq 'DisclosureStyle|\.style\(\)|let style' <<<"$text" && continue
    filtered+="$line"$'\n'
  done <<<"$hits"
  printf '%s' "${filtered%$'\n'}"
}

if [ "$#" -gt 0 ]; then
  SELF_FILES=("$@")
  check_hex_src() {
    local hits deny
    hits=$(grep -nE -e "$HEX" "${SELF_FILES[@]}" 2>/dev/null || true)
    hits=$(printf '%s\n' "$hits" | filter_comments | grep -vF -e '--' || true)
    [ -z "$hits" ] && { echo "  OK: 16進色直書き(自己テスト)"; return 0; }
    echo "ERROR: 16進色直書き(自己テスト)" >&2
    printf '%s\n' "$hits" >&2
    fail=1
  }
  check_func_src() {
    local hits deny
    hits=$(grep -nE -e "$FUNC_COLOR" "${SELF_FILES[@]}" 2>/dev/null || true)
    hits=$(printf '%s\n' "$hits" | filter_comments || true)
    deny=$(printf '%s\n' "$hits" | grep -vF 'var(--color-' || true)
    [ -z "$deny" ] && { echo "  OK: 関数色直書き(自己テスト)"; return 0; }
    echo "ERROR: 関数色直書き(自己テスト)" >&2
    printf '%s\n' "$deny" >&2
    fail=1
  }
  check_style_prop() {
    local hits
    hits=$(grep -nE -e "$STYLE_PROP" "${SELF_FILES[@]}" 2>/dev/null || true)
    hits=$(printf '%s\n' "$hits" | filter_comments || true)
    [ -z "$hits" ] && { echo "  OK: style指定の色・寸法(自己テスト)"; return 0; }
    echo "ERROR: style指定の色・寸法(自己テスト)" >&2
    printf '%s\n' "$hits" >&2
    fail=1
  }
  check_style_attr() {
    local hits filtered=""
    hits=$(grep -HnE -e "$STYLE_ATTR" "${SELF_FILES[@]}" 2>/dev/null || true)
    while IFS= read -r line; do
      [ -z "$line" ] && continue
      text=${line#*:*:}
      is_comment "$text" && continue
      grep -Eq 'DisclosureStyle|\.style\(\)|let style' <<<"$text" && continue
      filtered+="$line"$'\n'
    done <<<"$hits"
    deny="${filtered%$'\n'}"
    [ -z "$deny" ] && { echo "  OK: style属性(自己テスト)"; return 0; }
    echo "ERROR: style属性(自己テスト)" >&2
    printf '%s\n' "$deny" >&2
    fail=1
  }
  check_hex_src
  check_func_src
  check_style_prop
  check_style_attr
  [ "$fail" -ne 0 ] && exit 1
  exit 0
fi

check_hex_src
check_func_src
check_hex_css
check_func_css
check_style_prop
check_style_attr

check_baseline "hex-src" "$(collect_hex_src)"
check_baseline "func-src" "$(collect_func_src)"
check_baseline "hex-css" "$(collect_hex_css)"
check_baseline "func-css" "$(collect_func_css)"
check_baseline "style-prop" "$(collect_style_prop)"
check_baseline "style-attr" "$(collect_style_attr)"

[ "$fail" -ne 0 ] && exit 1

self_test_output=""
self_test_status=0
if self_test_output=$(bash "$SCRIPT_PATH" scripts/fixtures/css-colors-invalid.txt 2>&1); then
  self_test_status=0
else
  self_test_status=$?
fi

if [ "$self_test_status" -ne 1 ]; then
  echo "ERROR: 色検査の自己テストが exit 1 になりません" >&2
  exit 1
fi

for expected in 'fill="#FFFFFF"' 'rgba(15,23,42' 'style:width' 'style="width: 10%"'; do
  if ! grep -Fq "$expected" <<<"$self_test_output"; then
    echo "ERROR: 色検査の自己テストが $expected を検出しません" >&2
    exit 1
  fi
done

valid_output=""
if ! valid_output=$(bash "$SCRIPT_PATH" scripts/fixtures/css-colors-valid.txt 2>&1); then
  echo "ERROR: 色検査が正常fixtureを誤検知しました" >&2
  printf '%s\n' "$valid_output" >&2
  exit 1
fi
echo "  OK: 違反fixtureの自己テスト"
echo "  OK: 正常fixtureの自己テスト"

tmpdir=$(mktemp -d)
printf 'hex-src\tsrc/a.rs\t1\n' >"$tmpdir/baseline.txt"
over_hits=$(printf 'src/a.rs:1: fill="#FFFFFF"\nsrc/a.rs:2: fill="#000000"\n')
saved_baseline="$BASELINE"
saved_fail="$fail"
BASELINE="$tmpdir/baseline.txt"
fail=0
if check_baseline "hex-src" "$over_hits" 2>/dev/null; then
  echo "ERROR: 基準超過の自己テストが失敗を検出しません" >&2
  rm -rf "$tmpdir"
  exit 1
fi
fail=0
under_hits=$(printf 'src/a.rs:1: fill="#FFFFFF"\n')
if ! check_baseline "hex-src" "$under_hits" >/dev/null 2>&1; then
  echo "ERROR: 基準以内の自己テストが失敗になりました" >&2
  rm -rf "$tmpdir"
  exit 1
fi
BASELINE="$saved_baseline"
fail="$saved_fail"
echo "  OK: 基準超過の自己テスト"
rm -rf "$tmpdir"

echo "OK: 色の直書きはありません(基準内)"
