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

fail=0
FEATURE_FILES=(src/features/**/*.rs)

check() {
  local label="$1" pattern="$2"
  shift 2
  local hits
  hits=$(grep -HnoE -e "$pattern" "$@" 2>/dev/null || true)
  [ -z "$hits" ] && { echo "  OK: $label"; return 0; }
  echo "ERROR: $label" >&2
  printf '%s\n' "$hits" >&2
  fail=1
}

if [ "$#" -gt 0 ]; then
  check "要素の直書き(自己テスト)" "$RAW_ELEMENT" "$@"
  check "部品クラスの直書き(自己テスト)" "$PRIMITIVE_CLASS" "$@"
  [ "$fail" -ne 0 ] && exit 1
  exit 0
fi

check "要素の直書き(src/features/**/*.rs)" "$RAW_ELEMENT" "${FEATURE_FILES[@]}"
check "部品クラスの直書き(src/features/**/*.rs)" "$PRIMITIVE_CLASS" "${FEATURE_FILES[@]}"

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

for expected in '<button' '<select' 'panel-card' 'collapsible-trigger' 'search-submit' 'empty-state' 'code-badge'; do
  if ! grep -Fq -- "$expected" <<<"$self_test_output"; then
    echo "ERROR: 基本部品検査の自己テストが $expected を検出しません" >&2
    exit 1
  fi
done

echo "  OK: 違反fixtureの自己テスト"
echo "OK: 直書きはありません"
