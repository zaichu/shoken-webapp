#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND_DIR="$ROOT_DIR/backend"

BACKEND_URL="${BACKEND_URL:-}"
FRONTEND_URL="${FRONTEND_URL:-}"

BACKEND_PID_FILE_EXPLICIT="${BACKEND_PID_FILE:-}"
FRONTEND_PID_FILE_EXPLICIT="${FRONTEND_PID_FILE:-}"
BACKEND_PID_FILE="${BACKEND_PID_FILE_EXPLICIT}"
FRONTEND_PID_FILE="${FRONTEND_PID_FILE_EXPLICIT}"

KEEP_DB=0

usage() {
  cat <<'EOF'
Usage:
  ./stop-local.sh [--keep-db]
  ./scripts/stop-local.sh [--keep-db]

Stops the local dev stack used by ./scripts/start-local.sh:
- Frontend dev server (default: port 8081)
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
  # 03001 のような先頭ゼロ付き表記を 3001 と同一ポートとして扱うため 10 進数に揃える
  if [[ "${url}" =~ :([0-9]+)(/|$) ]]; then
    echo "$((10#${BASH_REMATCH[1]}))"
  fi
}

http_ok() {
  local url="$1"
  if ! have_cmd curl; then
    return 1
  fi

  curl -sSf --connect-timeout 1 --max-time 2 "${url}" >/dev/null 2>&1
}

pids_from_file() {
  local file="$1"
  local pid=""

  if [[ -f "${file}" ]]; then
    pid="$(sed -n '1p' "${file}" 2>/dev/null | tr -d '[:space:]' || true)"
  fi
  if [[ "${pid}" =~ ^[0-9]+$ ]] && kill -0 "${pid}" >/dev/null 2>&1; then
    printf '%s\n' "${pid}"
  fi
  return 0
}

scoped_pid_tag() {
  local base="worktree"
  local hash="0"
  base="$(basename "${ROOT_DIR}")"
  hash="$(printf '%s' "${ROOT_DIR}" | cksum 2>/dev/null | cut -d' ' -f1)"
  if [[ -z "${hash}" ]]; then
    hash="0"
  fi
  printf '%s-%s' "${base}" "${hash}"
}

pid_command() {
  ps -o command= -p "$1" 2>/dev/null || true
}

pid_started() {
  ps -p "$1" -o lstart= 2>/dev/null | sed 's/^ *//' || true
}

# PID ファイルの2行目と起動中プロセスの起動時刻が一致する場合だけ有効とする
file_pid_identity_ok() {
  local file="$1"
  local pid="$2"
  local recorded=""
  local current=""
  recorded="$(sed -n '2p' "${file}" 2>/dev/null | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
  if [[ -z "${recorded}" ]]; then
    return 1
  fi
  current="$(pid_started "${pid}")"
  [[ -n "${current}" && "${current}" == "${recorded}" ]]
}

file_pid_valid() {
  local service="$1"
  local pid="$2"
  local port="$3"
  local cmd=""
  local name=""
  local cwd=""
  cmd="$(pid_command "${pid}")"
  if [[ -z "${cmd}" ]]; then
    return 1
  fi
  name="$(ps -p "${pid}" -o comm= 2>/dev/null)" || return 1
  cwd="$(readlink -f "/proc/${pid}/cwd" 2>/dev/null)" || return 1
  if [[ "${service}" == "frontend" ]]; then
    if [[ "${name}" != "trunk" || "${cwd}" != "${ROOT_DIR}/frontend" ]]; then
      return 1
    fi
    if [[ "${cmd}" == *"--port "* ]] && [[ ! "${cmd}" =~ --port[[:space:]]+${port}([[:space:]]|$) ]]; then
      return 1
    fi
    return 0
  fi
  if [[ "${cwd}" != "${BACKEND_DIR}" ]]; then
    return 1
  fi
  if [[ "${name}" == "backend" ]] || [[ "${name}" == "cargo" && "${cmd}" =~ (^|[[:space:]])run[[:space:]]+--bin[[:space:]]+backend([[:space:]]|$) ]]; then
    return 0
  fi
  return 1
}

