#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND_DIR="$ROOT_DIR/backend"

BACKEND_URL="${BACKEND_URL:-http://127.0.0.1:3001}"
FRONTEND_URL="${FRONTEND_URL:-http://127.0.0.1:8080}"

KEEP_DB=0

usage() {
  cat <<'EOF'
Usage: ./scripts/stop-local.sh [--keep-db]

Stops the local dev stack used by ./scripts/start-local.sh:
- Frontend dev server (default: port 8080)
- Backend server (default: port 3001)
- Local PostgreSQL via docker compose (unless --keep-db)

You can override ports via env vars:
  BACKEND_URL / BACKEND_PORT
  FRONTEND_URL / FRONTEND_PORT
EOF
}

extract_port_from_url() {
  local url="$1"
  if [[ "${url}" =~ :([0-9]+)(/|$) ]]; then
    echo "${BASH_REMATCH[1]}"
  fi
}

pids_listening_on_port() {
  local port="$1"
  lsof -nP -tiTCP:"${port}" -sTCP:LISTEN 2>/dev/null | sort -u || true
}

print_pids() {
  local pid
  for pid in "$@"; do
    if [[ -n "${pid}" ]] && kill -0 "${pid}" >/dev/null 2>&1; then
      echo "  - ${pid}: $(ps -o command= -p "${pid}" 2>/dev/null || echo "?")"
    fi
  done
}

stop_pids() {
  local label="$1"
  shift
  local pids=("$@")

  if [[ "${#pids[@]}" -eq 0 ]]; then
    echo "${label}: not running"
    return 0
  fi

  echo "${label}: stopping..."
  print_pids "${pids[@]}"

  kill "${pids[@]}" >/dev/null 2>&1 || true

  local deadline=$((SECONDS + 10))
  while [[ "${SECONDS}" -lt "${deadline}" ]]; do
    local still=()
    local pid
    for pid in "${pids[@]}"; do
      if [[ -n "${pid}" ]] && kill -0 "${pid}" >/dev/null 2>&1; then
        still+=("${pid}")
      fi
    done

    if [[ "${#still[@]}" -eq 0 ]]; then
      echo "${label}: stopped"
      return 0
    fi

    sleep 0.2
  done

  local still=()
  local pid
  for pid in "${pids[@]}"; do
    if [[ -n "${pid}" ]] && kill -0 "${pid}" >/dev/null 2>&1; then
      still+=("${pid}")
    fi
  done

  if [[ "${#still[@]}" -gt 0 ]]; then
    echo "${label}: forcing stop (SIGKILL)..."
    print_pids "${still[@]}"
    kill -9 "${still[@]}" >/dev/null 2>&1 || true
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --keep-db)
      KEEP_DB=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

BACKEND_PORT="${BACKEND_PORT:-$(extract_port_from_url "${BACKEND_URL}")}"
BACKEND_PORT="${BACKEND_PORT:-3001}"

FRONTEND_PORT="${FRONTEND_PORT:-$(extract_port_from_url "${FRONTEND_URL}")}"
FRONTEND_PORT="${FRONTEND_PORT:-8080}"

mapfile -t front_pids < <(pids_listening_on_port "${FRONTEND_PORT}")
stop_pids "Frontend (port ${FRONTEND_PORT})" "${front_pids[@]}"

mapfile -t back_pids < <(pids_listening_on_port "${BACKEND_PORT}")
stop_pids "Backend (port ${BACKEND_PORT})" "${back_pids[@]}"

if [[ "${KEEP_DB}" -eq 1 ]]; then
  echo "DB: keep running (--keep-db)"
else
  echo "DB: stopping (docker compose down)..."
  (cd "${BACKEND_DIR}" && make db-down >/dev/null 2>&1) || true
  echo "DB: stopped"
fi

