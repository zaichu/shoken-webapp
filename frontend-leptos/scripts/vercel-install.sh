#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# package.json の file:../frontend/node_modules/@playwright/test を解決するため
# frontend 側の node_modules を先に用意する
(cd ../frontend && npm ci)
npm ci --include=dev
