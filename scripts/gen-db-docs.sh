#!/usr/bin/env bash
# DB スキーマのドキュメント(テーブル一覧と ER 図)を docs/db/ に生成する。
# 一時 PostgreSQL を起動 → 全 migration を適用 → tbls doc → 後片付け。
# 実データ・本番の接続情報は使わない(起動直後の空 DB にだけ migration を適用する)。
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

CONTAINER_NAME="shoken-db-docs-$$"
POSTGRES_IMAGE="postgres:16"
DB_NAME="shoken_db"
DB_USER="user"
DB_PASSWORD="password"

cleanup() {
  # 自分が起動したコンテナだけ止める(他の作業のコンテナには触らない)
  docker rm --force "${CONTAINER_NAME}" >/dev/null 2>&1 || true
}
trap cleanup EXIT

if ! command -v tbls >/dev/null 2>&1; then
  echo "tbls が見つかりません。README を参照してインストールしてください" >&2
  exit 1
fi
if ! command -v docker >/dev/null 2>&1; then
  echo "docker が見つかりません" >&2
  exit 1
fi

echo "1/3 一時 PostgreSQL を起動..."
docker run --detach --name "${CONTAINER_NAME}" \
  --env "POSTGRES_USER=${DB_USER}" \
  --env "POSTGRES_PASSWORD=${DB_PASSWORD}" \
  --env "POSTGRES_DB=${DB_NAME}" \
  --publish "127.0.0.1::5432" \
  "${POSTGRES_IMAGE}" >/dev/null

for _ in $(seq 1 60); do
  if docker exec "${CONTAINER_NAME}" pg_isready --username "${DB_USER}" --dbname "${DB_NAME}" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
if ! docker exec "${CONTAINER_NAME}" pg_isready --username "${DB_USER}" --dbname "${DB_NAME}" >/dev/null 2>&1; then
  echo "一時 PostgreSQL が起動しませんでした" >&2
  exit 1
fi

# ポートは空指定で自動割り当てさせる(他の作業の DB と衝突しない)
PORT="$(docker port "${CONTAINER_NAME}" 5432/tcp | head -n 1 | sed 's/.*://')"
DSN="postgresql://${DB_USER}:${DB_PASSWORD}@127.0.0.1:${PORT}/${DB_NAME}"

echo "2/3 migration を適用..."
(cd backend && DATABASE_URL="${DSN}" cargo run --bin migrate)

echo "3/3 tbls doc を実行..."
# --rm-dist: 消えたテーブルなどの古い生成物を残さない
TBLS_DSN="${DSN}?sslmode=disable" tbls doc --config .tbls.yml --rm-dist

echo "docs/db/ を更新しました。差分を確認してコミットしてください"
