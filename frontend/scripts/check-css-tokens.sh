#!/usr/bin/env bash
# frontend の Rust 側クラス文字列から、生パレット・旧トークン・白黒直書き・任意値を検出する。
# 新規の色・値は style/input.css の @theme トークンか ui/ コンポーネントで表すこと。
# CSS ファイル側(style/input.css)の宣言は Stylelint が見る。
# 使い方: frontend/ で `bash scripts/check-css-tokens.sh`
set -euo pipefail
shopt -s globstar nullglob

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
SCRIPT_PATH="$SCRIPT_DIR/$(basename "$0")"
cd "$SCRIPT_DIR/.."

PALETTE='(text|bg|border|border-[trblxyse]|divide|ring|outline|placeholder|from|via|to|fill|stroke|caret|decoration|shadow|ring-offset|accent)-(slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-[0-9]{2,3}'
OLD_TOKEN='(text|bg|border|border-[trblxyse]|divide|ring|outline|placeholder|from|via|to|fill|stroke|caret|decoration|shadow|ring-offset)-(primary|primary-hover|primary-dark|secondary|secondary-hover|success|success-hover|info-hover|warning|warning-hover|danger|danger-hover|light|dark|negative-dark|bg-body|bg-dark|bg-card-dark|border-dark)\b'
# 白・黒の直書き(bg-white など)。`bg-surface` / `text-text-inverse` / `bg-scrim` などのトークンを使う
MONO='([^[:space:]"]+:)*(text|bg|border|border-[trblxyse]|divide|ring|outline|placeholder|from|via|to|fill|stroke|caret|decoration|shadow|ring-offset|accent)-(white|black)\b'
# `-[...]` 形式の任意値・`-(var)` 形式の CSS 変数省略記法・`[property:value]` 形式の任意プロパティ
ARBITRARY='([a-z][a-z-]*-\[[^]]*\]|\[[a-z-]+:[^]]+\]|[a-z][a-z-]*-\([^)]*\))'
TYPE_SIZE='text-(xs|sm|base|lg|xl|2xl)\b'
FONT_PROP='font-size:'
SPACE_PROP='(padding|margin|gap)(-(top|right|bottom|left|inline|block)(-(start|end))?)?[[:space:]]*:'
# features/ の余白ユーティリティ直書き(margin/padding/gap/space 系)を数える。
# 部品の見た目は ui/ の部品の中だけで決め、features/ では足さない決まりのため件数を凍結する。
# `whitespace-nowrap`・`min-w-*`・`max-sm:*` は数えない。`scroll-mt-*` は mt 部分で数える
SPACING_RS='(^|[^A-Za-z0-9_])(m[trblxyse]?|p[trblxyse]?|gap(-[xy])?|space-[xy])-'
TYPE_BASELINE="scripts/css-type-spacing-baseline.txt"
SPACING_BASELINE="scripts/css-spacing-baseline.txt"

# 許可リスト: 追加するときは理由を添える。形式は grep -E の正規表現(一致する行を除外)。
ALLOWED_SRC=(
  'transition-\['   # 遷移プロパティの列挙は任意値でしか書けない
  'min-w-\[8ch\]'   # 表の金額列は桁幅(ch 単位)で指定する
  'min-h-\[50vh\]'  # 404・モーダルの縦位置はビューポート基準のため
  'shadow-\[0_1px_2px_rgba\(15,23,42,\.08\)\]'  # タブ選択時の微弱シャドウ
  'text-\[11px\]'   # タブ件数バッジの極小フォント
  'text-\[13px\]'   # 検索バーのチップ(デザイン指定)
  'p-\[3px\]'       # タブリストの内側余白(デザイン指定)
)
fail=0
RS_FILES=(src/**/*.rs)
FEATURE_RS_FILES=(src/features/**/*.rs)

join_or() {
  local IFS='|'
  echo "$*"
}

