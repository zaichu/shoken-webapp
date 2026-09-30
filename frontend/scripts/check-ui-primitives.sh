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
PRIMITIVE_CLASS='(panel-card|login-card|table-card|collapsible-card|feature-card|rail-panel|empty-state|summary-section-header|collapsible-trigger|rail-toggle|receipt-card-trigger|group-card-trigger|search-submit|chart-toggle-button|review-prompt-button|login-button|copyable-name|modal-close-button|filter-chip|filter-badge|code-badge|file-chip|skeleton-table-row)'
# 開閉トリガーは DisclosureToggle(または Button/FieldTrigger の aria_expanded)を使う
RAW_DISCLOSURE='aria-expanded'
# 状態表示は ui/ の部品(Alert/ListLoadError/ListSkeleton/Skeleton 等)を使う。
# role="alert"・animate-pulse は部品が持つ属性なので features/ では直書きしない
STATE_MARKUP='role="alert"|animate-pulse'

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
  # 第3引数 strip のときは文字列リテラル内の一致も違反にしない
  # (class="..." の値自体を検査する部品クラス検査には使えない)
  local label="$1" pattern="$2" strip="${3:-}" hits line text filtered=""
  shift 3
  hits=$(grep -HnE -e "$pattern" "$@" 2>/dev/null || true)
  while IFS= read -r line; do
    [ -z "$line" ] && continue
    text=${line#*:*:}
    # コメント行(// /* * で始まる行)は違反ではない
    grep -Eq '^[[:space:]]*(//|/\*|\*)' <<<"$text" && continue
    if [ -n "$strip" ] \
      && ! grep -qE "$pattern" <<<"$(sed -E 's/"[^"]*"//g' <<<"$text")"; then
      continue
    fi
    filtered+="$line"$'\n'
  done <<<"$hits"
  emit_check "$label" "${filtered%$'\n'}"
}

# タグ内の文字列と RSX 属性式の括弧を追跡し、属性値中の > を終端と誤認しない。
# タグ外では Rust の文字列・raw string・文字リテラル・行末コメントを読み飛ばし、
# それらに含まれるタグ風テキストを誤検知しない。
extract_class_tags() {
  awk '
    # Rust の文字リテラル(例: ( の3文字、\n の4文字)なら終端位置を返す。
    # ライフタイム(a のように閉じクォートを持たない)は -1 を返す。
    function char_lit_end(text, i,    n1, n3) {
      n1 = substr(text, i + 1, 1)
      if (n1 == "\\" && substr(text, i + 3, 1) == "\047") return i + 3
      if (substr(text, i + 2, 1) == "\047") return i + 2
      return -1
    }

    function start_tag() {
      in_tag = 1
      tag = "<"
      quote = ""
      escaped = 0
      paren = 0
      bracket = 0
      brace = 0
    }

    function finish_tag() {
      if (tag ~ /class/) {
        print tag
      }
      in_tag = 0
      tag = ""
    }

    /^[[:space:]]*(\/\/|\/\*|\*)/ { next }

    {
      text = $0 " "
      for (i = 1; i <= length(text); i++) {
        char = substr(text, i, 1)
        next_char = substr(text, i + 1, 1)

        if (!in_tag) {
          # 文字列リテラル "...": 中身とエスケープを読み飛ばす
          if (char == "\"") {
            j = i + 1
            while (j <= length(text)) {
              c2 = substr(text, j, 1)
              if (c2 == "\\") { j += 2; continue }
              if (c2 == "\"") break
              j++
            }
            i = j
            continue
          }
          # raw string r"..." / r#"..."#: 開始側の # の数と同じ閉端を探す
          if (char == "r" && match(substr(text, i + 1), /^#+"/)) {
            closer = "\"" substr(text, i + 1, RLENGTH - 1)
            j = index(substr(text, i + 1 + RLENGTH), closer)
            if (j > 0) i = i + RLENGTH + j + RLENGTH - 2
            continue
          }
          # 文字リテラル(ライフタイムは除く): 中の記号を構文と誤認しない
          if (char == "\047") {
            e = char_lit_end(text, i)
            if (e > 0) { i = e; continue }
          }
          # // 以降は行末コメント
          if (char == "/" && next_char == "/") break
          if (char == "<" && next_char ~ /[[:alpha:]\/]/) {
            start_tag()
          }
          continue
        }

        tag = tag char
        if (quote != "") {
          if (escaped) {
            escaped = 0
          } else if (char == "\\") {
            escaped = 1
          } else if (char == quote) {
            quote = ""
          }
        } else if (char == "\"") {
          quote = char
        } else if (char == "\047" && char_lit_end(text, i) > 0) {
          # 属性式中の文字リテラル(例: ()は括弧カウントを増やさない
          tag = tag substr(text, i + 1, char_lit_end(text, i) - i)
          i = char_lit_end(text, i)
        } else if (char == "(") {
          paren++
        } else if (char == ")" && paren > 0) {
          paren--
        } else if (char == "[") {
          bracket++
        } else if (char == "]" && bracket > 0) {
          bracket--
        } else if (char == "{") {
          brace++
        } else if (char == "}" && brace > 0) {
          brace--
        } else if (char == ">" && paren == 0 && bracket == 0 && brace == 0) {
          finish_tag()
        }
      }
    }
  ' "$@"
}

# カードに相当するユーティリティの組み合わせ(角丸・枠・面の直書きは Card の variant にする)。
# border-none / border-x はカード全体の枠ではないため除外し、完全な border トークンだけを見る。
# bg-surface-2 のような別トークンは除外し、bg-surface と opacity 修飾だけを同じ面として扱う。
check_card_like() {
  local label="$1" file tag hits=""
  local border_pattern='(^|[^[:alnum:]_-])border([^[:alnum:]_/-]|$)'
  local surface_pattern='(^|[^[:alnum:]_-])bg-surface(/[[:alnum:]_.-]+)?([^[:alnum:]_/-]|$)'
  shift
  for file in "$@"; do
    while IFS= read -r tag; do
      if grep -qE 'rounded-(lg|xl|2xl)' <<<"$tag" \
        && grep -qE "$border_pattern" <<<"$tag" \
        && grep -qE "$surface_pattern" <<<"$tag"; then
        hits+="${file}: $(printf '%.120s' "$tag")"$'\n'
      fi
    done < <(extract_class_tags "$file")
  done
  emit_check "$label" "${hits%$'\n'}"
}

if [ "$#" -gt 0 ]; then
  check "要素の直書き(自己テスト)" "$RAW_ELEMENT" strip "$@"
  check "部品クラスの直書き(自己テスト)" "$PRIMITIVE_CLASS" "" "$@"
  check_card_like "カード相当のクラス組み合わせ(自己テスト)" "$@"
  check "開閉属性の直書き(自己テスト)" "$RAW_DISCLOSURE" strip "$@"
  check "状態表示の直書き(自己テスト)" "$STATE_MARKUP" "" "$@"
  [ "$fail" -ne 0 ] && exit 1
  exit 0
fi

check "要素の直書き(src/features/**/*.rs)" "$RAW_ELEMENT" strip "${FEATURE_FILES[@]}"
check "部品クラスの直書き(src/features/**/*.rs)" "$PRIMITIVE_CLASS" "" "${FEATURE_FILES[@]}"
check_card_like "カード相当のクラス組み合わせ(src/features/**/*.rs)" "${FEATURE_FILES[@]}"
check "開閉属性の直書き(src/features/**/*.rs)" "$RAW_DISCLOSURE" strip "${FEATURE_FILES[@]}"
check "状態表示の直書き(src/features/**/*.rs)" "$STATE_MARKUP" "" "${FEATURE_FILES[@]}"

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

for expected in '<button' '<select' 'panel-card' 'collapsible-trigger' 'search-submit' 'empty-state' 'code-badge' 'bg-surface' 'class:bg-surface' 'mb-7' 'string-chevron' 'event-chevron' 'dynamic-card' 'char-lit-open' 'char-lit-close' 'aria-expanded' 'role="alert"' 'animate-pulse'; do
  if ! grep -Fq -- "$expected" <<<"$self_test_output"; then
    echo "ERROR: 基本部品検査の自己テストが $expected を検出しません" >&2
    exit 1
  fi
done

echo "  OK: 違反fixtureの自己テスト"

valid_output=""
if ! valid_output=$(bash "$SCRIPT_PATH" scripts/fixtures/ui-primitives-valid.txt 2>&1); then
  echo "ERROR: 基本部品検査がコメント・文字列リテラルを誤検知しました" >&2
  printf '%s\n' "$valid_output" >&2
  exit 1
fi
echo "  OK: 正常fixtureの自己テスト"
echo "OK: 直書きはありません"
