#!/usr/bin/env bash
# `vercel build` の buildCommand。Rust toolchain と trunk は GitHub Actions
# (deploy-frontend.yml) またはローカル環境側で用意する。
set -euo pipefail
cd "$(dirname "$0")/.."

DIST_DIR="${VERCEL_DIST_DIR:-dist}"

export PATH="$HOME/.cargo/bin:$PATH"

# trunk 0.21.4 は NO_COLOR=1 を予期しない引数として解釈するため解除する
env -u NO_COLOR trunk build --release --dist "$DIST_DIR"
node scripts/prepare-vercel-dist.mjs "$DIST_DIR"