check() {
  local label="$1" pattern="$2"
  shift 2
  local hits
  hits=$(grep -noE -e "$pattern" "$@" 2>/dev/null || true)
  [ -z "$hits" ] && { echo "  OK: $label"; return 0; }
  echo "ERROR: $label" >&2
  printf '%s\n' "$hits" >&2
  fail=1
}

check_allowed() {
  local label="$1" pattern="$2" allowed="$3"
  shift 3
  local hits deny
  hits=$(grep -HnoE -e "$pattern" "$@" 2>/dev/null || true)
  [ -z "$hits" ] && { echo "  OK: $label"; return 0; }
  deny=$(printf '%s\n' "$hits" | grep -vE "$allowed" || true)
  [ -z "$deny" ] && { echo "  OK: $label"; return 0; }
  echo "ERROR: $label" >&2
  printf '%s\n' "$deny" >&2
  fail=1
}

check_arbitrary() {
  local label="$1" allowed="$2"
  shift 2
  local hits deny
  hits=$(grep -noE -e "$ARBITRARY" "$@" 2>/dev/null || true)
  [ -z "$hits" ] && { echo "  OK: 任意値($label)"; return 0; }
  # Rust のパス区切り(::)を含む [...] は配列リテラルなので対象外にする
  # (Tailwind の任意プロパティ・任意値に :: は使わない)
  deny=$(printf '%s\n' "$hits" | grep -vE "$allowed" | grep -vF '::' || true)
  [ -z "$deny" ] && { echo "  OK: 任意値(すべて許可リスト内: $label)"; return 0; }
  echo "ERROR: 許可リスト外の任意値($label)" >&2
  printf '%s\n' "$deny" >&2
  fail=1
}

collect_type_rs() {
  grep -HoE -e "$TYPE_SIZE" "${RS_FILES[@]}" 2>/dev/null || true
}

collect_spacing_rs() {
  grep -HoE -e "$SPACING_RS" "${FEATURE_RS_FILES[@]}" 2>/dev/null || true
}

