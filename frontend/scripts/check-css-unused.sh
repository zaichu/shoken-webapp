#!/usr/bin/env bash
# CSS で定義した独自クラスが Rust 側で使われているかを確かめる。
# 棚卸し(docs/design/css-inventory.md §2)の手順を固定したもので、
# class="..."・class:name= 指令・文字列リテラル・format! の静的部分を拾い、
# data-testid/testid のみの一致は未使用と数える。
# 使い方: frontend/ で `bash scripts/check-css-unused.sh`
set -euo pipefail
shopt -s globstar nullglob

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
SCRIPT_PATH="$SCRIPT_DIR/$(basename "$0")"
cd "$SCRIPT_DIR/.."

CSS_FILE="style/input.css"
RS_FILES=(src/**/*.rs)

# 引数があるときは自己テスト用の指定として扱う(第1引数=CSS、第2引数以降=Rust)
if [ "$#" -gt 0 ]; then
  CSS_FILE="$1"
  shift
  RS_FILES=("$@")
fi

# テストで CSS 対応を保証している動的クラス(棚卸し§2の要確認)。
# 文字列合成のため静的検索では拾えないが、Rust テストが対応を保証する
TEST_BACKED='^(receipt-tier-|receipt-span-|comp-slice-|comp-bar-fill|comp-legend-dot)'

# CSS のブロックコメント(/* ... */、複数行も)を取り除く。
# コメント内の .rs などの文字列をクラス定義と誤認しないため
strip_css_comments() {
  perl -0777 -pe 's{/\*.*?\*/}{}gs'
}

# data-testid/testid の値を取り除く(同名の testid があっても使用中とは数えない)
strip_testids() {
  sed -E -e 's/(data-)?testid="[^"]*"//g' \
    -e "s/(data-)?testid='[^']*'//g" \
    -e 's/(data-)?testid=\{[^}]*\}//g'
}

# Rust のコメントを取り除く。行コメント(// ...)と RSX 内の {/* ... */} が対象。
# 通常の文字列リテラル内の // (URL など)は残す。raw string は src/ に無いため対象外
strip_rs_comments() {
  perl -ne '
    BEGIN { $q = 0 }
    @c = split //, $_;
    $out = "";
    for ($i = 0; $i < @c; $i++) {
      if ($q) {
        $out .= $c[$i];
        if ($c[$i] eq "\\") { $i++; $out .= $c[$i] if $i < @c; }
        elsif ($c[$i] eq "\"") { $q = 0; }
      } else {
        if ($c[$i] eq "\"") { $q = 1; $out .= $c[$i]; }
        elsif ($c[$i] eq "/" && $i + 1 < @c && $c[$i + 1] eq "/") { last; }
        elsif ($c[$i] eq "/" && $i + 1 < @c && $c[$i + 1] eq "*") {
          $i += 2;
          while ($i < @c && !($c[$i] eq "*" && $i + 1 < @c && $c[$i + 1] eq "/")) { $i++; }
          $i++;
        } else { $out .= $c[$i]; }
      }
    }
    print $out;
  '
}

check_unused() {
  local css="$1"
  shift
  local defined corpus tmp_corpus
  tmp_corpus=$(mktemp)
  for file in "$@"; do
    strip_rs_comments <"$file" | strip_testids >>"$tmp_corpus"
  done
  defined=$(strip_css_comments <"$css" | grep -oE '\.[a-z][a-z0-9_-]*' | sed 's/^\.//' | sort -u)
  local fail=0 class
  for class in $defined; do
    if grep -Eq "$TEST_BACKED" <<<"$class"; then
      continue
    fi
    if ! grep -Eq "(^|[^A-Za-z0-9_-])${class}([^A-Za-z0-9_-]|$)" "$tmp_corpus"; then
      echo "ERROR: 未使用のCSSクラス: .$class" >&2
      fail=1
    fi
  done
  rm -f "$tmp_corpus"
  if [ "$fail" -eq 0 ]; then
    echo "  OK: 未使用クラスなし"
  fi
  return "$fail"
}

if [ "$#" -eq 0 ]; then
  check_unused "$CSS_FILE" "${RS_FILES[@]}"
  status=$?
  [ "$status" -ne 0 ] && exit "$status"

  # 自己テスト: testid のみの一致は未使用、class=/class:/format! の静的部分は使用中、
  # テスト保証の動的クラスは対象外。CSS/Rust のコメントは無視する
  tmp_invalid=$(mktemp -d)
  printf '/* See component.rs for .commented-gone */\n.used-basic{color:red}.unused-gone{color:red}.testid-only{color:red}.used-directive{color:red}.used-format{color:red}.gone-widget{color:red}.receipt-tier-fixture{color:red}\n' >"$tmp_invalid/check.css"
  cat >"$tmp_invalid/check.rs" <<'EOF'
view! {
  // gone-widget was deleted
  <div class="used-basic">
    <span data-testid="testid-only">x</span>
    <div class:used-directive=flag />
    <p class=format!("used-format {extra}")>y</p>
    <p class=format!("receipt-tier-{}", level)>z</p>
  </div>
}
EOF
  invalid_output=""
  invalid_status=0
  if invalid_output=$(bash "$SCRIPT_PATH" "$tmp_invalid/check.css" "$tmp_invalid/check.rs" 2>&1); then
    invalid_status=0
  else
    invalid_status=$?
  fi
  rm -rf "$tmp_invalid"
  if [ "$invalid_status" -ne 1 ]; then
    echo "ERROR: 未使用クラス検査の自己テストが exit 1 になりません" >&2
    exit 1
  fi
  for expected in '.unused-gone' '.testid-only' '.gone-widget'; do
    if ! grep -Fq "$expected" <<<"$invalid_output"; then
      echo "ERROR: 未使用クラス検査の自己テストが $expected を検出しません" >&2
      exit 1
    fi
  done
  for unexpected in '.used-basic' '.used-directive' '.used-format' '.receipt-tier-fixture' '.commented-gone' '.rs'; do
    if grep -Fq "$unexpected" <<<"$invalid_output"; then
      echo "ERROR: 未使用クラス検査が使用中の $unexpected を未使用としました" >&2
      exit 1
    fi
  done
  echo "  OK: 違反fixtureの自己テスト"

  tmp_valid=$(mktemp -d)
  printf '/* See component.rs for details */\n.ok-one{color:red}\n' >"$tmp_valid/check.css"
  printf 'let url = "https://example.com/x";\n<div class="ok-one">x</div>\n' >"$tmp_valid/check.rs"
  valid_output=""
  if ! valid_output=$(bash "$SCRIPT_PATH" "$tmp_valid/check.css" "$tmp_valid/check.rs" 2>&1); then
    echo "ERROR: 未使用クラス検査が使用中クラスを誤検知しました" >&2
    printf '%s\n' "$valid_output" >&2
    rm -rf "$tmp_valid"
    exit 1
  fi
  rm -rf "$tmp_valid"
  echo "  OK: 正常fixtureの自己テスト"
  echo "OK: 未使用クラスはありません"
  exit 0
fi

check_unused "$CSS_FILE" "${RS_FILES[@]}"
