#!/usr/bin/env bash
# features/ で ui/ の基本部品を通さない直書きマークアップを検出する。
# ボタン・カード・開閉・選択・金額は src/ui/ の部品(Button/Card/DisclosureToggle/Chip/Amount 等)を使うこと。
# 使い方: frontend/ で `bash scripts/check-ui-primitives.sh`
set -euo pipefail
shopt -s globstar nullglob

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
SCRIPT_PATH="$SCRIPT_DIR/$(basename "$0")"
cd "$SCRIPT_DIR/.."

# features/ では <button> / <select> を直書きしない(ui/ の部品を使う)
RAW_ELEMENT='<button|<select'
# カード・空状態・開閉・ボタン・バッジのコンポーネントクラスの直書き
PRIMITIVE_CLASS='(panel-card|login-card|table-card|collapsible-card|feature-card|rail-panel|empty-state|summary-section-header|collapsible-trigger|rail-toggle|receipt-card-trigger|group-card-trigger|search-submit|chart-toggle-button|review-prompt-button|login-button|copyable-name|modal-close-button|filter-chip|filter-badge|code-badge|file-chip)'
# カードに相当するユーティリティの組み合わせ(角丸・枠・面・影の直書きは Card の variant にする)。
# class 属性内に rounded-(lg|xl|2xl)・border・bg-surface が揃うものを検出する(順不同)。
CARD_LIKE='class="[^"]*(rounded-(lg|xl|2xl)[^"]*border[^"]*bg-surface|rounded-(lg|xl|2xl)[^"]*bg-surface[^"]*border|border[^"]*rounded-(lg|xl|2xl)[^"]*bg-surface|border[^"]*bg-surface[^"]*rounded-(lg|xl|2xl)|bg-surface[^"]*rounded-(lg|xl|2xl)[^"]*border|bg-surface[^"]*border[^"]*rounded-(lg|xl|2xl))'
# 開閉トリガーは DisclosureToggle(または Button/FieldTrigger の aria_expanded)を使う
RAW_DISCLOSURE='aria-expanded'

# 許可リスト: "ファイルパス:行" 形式。部品で表せない正当な直書きだけを理由付きで列挙する
ALLOWED=()

fail=0
FEATURE_FILES=(src/features/**/*.rs)

check() {
  local label="$1" pattern="$2"
  shift 2
  local hits
  hits=$(grep -HnoE -e "$pattern" "$@" 2>/dev/null || true)
  if [ -n "$hits" ] && [ "${#ALLOWED[@]}" -gt 0 ]; then
    local allowed
    for allowed in "${ALLOWED[@]}"; do
      hits=$(printf '%s\n' "$hits" | grep -vF -- "${allowed}:" || true)
    done
  fi
  [ -z "$hits" ] && { echo "  OK: $label"; return 0; }
  echo "ERROR: $label" >&2
  printf '%s\n' "$hits" >&2
  fail=1
}

if [ "$#" -gt 0 ]; then
  check "要素の直書き(自己テスト)" "$RAW_ELEMENT" "$@"
  check "部品クラスの直書き(自己テスト)" "$PRIMITIVE_CLASS" "$@"
  check "カード相当のクラス組み合わせ(自己テスト)" "$CARD_LIKE" "$@"
  check "開閉属性の直書き(自己テスト)" "$RAW_DISCLOSURE" "$@"
  [ "$fail" -ne 0 ] && exit 1
  exit 0
fi

check "要素の直書き(src/features/**/*.rs)" "$RAW_ELEMENT" "${FEATURE_FILES[@]}"
check "部品クラスの直書き(src/features/**/*.rs)" "$PRIMITIVE_CLASS" "${FEATURE_FILES[@]}"
check "カード相当のクラス組み合わせ(src/features/**/*.rs)" "$CARD_LIKE" "${FEATURE_FILES[@]}"
check "開閉属性の直書き(src/features/**/*.rs)" "$RAW_DISCLOSURE" "${FEATURE_FILES[@]}"

[ "$fail" -ne 0 ] && exit 1

self_test_output=""
self_test_status=0
if self_test_output=$(bash "$SCRIPT_PATH" scripts/fixtures/ui-primitives-invalid.txt 2>&1); then
  self_test_status=0
else
  self_test_status=$?
fi

if [ "$self_test_status" -ne 1 ]; then
  echo "ERROR: 基本部品検査の自己テストが exit 1 になりません" >&2
  exit 1
fi

for expected in '<button' '<select' 'panel-card' 'collapsible-trigger' 'search-submit' 'empty-state' 'code-badge' 'bg-surface' 'aria-expanded'; do
  if ! grep -Fq -- "$expected" <<<"$self_test_output"; then
    echo "ERROR: 基本部品検査の自己テストが $expected を検出しません" >&2
    exit 1
  fi
done

echo "  OK: 違反fixtureの自己テスト"
echo "OK: 直書きはありません"
