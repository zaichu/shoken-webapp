#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

RUST_TOOLCHAIN="1.96.0"
TRUNK_VERSION="0.21.4"
DIST_DIR="${VERCEL_DIST_DIR:-dist}"

export PATH="$HOME/.cargo/bin:$PATH"

if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh
  sh /tmp/rustup-init.sh -y --profile minimal --default-toolchain "$RUST_TOOLCHAIN" --target wasm32-unknown-unknown
else
  rustup toolchain install "$RUST_TOOLCHAIN" --profile minimal --target wasm32-unknown-unknown
fi
rustup default "$RUST_TOOLCHAIN"

if ! command -v trunk >/dev/null 2>&1; then
  curl -sSfL "https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz" -o /tmp/trunk.tar.gz
  curl -sSfL "https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz.sha256" -o /tmp/trunk.tar.gz.sha256
  (cd /tmp && echo "$(tr -d '[:space:]' < trunk.tar.gz.sha256)  trunk.tar.gz" | sha256sum --check)
  tar -xzf /tmp/trunk.tar.gz -C /tmp
  install -m 755 /tmp/trunk "$HOME/.cargo/bin/trunk"
fi

# vercel.json の connect-src が許可する唯一の外部 origin。
# これ以外を埋め込むと CSP が API 通信をブロックするためビルド時に弾く。
ALLOWED_API_ORIGIN="https://shoken-backend.fly.dev"

if [[ -z "${SHOKEN_WEBAPI_URL:-}" ]]; then
  echo "SHOKEN_WEBAPI_URL is not set. The bundle would call same-origin /api, which Vercel rewrites to index.html." >&2
  exit 1
fi
if [[ "$SHOKEN_WEBAPI_URL" != "$ALLOWED_API_ORIGIN" ]]; then
  echo "SHOKEN_WEBAPI_URL must be ${ALLOWED_API_ORIGIN}. Other origins are blocked by connect-src in vercel.json." >&2
  exit 1
fi

env -u NO_COLOR trunk build --release --dist "$DIST_DIR"
node scripts/prepare-vercel-dist.mjs "$DIST_DIR"
