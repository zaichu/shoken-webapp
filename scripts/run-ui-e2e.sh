#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FRONTEND_DIR="$ROOT_DIR/frontend"

BACKEND_URL="${BACKEND_URL:-http://127.0.0.1:3001}"
FRONTEND_URL="${FRONTEND_URL:-http://127.0.0.1:8080}"
FRONTEND_PORT="${FRONTEND_PORT:-8080}"
STORAGE_STATE="${STORAGE_STATE:-$FRONTEND_DIR/.auth/storage-state.json}"

BACKEND_LOG="${BACKEND_LOG:-/tmp/shoken-backend-e2e.log}"
FRONTEND_LOG="${FRONTEND_LOG:-/tmp/shoken-frontend-e2e.log}"
DB_LOG="${DB_LOG:-/tmp/shoken-db-e2e.log}"
START_LOCAL_LOG="${START_LOCAL_LOG:-/tmp/shoken-start-local-e2e.log}"

RUN_MAIN=1
RUN_CSV=1
RUN_SAVE_AUTH=0
KEEP_RUNNING=0
START_ONLY=0

START_LOCAL_PID=""
STARTED_STACK=0

usage() {
  cat <<EOF
Usage: $(basename "$0") [options]

フロントエンド / バックエンドを起動または再利用し、
主要ページのスクショと CSV CRUD の Playwright をまとめて実行します。

Options:
  --skip-main     主要ページスクショ (npm run ui:screenshot:auth) をスキップ
  --skip-csv      CSV CRUD スクショ (npm run ui:screenshot:csv-crud) をスキップ
  --save-auth     実行前に npm run ui:save-auth を実行
  --start-only    サーバー起動 / 再利用だけ行って終了
  --keep-running  実行後も、このスクリプトが起動した backend / frontend を止めない
  --help          このヘルプを表示

Examples:
  $(basename "$0")
  $(basename "$0") --skip-csv
  $(basename "$0") --save-auth --keep-running
EOF
}

cleanup() {
  if [[ "$KEEP_RUNNING" -eq 1 ]]; then
    return 0
  fi

  if [[ "$STARTED_STACK" -eq 1 ]]; then
    if [[ -n "$START_LOCAL_PID" ]] && kill -0 "$START_LOCAL_PID" >/dev/null 2>&1; then
      kill "$START_LOCAL_PID" >/dev/null 2>&1 || true
      wait "$START_LOCAL_PID" >/dev/null 2>&1 || true
    else
      "$ROOT_DIR/scripts/stop-local.sh" --keep-db >/dev/null 2>&1 || true
    fi
  fi
}

print_log_tail() {
  local label="$1"
  local path="$2"

  if [[ -f "$path" ]]; then
    echo "--- $label: $path ---" >&2
    tail -n 80 "$path" >&2 || true
  fi
}

http_ok() {
  local url="$1"
  curl -sSf --connect-timeout 1 --max-time 2 "$url" >/dev/null 2>&1
}

ensure_stack() {
  echo "1/2 Ensuring local stack..."

  if http_ok "$BACKEND_URL/health" && http_ok "$FRONTEND_URL/"; then
    echo "  Reusing backend: $BACKEND_URL"
    echo "  Reusing frontend: $FRONTEND_URL"
    return 0
  fi

  "$ROOT_DIR/scripts/stop-local.sh" --keep-db >/dev/null 2>&1 || true

  if [[ "$KEEP_RUNNING" -eq 1 || "$START_ONLY" -eq 1 ]]; then
    nohup env \
      BACKEND_URL="$BACKEND_URL" \
      FRONTEND_URL="$FRONTEND_URL" \
      FRONTEND_PORT="$FRONTEND_PORT" \
      BACKEND_LOG="$BACKEND_LOG" \
      FRONTEND_LOG="$FRONTEND_LOG" \
      DB_LOG="$DB_LOG" \
      "$ROOT_DIR/scripts/start-local.sh" >"$START_LOCAL_LOG" 2>&1 &
  else
    env \
      BACKEND_URL="$BACKEND_URL" \
      FRONTEND_URL="$FRONTEND_URL" \
      FRONTEND_PORT="$FRONTEND_PORT" \
      BACKEND_LOG="$BACKEND_LOG" \
      FRONTEND_LOG="$FRONTEND_LOG" \
      DB_LOG="$DB_LOG" \
      "$ROOT_DIR/scripts/start-local.sh" >"$START_LOCAL_LOG" 2>&1 &
  fi
  START_LOCAL_PID=$!
  STARTED_STACK=1

  for _ in $(seq 1 120); do
    if ! kill -0 "$START_LOCAL_PID" >/dev/null 2>&1; then
      echo "start-local.sh exited before services became ready." >&2
      print_log_tail "start-local" "$START_LOCAL_LOG"
      print_log_tail "db" "$DB_LOG"
      print_log_tail "backend" "$BACKEND_LOG"
      print_log_tail "frontend" "$FRONTEND_LOG"
      exit 1
    fi

    if http_ok "$BACKEND_URL/health" && http_ok "$FRONTEND_URL/"; then
      return 0
    fi

    sleep 0.5
  done

  echo "Local stack did not become ready." >&2
  print_log_tail "start-local" "$START_LOCAL_LOG"
  print_log_tail "db" "$DB_LOG"
  print_log_tail "backend" "$BACKEND_LOG"
  print_log_tail "frontend" "$FRONTEND_LOG"
  exit 1
}

ensure_storage_state() {
  echo "2/2 Ensuring auth state..."

  if [[ "$RUN_SAVE_AUTH" -eq 1 ]]; then
    (cd "$FRONTEND_DIR" && npm run ui:save-auth)
    return 0
  fi

  if [[ ! -f "$STORAGE_STATE" ]]; then
    echo "storage state not found: $STORAGE_STATE" >&2
    echo "Run with --save-auth or create frontend/.auth/storage-state.json first." >&2
    exit 1
  fi
}

run_main_screenshots() {
  echo "Running main page screenshots..."
  (cd "$FRONTEND_DIR" && npm run ui:screenshot:auth)
}

run_csv_screenshots() {
  echo "Running CSV CRUD screenshots..."
  (cd "$FRONTEND_DIR" && npm run ui:screenshot:csv-crud)
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-main)
      RUN_MAIN=0
      shift
      ;;
    --skip-csv)
      RUN_CSV=0
      shift
      ;;
    --save-auth)
      RUN_SAVE_AUTH=1
      shift
      ;;
    --keep-running)
      KEEP_RUNNING=1
      shift
      ;;
    --start-only)
      START_ONLY=1
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ "$RUN_MAIN" -eq 0 && "$RUN_CSV" -eq 0 && "$START_ONLY" -eq 0 ]]; then
  echo "Nothing to run. Remove --skip-main/--skip-csv or use --start-only." >&2
  exit 1
fi

trap cleanup EXIT INT TERM

ensure_stack
if [[ "$START_ONLY" -eq 1 ]]; then
  echo "Ready:"
  echo "  Backend  : $BACKEND_URL"
  echo "  Frontend : $FRONTEND_URL"
  echo "Start only mode completed."
  exit 0
fi

ensure_storage_state

echo "Ready:"
echo "  Backend  : $BACKEND_URL"
echo "  Frontend : $FRONTEND_URL"
echo "  Screens  : $ROOT_DIR/.playwright-mcp"

if [[ "$RUN_MAIN" -eq 1 ]]; then
  run_main_screenshots
fi

if [[ "$RUN_CSV" -eq 1 ]]; then
  run_csv_screenshots
fi

echo "Completed."