collect_service_pids() {
  local service="$1"
  local file="$2"
  local port="$3"
  local file_pid=""
  local port_out=""
  local scan_st=0
  local -A seen=()
  local pid=""

  file_pid="$(pids_from_file "${file}")"
  if [[ -n "${file_pid}" ]]; then
    if file_pid_valid "${service}" "${file_pid}" "${port}" \
      && file_pid_identity_ok "${file}" "${file_pid}"; then
      printf '%s\n' "${file_pid}"
      return 0
    else
      rm -f "${file}"
    fi
  else
    rm -f "${file}"
  fi

  port_out=""
  scan_st=0
  port_out="$(pids_listening_on_port "${port}")" || scan_st=$?
  if [ "${scan_st}" -ne 0 ]; then
    return "${scan_st}"
  fi
  if [[ -n "${port_out}" ]]; then
    while IFS= read -r pid; do
      pid="$(printf '%s' "${pid}" | tr -d '[:space:]')"
      if [[ "${pid}" =~ ^[0-9]+$ ]] && [[ -z "${seen[${pid}]:-}" ]] \
        && file_pid_valid "${service}" "${pid}" "${port}"; then
        printf '%s\n' "${pid}"
        seen["${pid}"]=1
      fi
    done <<< "${port_out}"
  fi
  return 0
}

