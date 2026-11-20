#!/usr/bin/env bash
# 画面を1コマンドで撮るスクリプト。使い方は usage 参照。
# データは全て架空のモック(frontend/scripts/ui-shots/data.mjs)で、
# 本番の接続情報・実保有データには触らない。
#
# latest/ が全画面の最新(390・1280・1920 × ライト・ダーク)。
# PR の確認用は pr<番号>/before・after(開いている間だけ置き、マージや close で消す)。

set -euo pipefail
unset NO_COLOR

usage() {
  cat <<'EOF'
使い:
  frontend/scripts/ui-shots.sh <名前> <before|after> --pr <番号> [オプション]
  frontend/scripts/ui-shots.sh <名前> latest [オプション]
  frontend/scripts/ui-shots.sh --cleanup-pr <番号> [オプション]

  <名前>           コミットメッセージに出すラベル(例: i1234)
  <before|after>   変更前 / 変更後のどちらを撮るか(PR 確認用。pr<番号>/ に置く)
  latest           全画面の最新を撮り直し、latest/ を置き換える
  --cleanup-pr     pr<番号>/ を消す(PR のマージ・close 後に使う。撮らない)

オプション:
  --pr <番号>            画像を置く pr<番号>/ の番号(before・after で必須。ブランチ名には Issue 番号が入るため)
  --only <画面一覧>      撮る画面を絞る(例: --only home,asset-stock)
  --viewports <幅一覧>   撮る幅を絞る(latest の既定: 390,1280,1920。PR 確認用は既定で全幅)
  --shots-branch <名>    画像を置くブランチ(既定: screenshots。push 動作の検証用に一時名を指定できる)
  --no-push              画像を push しない(撮るだけ。--cleanup-pr とは併用不可)
  --detail-mobile        詳細画面を360pxと390pxで撮る(1280pxは一覧画面を撮る)
  -h, --help             この説明を表示する
EOF
}

CLEANUP_PR=""
NAME=""
SIDE=""
ONLY=""
PR_NUMBER=""
VIEWPORTS=""
SHOTS_BRANCH="screenshots"
PUSH=1
DETAIL_MOBILE=0
while [ "$#" -gt 0 ]; do
  case "$1" in
    --cleanup-pr)
      CLEANUP_PR="$2"
      shift 2
      ;;
    --only)
      ONLY="$2"
      shift 2
      ;;
    --pr)
      PR_NUMBER="$2"
      shift 2
      ;;
    --viewports)
      VIEWPORTS="$2"
      shift 2
      ;;
    --shots-branch)
      SHOTS_BRANCH="$2"
      shift 2
      ;;
    --no-push)
      PUSH=0
      shift
      ;;
    --detail-mobile)
      DETAIL_MOBILE=1
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    -*)
      echo "不明な引数: $1" >&2
      usage >&2
      exit 2
      ;;
    *)
      if [ -z "$NAME" ]; then
        NAME="$1"
      elif [ -z "$SIDE" ]; then
        SIDE="$1"
      else
        echo "引数が多すぎます: $1" >&2
        usage >&2
        exit 2
      fi
      shift
      ;;
  esac
done

if [ -n "$CLEANUP_PR" ]; then
  if [ -n "$NAME" ] || [ -n "$SIDE" ] || [ -n "$PR_NUMBER" ]; then
    echo "--cleanup-pr と撮影引数は同時に指定できません" >&2
    exit 2
  fi
  if [ "$PUSH" -eq 0 ]; then
    echo "--cleanup-pr と --no-push は同時に指定できません(削除の publish が目的なため)" >&2
    exit 2
  fi
elif [ -z "$NAME" ] || [ -z "$SIDE" ]; then
  usage >&2
  exit 2
fi

case "$SIDE" in
  "" | before | after | latest) ;;
  *)
    echo "第2引数は before / after / latest です: $SIDE" >&2
    exit 2
    ;;
esac

if { [ "$SIDE" = "before" ] || [ "$SIDE" = "after" ]; } && [ -z "$PR_NUMBER" ]; then
  echo "--pr で PR 番号を指定してください" >&2
  usage >&2
  exit 2
