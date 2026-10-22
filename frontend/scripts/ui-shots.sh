#!/usr/bin/env bash
# 画面を1コマンドで撮るスクリプト(Issue #1484)。
#
#   frontend/scripts/ui-shots.sh <名前> <before|after> [--only 画面一覧] [--pr 番号] [--no-push]
#
# 1. フレッシュにビルド(test-results/dist-leptos を作り直す)
# 2. 空きポート(8160 以降)で配信を起動する(自分が起動したプロセスだけ後で止める)
# 3. 標準セットの画面をダーク・ライト × 1280×800 / 1920×1080 / 390×844 で撮る
# 4. 画像を screenshots ブランチの pr<番号>/<before|after>/ に push して
#    PR 本文に貼る Markdown(変更前後の表)を出力する
#
# データの出所は全て架空のモック(frontend/scripts/ui-shots/data.mjs)。
# 本番の接続情報・実保有データには触らない。

set -euo pipefail

usage() {
  cat <<'EOF'
使い: frontend/scripts/ui-shots.sh <名前> <before|after> [オプション]

  <名前>           コミットメッセージに出すラベル(例: i1484)
  <before|after>   変更前 / 変更後のどちらを撮るか

オプション:
  --only <画面一覧>  撮る画面を絞る(例: --only home,asset-stock)
  --pr <番号>        screenshots ブランチの pr<番号>/ に置く。省略時はブランチ名から取る
  --no-push          画像を push しない(撮るだけ)
  -h, --help         この説明を表示する
EOF
}

if [ "$#" -lt 2 ]; then
  usage >&2
  exit 2
fi

NAME="$1"
SIDE="$2"
shift 2

ONLY=""
PR_NUMBER=""
PUSH=1
while [ "$#" -gt 0 ]; do
  case "$1" in
    --only)
      ONLY="$2"
      shift 2
      ;;
    --pr)
      PR_NUMBER="$2"
      shift 2
      ;;
    --no-push)
      PUSH=0
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "不明な引数: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

case "$SIDE" in
  before | after) ;;
  *)
    echo "第2引数は before か after です: $SIDE" >&2
    exit 2
    ;;
esac

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FRONTEND_DIR="$(dirname "$SCRIPT_DIR")"
cd "$FRONTEND_DIR"

REPO_ROOT="$(git rev-parse --show-toplevel)"
DIST_DIR="test-results/dist-leptos"
SHOT_DIR="test-results/shots"
MARKDOWN_FILE="test-results/shots.md"

if [ -z "$PR_NUMBER" ]; then
  BRANCH="$(git rev-parse --abbrev-ref HEAD)"
  if [[ "$BRANCH" =~ ([0-9]+) ]]; then
    PR_NUMBER="${BASH_REMATCH[1]}"
  fi
fi
if [ -z "$PR_NUMBER" ]; then
  echo "PR 番号が分かりません。--pr で指定してください(ブランチ: $(git rev-parse --abbrev-ref HEAD))" >&2
  exit 2
fi

SERVER_PID=""
stop_server() {
  if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  SERVER_PID=""
}
trap stop_server EXIT INT TERM

port_in_use() {
  (exec 3<>"/dev/tcp/127.0.0.1/$1") >/dev/null 2>&1
}

echo "==> ビルド(${DIST_DIR})"
rm -rf "$DIST_DIR" "$SHOT_DIR"
trunk build --release --dist "$DIST_DIR"
node scripts/prepare-dist.mjs "$DIST_DIR" ''

PORT=""
for candidate in $(seq 8160 8199); do
  if ! port_in_use "$candidate"; then
    PORT="$candidate"
    break
  fi
done
if [ -z "$PORT" ]; then
  echo "8160〜8199 に空いているポートがありません" >&2
  exit 1
fi

echo "==> 配信(ポート $PORT)"
node scripts/serve-dist.mjs "$DIST_DIR" "$PORT" >test-results/ui-shots-server.log 2>&1 &
SERVER_PID=$!
for _ in $(seq 1 60); do
  if port_in_use "$PORT"; then
    break
  fi
  sleep 0.5
done
if ! port_in_use "$PORT"; then
  echo "サーバーが起動しませんでした(test-results/ui-shots-server.log を確認)" >&2
  exit 1
fi

echo "==> 撮影"
mkdir -p "$SHOT_DIR"
if [ -n "$ONLY" ]; then
  node scripts/ui-shots/shoot.mjs --port "$PORT" --out "$SHOT_DIR" --only "$ONLY"
else
  node scripts/ui-shots/shoot.mjs --port "$PORT" --out "$SHOT_DIR"
fi

stop_server
echo "==> サーバー停止(ポート $PORT)"

if [ "$PUSH" -eq 0 ]; then
  echo "==> push しないため画像は $SHOT_DIR に残しました"
  exit 0
fi

REMOTE="$(git remote get-url origin)"
REPO_SLUG="${REMOTE##*github.com}"
REPO_SLUG="${REPO_SLUG#:}"
REPO_SLUG="${REPO_SLUG#/}"
REPO_SLUG="${REPO_SLUG%.git}"
if ! [[ "$REPO_SLUG" =~ ^[A-Za-z0-9._-]+/[A-Za-z0-9._-]+$ ]]; then
  echo "origin からリポジトリ名を取り出せません: $REMOTE" >&2
  exit 1
fi

SHOTS_DIR="$(dirname "$REPO_ROOT")/shots-pr${PR_NUMBER}"
echo "==> screenshots ブランチへ push(${SHOTS_DIR})"
git fetch origin screenshots
git worktree prune
if [ -d "$SHOTS_DIR" ]; then
  git worktree remove --force "$SHOTS_DIR" 2>/dev/null || rm -rf "$SHOTS_DIR"
fi
git worktree add --detach "$SHOTS_DIR" origin/screenshots

TARGET_DIR="$SHOTS_DIR/pr${PR_NUMBER}/${SIDE}"
mkdir -p "$TARGET_DIR"
cp "$SHOT_DIR"/*.png "$TARGET_DIR/"

cd "$SHOTS_DIR"
git add "pr${PR_NUMBER}/${SIDE}"
git commit -m "pr${PR_NUMBER}: ${NAME} のスクリーンショット(${SIDE})"
if ! git push origin HEAD:screenshots; then
  # 他の lane が先に push していたら取り込んでからもう1度 push する(force push はしない)
  git fetch origin screenshots
  git merge origin/screenshots
  git push origin HEAD:screenshots
fi
SHA="$(git rev-parse HEAD)"

echo "==> Markdown($MARKDOWN_FILE にも出しました)"
node "$FRONTEND_DIR/scripts/ui-shots/markdown.mjs" \
  --pr "$PR_NUMBER" \
  --sha "$SHA" \
  --repo "$REPO_SLUG" \
  --dir "$SHOTS_DIR" | tee "$FRONTEND_DIR/$MARKDOWN_FILE"

cd "$REPO_ROOT"
git worktree remove --force "$SHOTS_DIR"