pids_listening_on_port() {
  local port="$1"
  local scan_st=127

  if have_cmd lsof; then
    local out=""
    local st=0
    out="$(lsof -nP -tiTCP:"${port}" -sTCP:LISTEN 2>/dev/null)" || st=$?
    if [[ "${st}" -eq 0 ]]; then
      printf '%s\n' "${out}" | sed '/^$/d' | sort -u
      return 0
    fi

    if [[ "${st}" -eq 1 ]]; then
      # No matches.
      return 0
    fi

    scan_st="${st}"
    echo "WARN: lsof failed while checking port ${port} (exit ${st}); trying fallback..." >&2
  fi

  if have_cmd ss; then
    # Example: users:(("node",pid=1234,fd=23))
    local ss_out=""
    local ss_st=0
    ss_out="$(ss -H -ltnp "sport = :${port}" 2>/dev/null)" || ss_st=$?
    if [[ "${ss_st}" -eq 0 ]]; then
      printf '%s\n' "${ss_out}" \
        | grep -oE 'pid=[0-9]+' \
        | cut -d= -f2 \
        | sort -u \
        || true
      return 0
    fi

    # ss は lsof/fuser と違い該当なしでも exit 0(空出力)を返すので、非0はすべて失敗
    scan_st="${ss_st}"
    echo "WARN: ss failed while checking port ${port} (exit ${ss_st}); trying fallback..." >&2
  fi

  if have_cmd fuser; then
    local out=""
    local st=0
    out="$(fuser -n tcp "${port}" 2>/dev/null)" || st=$?
    if [[ "${st}" -eq 0 ]]; then
      printf '%s\n' "${out}" \
        | tr ' ' '\n' \
        | sed '/^$/d' \
        | grep -E '^[0-9]+$' \
        | sort -u \
        || true
      return 0
    fi

    if [[ "${st}" -eq 1 ]]; then
      # No matches.
      return 0
    fi

    echo "WARN: fuser failed while checking port ${port} (exit ${st})." >&2
    return "${st}"
  fi

  echo "ERROR: Cannot identify PID(s) listening on port ${port} using lsof/ss/fuser." >&2
  return "${scan_st}"
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
if [[ "${BACKEND_PORT}" =~ ^[0-9]+$ ]]; then BACKEND_PORT=$((10#${BACKEND_PORT})); fi
BACKEND_URL="${BACKEND_URL:-http://127.0.0.1:${BACKEND_PORT}}"

FRONTEND_PORT="${FRONTEND_PORT:-$(extract_port_from_url "${FRONTEND_URL}")}"
FRONTEND_PORT="${FRONTEND_PORT:-8081}"
if [[ "${FRONTEND_PORT}" =~ ^[0-9]+$ ]]; then FRONTEND_PORT=$((10#${FRONTEND_PORT})); fi
FRONTEND_URL="${FRONTEND_URL:-http://127.0.0.1:${FRONTEND_PORT}}"

# PORT と URL を両方明示してポートが食い違うと、走査と応答確認が別サーバーを見るので拒否する
_url_port="$(extract_port_from_url "${BACKEND_URL}")"
if [[ -n "${_url_port}" && "${_url_port}" != "${BACKEND_PORT}" ]]; then
  echo "ERROR: BACKEND_PORT (${BACKEND_PORT}) and BACKEND_URL port (${_url_port}) disagree." >&2
  exit 2
fi
_url_port="$(extract_port_from_url "${FRONTEND_URL}")"
if [[ -n "${_url_port}" && "${_url_port}" != "${FRONTEND_PORT}" ]]; then
  echo "ERROR: FRONTEND_PORT (${FRONTEND_PORT}) and FRONTEND_URL port (${_url_port}) disagree." >&2
  exit 2
fi

_SCOPED_TAG="$(scoped_pid_tag)"
if [[ -z "${BACKEND_PID_FILE}" ]]; then
  BACKEND_PID_FILE="/tmp/shoken-backend-dev-${_SCOPED_TAG}-${BACKEND_PORT}.pid"
fi
if [[ -z "${FRONTEND_PID_FILE}" ]]; then
  FRONTEND_PID_FILE="/tmp/shoken-frontend-dev-${_SCOPED_TAG}-${FRONTEND_PORT}.pid"
fi

front_pids=()
front_out=""
if ! front_out="$(collect_service_pids "frontend" "${FRONTEND_PID_FILE}" "${FRONTEND_PORT}")"; then
  echo "ERROR: Failed to detect frontend PID(s) for port ${FRONTEND_PORT}." >&2
  exit 1
fi
mapfile -t front_pids < <(printf '%s\n' "${front_out}" | sed '/^$/d' | grep -E '^[0-9]+$' || true)

back_pids=()
back_out=""
if ! back_out="$(collect_service_pids "backend" "${BACKEND_PID_FILE}" "${BACKEND_PORT}")"; then
  echo "ERROR: Failed to detect backend PID(s) for port ${BACKEND_PORT}." >&2
  exit 1
fi
mapfile -t back_pids < <(printf '%s\n' "${back_out}" | sed '/^$/d' | grep -E '^[0-9]+$' || true)
if [[ "${#front_pids[@]}" -eq 0 ]] && http_ok "${FRONTEND_URL}/"; then
  echo "ERROR: Frontend responds at ${FRONTEND_URL} but no PID was detected for port ${FRONTEND_PORT}." >&2
  echo "       Install lsof/ss/fuser or run with sufficient permissions." >&2
  exit 1
fi
stop_pids "Frontend (port ${FRONTEND_PORT})" "${front_pids[@]}"
rm -f "${FRONTEND_PID_FILE}"

if [[ "${#back_pids[@]}" -eq 0 ]] && http_ok "${BACKEND_URL}/health"; then
  echo "ERROR: Backend responds at ${BACKEND_URL} but no PID was detected for port ${BACKEND_PORT}." >&2
  echo "       Install lsof/ss/fuser or run with sufficient permissions." >&2
  exit 1
fi
stop_pids "Backend (port ${BACKEND_PORT})" "${back_pids[@]}"
rm -f "${BACKEND_PID_FILE}"

if [[ "${KEEP_DB}" -eq 1 ]]; then
  echo "DB: keep running (--keep-db)"
else
  echo "DB: stopping (docker compose down)..."
  (cd "${BACKEND_DIR}" && make db-down >/dev/null 2>&1) || true
  echo "DB: stopped"
fi
