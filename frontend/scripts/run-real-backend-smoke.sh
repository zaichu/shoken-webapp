#!/usr/bin/env bash
set -euo pipefail

# 実 backend に接続して取引明細の smoke を実行する。
# 起動順は DB -> backend -> frontend、終了時にスタックを止める。

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="$ROOT_DIR/backend"
LEPTOS_DIR="$ROOT_DIR/frontend"

BACKEND_URL="${BACKEND_URL:-http://127.0.0.1:3001}"
LEPTOS_PORT="${LEPTOS_PORT:-8096}"
FRONTEND_URL="http://127.0.0.1:${LEPTOS_PORT}"

SMOKE_USER_ID="${SMOKE_USER_ID:-00000000-0000-0000-0000-000000000101}"
SMOKE_SESSION_TOKEN="${SMOKE_SESSION_TOKEN:-00000000-0000-0000-0000-000000000102}"
SMOKE_SECURITY_NAME="${SMOKE_SECURITY_NAME:-ＫＤＤＩ}"

START_LOCAL_LOG="${START_LOCAL_LOG:-/tmp/shoken-leptos-smoke-start-local.log}"
START_LOCAL_PID=""

stop_stack() {
  FRONTEND_URL="$FRONTEND_URL" \
    FRONTEND_PORT="$LEPTOS_PORT" \
    BACKEND_URL="$BACKEND_URL" \
    "$ROOT_DIR/scripts/stop-local.sh" >/dev/null 2>&1
}

cleanup() {
  local status=$?
  if [[ -n "$START_LOCAL_PID" ]] && kill -0 "$START_LOCAL_PID" >/dev/null 2>&1; then
    kill "$START_LOCAL_PID" >/dev/null 2>&1 || true
    wait "$START_LOCAL_PID" >/dev/null 2>&1 || true
  fi
  if ! stop_stack; then
    echo "WARNING: stop-local.sh failed; backend/DB may still be running" >&2
  fi
  if [[ "$status" -ne 0 ]]; then
    tail -n 100 "$START_LOCAL_LOG" >&2 || true
  fi
  exit "$status"
}
trap cleanup EXIT INT TERM

# 既存スタックの残骸を止める
stop_stack || echo "WARNING: 既存スタックの停止に失敗しました" >&2

echo "Starting DB, backend, and Leptos via start-local.sh..."
BACKEND_URL="$BACKEND_URL" \
  FRONTEND_URL="$FRONTEND_URL" \
  FRONTEND_PORT="$LEPTOS_PORT" \
  CORS_ORIGINS="http://localhost:${LEPTOS_PORT},${FRONTEND_URL}" \
  "$ROOT_DIR/scripts/start-local.sh" >"$START_LOCAL_LOG" 2>&1 &
START_LOCAL_PID=$!

for _ in $(seq 1 240); do
  if curl -sSf --connect-timeout 1 --max-time 2 "$BACKEND_URL/ready" >/dev/null 2>&1; then
    break
  fi
  if ! kill -0 "$START_LOCAL_PID" >/dev/null 2>&1; then
    echo "start-local.sh exited before backend became ready." >&2
    exit 1
  fi
  sleep 0.5
done
curl -sSf "$BACKEND_URL/ready" >/dev/null

for _ in $(seq 1 240); do
  if curl -sSf --connect-timeout 1 --max-time 2 "$FRONTEND_URL/" >/dev/null 2>&1; then
    break
  fi
  if ! kill -0 "$START_LOCAL_PID" >/dev/null 2>&1; then
    echo "start-local.sh exited before frontend became ready." >&2
    exit 1
  fi
  sleep 0.5
done
curl -sSf "$FRONTEND_URL/" >/dev/null

echo "Seeding smoke fixture..."
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
    user_id, settlement_date, product, account, security_code, security_name,
    unit_price, shares, dividends_before_tax, taxes, net_amount_received
) VALUES (
    :'user_id'::uuid, DATE '2026-06-01', '国内株式', '特定・一般', '9433',
    :'security_name', 7.95, 2000, 15900, 3228, 12672
);
COMMIT;
SQL
)

echo "Running Leptos real-backend smoke spec..."
export PATH="$LEPTOS_DIR/node_modules/.bin:$PATH"
(
  cd "$LEPTOS_DIR"
  REAL_BACKEND_URL="$BACKEND_URL" \
    REAL_FRONTEND_URL="$FRONTEND_URL" \
    REAL_BACKEND_SESSION_TOKEN="$SMOKE_SESSION_TOKEN" \
    REAL_BACKEND_EXPECTED_SECURITY_NAME="$SMOKE_SECURITY_NAME" \
    BASE_URL="$FRONTEND_URL" \
    LEPTOS_E2E_PORT="$LEPTOS_PORT" \
    npx playwright test --config playwright.receipts.config.ts real-backend-receipts-smoke.spec.ts
)

echo "Leptos real-backend receipts smoke passed."
