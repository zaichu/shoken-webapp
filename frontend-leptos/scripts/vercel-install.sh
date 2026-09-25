#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# package.json の @playwright/test は file:../frontend/... を指すが、
# Vercel の Root Directory ビルドには ../frontend が含まれない。
# ビルドでは playwright を使わないため、npm のリンク解決用にスタブだけ置く。
# ルート外ソース含有が有効で実在する場合は触らない。
stub="../frontend/node_modules/@playwright/test"
if [[ ! -f "$stub/package.json" ]]; then
  mkdir -p "$stub"
  printf '{"name":"@playwright/test","version":"1.63.0"}\n' > "$stub/package.json"
fi

npm ci