fi
if [ "$SIDE" = "latest" ] && [ -z "$VIEWPORTS" ]; then
  VIEWPORTS="390,1280,1920"
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
  if command -v ss >/dev/null 2>&1; then
    ss -ltnH "sport = :$1" | grep -q .
  else
    timeout 1 bash -c 'exec 3<>"/dev/tcp/127.0.0.1/$1"' _ "$1" >/dev/null 2>&1
  fi
}

# 選んだポートを自分のサーバーが保持しているかを確認する。
# 他の撮影とポートを競合すると相手のポートに接続してしまうため、プロセスまで確かめる
server_owns_port() {
  if command -v ss >/dev/null 2>&1; then
    local listeners pid
    listeners="$(ss -ltnpH "sport = :$1" 2>/dev/null)"
    printf '%s\n' "$listeners" | grep -q "pid=$SERVER_PID," && return 0
    while read -r pid; do
      [ -n "$pid" ] || continue
      if [ "$(ps -o ppid= -p "$pid" | tr -d ' ')" = "$SERVER_PID" ]; then
        printf '%s\n' "$listeners" | grep -q "pid=$pid," && return 0
      fi
    done < <(pgrep -P "$SERVER_PID" || true)
    # ここまで一致なし=待ち受けの持ち主が別人(または不在)のため失敗にする。
    # 素の return は直前の while の終了状態(空読みで 0)を返し、
    # 待ち受け未確認のまま成功扱いになるため使わない
    return 1
  fi
  kill -0 "$SERVER_PID" 2>/dev/null
}

# 画像ブランチの worktree を用意する。検証用の一時ブランチ名を指定したときは
# まだ存在しないため、起点は screenshots にして push 先だけ一時名にする
with_shots_worktree() {
  local work_dir="$1"
  REMOTE="$(git remote get-url origin)"
  REPO_SLUG="${REMOTE##*github.com}"
  REPO_SLUG="${REPO_SLUG#:}"
  REPO_SLUG="${REPO_SLUG#/}"
  REPO_SLUG="${REPO_SLUG%.git}"
  if ! [[ "$REPO_SLUG" =~ ^[A-Za-z0-9._-]+/[A-Za-z0-9._-]+$ ]]; then
    echo "origin からリポジトリ名を取り出せません: $REMOTE" >&2
    exit 1
  fi

  echo "==> 画像ブランチ ${SHOTS_BRANCH} へ push(${work_dir})"
  git fetch origin
  git fetch origin "$SHOTS_BRANCH" 2>/dev/null || true
  git worktree prune
  if [ -e "$work_dir" ]; then
    # 名前が同じでも持ち主が違う可能性があるため、git に worktree として登録されているか確かめる
    if ! git worktree list --porcelain | grep -Fxq "worktree $work_dir"; then
      echo "同名ディレクトリがあります。$work_dir を確認してください" >&2
      exit 1
    fi
    # 変更や未 push のコミットがある worktree は git が削除を拒む
    if ! git worktree remove "$work_dir"; then
      echo "撮影用 worktree に変更があります。$work_dir を確認してください" >&2
      exit 1
    fi
  fi
  if git rev-parse --verify --quiet "origin/${SHOTS_BRANCH}" >/dev/null; then
    git worktree add --detach "$work_dir" "origin/${SHOTS_BRANCH}"
  else
    git worktree add --detach "$work_dir" origin/screenshots
  fi
}

# 画像ブランチへ通常 push する(force push しない)。
# 新規ブランチにも押せるよう送り先は refs/heads/ で修飾する。
# 他の lane が先に push していたら取り込んでからもう1度 push する。
# 呼び出し側はコマンド置換で使うため set -e が効かない。
# 失敗は明示的に return し、merge の標準出力は SHA の取り出しに混ざるため捨てる
push_shots() {
  local dest="refs/heads/${SHOTS_BRANCH}"
  if [ "$(git rev-parse HEAD)" != "$(git rev-parse "origin/${SHOTS_BRANCH}" 2>/dev/null || echo none)" ]; then
    if ! git push origin "HEAD:${dest}"; then
      git fetch origin "$SHOTS_BRANCH" || return 1
      git merge "origin/${SHOTS_BRANCH}" >&2 || return 1
      git push origin "HEAD:${dest}" || return 1
    fi
  fi
  git rev-parse HEAD
}

