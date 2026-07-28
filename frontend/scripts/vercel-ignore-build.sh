#!/usr/bin/env bash
set -euo pipefail

if [[ "${VERCEL_ENV:-}" == "production" ]]; then
  echo "Production deployment: build required."
  exit 1
fi

base_ref="origin/main"
if ! git rev-parse --verify "$base_ref" >/dev/null 2>&1; then
  git fetch --quiet --depth=50 origin main:refs/remotes/origin/main || true
fi

if git rev-parse --verify "$base_ref" >/dev/null 2>&1; then
  set +e
  git diff --quiet "$base_ref"...HEAD -- .
  diff_status=$?
  set -e

  case "$diff_status" in
    0)
      echo "No frontend changes against main: skip Vercel build."
      exit 0
      ;;
    1)
      echo "Frontend changes detected against main: build required."
      exit 1
      ;;
    *)
      echo "Unable to compare against main; falling back to last commit diff."
      ;;
  esac
fi

if git diff --quiet HEAD^ HEAD -- .; then
  echo "No frontend changes in last commit: skip Vercel build."
  exit 0
fi

echo "Frontend changes detected in last commit: build required."
exit 1
