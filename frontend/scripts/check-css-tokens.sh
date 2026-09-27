#!/usr/bin/env bash
# frontend のクラス文字列から、生パレット・旧トークン・白黒直書き・任意値を検出する。
# 新規の色・値は style/input.css の @theme トークンか ui/ コンポーネントで表すこと。
# 使い方: frontend/ で `bash scripts/check-css-tokens.sh`
set -euo pipefail
shopt -s globstar nullglob

cd "$(dirname "$0")/.."

PALETTE='-(slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-[0-9]{2,3}'
OLD_TOKEN='(text|bg|border|divide|ring|outline|placeholder|from|via|to|fill|stroke|caret|decoration|shadow|ring-offset)-(primary|primary-hover|primary-dark|secondary|secondary-hover|success|success-hover|info-hover|warning|warning-hover|danger|danger-hover|light|dark|negative-dark|bg-body|bg-dark|bg-card-dark|border-dark)\b'
# 白・黒の直書き(bg-white など)。`bg-surface` / `text-text-inverse` / `bg-scrim` などのトークンを使う
MONO='(text|bg|border|divide|ring|outline|placeholder|from|via|to|fill|stroke|caret|decoration|shadow|ring-offset|accent)-(white|black)\b'
# `-[...]` 形式の任意値。`]` 以外は許すので content-["..."] など引用符を含むものも拾う
ARBITRARY='[a-z][a-z-]*-\[[^]]*\]'

# 許可リスト: 追加するときは理由を添える。形式は grep -E の正規表現(一致する行を除外)。
ALLOWED_SRC=(
  'transition-\['   # 遷移プロパティの列挙は任意値でしか書けない
  'min-w-\[8ch\]'   # 表の金額列は桁幅(ch 単位)で指定する
  'min-h-\[50vh\]'  # 404・モーダルの縦位置はビューポート基準のため
)
# input.css の @layer components 内ではレイアウトの任意値を許す(レール付き 2 カラムの grid template 等)。
# src 側の要素直書きは許可しない
ALLOWED_CSS=("${ALLOWED_SRC[@]}" 'grid-cols-\[')

fail=0
RS_FILES=(src/**/*.rs)

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

check_arbitrary() {
  local label="$1" allowed="$2"
  shift 2
  local hits deny
  hits=$(grep -noE -e "$ARBITRARY" "$@" 2>/dev/null || true)
  [ -z "$hits" ] && { echo "  OK: 任意値($label)"; return 0; }
  deny=$(printf '%s\n' "$hits" | grep -vE "$allowed" || true)
  [ -z "$deny" ] && { echo "  OK: 任意値(すべて許可リスト内: $label)"; return 0; }
  echo "ERROR: 許可リスト外の任意値($label)" >&2
  printf '%s\n' "$deny" >&2
  fail=1
}

check "生パレット(src/**/*.rs)" "$PALETTE" "${RS_FILES[@]}"
check "旧トークン(src/**/*.rs)" "$OLD_TOKEN" "${RS_FILES[@]}"
check "白・黒の直書き(src/**/*.rs)" "$MONO" "${RS_FILES[@]}"
check_arbitrary "src/**/*.rs" "$(join_or "${ALLOWED_SRC[@]}")" "${RS_FILES[@]}"
check "生パレット(style/input.css)" "$PALETTE" style/input.css
check "旧トークン(style/input.css)" "$OLD_TOKEN" style/input.css
check_arbitrary "style/input.css" "$(join_or "${ALLOWED_CSS[@]}")" style/input.css

[ "$fail" -ne 0 ] && exit 1
echo "OK: 違反はありません"
