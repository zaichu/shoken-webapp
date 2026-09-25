#!/usr/bin/env bash
# `vercel build` の buildCommand。Rust toolchain と trunk は GitHub Actions
# (deploy-frontend-leptos.yml) またはローカル環境側で用意する。
set -euo pipefail
cd "$(dirname "$0")/.."

DIST_DIR="${VERCEL_DIST_DIR:-dist}"

export PATH="$HOME/.cargo/bin:$PATH"

# vercel.json の connect-src が許可する唯一の外部 origin。
# これ以外を埋め込むと CSP が API 通信をブロックするためビルド時に弾く。
ALLOWED_API_ORIGIN="$(node scripts/backend-origin.cjs)"

if [[ -z "${SHOKEN_WEBAPI_URL:-}" ]]; then
  echo "SHOKEN_WEBAPI_URL is not set. The bundle would call same-origin /api, which Vercel rewrites to index.html." >&2
  exit 1
fi
if [[ "$SHOKEN_WEBAPI_URL" != "$ALLOWED_API_ORIGIN" ]]; then
  echo "SHOKEN_WEBAPI_URL must be ${ALLOWED_API_ORIGIN}. Other origins are blocked by connect-src in vercel.json." >&2
  exit 1
fi

# trunk 0.21.4 は NO_COLOR=1 を予期しない引数として解釈するため解除する
env -u NO_COLOR trunk build --release --dist "$DIST_DIR"
node scripts/prepare-vercel-dist.mjs "$DIST_DIR"
