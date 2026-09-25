#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

DIST_DIR="${1:-dist}"
PORT="${2:-8190}"

node scripts/prepare-vercel-dist.mjs "$DIST_DIR"
exec node scripts/serve-dist.mjs "$DIST_DIR" "$PORT"
