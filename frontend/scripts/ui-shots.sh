#!/usr/bin/env bash
# 画面を1コマンドで撮るスクリプト。使い方は usage 参照。
# データは全て架空のモック(frontend/scripts/ui-shots/data.mjs)で、
# 本番の接続情報・実保有データには触らない。

set -euo pipefail

usage() {
  cat <<'EOF'
使い: frontend/scripts/ui-shots.sh <名前> <before|after> --pr <番号> [オプション]

  <名前>           コミットメッセージに出すラベル(例: i1234)
  <before|after>   変更前 / 変更後のどちらを撮るか

オプション:
  --pr <番号>        画像を置く pr<番号>/ の番号(必須。ブランチ名には Issue 番号が入るため)
  --only <画面一覧>  撮る画面を絞る(例: --only home,asset-stock)
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

if [ -z "$PR_NUMBER" ]; then
  echo "--pr で PR 番号を指定してください" >&2
  usage >&2
  exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FRONTEND_DIR="$(dirname "$SCRIPT_DIR")"
cd "$FRONTEND_DIR"

REPO_ROOT="$(git rev-parse --show-toplevel)"
DIST_DIR="test-results/dist-leptos"
SHOT_DIR="test-results/shots"
MARKDOWN_FILE="test-results/shots.md"
SERVER_LOG="test-results/ui-shots-server.log"

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

# 選んだポートを自分のサーバーが保持しているかを確認する。
# 他の撮影とポートを競合すると相手のポートに接続してしまうため、プロセスまで確かめる
server_owns_port() {
  if command -v ss >/dev/null 2>&1; then
    ss -ltnpH "sport = :$1" 2>/dev/null | grep -q "pid=$SERVER_PID,"
    return
  fi
  kill -0 "$SERVER_PID" 2>/dev/null
}

echo "==> ビルド(${DIST_DIR})"
rm -rf "$DIST_DIR" "$SHOT_DIR"
trunk build --release --dist "$DIST_DIR"
node scripts/prepare-dist.mjs "$DIST_DIR" ''

PORT=""
for candidate in $(seq 8160 8199); do
  if port_in_use "$candidate"; then
    continue
  fi
  node scripts/serve-dist.mjs "$DIST_DIR" "$candidate" >"$SERVER_LOG" 2>&1 &
  SERVER_PID=$!
  for _ in $(seq 1 60); do
    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
      break
    fi
    if server_owns_port "$candidate"; then
      PORT="$candidate"
      break
    fi
    sleep 0.5
  done
  if [ -n "$PORT" ]; then
    break
  fi
  echo "ポート $candidate で起動できなかったため次のポートを試します"
  stop_server
done
if [ -z "$PORT" ]; then
  echo "8160〜8199 にサーバーを起動できませんでした(${SERVER_LOG} を確認)" >&2
  exit 1
fi

echo "==> 配信(ポート $PORT)"
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
git fetch origin
git worktree prune
if [ -e "$SHOTS_DIR" ]; then
  # 名前が同じでも持ち主が違う可能性があるため、git に worktree として登録されているか確かめる
  if ! git worktree list --porcelain | grep -Fxq "worktree $SHOTS_DIR"; then
    echo "同名ディレクトリがあります。$SHOTS_DIR を確認してください" >&2
    exit 1
  fi
  # 変更や未 push のコミットがある worktree は git が削除を拒む
  if ! git worktree remove "$SHOTS_DIR"; then
    echo "撮影用 worktree に変更があります。$SHOTS_DIR を確認してください" >&2
    exit 1
  fi
fi
git worktree add --detach "$SHOTS_DIR" origin/screenshots

TARGET_DIR="$SHOTS_DIR/pr${PR_NUMBER}/${SIDE}"
mkdir -p "$TARGET_DIR"
cp "$SHOT_DIR"/*.png "$TARGET_DIR/"

cd "$SHOTS_DIR"
git add "pr${PR_NUMBER}/${SIDE}"
if ! git diff --cached --quiet; then
  git commit -m "pr${PR_NUMBER}: ${NAME} のスクリーンショット(${SIDE})"
fi
if [ "$(git rev-parse HEAD)" != "$(git rev-parse origin/screenshots)" ]; then
  if ! git push origin HEAD:screenshots; then
    # 他の lane が先に push していたら取り込んでからもう1度 push する(force push はしない)
    git fetch origin screenshots
    git merge origin/screenshots
    git push origin HEAD:screenshots
  fi
fi
SHA="$(git rev-parse HEAD)"

echo "==> Markdown($MARKDOWN_FILE にも出しました)"
node "$FRONTEND_DIR/scripts/ui-shots/markdown.mjs" \
  --pr "$PR_NUMBER" \
  --sha "$SHA" \
  --repo "$REPO_SLUG" \
  --dir "$SHOTS_DIR" | tee "$FRONTEND_DIR/$MARKDOWN_FILE"

cd "$REPO_ROOT"
git worktree remove "$SHOTS_DIR"
