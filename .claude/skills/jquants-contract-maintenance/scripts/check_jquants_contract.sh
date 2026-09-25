#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT"

echo "[1/3] backend jquants tests"
(cd backend && cargo test jquants)

echo "[2/3] openapi sync"
bash scripts/check-openapi.sh

echo "[3/3] frontend contract and dividend tests"
(cd frontend-leptos && cargo test contract_matches_openapi)
(cd frontend-leptos && cargo test dividend)
