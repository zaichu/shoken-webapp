#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND_DIR="$ROOT_DIR/backend"

BACKEND_URL="${BACKEND_URL:-http://127.0.0.1:3001}"
FRONTEND_URL="${FRONTEND_URL:-http://127.0.0.1:8080}"

KEEP_DB=0

usage() {
  cat <<'EOF'
Usage:
  ./stop-local.sh [--keep-db]
  ./scripts/stop-local.sh [--keep-db]

Stops the local dev stack used by ./scripts/start-local.sh:
- Frontend dev server (default: port 8080)
- Backend server (default: port 3001)
- Local PostgreSQL via docker compose (unless --keep-db)

You can override ports via env vars:
  BACKEND_URL / BACKEND_PORT
  FRONTEND_URL / FRONTEND_PORT

Requires one of:
  - lsof (recommended)
  - ss
  - fuser
EOF
}

have_cmd() {
  command -v "$1" >/dev/null 2>&1
}

extract_port_from_url() {
  local url="$1"
  if [[ "${url}" =~ :([0-9]+)(/|$) ]]; then
    echo "${BASH_REMATCH[1]}"
  fi
}

http_ok() {
  local url="$1"
  if ! have_cmd curl; then
    return 1
  fi

  curl -sSf --connect-timeout 1 --max-time 2 "${url}" >/dev/null 2>&1
}

pids_listening_on_port() {
  local port="$1"

  if have_cmd lsof; then
    local out=""
    if out="$(lsof -nP -tiTCP:"${port}" -sTCP:LISTEN 2>/dev/null)"; then
      printf '%s\n' "${out}" | sed '/^$/d' | sort -u
      return 0
    fi

    local st=$?
    if [[ "${st}" -eq 1 ]]; then
      # No matches.
      return 0
    fi

    echo "WARN: lsof failed while checking port ${port} (exit ${st}); trying fallback..." >&2
  fi

  if have_cmd ss; then
    # Example: users:(("node",pid=1234,fd=23))
    ss -H -ltnp "sport = :${port}" 2>/dev/null \
      | grep -oE 'pid=[0-9]+' \
      | cut -d= -f2 \
      | sort -u \
      || true
    return 0
  fi

  if have_cmd fuser; then
    local out=""
    if out="$(fuser -n tcp "${port}" 2>/dev/null)"; then
      printf '%s\n' "${out}" \
        | tr ' ' '\n' \
        | sed '/^$/d' \
        | grep -E '^[0-9]+$' \
        | sort -u \
        || true
      return 0
    fi

    local st=$?
    if [[ "${st}" -eq 1 ]]; then
      # No matches.
      return 0
    fi

    echo "WARN: fuser failed while checking port ${port} (exit ${st})." >&2
    return "${st}"
  fi

  echo "ERROR: Cannot identify PID(s) listening on port ${port} (missing: lsof/ss/fuser)." >&2
  return 127
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

front_out=""
if ! front_out="$(pids_listening_on_port "${FRONTEND_PORT}")"; then
  echo "ERROR: Failed to detect frontend PID(s) for port ${FRONTEND_PORT}." >&2
  exit 1
fi
mapfile -t front_pids < <(printf '%s\n' "${front_out}" | sed '/^$/d' | grep -E '^[0-9]+$' || true)
if [[ "${#front_pids[@]}" -eq 0 ]] && http_ok "${FRONTEND_URL}/"; then
  echo "ERROR: Frontend responds at ${FRONTEND_URL} but no PID was detected for port ${FRONTEND_PORT}." >&2
  echo "       Install lsof/ss/fuser or run with sufficient permissions." >&2
  exit 1
fi
stop_pids "Frontend (port ${FRONTEND_PORT})" "${front_pids[@]}"

back_out=""
if ! back_out="$(pids_listening_on_port "${BACKEND_PORT}")"; then
  echo "ERROR: Failed to detect backend PID(s) for port ${BACKEND_PORT}." >&2
  exit 1
fi
mapfile -t back_pids < <(printf '%s\n' "${back_out}" | sed '/^$/d' | grep -E '^[0-9]+$' || true)
if [[ "${#back_pids[@]}" -eq 0 ]] && http_ok "${BACKEND_URL}/health"; then
  echo "ERROR: Backend responds at ${BACKEND_URL} but no PID was detected for port ${BACKEND_PORT}." >&2
  echo "       Install lsof/ss/fuser or run with sufficient permissions." >&2
  exit 1
fi
stop_pids "Backend (port ${BACKEND_PORT})" "${back_pids[@]}"

if [[ "${KEEP_DB}" -eq 1 ]]; then
  echo "DB: keep running (--keep-db)"
else
  echo "DB: stopping (docker compose down)..."
  (cd "${BACKEND_DIR}" && make db-down >/dev/null 2>&1) || true
  echo "DB: stopped"
fi
