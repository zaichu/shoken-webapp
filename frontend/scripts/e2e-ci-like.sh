#!/usr/bin/env bash
# 手元の E2E を CI と同じ Ubuntu 24.04・同じフォントで回す。
# 使い方: frontend/ で `bash scripts/e2e-ci-like.sh [オプション] [playwright に渡す引数]`
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
FRONTEND_DIR=$(cd "$SCRIPT_DIR/.." && pwd)
ROOT=$(cd "$FRONTEND_DIR/.." && pwd)
WORKFLOW="$ROOT/.github/workflows/frontend.yml"
DOCKERFILE="$SCRIPT_DIR/e2e-ci-like.Dockerfile"
IMAGE_BASE="shoken-e2e-ci-like"

REBUILD=0
NO_BUILD=0
SHOW_FONTS=0
SHOW_FONTS_ONLY=0
BUILD_ONLY=0
PLAYWRIGHT_ARGS=()

for arg in "$@"; do
  case "$arg" in
    --rebuild) REBUILD=1 ;;
    --no-build) NO_BUILD=1 ;;
    --show-fonts) SHOW_FONTS=1 ;;
    --show-fonts-only) SHOW_FONTS=1; SHOW_FONTS_ONLY=1 ;;
    --build-only) BUILD_ONLY=1 ;;
    -h|--help)
      cat <<'USAGE'
使い方: bash scripts/e2e-ci-like.sh [オプション] [playwright への引数]

例:
  bash scripts/e2e-ci-like.sh e2e/receipts-desktop-layout.spec.ts
  bash scripts/e2e-ci-like.sh e2e/receipts-desktop-layout.spec.ts --grep "768px"
  bash scripts/e2e-ci-like.sh --show-fonts e2e/receipts-desktop-layout.spec.ts

オプション:
  --rebuild           イメージを作り直す(既定はタグがあれば使い回す)
  --no-build          ホストの trunk build を省き既存の dist-e2e を使う
  --show-fonts        ホストとコンテナの fc-match を並べて表示してから実行する
  --show-fonts-only   fc-match の表示だけして終える
  --build-only        イメージの用意(と dist の build)だけして終える
  -h, --help          この表示

playwright への引数はそのまま渡す(spec・--grep・--repeat-each など)。
ポートは LEPTOS_E2E_PORT がなければ 8140 を使う。
USAGE
      exit 0
      ;;
    *) PLAYWRIGHT_ARGS+=("$arg") ;;
  esac
done

NODE_MAJOR=$(grep 'node-version:' "$WORKFLOW" | head -n 1 | sed -e 's/^[^0-9]*//' -e 's/[^0-9].*//' || true)
APT_PACKAGES=$(grep 'apt-get install' "$WORKFLOW" | sed -e 's/^.*apt-get install//' | tr ' ' '\n' | { grep -v -e '^-' -e '^$' -e '::' -e '=' -e 'apt-packages' -e '^\$' || true; } | sort -u | tr '\n' ' ' || true)
PLAYWRIGHT_VERSION=$(grep -A2 '"node_modules/@playwright/test"' "$ROOT/frontend/package-lock.json" | sed -n 's/.*"version": *"\([0-9][0-9.]*\)".*/\1/p' | head -n 1 || true)
[ -n "$NODE_MAJOR" ] || { echo "node-version を $WORKFLOW から読めません" >&2; exit 1; }
[ -n "$APT_PACKAGES" ] || { echo "apt の一覧を $WORKFLOW から読めません" >&2; exit 1; }
[ -n "$PLAYWRIGHT_VERSION" ] || { echo "@playwright/test の版を読めません" >&2; exit 1; }
IMAGE_TAG=$( { cat "$DOCKERFILE"; printf '%s\n' "ubuntu24.04" "$NODE_MAJOR" "$APT_PACKAGES" "$PLAYWRIGHT_VERSION"; } | sha256sum | cut -c1-12)
IMAGE="$IMAGE_BASE:$IMAGE_TAG"
PORT="${LEPTOS_E2E_PORT:-8140}"

host_fc() {
  if ! command -v fc-match >/dev/null 2>&1; then
    echo "fc-match なし"
    return 0
  fi
  fc-match "$1" | head -n 1
}

container_fc() {
  docker run --rm "$IMAGE" fc-match "$1" | head -n 1
}

show_fonts() {
  for pattern in "sans-serif" "monospace" "sans-serif:lang=ja"; do
    printf 'ホスト[%s]: %s\n' "$pattern" "$(host_fc "$pattern")"
    printf 'コンテナ[%s]: %s\n' "$pattern" "$(container_fc "$pattern")"
  done
}

if [ "$(docker images -q "$IMAGE" 2>/dev/null)" = "" ] || [ "$REBUILD" -eq 1 ]; then
  docker build -f "$DOCKERFILE" -t "$IMAGE" \
    --build-arg "NODE_MAJOR=$NODE_MAJOR" \
    --build-arg "APT_PACKAGES=$APT_PACKAGES" \
    --build-arg "PLAYWRIGHT_VERSION=$PLAYWRIGHT_VERSION" \
    "$FRONTEND_DIR"
fi

if [ "$SHOW_FONTS" -eq 1 ] && [ "$SHOW_FONTS_ONLY" -eq 1 ]; then
  show_fonts
  exit 0
fi

if [ "$NO_BUILD" -eq 0 ]; then
  export PATH="$HOME/.cargo/bin:$PATH"
  cd "$FRONTEND_DIR"
  npm ci --no-audit --no-fund
  env -u NO_COLOR trunk build --release --dist dist-e2e
fi

if [ "$BUILD_ONLY" -eq 1 ]; then
  exit 0
fi

if [ "$SHOW_FONTS" -eq 1 ]; then
  show_fonts
fi

cd "$FRONTEND_DIR"
docker run --rm --ipc=host \
  --user "$(id -u):$(id -g)" \
  -v "$ROOT:/work" \
  -w /work/frontend \
  -e LEPTOS_E2E_DIST_DIR=dist-e2e \
  -e "LEPTOS_E2E_PORT=$PORT" \
  -e "LEPTOS_E2E_OUTPUT_DIR=${LEPTOS_E2E_OUTPUT_DIR:-test-results}" \
  -e HOME=/tmp \
  -e CI=1 \
  "$IMAGE" \
  bash -c 'npm ci --no-audit --no-fund && npx playwright install chromium && env -u NO_COLOR npx playwright test --config playwright.leptos.config.ts "$@"' bash "${PLAYWRIGHT_ARGS[@]}"