check_type_baseline() {
  local kind="$1" hits="$2" baseline="${3:-$TYPE_BASELINE}" tmp_current tmp_base
  [ -z "$hits" ] && return 0
  tmp_current=$(mktemp)
  tmp_base=$(mktemp)
  awk -F: '{n=$1; c[n]++} END {for (f in c) print f"\t"c[f]}' <<<"$hits" | sort >"$tmp_current"
  awk -v k="$kind" -F'\t' '$1==k {print $2"\t"$3}' "$baseline" | sort >"$tmp_base"
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

if [ "$#" -gt 0 ]; then
  check "生パレット(自己テスト)" "$PALETTE" "$@"
  check "旧トークン(自己テスト)" "$OLD_TOKEN" "$@"
  check_allowed "白・黒の直書き(自己テスト)" "$MONO" 'print:(\[[^]]+\]:)?border-black$' "$@"
  check_arbitrary "自己テスト" '^$' "$@"
  check "文字サイズ直書き(自己テスト)" "$TYPE_SIZE" "$@"
  check "font-size直書き(自己テスト)" "$FONT_PROP" "$@"
  check "余白直書き(自己テスト)" "$SPACE_PROP" "$@"
  check "余白ユーティリティ直書き(自己テスト)" "$SPACING_RS" "$@"
  [ "$fail" -ne 0 ] && exit 1
  exit 0
fi

check "生パレット(src/**/*.rs)" "$PALETTE" "${RS_FILES[@]}"
check "旧トークン(src/**/*.rs)" "$OLD_TOKEN" "${RS_FILES[@]}"
check "白・黒の直書き(src/**/*.rs)" "$MONO" "${RS_FILES[@]}"
check_arbitrary "src/**/*.rs" "$(join_or "${ALLOWED_SRC[@]}")" "${RS_FILES[@]}"
check_type_baseline "type-rs" "$(collect_type_rs)"
check_type_baseline "spacing-features" "$(collect_spacing_rs)" "$SPACING_BASELINE"

[ "$fail" -ne 0 ] && exit 1

self_test_output=""
self_test_status=0
if self_test_output=$(bash "$SCRIPT_PATH" scripts/fixtures/css-tokens-invalid.txt 2>&1); then
  self_test_status=0
else
  self_test_status=$?
fi

if [ "$self_test_status" -ne 1 ]; then
  echo "ERROR: CSS トークン検査の自己テストがexit 1になりません" >&2
  exit 1
fi

for expected in 'bg-white' 'text-slate-500' 'shadow-[0_1px_2px_#000]' 'text-primary' '[min-height:44px]' 'text-(--color-red-700)' 'text-sm' 'font-size:' 'padding:' 'mt-'; do
  if ! grep -Fq "$expected" <<<"$self_test_output"; then
    echo "ERROR: CSS トークン検査の自己テストが $expected を検出しません" >&2
    exit 1
  fi
done

if grep -Fq 'print:border-black' <<<"$self_test_output"; then
  echo "ERROR: CSS トークン検査が print:border-black を違反として扱いました" >&2
  exit 1
fi

echo "  OK: 違反fixtureの自己テスト"

tmp_ts=$(mktemp -d)
printf '<div class="text-sm max-sm:text-xs"></div>\n' >"$tmp_ts/two.rs"
if [ "$(grep -HoE -e "$TYPE_SIZE" "$tmp_ts/two.rs" | wc -l)" -ne 2 ]; then
  echo "ERROR: 1行2件の文字サイズ直書きを2件と数えません" >&2
  rm -rf "$tmp_ts"
  exit 1
fi
printf '.a{margin:var(--space-page);padding:4px}.b{padding-inline-start:4px}\n' >"$tmp_ts/two.css"
if [ "$(grep -HoE -e "${SPACE_PROP}[^;}]*" "$tmp_ts/two.css" | grep -v 'var(--space' | wc -l)" -ne 2 ]; then
  echo "ERROR: トークン行に混ざった余白直書き・論理辺の余白を数えません" >&2
  rm -rf "$tmp_ts"
  exit 1
fi
rm -rf "$tmp_ts"
echo "  OK: 件数・宣言単位の自己テスト"

tmp_sp=$(mktemp -d)
printf '<div class="mt-4 px-3 gap-2 space-y-4 p-5 scroll-mt-18 lg:mx-0"></div>\n<div class="whitespace-nowrap min-w-0 max-sm:flex-col"></div>\n' >"$tmp_sp/sp.rs"
if [ "$(grep -HoE -e "$SPACING_RS" "$tmp_sp/sp.rs" | wc -l)" -ne 7 ]; then
  echo "ERROR: 余白ユーティリティ7件を数えません(whitespace/min-w/max-sm は対象外)" >&2
  rm -rf "$tmp_sp"
  exit 1
fi
printf 'spacing-features\t%s\t1\n' "$tmp_sp/sp.rs" >"$tmp_sp/baseline.txt"
if check_type_baseline "spacing-features" "$(grep -HoE -e "$SPACING_RS" "$tmp_sp/sp.rs")" "$tmp_sp/baseline.txt"; then
  echo "ERROR: 余白ユーティリティの基準超えを検出しません" >&2
  rm -rf "$tmp_sp"
  exit 1
fi
fail=0
rm -rf "$tmp_sp"
echo "  OK: 余白ユーティリティの自己テスト"
echo "OK: 違反はありません"

# 続けて features/ の直書きマークアップ検査を実行する
bash "$SCRIPT_DIR/check-ui-primitives.sh"
# 続けて色の直書き・style属性の色/寸法検査を実行する
bash "$SCRIPT_DIR/check-css-colors.sh"
# 続けて use の取り込み方検査を実行する
bash "$SCRIPT_DIR/check-use-paths.sh"