build_and_shoot() {
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
  SHOOT_ARGS=(--port "$PORT" --out "$SHOT_DIR")
  if [ -n "$ONLY" ]; then
    SHOOT_ARGS+=(--only "$ONLY")
  fi
  if [ -n "$VIEWPORTS" ]; then
    SHOOT_ARGS+=(--viewports "$VIEWPORTS")
  fi
  if [ "$DETAIL_MOBILE" -eq 1 ]; then
    node scripts/ui-shots/shoot.mjs "${SHOOT_ARGS[@]}" --detail-mobile
  else
    node scripts/ui-shots/shoot.mjs "${SHOOT_ARGS[@]}"
  fi

  stop_server
  echo "==> サーバー停止(ポート $PORT)"
}

if [ -n "$CLEANUP_PR" ]; then
  SHOTS_DIR="$(dirname "$REPO_ROOT")/shots-cleanup-${CLEANUP_PR}"
  with_shots_worktree "$SHOTS_DIR"
  cd "$SHOTS_DIR"
  if [ ! -d "pr${CLEANUP_PR}" ]; then
    echo "pr${CLEANUP_PR}/ はありません。消すものはありません"
    cd "$REPO_ROOT"
    git worktree remove "$SHOTS_DIR"
    exit 0
  fi
  git rm -r "pr${CLEANUP_PR}"
  git commit -m "pr${CLEANUP_PR}: 確認用スクリーンショットを削除"
  push_shots
  cd "$REPO_ROOT"
  git worktree remove "$SHOTS_DIR"
  exit 0
fi

build_and_shoot

if [ "$PUSH" -eq 0 ]; then
  echo "==> push しないため画像は $SHOT_DIR に残しました"
  exit 0
fi

if [ "$SIDE" = "latest" ]; then
  SHOTS_DIR="$(dirname "$REPO_ROOT")/shots-latest"
  with_shots_worktree "$SHOTS_DIR"
  MAIN_SHA="$(git rev-parse origin/main)"
  NOW="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

  TARGET_DIR="$SHOTS_DIR/latest"
  mkdir -p "$TARGET_DIR"
  rm -f "$TARGET_DIR"/*.png
  cp "$SHOT_DIR"/*.png "$TARGET_DIR/"
  node "$FRONTEND_DIR/scripts/ui-shots/latest-readme.mjs" \
    --dir "$TARGET_DIR" --commit "$MAIN_SHA" --date "$NOW"

  cd "$SHOTS_DIR"
  git add latest
  if ! git diff --cached --quiet; then
    git commit -m "${NAME}: 最新の全画面スクリーンショット"
  fi
  SHA="$(push_shots)" || exit 1

  cd "$REPO_ROOT"
  git worktree remove "$SHOTS_DIR"
  exit 0
fi

SHOTS_DIR="$(dirname "$REPO_ROOT")/shots-pr${PR_NUMBER}"
with_shots_worktree "$SHOTS_DIR"

TARGET_DIR="$SHOTS_DIR/pr${PR_NUMBER}/${SIDE}"
mkdir -p "$TARGET_DIR"
cp "$SHOT_DIR"/*.png "$TARGET_DIR/"

cd "$SHOTS_DIR"
git add "pr${PR_NUMBER}/${SIDE}"
if ! git diff --cached --quiet; then
  git commit -m "pr${PR_NUMBER}: ${NAME} のスクリーンショット(${SIDE})"
fi
SHA="$(push_shots)" || exit 1

echo "==> Markdown($MARKDOWN_FILE にも出しました)"
node "$FRONTEND_DIR/scripts/ui-shots/markdown.mjs" \
  --pr "$PR_NUMBER" \
  --sha "$SHA" \
  --repo "$REPO_SLUG" \
  --dir "$SHOTS_DIR" | tee "$FRONTEND_DIR/$MARKDOWN_FILE"

cd "$REPO_ROOT"
git worktree remove "$SHOTS_DIR"
