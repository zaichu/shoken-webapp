#!/usr/bin/env bash
# 旧 migration(backend/migrations/archive)を順に当てた DB と、土台(backend/migrations)だけを
# 当てた DB を使い捨ての Docker Postgres で作り、スキーマ・ロール・権限・初期行が一致するかを比べる。
#
# 使い方: scripts/check-migration-baseline.sh [ダンプの保存先]
#   POSTGRES_IMAGE  比べる Postgres(既定は本番と同じ 17)
#   MODES           移行ロールの種類。superuser(既定の postgres)と owner(本番と同じ非 superuser の
#                   CREATEROLE ロール)。既定は両方
set -euo pipefail

ROOT="$(cd "${BASH_SOURCE[0]%/*}/.." && pwd)"
IMAGE="${POSTGRES_IMAGE:-postgres:17}"
MODES="${MODES:-superuser owner}"
LEGACY_DIR="${ROOT}/backend/migrations/archive"
BASELINE_DIR="${ROOT}/backend/migrations"
OUT="${1:-$(mktemp -d)}"
mkdir -p "${OUT}"

PREFIX="shoken-baseline-check-$$"
STARTED=()
cleanup() {
    for name in "${STARTED[@]}"; do
        docker rm -f "${name}" >/dev/null 2>&1 || true
    done
}
trap cleanup EXIT

start_db() {
    local name="$1"
    docker run -d --rm --name "${name}" -e POSTGRES_PASSWORD=postgres "${IMAGE}" >/dev/null
    STARTED+=("${name}")
}

# 初期化中の一時サーバーは unix socket だけで待つため、TCP で応答するまで待つ
wait_db() {
    local name="$1"
    for _ in $(seq 1 60); do
        if docker exec "${name}" pg_isready -q -h 127.0.0.1 -U postgres; then
            return 0
        fi
        sleep 1
    done
    echo "::error::${name} が起動しませんでした" >&2
    return 1
}

run_psql() {
    local name="$1" user="$2"
    shift 2
    docker exec -i -e PGPASSWORD="${user}" -e PGOPTIONS='-c client_min_messages=error' "${name}" \
        psql -X -q -h 127.0.0.1 -U "${user}" -d postgres -v ON_ERROR_STOP=1 "$@"
}

prepare_owner() {
    local name="$1"
    run_psql "${name}" postgres -c "
        CREATE ROLE migrator LOGIN PASSWORD 'migrator' CREATEROLE NOSUPERUSER BYPASSRLS;
        GRANT CREATE ON DATABASE postgres TO migrator;
        GRANT CREATE ON SCHEMA public TO migrator WITH GRANT OPTION;"
}

# ランナー(backend/src/db/migrate.rs)と同じく 1 ファイル 1 トランザクションで番号順に当てる
apply_dir() {
    local name="$1" user="$2" dir="$3" file
    for file in "${dir}"/[0-9][0-9][0-9][0-9]_*.sql; do
        if [[ "$(head -n1 "${file}")" == "-- no-transaction"* ]]; then
            run_psql "${name}" "${user}" -f - <"${file}"
        else
            run_psql "${name}" "${user}" --single-transaction -f - <"${file}"
        fi || {
            echo "::error::$(basename "${file}") の適用に失敗しました" >&2
            return 1
        }
    done
}

# pg_dump 17 の \restrict 行は実行ごとに乱数の鍵を含むため除く
dump_db() {
    local name="$1" out="$2"
    {
        echo '-- schema'
        docker exec -e PGPASSWORD=postgres "${name}" pg_dump -h 127.0.0.1 -U postgres \
            --schema-only --exclude-table=public._sqlx_migrations postgres
        echo '-- roles'
        docker exec -e PGPASSWORD=postgres "${name}" pg_dumpall -h 127.0.0.1 -U postgres \
            --roles-only --no-role-passwords
        echo '-- seed rows'
        run_psql "${name}" postgres -At -c "
            SELECT 'fund_codes', normalized_name, association_fund_cd, fund_name
              FROM public.fund_codes ORDER BY normalized_name;
            SELECT 'market_data_provider_rate_control', provider
              FROM public.market_data_provider_rate_control ORDER BY provider;"
    } | grep -vE '^\\(un)?restrict ' >"${out}"
}

build() {
    local mode="$1" kind="$2" dir="$3"
    local name="${PREFIX}-${mode}-${kind}" user=postgres
    wait_db "${name}"
    if [[ "${mode}" == owner ]]; then
        prepare_owner "${name}"
        user=migrator
    fi
    apply_dir "${name}" "${user}" "${dir}"
    dump_db "${name}" "${OUT}/${mode}-${kind}.sql"
    docker rm -f "${name}" >/dev/null
}

status=0
for mode in ${MODES}; do
    start_db "${PREFIX}-${mode}-legacy"
    start_db "${PREFIX}-${mode}-baseline"
    build "${mode}" legacy "${LEGACY_DIR}"
    build "${mode}" baseline "${BASELINE_DIR}"
    if diff -u "${OUT}/${mode}-legacy.sql" "${OUT}/${mode}-baseline.sql" >"${OUT}/${mode}.diff"; then
        echo "${mode}: 旧 migration と土台の結果は一致しました($(wc -l <"${OUT}/${mode}-legacy.sql") 行)"
    else
        echo "::error::${mode}: 旧 migration と土台の結果が一致しません" >&2
        cat "${OUT}/${mode}.diff"
        status=1
    fi
done
echo "ダンプ: ${OUT}"
exit "${status}"
