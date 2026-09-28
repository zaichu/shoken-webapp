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
# 開閉トリガーは DisclosureToggle(または Button/FieldTrigger の aria_expanded)を使う
RAW_DISCLOSURE='aria-expanded'

# 許可リスト: "ファイルパス:行" 形式。部品で表せない正当な直書きだけを理由付きで列挙する
# (カード相当の組み合わせはタグ単位で検出するため行番号を出さず、許可リストは適用できない)
ALLOWED=()

fail=0
FEATURE_FILES=(src/features/**/*.rs)

emit_check() {
  local label="$1" hits="$2" allowed
  if [ -n "$hits" ] && [ "${#ALLOWED[@]}" -gt 0 ]; then
    for allowed in "${ALLOWED[@]}"; do
      hits=$(printf '%s\n' "$hits" | grep -vF -- "${allowed}:" || true)
    done
  fi
  [ -z "$hits" ] && { echo "  OK: $label"; return 0; }
  echo "ERROR: $label" >&2
  printf '%s\n' "$hits" >&2
  fail=1
}

check() {
  local label="$1" pattern="$2" hits
  shift 2
  hits=$(grep -HnoE -e "$pattern" "$@" 2>/dev/null || true)
  emit_check "$label" "$hits"
}

# カードに相当するユーティリティの組み合わせ(角丸・枠・面の直書きは Card の variant にする)。
# タグは複数行にまたがり得るのでファイルを1行に潰して <...> 単位で切り出し、
# class="..."(改行を含む)と class:xxx の両方の指定を同じタグ内の組み合わせとして見る
check_card_like() {
  local label="$1" file tag hits=""
  shift
  for file in "$@"; do
    while IFS= read -r tag; do
      if grep -qE 'rounded-(lg|xl|2xl)' <<<"$tag" \
        && grep -q 'border' <<<"$tag" \
        && grep -q 'bg-surface' <<<"$tag"; then
        hits+="${file}: $(printf '%.120s' "$tag")"$'\n'
      fi
    done < <(tr '\n' ' ' <"$file" | grep -oE '<[^>]*class[^>]*>' || true)
  done
  emit_check "$label" "${hits%$'\n'}"
}

if [ "$#" -gt 0 ]; then
  check "要素の直書き(自己テスト)" "$RAW_ELEMENT" "$@"
  check "部品クラスの直書き(自己テスト)" "$PRIMITIVE_CLASS" "$@"
  check_card_like "カード相当のクラス組み合わせ(自己テスト)" "$@"
  check "開閉属性の直書き(自己テスト)" "$RAW_DISCLOSURE" "$@"
  [ "$fail" -ne 0 ] && exit 1
  exit 0
fi

check "要素の直書き(src/features/**/*.rs)" "$RAW_ELEMENT" "${FEATURE_FILES[@]}"
check "部品クラスの直書き(src/features/**/*.rs)" "$PRIMITIVE_CLASS" "${FEATURE_FILES[@]}"
check_card_like "カード相当のクラス組み合わせ(src/features/**/*.rs)" "${FEATURE_FILES[@]}"
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

for expected in '<button' '<select' 'panel-card' 'collapsible-trigger' 'search-submit' 'empty-state' 'code-badge' 'bg-surface' 'class:bg-surface' 'mb-7' 'aria-expanded'; do
  if ! grep -Fq -- "$expected" <<<"$self_test_output"; then
    echo "ERROR: 基本部品検査の自己テストが $expected を検出しません" >&2
    exit 1
  fi
done

echo "  OK: 違反fixtureの自己テスト"
echo "OK: 直書きはありません"
