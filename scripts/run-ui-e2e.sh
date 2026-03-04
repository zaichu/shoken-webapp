#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND_DIR="$ROOT_DIR/backend"
FRONTEND_DIR="$ROOT_DIR/frontend"

BACKEND_URL="${BACKEND_URL:-http://127.0.0.1:3001}"
FRONTEND_URL="${FRONTEND_URL:-http://127.0.0.1:8080}"
FRONTEND_PORT="${FRONTEND_PORT:-8080}"
STORAGE_STATE="${STORAGE_STATE:-$FRONTEND_DIR/.auth/storage-state.json}"

BACKEND_LOG="${BACKEND_LOG:-/tmp/shoken-backend-e2e.log}"
FRONTEND_LOG="${FRONTEND_LOG:-/tmp/shoken-frontend-e2e.log}"
DB_LOG="${DB_LOG:-/tmp/shoken-db-e2e.log}"

RUN_MAIN=1
RUN_CSV=1
RUN_SAVE_AUTH=0
KEEP_RUNNING=0
START_ONLY=0

BACK_PID=""
FRONT_PID=""

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

  if [[ -n "$FRONT_PID" ]] && kill -0 "$FRONT_PID" >/dev/null 2>&1; then
    kill "$FRONT_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "$BACK_PID" ]] && kill -0 "$BACK_PID" >/dev/null 2>&1; then
    kill "$BACK_PID" >/dev/null 2>&1 || true
  fi
}

wait_for_http_ok() {
  local url="$1"
  local label="$2"
  local retries="${3:-120}"
  local interval="${4:-0.5}"

  for _ in $(seq 1 "$retries"); do
    if curl -sSf "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep "$interval"
  done

  echo "$label did not become ready: $url" >&2
  return 1
}

ensure_db() {
  echo "1/4 Ensuring local PostgreSQL..."
  (cd "$BACKEND_DIR" && make db-up >"$DB_LOG" 2>&1)

  for i in $(seq 1 120); do
    if (cd "$BACKEND_DIR" && docker compose -f docker-compose.yml exec -T postgres pg_isready -U user -d shoken_db >/dev/null 2>&1); then
      return 0
    fi
    sleep 0.5
    if [[ "$i" -eq 120 ]]; then
      echo "Local DB did not become ready." >&2
      tail -n 80 "$DB_LOG" >&2 || true
      exit 1
    fi
  done
}

ensure_backend() {
  echo "2/4 Ensuring backend..."
  if curl -sSf "$BACKEND_URL/health" >/dev/null 2>&1; then
    echo "  Reusing backend: $BACKEND_URL"
    return 0
  fi

  (cd "$BACKEND_DIR" && make run >"$BACKEND_LOG" 2>&1) &
  BACK_PID=$!

  if ! wait_for_http_ok "$BACKEND_URL/health" "Backend" 120 0.5; then
    tail -n 80 "$BACKEND_LOG" >&2 || true
    exit 1
  fi
}

ensure_frontend() {
  echo "3/4 Ensuring frontend..."
  if curl -sSf "$FRONTEND_URL/" >/dev/null 2>&1; then
    echo "  Reusing frontend: $FRONTEND_URL"
    return 0
  fi

  (
    cd "$FRONTEND_DIR"
    VITE_SHOKEN_WEBAPI_API_URL="$BACKEND_URL" \
      npm run dev -- --host 127.0.0.1 --port "$FRONTEND_PORT" --strictPort >"$FRONTEND_LOG" 2>&1
  ) &
  FRONT_PID=$!

  if ! wait_for_http_ok "$FRONTEND_URL/" "Frontend" 120 0.5; then
    tail -n 80 "$FRONTEND_LOG" >&2 || true
    exit 1
  fi
}

ensure_storage_state() {
  if [[ "$RUN_SAVE_AUTH" -eq 1 ]]; then
    echo "4/4 Saving auth state..."
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

ensure_db
ensure_backend
ensure_frontend
ensure_storage_state

echo "Ready:"
echo "  Backend  : $BACKEND_URL"
echo "  Frontend : $FRONTEND_URL"
echo "  Screens  : $ROOT_DIR/.playwright-mcp"

if [[ "$START_ONLY" -eq 1 ]]; then
  echo "Start only mode completed."
  exit 0
fi

if [[ "$RUN_MAIN" -eq 1 ]]; then
  run_main_screenshots
fi

if [[ "$RUN_CSV" -eq 1 ]]; then
  run_csv_screenshots
fi

echo "Completed."
