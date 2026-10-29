#!/usr/bin/env bash
set +x
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RECIPIENT_FILE="${RECIPIENT_FILE:-$ROOT_DIR/.github/backup-recipient.txt}"
OUT="${OUT:-db-backup-$(date -u +%Y%m%dT%H%M%SZ).sql.gz.age}"

if [[ $# -gt 1 ]]; then
  echo "使いかた: PGURL=<接続文字列> [RECIPIENT_FILE=<公開鍵>] [OUT=<出力>] bash scripts/db-backup.sh [出力]" >&2
  exit 2
fi
if [[ $# -eq 1 ]]; then
  OUT="$1"
fi

case "$OUT" in
  *.sql.gz.age) ;;
  *)
    echo "バックアップの出力先は .sql.gz.age で終わる必要があります" >&2
    exit 1
    ;;
esac

if [[ -z "${PGURL:-}" ]]; then
  echo "バックアップに必要な接続情報がありません" >&2
  exit 1
fi

for tool in pg_dump gzip age; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "バックアップに必要な道具がありません" >&2
    exit 1
  fi
done

if [[ ! -f "$RECIPIENT_FILE" ]]; then
  echo "バックアップの公開鍵がありません" >&2
  exit 1
fi
recipient="$(tr -d ' \t\r\n' < "$RECIPIENT_FILE")"
if [[ -z "$recipient" ]]; then
  echo "バックアップの公開鍵がありません" >&2
  exit 1
fi
if ! printf '%s' "$recipient" | grep -Eq '^age1[0-9a-z]{58}$'; then
  echo "バックアップの公開鍵の形式が正しくありません" >&2
  exit 1
fi

err_file="$(mktemp)"
tmp_out="${OUT}.tmp.$$"
cleanup() {
  rm -f "$err_file" "$tmp_out"
}
trap cleanup EXIT

if ! pg_dump --schema=public "$PGURL" 2>"$err_file" | gzip -c | age --recipient "$recipient" --output "$tmp_out"; then
  echo "バックアップの書き出しに失敗しました" >&2
  exit 1
fi
if [[ ! -s "$tmp_out" ]]; then
  echo "バックアップの書き出しに失敗しました" >&2
  exit 1
fi
mv "$tmp_out" "$OUT"
trap - EXIT
rm -f "$err_file"
printf '%s\n' "$OUT"
