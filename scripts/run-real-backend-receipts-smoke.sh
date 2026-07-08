#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND_DIR="$ROOT_DIR/backend"
FRONTEND_DIR="$ROOT_DIR/frontend"

BACKEND_URL="${BACKEND_URL:-http://127.0.0.1:3001}"
FRONTEND_URL="${FRONTEND_URL:-http://127.0.0.1:8080}"
FRONTEND_PORT="${FRONTEND_PORT:-8080}"

BACKEND_LOG="${BACKEND_LOG:-/tmp/shoken-real-backend-smoke-backend.log}"
FRONTEND_LOG="${FRONTEND_LOG:-/tmp/shoken-real-backend-smoke-frontend.log}"
DB_LOG="${DB_LOG:-/tmp/shoken-real-backend-smoke-db.log}"
START_LOCAL_LOG="${START_LOCAL_LOG:-/tmp/shoken-real-backend-smoke-start-local.log}"

SMOKE_USER_ID="${SMOKE_USER_ID:-00000000-0000-0000-0000-000000000101}"
SMOKE_SESSION_TOKEN="${SMOKE_SESSION_TOKEN:-00000000-0000-0000-0000-000000000102}"
SMOKE_SECURITY_NAME="${SMOKE_SECURITY_NAME:-ＫＤＤＩ}"

START_LOCAL_PID=""

usage() {
  cat <<'EOF'
Usage:
  ./scripts/run-real-backend-receipts-smoke.sh

Starts the local stack, seeds an authenticated dividend fixture into the local
PostgreSQL database, runs a Playwright smoke test against the real backend, and
stops the local stack afterward.

Environment overrides:
  BACKEND_URL / FRONTEND_URL / FRONTEND_PORT
  SMOKE_USER_ID / SMOKE_SESSION_TOKEN / SMOKE_SECURITY_NAME
  BACKEND_LOG / FRONTEND_LOG / DB_LOG / START_LOCAL_LOG
EOF
}

print_log_tail() {
  local label="$1"
  local path="$2"

  if [[ -f "$path" ]]; then
    echo "--- ${label}: ${path} ---" >&2
    tail -n 100 "$path" >&2 || true
  fi
}

cleanup() {
  local status=$?

  if [[ -n "$START_LOCAL_PID" ]] && kill -0 "$START_LOCAL_PID" >/dev/null 2>&1; then
    kill "$START_LOCAL_PID" >/dev/null 2>&1 || true
    wait "$START_LOCAL_PID" >/dev/null 2>&1 || true
  fi

  "$ROOT_DIR/scripts/stop-local.sh" >/dev/null 2>&1 || true

  if [[ "$status" -ne 0 ]]; then
    print_log_tail "start-local" "$START_LOCAL_LOG"
    print_log_tail "db" "$DB_LOG"
    print_log_tail "backend" "$BACKEND_LOG"
    print_log_tail "frontend" "$FRONTEND_LOG"
  fi

  exit "$status"
}

http_ok() {
  local url="$1"
  curl -sSf --connect-timeout 1 --max-time 2 "$url" >/dev/null 2>&1
}

wait_for_http_ok() {
  local url="$1"
  local label="$2"

  for _ in $(seq 1 180); do
    if http_ok "$url"; then
      return 0
    fi

    if [[ -n "$START_LOCAL_PID" ]] && ! kill -0 "$START_LOCAL_PID" >/dev/null 2>&1; then
      echo "start-local.sh exited before ${label} became ready." >&2
      return 1
    fi

    sleep 0.5
  done

  echo "${label} did not become ready: ${url}" >&2
  return 1
}

seed_fixture() {
  echo "Seeding real-backend smoke fixture..."
  (
    cd "$BACKEND_DIR"
    docker compose -f docker-compose.yml exec -T postgres \
      psql -U user -d shoken_db -v ON_ERROR_STOP=1 \
        -v user_id="$SMOKE_USER_ID" \
        -v session_token="$SMOKE_SESSION_TOKEN" \
        -v security_name="$SMOKE_SECURITY_NAME" <<'SQL'
BEGIN;

DELETE FROM dividends WHERE user_id = :'user_id'::uuid;
DELETE FROM sessions WHERE id = :'session_token'::uuid OR user_id = :'user_id'::uuid;
DELETE FROM users WHERE id = :'user_id'::uuid;

INSERT INTO users (id, google_id, email, name, picture_url)
VALUES (:'user_id'::uuid, 'real-backend-smoke-user', 'real-backend-smoke@example.com', '実backendスモーク', NULL);

INSERT INTO sessions (id, user_id, expires_at)
VALUES (:'session_token'::uuid, :'user_id'::uuid, NOW() + INTERVAL '7 days');

INSERT INTO dividends (
    user_id,
    settlement_date,
    product,
    account,
    security_code,
    security_name,
    unit_price,
    shares,
    dividends_before_tax,
    taxes,
    net_amount_received
)
VALUES (
    :'user_id'::uuid,
    DATE '2026-06-01',
    '国内株式',
    '特定・一般',
    '9433',
    :'security_name',
    7.95,
    2000,
    15900,
    3228,
    12672
);

COMMIT;
SQL
  )
}

run_smoke() {
  echo "Running Playwright real-backend receipts smoke..."
  (
    cd "$FRONTEND_DIR"
    REAL_BACKEND_URL="$BACKEND_URL" \
      REAL_FRONTEND_URL="$FRONTEND_URL" \
      REAL_BACKEND_SESSION_TOKEN="$SMOKE_SESSION_TOKEN" \
      REAL_BACKEND_EXPECTED_SECURITY_NAME="$SMOKE_SECURITY_NAME" \
      BASE_URL="$FRONTEND_URL" \
      VITE_SHOKEN_WEBAPI_API_URL="$BACKEND_URL" \
      npx playwright test e2e/real-backend-receipts-smoke.spec.ts
  )
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

trap cleanup EXIT INT TERM

echo "Stopping existing local stack..."
"$ROOT_DIR/scripts/stop-local.sh" >/dev/null 2>&1 || true

echo "Starting local stack..."
BACKEND_URL="$BACKEND_URL" \
  FRONTEND_URL="$FRONTEND_URL" \
  FRONTEND_PORT="$FRONTEND_PORT" \
  BACKEND_LOG="$BACKEND_LOG" \
  FRONTEND_LOG="$FRONTEND_LOG" \
  DB_LOG="$DB_LOG" \
  "$ROOT_DIR/scripts/start-local.sh" >"$START_LOCAL_LOG" 2>&1 &
START_LOCAL_PID=$!

wait_for_http_ok "$BACKEND_URL/ready" "Backend readiness"
wait_for_http_ok "$FRONTEND_URL/" "Frontend"

seed_fixture
run_smoke

echo "Real-backend receipts smoke passed."
