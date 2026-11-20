#!/usr/bin/env bash
# 本番と同じ形の DB(旧 migration の backend/migrations/archive を順に当て、土台の後ろの migration を足す)と、
# 新しい DB と同じ形の DB(backend/migrations を全て当てる)を使い捨ての Docker Postgres で作り、
# スキーマ・ロール・権限・初期行が一致するかを比べる。土台の後ろに足した migration が
# 本番と同じ形の DB にも当たることを毎回確かめる。
#
# 使い方: scripts/check-migration-baseline.sh [ダンプの保存先]
#   POSTGRES_IMAGE  比べる Postgres(既定は本番と同じ 17)
#   MODES           移行ロールの種類。superuser(既定の postgres)と owner(新しい環境と同じ条件の
#                   非 superuser ロール: NOSUPERUSER BYPASSRLS CREATEROLE。土台 0100 が BYPASSRLS の
#                   限定関数所有ロールを作るため)。既定は両方
set -euo pipefail

ROOT="$(cd "${BASH_SOURCE[0]%/*}/.." && pwd)"
IMAGE="${POSTGRES_IMAGE:-postgres:17}"
MODES="${MODES:-superuser owner}"
LEGACY_DIR="${ROOT}/backend/migrations/archive"
CURRENT_DIR="${ROOT}/backend/migrations"
# 土台の最後の版はランナーの定数を正本にする
BASELINE_LAST_VERSION="$(sed -nE 's/^const BASELINE_LAST_VERSION: i64 = ([0-9]+);$/\1/p' "${ROOT}/backend/src/db/migrate.rs")"
[[ -n "${BASELINE_LAST_VERSION}" ]] || { echo "::error::BASELINE_LAST_VERSION を読めません" >&2; exit 1; }
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

# 番号順のファイル一覧。after を渡すとその版より後ろだけ
migration_files() {
    local dir="$1" after="${2:--1}" file version
    for file in "${dir}"/[0-9][0-9][0-9][0-9]_*.sql; do
        [[ -e "${file}" ]] || continue
        version="$(basename "${file}")"
        version=$((10#${version:0:4}))
        if ((version > after)); then
            echo "${file}"
        fi
    done
}

# ランナー(backend/src/db/migrate.rs)と同じく 1 ファイル 1 トランザクションで順に当てる
apply_files() {
    local name="$1" user="$2" file
    shift 2
    for file in "$@"; do
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
    local mode="$1" kind="$2"
    shift 2
    local name="${PREFIX}-${mode}-${kind}" user=postgres
    wait_db "${name}"
    if [[ "${mode}" == owner ]]; then
        prepare_owner "${name}"
        user=migrator
    fi
    apply_files "${name}" "${user}" "$@"
    dump_db "${name}" "${OUT}/${mode}-${kind}.sql"
    docker rm -f "${name}" >/dev/null
}

status=0
for mode in ${MODES}; do
    start_db "${PREFIX}-${mode}-legacy"
    start_db "${PREFIX}-${mode}-baseline"
    mapfile -t legacy_files < <(migration_files "${LEGACY_DIR}"; migration_files "${CURRENT_DIR}" "${BASELINE_LAST_VERSION}")
    mapfile -t current_files < <(migration_files "${CURRENT_DIR}")
    build "${mode}" legacy "${legacy_files[@]}"
    build "${mode}" baseline "${current_files[@]}"
    if diff -u "${OUT}/${mode}-legacy.sql" "${OUT}/${mode}-baseline.sql" >"${OUT}/${mode}.diff"; then
        echo "${mode}: 本番と同じ形(archive + ${BASELINE_LAST_VERSION} より後)と新しい DB(土台から)の結果は一致しました($(wc -l <"${OUT}/${mode}-legacy.sql") 行)"
    else
        echo "::error::${mode}: 本番と同じ形の DB と新しい DB の結果が一致しません" >&2
        cat "${OUT}/${mode}.diff"
        status=1
    fi
done
echo "ダンプ: ${OUT}"
exit "${status}"
