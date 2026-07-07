#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND_DIR="$ROOT_DIR/backend"
FRONTEND_DIR="$ROOT_DIR/frontend"

BACKEND_URL="${BACKEND_URL:-http://127.0.0.1:3001}"
FRONTEND_URL="${FRONTEND_URL:-http://127.0.0.1:8080}"
FRONTEND_PORT="${FRONTEND_PORT:-8080}"
DATABASE_URL="${DATABASE_URL:-postgresql://user:password@localhost:5432/shoken_db}"
CORS_ORIGINS="${CORS_ORIGINS:-http://localhost:${FRONTEND_PORT},${FRONTEND_URL}}"

BACKEND_LOG="${BACKEND_LOG:-/tmp/shoken-backend-dev.log}"
FRONTEND_LOG="${FRONTEND_LOG:-/tmp/shoken-frontend-dev.log}"
DB_LOG="${DB_LOG:-/tmp/shoken-db-up.log}"

BACK_PID=""
FRONT_PID=""

cleanup() {
  if [[ -n "${FRONT_PID}" ]] && kill -0 "${FRONT_PID}" >/dev/null 2>&1; then
    kill "${FRONT_PID}" >/dev/null 2>&1 || true
  fi
  if [[ -n "${BACK_PID}" ]] && kill -0 "${BACK_PID}" >/dev/null 2>&1; then
    kill "${BACK_PID}" >/dev/null 2>&1 || true
  fi
}

wait_for_http_ok() {
  local url="$1"
  local label="$2"
  local retries="${3:-120}"
  local interval_sec="${4:-0.5}"

  for i in $(seq 1 "${retries}"); do
    if curl -sSf "${url}" >/dev/null 2>&1; then
      return 0
    fi
    sleep "${interval_sec}"
  done

  echo "${label} did not become ready: ${url}" >&2
  return 1
}

echo "1/3 Starting local PostgreSQL..."
(cd "${BACKEND_DIR}" && make db-up >"${DB_LOG}" 2>&1)

for i in $(seq 1 120); do
  if (cd "${BACKEND_DIR}" && docker compose -f docker-compose.yml exec -T postgres pg_isready -U user -d shoken_db >/dev/null 2>&1); then
    break
  fi
  sleep 0.5
  if [[ "${i}" -eq 120 ]]; then
    echo "Local DB did not become ready." >&2
    tail -n 80 "${DB_LOG}" >&2 || true
    exit 1
  fi
done

echo "2/3 Starting backend..."
(
  cd "${BACKEND_DIR}"
  DATABASE_URL="${DATABASE_URL}" \
    BACKEND_URL="${BACKEND_URL}" \
    FRONTEND_URL="${FRONTEND_URL}" \
    CORS_ORIGINS="${CORS_ORIGINS}" \
    make run >"${BACKEND_LOG}" 2>&1
) &
BACK_PID=$!

if ! wait_for_http_ok "${BACKEND_URL}/health" "Backend" 120 0.5; then
  tail -n 80 "${BACKEND_LOG}" >&2 || true
  cleanup
  exit 1
fi

echo "3/3 Starting frontend..."
(
  cd "${FRONTEND_DIR}"
  VITE_SHOKEN_WEBAPI_API_URL="${BACKEND_URL}" \
    npm run dev -- --host 127.0.0.1 --port "${FRONTEND_PORT}" --strictPort >"${FRONTEND_LOG}" 2>&1
) &
FRONT_PID=$!

if ! wait_for_http_ok "${FRONTEND_URL}/" "Frontend" 120 0.5; then
  tail -n 80 "${FRONTEND_LOG}" >&2 || true
  cleanup
  exit 1
fi

trap cleanup EXIT INT TERM

echo "Ready:"
echo "  DB       : postgres://user:password@localhost:5432/shoken_db"
echo "  Backend  : ${BACKEND_URL}"
echo "  Frontend : ${FRONTEND_URL}"
echo
echo "Logs:"
echo "  ${DB_LOG}"
echo "  ${BACKEND_LOG}"
echo "  ${FRONTEND_LOG}"
echo
echo "Press Ctrl+C to stop backend/frontend."

wait -n "${BACK_PID}" "${FRONT_PID}" || true
echo "One of the app servers exited. Stopping the remaining process..." >&2
